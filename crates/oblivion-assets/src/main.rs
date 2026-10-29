//! `oblivion-extract`: reads the user's own Oblivion install (read-only).
//!
//!     oblivion-extract [--game <Oblivion folder>] check <prefix>...
//!     oblivion-extract export [--out <folder>] <prefix>...
//!     oblivion-extract preview <model> <png>
//!     oblivion-extract anims <prefix>...
//!
//! `check` parses every model under the prefixes and reports which it can't read (a test of the
//! model reader against the real files). `export` writes every model under the prefixes, and the
//! textures they use, into the runtime's cache (`local_assets/oblivion`, see
//! `oblivion_assets::export`; a `textures/...` prefix brings every texture under it, e.g. the races'
//! skins). `preview` draws one converted model from three sides.

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
        Some("anims") => {
            // Each model's animations: what each moves, from where it starts to where it ends (a
            // door's leaf turned about its hinge, an activator's lever, a gate slid up).
            let prefixes: Vec<String> = args[1..].iter().map(|p| oblivion_assets::bsa::normalise(p)).collect();
            let names: Vec<String> = archives.names().filter(|n| n.ends_with(".nif") && prefixes.iter().any(|p| n.starts_with(p.as_str()))).map(str::to_owned).collect();
            for name in names {
                let Ok(nif) = archives.read(&name).and_then(|d| Nif::parse(&d)) else { continue };
                let seqs = nif.sequences();
                if seqs.is_empty() {
                    continue;
                }
                println!("{name}{}", nif.stopped_at.as_ref().map(|k| format!(" (stopped at {k})")).unwrap_or_default());
                for s in seqs {
                    let moved: Vec<String> = nif
                        .sequence_ends(s)
                        .into_iter()
                        .filter_map(|(node, a, b)| {
                            let turn = glam::Quat::from_mat3(&(b.rotation * a.rotation.transpose()));
                            let (axis, angle) = turn.to_axis_angle();
                            let slid = b.translation - a.translation;
                            (angle.to_degrees() > 1.0 || slid.length() > 0.5).then(|| format!("'{node}' {:.0} deg about {axis:.2?} slid {slid:.1?}", angle.to_degrees()))
                        })
                        .collect();
                    println!("  {} {:.2}..{:.2}: {}", s.name, s.start, s.stop, moved.join("; "));
                }
            }
        }
        Some("export") => export(&archives, &args[1..])?,
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
            Block::RigidBody(r) => println!("{i:3} rigid body shape {} mass {} centre {:?} layer {} motion {}", r.shape, r.mass, r.center, r.layer, r.motion),
            Block::Sequence(s) => {
                println!("{i:3} sequence '{}' {}..{} text keys {:?}", s.name, s.start, s.stop, nif.get(s.text_keys));
                for (name, a, b) in nif.sequence_ends(s) {
                    let turn = glam::Quat::from_mat3(&(b.rotation * a.rotation.transpose()));
                    let (axis, angle) = turn.to_axis_angle();
                    println!("      '{name}': {:?} -> {:?}, turned {:.1} deg about {axis:.3?}", a.translation, b.translation, angle.to_degrees());
                }
            }
            Block::Keyframes(k) => println!("{i:3} keys: {} rotations, xyz {:?}, {} translations, span {:?}", k.rotations.len(), k.xyz.iter().map(Vec::len).collect::<Vec<_>>(), k.translations.len(), k.span()),
            Block::StringExtra { name, value } => println!("{i:3} string '{name}' = '{value}'"),
            Block::IntegerExtra { name, value } => println!("{i:3} int '{name}' = {value:#x}"),
            other => println!("{i:3} {}", format!("{other:?}").chars().take(120).collect::<String>()),
        }
    }
    if let Some(k) = &nif.stopped_at {
        println!("(stopped at {k})");
    }
}
