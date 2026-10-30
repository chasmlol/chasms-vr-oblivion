//! `oblivion-extract`: reads the user's own Oblivion install (read-only).
//!
//!     oblivion-extract [--game <Oblivion folder>] check <prefix>...
//!     oblivion-extract export [--out <folder>] <prefix>...
//!     oblivion-extract preview <model> <png>
//!     oblivion-extract bench <prefix>...
//!     oblivion-extract export-ui <prefix>...
//!
//! `check` parses every model under the prefixes and reports which it can't read (a test of the
//! model reader against the real files). `export` writes every model under the prefixes, and the
//! textures they use, into the runtime's cache (`local_assets/oblivion`, see
//! `oblivion_assets::export`; a `textures/...` prefix brings every texture under it, e.g. the races'
//! skins). `preview` draws one converted model from three sides. `bench` splits each mortar and pestle
//! under the prefixes into its two pieces and measures them (`oblivion_assets::bench`: the pieces in
//! the cache, the measurements in its `bench.toml`). `export-ui` writes the game's fonts and menu
//! art under the prefixes for the runtime to paint (`oblivion_assets::ui`: `fonts\...` and
//! `textures\menus\...`, the art as RGBA).

use anyhow::Result;
use oblivion_assets::export::{self, Part};
use oblivion_assets::{bsa::Archives, nif::Nif, preview};
use std::path::PathBuf;

const DEFAULT_GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion";

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut game = PathBuf::from(DEFAULT_GAME);
    if args.first().map(String::as_str) == Some("--game") {
        game = PathBuf::from(args.remove(1));
        args.remove(0);
    }
    let archives = Archives::open(&game)?;
    match args.first().map(String::as_str) {
        Some("check") => {
            let prefixes: Vec<String> = args[1..].iter().map(|p| oblivion_assets::bsa::normalise(p)).collect();
            let (mut ok, mut short, mut bad) = (0, 0, 0);
            let mut stopped = std::collections::BTreeMap::new();
            let names: Vec<String> = archives.names().filter(|n| n.ends_with(".nif") && prefixes.iter().any(|p| n.starts_with(p.as_str()))).map(str::to_owned).collect();
            for name in names {
                match archives.read(&name).and_then(|d| Nif::parse(&d)) {
                    Ok(n) if n.stopped_at.is_none() => ok += 1,
                    Ok(n) => {
                        short += 1;
                        *stopped.entry(n.stopped_at.unwrap()).or_insert(0) += 1;
                    }
                    Err(e) => {
                        bad += 1;
                        if bad <= 10 {
                            println!("{name}: {e:#}");
                        }
                    }
                }
            }
            println!("{ok} read whole, {short} stopped at an unknown block, {bad} failed");
            for (kind, n) in stopped {
                println!("  stopped at {kind}: {n}");
            }
        }
        Some("dump") => {
            for path in &args[1..] {
                let nif = Nif::parse(&archives.read(path)?)?;
                dump(&nif);
            }
        }
        Some("export") => export(&archives, &args[1..])?,
        Some("bench") => bench(&archives, &args[1..])?,
        Some("export-ui") => export_ui(&archives, &args[1..])?,
        Some("preview") if args.len() == 3 => {
            let skeletons = Skeletons::load(&archives)?;
            let path = oblivion_assets::bsa::normalise(&args[1]);
            let nif = Nif::parse(&archives.read(&path)?)?;
            let c = export::convert(&nif, &path, skeletons.for_model(&nif), Part::All, &|p| archives.contains(p));
            preview::save(&c.model, std::path::Path::new(&args[2]))?;
            println!("{path}: {} meshes, {} bones -> {}", c.model.meshes.len(), c.model.bones.len(), args[2]);
        }
        _ => println!("usage: oblivion-extract [--game <folder>] check <prefix>... | dump <model>... | export [--out <folder>] <prefix>... | preview <model> <png>"),
    }
    Ok(())
}

/// The game's two skeletons: everyone's, and the beast races' (with a tail).
struct Skeletons {
    human: (String, export::Skeleton),
    beast: (String, export::Skeleton),
}

impl Skeletons {
    fn load(archives: &Archives) -> Result<Skeletons> {
        let load = |p: &str| -> Result<(String, export::Skeleton)> { Ok((p.to_owned(), export::Skeleton::from_nif(&Nif::parse(&archives.read(p)?)?))) };
        Ok(Skeletons { human: load("meshes\\characters\\_male\\skeleton.nif")?, beast: load("meshes\\characters\\_male\\skeletonbeast.nif")? })
    }

