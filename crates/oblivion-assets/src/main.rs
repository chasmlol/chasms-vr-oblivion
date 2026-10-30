//! `oblivion-extract`: reads the user's own Oblivion install (read-only).
//!
//!     oblivion-extract [--game <Oblivion folder>] check <prefix>...
//!     oblivion-extract export [--out <folder>] <prefix>...
//!     oblivion-extract preview <model> <png>
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
        Some("export") => export(&archives, &args[1..])?,
        Some("preview") if args.len() == 3 => {
            // (Several models joined by `+` are drawn together: a head with its eyes and hair.)
            let skeletons = Skeletons::load(&archives)?;
            let mut model = rustvr_asset::Model::default();
            for path in args[1].split('+') {
                let path = oblivion_assets::bsa::normalise(path);
                let nif = Nif::parse(&archives.read(&path)?)?;
                let exists = |p: &str| archives.contains(p);
                let c = if head_part(&archives, &nif, &path) {
                    export::convert_attached(&nif, &path, &skeletons.human.1, "Bip01 Head", &exists)
                } else {
                    export::convert(&nif, &path, skeletons.for_model(&nif), Part::All, &exists)
                };
                // (Joined, the models alone, drawn as big as they are: a head and its parts.)
                if !args[1].contains('+') {
                    model.bones = c.model.bones.clone();
                }
                model.meshes.extend(c.model.meshes);
            }
            preview::save(&model, std::path::Path::new(&args[2]))?;
            println!("{}: {} meshes, {} bones -> {}", args[1], model.meshes.len(), model.bones.len(), args[2]);
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
    let (mut models, mut failed, mut faces) = (0, 0, 0);
    // FaceGen's age control, which picks a face's age map (the adapter's).
    match archives.read("facegen\\si.ctl").and_then(|d| oblivion_assets::facegen::parse_age(&d)) {
        Ok((age, offset)) => std::fs::write(out.join("facegen.toml"), toml::to_string(&FaceGenFile { age, age_offset: offset })?)?,
        Err(e) => eprintln!("facegen\\si.ctl: {e:#}"),
    }
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
        let stem = path.strip_suffix(".nif").unwrap_or(&path).to_owned();
        let (egm, egt) = (format!("{stem}.egm"), format!("{stem}.egt"));
        let head_part = head_part(archives, &nif, &path);
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
            let c = if head_part { export::convert_attached(&nif, &path, &skeletons.human.1, "Bip01 Head", &exists) } else { export::convert(&nif, &path, skeleton, part, &exists) };
            if c.model.meshes.is_empty() {
                continue;
            }
            // Its face morphs, in the cache's space.
            if archives.contains(&egm) {
                match face_morphs(archives, &c, &egm, &egt) {
                    Ok(m) => {
                        rustvr_asset::save_morphs(&out, &key, &m)?;
                        faces += 1;
                    }
                    Err(e) => eprintln!("{path}: its face morphs: {e:#}"),
                }
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
    println!("{models} models ({failed} unreadable, {faces} with face morphs), {wrote} new textures ({missing} missing) in {}", out.display());
    Ok(())
}

/// A rigid part of the head the game hangs on the head bone: FaceGen's (with an `.egm`: the ears,
/// eyes, mouth, teeth, tongue, hair) or one naming the head as its parent (`Prn`).
fn head_part(archives: &Archives, nif: &Nif, path: &str) -> bool {
    let skinned = nif.blocks.iter().any(|b| matches!(b, oblivion_assets::nif::Block::Shape { skin, .. } if *skin >= 0));
    let on_head = nif.blocks.iter().any(|b| matches!(b, oblivion_assets::nif::Block::StringExtra { name, value } if name == "Prn" && value.eq_ignore_ascii_case("Bip01 Head")));
    let egm = format!("{}.egm", path.strip_suffix(".nif").unwrap_or(path));
    export::normalise(path, "meshes\\").starts_with("characters\\") && !skinned && (on_head || archives.contains(&egm))
}

/// FaceGen's controls the adapter needs (`facegen.toml`): the age, from the symmetric geometry
/// coordinates (`age = Σ age[i]·gs[i] + age_offset`).
#[derive(serde::Serialize)]
struct FaceGenFile {
    age: Vec<f32>,
    age_offset: f32,
}

/// A head part's FaceGen morphs (its `.egm`, and the `.egt` when it has one) on its converted
/// model's mesh: each move taken into the cache's space by that vertex's own map.
fn face_morphs(archives: &Archives, c: &export::Converted, egm: &str, egt: &str) -> Result<rustvr_asset::FaceMorphs> {
    use glam::Vec3;
    let e = oblivion_assets::facegen::parse_egm(&archives.read(egm)?)?;
    let (mesh, maps) = (&c.model.meshes[0], &c.linear[0]);
    anyhow::ensure!(e.vertices >= mesh.vertices.len(), "{} vertices for a mesh of {}", e.vertices, mesh.vertices.len());
    let take = |morphs: &[(f32, Vec<[i16; 3]>)]| -> Vec<Vec<[f32; 3]>> {
        morphs
            .iter()
            .map(|(scale, d)| maps.iter().zip(d).map(|(m, v)| (*m * (Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32) * *scale)).to_array()).collect())
            .collect()
    };
    let texture = if archives.contains(egt) {
        let t = oblivion_assets::facegen::parse_egt(&archives.read(egt)?)?;
        Some(rustvr_asset::TextureMorphs {
            width: t.cols as u32,
            height: t.rows as u32,
            symmetric: t.symmetric.into_iter().map(|(scale, delta)| rustvr_asset::TextureMorph { scale, delta: oblivion_assets::facegen::top_first(&delta, t.rows, t.cols) }).collect(),
        })
    } else {
        None
    };
    Ok(rustvr_asset::FaceMorphs {
        version: rustvr_asset::FORMAT_VERSION,
        source: egm.to_owned(),
        mesh: 0,
        symmetric: take(&e.symmetric),
        asymmetric: take(&e.asymmetric),
        texture,
    })
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
                println!("{i:3} {pad}node '{}' at {:?} rot {:?} children {children:?} flags {:#x} collision {}", av.name, w.translation, w.rotation.to_cols_array().map(|x| (x * 1000.0).round() / 1000.0), av.flags, av.collision);
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