    /// The skeleton a skinned model was made for: the beasts' when it has a tail.
    fn for_model(&self, nif: &Nif) -> Option<&export::Skeleton> {
        let tail = nif.blocks.iter().any(|b| matches!(b, oblivion_assets::nif::Block::Node { av, .. } if av.name.to_ascii_lowercase().starts_with("bip01 tail")));
        Some(if tail { &self.beast.1 } else { &self.human.1 })
    }
}

/// Converts every model under the prefixes (and the textures they use) into `local_assets/oblivion`
/// (or `--out`), with an `index.toml` of what each key is. A prefix under `textures\` brings every
/// texture there.
fn export(archives: &Archives, args: &[String]) -> Result<()> {
    use rustvr_asset::{Index, Item, ItemKind};
    let mut args = args.to_vec();
    let mut out = rustvr_asset::default_root().join("oblivion");
    if args.first().map(String::as_str) == Some("--out") && args.len() > 1 {
        out = PathBuf::from(args.remove(1));
        args.remove(0);
    }
    let prefixes: Vec<String> = args.iter().map(|p| oblivion_assets::bsa::normalise(p)).collect();
    let skeletons = Skeletons::load(archives)?;
    let index_path = out.join("index.toml");
    let mut index: Index = std::fs::read_to_string(&index_path).ok().and_then(|s| toml::from_str(&s).ok()).unwrap_or_default();
    index.game = "oblivion".into();
    index.version = rustvr_asset::FORMAT_VERSION;
    let mut items: std::collections::BTreeMap<String, Item> = index.items.drain(..).map(|i| (i.key.clone(), i)).collect();
    let mut textures = std::collections::BTreeSet::new();
    let mut add = |key: String, kind: ItemKind, name: String, model: &rustvr_asset::Model| -> Result<()> {
        rustvr_asset::save_model(&out, &key, model)?;
        items.insert(key.clone(), Item { key, kind, name, rig: None, hands: None, mag_size: None });
        Ok(())
    };
    // The skeletons themselves (bones, no meshes).
    for (path, s) in [&skeletons.human, &skeletons.beast] {
        let model = rustvr_asset::Model { version: rustvr_asset::FORMAT_VERSION, source: path.clone(), bones: s.bones.clone(), ..Default::default() };
        add(export::model_key(path), ItemKind::Body, export::normalise(path, "meshes\\"), &model)?;
    }
    let names: Vec<String> = archives.names().filter(|n| n.ends_with(".nif") && prefixes.iter().any(|p| n.starts_with(p.as_str()))).map(str::to_owned).collect();
    let (mut models, mut failed) = (0, 0);
    for path in names {
        if path.ends_with("\\skeleton.nif") || path.ends_with("\\skeletonbeast.nif") {
            continue;
        }
        let nif = match archives.read(&path).and_then(|d| Nif::parse(&d)) {
            Ok(n) => n,
            Err(e) => {
                failed += 1;
                eprintln!("{path}: {e:#}");
                continue;
            }
        };
        let skinned = nif.blocks.iter().any(|b| matches!(b, oblivion_assets::nif::Block::Shape { skin, .. } if *skin >= 0));
        let skeleton = if skinned { skeletons.for_model(&nif) } else { None };
        let exists = |p: &str| archives.contains(p);
        let rel = export::normalise(&path, "meshes\\");
        let kind = if rel.starts_with("weapons\\") {
            ItemKind::Weapon
        } else if skinned || rel.starts_with("characters\\") {
            ItemKind::Body
        } else {
            ItemKind::Prop
        };
        // A weapon with its scabbard: the weapon alone, and the scabbard alone.
        let parts: Vec<(String, String, Part)> = if nif.find("Scb").is_some() {
            vec![(export::model_key(&path), rel.clone(), Part::Without("Scb")), (export::variant_key(&path, "scb"), format!("{rel}#scb"), Part::Only("Scb"))]
        } else {
            vec![(export::model_key(&path), rel.clone(), Part::All)]
        };
        for (key, name, part) in parts {
            let c = export::convert(&nif, &path, skeleton, part, &exists);
            if c.model.meshes.is_empty() {
                continue;
            }
            textures.extend(c.textures);
            add(key, kind, name, &c.model)?;
            models += 1;
        }
    }
    // Textures asked for by folder (a race's skins, which no model names: the game swaps them in).
    textures.extend(archives.names().filter(|n| n.ends_with(".dds") && prefixes.iter().any(|p| p.starts_with("textures\\") && n.starts_with(p.as_str()))).map(str::to_owned));
    let (mut wrote, mut missing) = (0, 0);
    for path in &textures {
        let key = export::texture_key(path);
        if out.join("textures").join(format!("{key}.{}", rustvr_asset::TEXTURE_EXT)).is_file() {
            continue;
        }
        match archives.read(path).and_then(|d| oblivion_assets::dds::parse(&d, path)) {
            Ok(t) => {
                rustvr_asset::save_texture(&out, &key, &t)?;
                wrote += 1;
            }
            Err(e) => {
                missing += 1;
                if missing <= 10 {
                    eprintln!("{path}: {e:#}");
                }
            }
        }
    }
    index.items = items.into_values().collect();
    std::fs::write(&index_path, toml::to_string(&index)?)?;
    println!("{models} models ({failed} unreadable), {wrote} new textures ({missing} missing) in {}", out.display());
    Ok(())
}

/// Splits and measures every mortar and pestle under the prefixes (`oblivion_assets::bench`).
fn bench(archives: &Archives, args: &[String]) -> Result<()> {
    use oblivion_assets::bench::{self, Benches};
    let out = rustvr_asset::default_root().join("oblivion");
    let prefixes: Vec<String> = args.iter().map(|p| oblivion_assets::bsa::normalise(p)).collect();
    let mut benches = Benches::load(&out).unwrap_or_default();
    let index_path = out.join("index.toml");
    let mut index: rustvr_asset::Index = std::fs::read_to_string(&index_path).ok().and_then(|s| toml::from_str(&s).ok()).unwrap_or_default();
    let names: Vec<String> = archives.names().filter(|n| n.ends_with(".nif") && prefixes.iter().any(|p| n.starts_with(p.as_str()))).map(str::to_owned).collect();
    let mut textures = std::collections::BTreeSet::new();
    for path in names {
        let nif = Nif::parse(&archives.read(&path)?)?;
        let c = export::convert(&nif, &path, None, Part::All, &|p| archives.contains(p));
        let Some((b, mortar, pestle)) = bench::measure(&c.model, &path) else {
            println!("{path}: no pestle of its own");
            continue;
        };
        textures.extend(c.textures);
        let rel = export::normalise(&path, "meshes\\");
        for (key, name, model) in [(&b.mortar, format!("{rel}#mortar"), &mortar), (&b.pestle, format!("{rel}#pestle"), &pestle)] {
            rustvr_asset::save_model(&out, key, model)?;
            index.items.retain(|i| &i.key != key);
            index.items.push(rustvr_asset::Item { key: key.clone(), kind: rustvr_asset::ItemKind::Prop, name, rig: None, hands: None, mag_size: None });
        }
        println!(
            "{path}: bowl floor {:.3?} radius {:.3} depth {:.3} wall {:.3}; pestle {:.3} long, {:.3} thick, tip {:.3} at {:.3?}",
            b.floor, b.radius, b.depth, b.wall, b.pestle_half * 2.0, b.pestle_radius, b.tip_radius, b.pestle_pos
        );
        // (`RUSTVR_BENCH_PREVIEW=<folder>`: each piece drawn from three sides, to look at.)
        if let Some(dir) = std::env::var_os("RUSTVR_BENCH_PREVIEW") {
            let stem = rel.rsplit('\\').next().unwrap_or("bench").trim_end_matches(".nif").to_owned();
            preview::save(&mortar, &std::path::Path::new(&dir).join(format!("{stem}_mortar.png")))?;
            preview::save(&pestle, &std::path::Path::new(&dir).join(format!("{stem}_pestle.png")))?;
        }
        benches.benches.retain(|x| x.model != b.model);
        benches.benches.push(b);
    }
    for path in &textures {
        let key = export::texture_key(path);
        if !out.join("textures").join(format!("{key}.{}", rustvr_asset::TEXTURE_EXT)).is_file()
            && let Ok(t) = archives.read(path).and_then(|d| oblivion_assets::dds::parse(&d, path))
        {
            rustvr_asset::save_texture(&out, &key, &t)?;
        }
    }
    benches.benches.sort_by(|a, b| a.model.cmp(&b.model));
    std::fs::write(out.join(bench::FILE), toml::to_string(&benches)?)?;
    index.items.sort_by(|a, b| a.key.cmp(&b.key));
    std::fs::write(&index_path, toml::to_string(&index)?)?;
    println!("{} benches in {}", benches.benches.len(), out.join(bench::FILE).display());
    Ok(())
}

/// The game's fonts and menu art under the prefixes, for the runtime to paint (`oblivion_assets::ui`):
/// each font with its atlas, each texture as RGBA.
fn export_ui(archives: &Archives, args: &[String]) -> Result<()> {
    let out = rustvr_asset::default_root().join("oblivion");
    let prefixes: Vec<String> = args.iter().map(|p| oblivion_assets::bsa::normalise(p)).collect();
    let names: Vec<String> = archives.names().filter(|n| prefixes.iter().any(|p| n.starts_with(p.as_str()))).map(str::to_owned).collect();
    let (mut fonts, mut images, mut failed) = (0, 0, 0);
    for path in names {
        let done = if path.ends_with(".fnt") {
            (|| -> Result<()> {
                let atlas_path = oblivion_assets::ui::atlas_path(&path);
                let atlas = oblivion_assets::ui::atlas(&archives.read(&atlas_path)?, &atlas_path)?;
                let atlas_key = export::texture_key(&atlas_path);
                rustvr_asset::save_texture(&out, &atlas_key, &atlas)?;
                let font = oblivion_assets::ui::font(&archives.read(&path)?, &path, &atlas_key)?;
                rustvr_asset::save_font(&out, &export::font_key(&path), &font)?;
                fonts += 1;
                Ok(())
            })()
        } else if path.ends_with(".dds") {
            (|| -> Result<()> {
                let t = oblivion_assets::dds::to_rgba(&oblivion_assets::dds::parse(&archives.read(&path)?, &path)?)?;
                rustvr_asset::save_texture(&out, &export::texture_key(&path), &t)?;
                images += 1;
                Ok(())
            })()
        } else {
            Ok(())
        };
        if let Err(e) = done {
            failed += 1;
            if failed <= 10 {
                eprintln!("{path}: {e:#}");
            }
        }
    }
    println!("{fonts} fonts, {images} pieces of art ({failed} failed) in {}", out.display());
    Ok(())
}

fn dump(nif: &Nif) {
    use oblivion_assets::nif::Block;
    let parents = nif.parents();
    for (i, b) in nif.blocks.iter().enumerate() {
        let depth = {
            let (mut d, mut p) = (0, parents[i]);
            while p >= 0 {
                d += 1;
                p = parents[p as usize];
            }
            d
        };
        let pad = "  ".repeat(depth);
        match b {
            Block::Node { av, children } => {
                let w = nif.world(i as i32, &parents);
                println!("{i:3} {pad}node '{}' at {:?} children {children:?} flags {:#x} collision {}", av.name, w.translation, av.flags, av.collision);
            }
            Block::Shape { av, data, skin } => {
                let w = nif.world(i as i32, &parents);
                let (n, bounds) = match nif.get(*data) {
                    Some(Block::ShapeData(d)) => {
                        let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
                        for v in &d.vertices {
                            let p = w.apply(*v);
                            lo = lo.min(p);
                            hi = hi.max(p);
                        }
                        (format!("{} verts {} tris", d.vertices.len(), d.triangles.len()), format!("{lo:?}..{hi:?}"))
                    }
                    _ => (String::new(), String::new()),
                };
                println!("{i:3} {pad}shape '{}' {n} skin {skin} props {:?} bounds {bounds}", av.name, av.properties);
            }
            Block::SourceTexture { file } => println!("{i:3} texture {file}"),
            Block::RigidBody(r) => println!("{i:3} rigid body shape {} mass {} centre {:?} layer {}", r.shape, r.mass, r.center, r.layer),
            Block::StringExtra { name, value } => println!("{i:3} string '{name}' = '{value}'"),
            Block::IntegerExtra { name, value } => println!("{i:3} int '{name}' = {value:#x}"),
            other => println!("{i:3} {}", format!("{other:?}").chars().take(120).collect::<String>()),
        }
    }
    if let Some(k) = &nif.stopped_at {
        println!("(stopped at {k})");
    }
}
