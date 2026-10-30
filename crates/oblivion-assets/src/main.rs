//! `oblivion-extract`: reads the user's own Oblivion install (read-only).
//!
//!     oblivion-extract [--game <Oblivion folder>] check <prefix>...
//!     oblivion-extract export [--out <folder>] <prefix>...
//!     oblivion-extract sounds [--out <folder>]
//!     oblivion-extract preview <model> <png>
//!     oblivion-extract anims <prefix>...
//!     oblivion-extract bench <prefix>...
//!     oblivion-extract export-ui <prefix>...
//!
//! `check` parses every model under the prefixes and reports which it can't read (a test of the
//! model reader against the real files). `export` writes every model under the prefixes, and the
//! textures they use, into the runtime's cache (`local_assets/oblivion`, see
//! `oblivion_assets::export`; a `textures/...` prefix brings every texture under it, e.g. the races'
//! skins). `sounds` writes the game's own sounds as the runtime's sound pack
//! (`local_assets/oblivion/sounds`, see `oblivion_assets::sounds`). `preview` draws one converted
//! model from three sides. `anims` lists each model's animations and what each moves (a door's leaf
//! turned about its hinge, a lever's handle, a gate slid up). `bench` splits each mortar and pestle
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
        Some("anims") => {
            // Each model's animations: what each moves, from where it starts to where it ends (a
            // door's leaf turned about its hinge, an activator's lever, a gate slid up), or to the
            // farthest it goes when it comes back (a button pushed in and out again).
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
                    let (t0, t1) = nif.sequence_span(s);
                    let poses: Vec<_> = (0..=32).map(|k| (t0 + (t1 - t0) * k as f32 / 32.0, nif.sequence_pose(s, t0 + (t1 - t0) * k as f32 / 32.0))).collect();
                    let moved: Vec<String> = (0..poses[0].1.len())
                        .filter_map(|n| {
                            let a = &poses[0].1[n].1;
                            let moved = |b: &oblivion_assets::nif::Transform| {
                                let turn = glam::Quat::from_mat3(&(b.rotation * a.rotation.transpose()));
                                let (axis, angle) = turn.to_axis_angle();
                                (angle.to_degrees(), axis, b.translation - a.translation)
                            };
                            let far = poses.iter().max_by(|x, y| {
                                let (ax, _, sx) = moved(&x.1[n].1);
                                let (ay, _, sy) = moved(&y.1[n].1);
                                (ax + sx.length()).total_cmp(&(ay + sy.length()))
                            })?;
                            let (angle, axis, slid) = moved(&poses.last()?.1[n].1);
                            let (far_angle, _, far_slid) = moved(&far.1[n].1);
                            let back = (far_angle > angle + 1.0 || far_slid.length() > slid.length() + 0.5).then(|| format!(" (farthest at {:.2}: {far_angle:.0} deg, slid {far_slid:.1?})", far.0)).unwrap_or_default();
                            (far_angle > 1.0 || far_slid.length() > 0.5).then(|| format!("'{}' {angle:.0} deg about {axis:.2?} slid {slid:.1?}{back}", poses[0].1[n].0))
                        })
                        .collect();
                    println!("  {} {:.2}..{:.2}: {}", s.name, s.start, s.stop, moved.join("; "));
                }
            }
        }
        Some("export") => export(&archives, &args[1..])?,
        Some("sounds") => {
            let out = match args.get(1).map(String::as_str) {
                Some("--out") if args.len() > 2 => PathBuf::from(&args[2]),
                _ => rustvr_asset::default_root().join("oblivion").join("sounds"),
            };
            let pack = oblivion_assets::sounds::write(&game, &archives, &out)?;
            println!("{} files and sounds.toml -> {}", pack.files.len(), out.display());
            for m in &pack.missing {
                println!("  the game hasn't got {m}");
            }
        }
        Some("bench") => bench(&archives, &args[1..])?,
        Some("export-ui") => export_ui(&archives, &args[1..])?,
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
        _ => println!("usage: oblivion-extract [--game <folder>] check <prefix>... | dump <model>... | export [--out <folder>] <prefix>... | sounds [--out <folder>] | preview <model> <png>"),
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
        // A weapon with its scabbard: the weapon alone, and the scabbard alone. Arrows: the whole
        // model, and the arrow alone (what the hand nocks, `#arrow`), with the quiver and its arrows
        // (what the body wears) below. The arrow alone has a key of its own because every other
        // extractor writes the plain key whole, and `local_assets` is shared.
        let parts: Vec<(String, String, Part)> = if nif.find("Scb").is_some() {
            vec![(export::model_key(&path), rel.clone(), Part::Without("Scb")), (export::variant_key(&path, "scb"), format!("{rel}#scb"), Part::Only("Scb"))]
        } else if nif.find("ArrowQuiver").is_some() {
            vec![(export::model_key(&path), rel.clone(), Part::All), (export::variant_key(&path, "arrow"), format!("{rel}#arrow"), Part::Without("ArrowQuiver"))]
        } else {
            vec![(export::model_key(&path), rel.clone(), Part::All)]
        };
        // A quiver, worn on the body where the game hangs it (its `Prn`, the skeleton's `Quiver`),
        // showing as many of its arrows as the game would (`#quiver0` .. `#quiver5`).
        if nif.find("ArrowQuiver").is_some() {
            let prn = nif.blocks.iter().find_map(|b| match b {
                oblivion_assets::nif::Block::StringExtra { name, value } if name == "Prn" => Some(value.clone()),
                _ => None,
            });
            const ARROWS: [&str; 5] = ["Arrow1", "Arrow2", "Arrow3", "Arrow4", "Arrow5"];
            for shown in 0..=ARROWS.len() {
                let mut c = export::convert(&nif, &path, None, Part::OnlyBut("ArrowQuiver", &ARROWS[shown..]), &exists);
                let on = prn.as_deref().unwrap_or("Quiver");
                if c.model.meshes.is_empty() || !export::worn_on(&mut c.model, &skeletons.human.1, on) {
                    continue;
                }
                textures.extend(c.textures);
                add(export::variant_key(&path, &format!("quiver{shown}")), ItemKind::Body, format!("{rel}#quiver{shown}"), &c.model)?;
                models += 1;
            }
        }
        // A model the game bends (a bow drawn): bent all the way too, the same meshes and vertices.
        let bent = nif.morphed(1.0);
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
            add(key, kind, name.clone(), &c.model)?;
            models += 1;
            if let Some(b) = &bent {
                let c = export::convert(b, &path, skeleton, part, &exists);
                add(export::variant_key(&path, "bent"), kind, format!("{name}#bent"), &c.model)?;
                models += 1;
            }
        }
    }
    // With bows: how their limbs spring back after a shot, as the player's bow animation has it
    // (`bows.toml`, for the adapter).
    if items.values().any(|i| i.name.ends_with("bow.nif#bent")) {
        const KF: &str = "meshes\\characters\\_1stperson\\bowattack.kf";
        match archives.read(KF).and_then(|d| Nif::parse(&d)).map(|kf| kf.bow_spring("BowMorph")) {
            Ok(Some(spring)) => {
                let keys: Vec<String> = spring.iter().map(|[t, w]| format!("[{t:.4}, {w:.4}]")).collect();
                let text = format!("# How a bow's limbs spring back after a shot: the player's bow animation ({KF}),\n# its BowMorph from the Hold key on: [seconds after the hold, bend as a share of full draw].\nspring = [{}]\n", keys.join(", "));
                std::fs::write(out.join("bows.toml"), text)?;
                println!("bows.toml: the limbs' spring, {} keys", spring.len());
            }
            Ok(None) => eprintln!("{KF}: no BowMorph after a Hold key"),
            Err(e) => eprintln!("{KF}: {e:#}"),
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
    // A horse's saddle, measured for the hands to find (`oblivion_assets::horse`).
    if prefixes.iter().any(|p| oblivion_assets::horse::SADDLE.starts_with(p.as_str())) {
        let read = |p: &str| archives.read(p).and_then(|d| Nif::parse(&d));
        let tack = archives.read(oblivion_assets::horse::TACK).and_then(|d| oblivion_assets::dds::parse(&d, oblivion_assets::horse::TACK)).ok();
        match (read(oblivion_assets::horse::SADDLE), read(oblivion_assets::horse::SKELETON), read(oblivion_assets::horse::BODY), read(oblivion_assets::horse::HEAD)) {
            (Ok(saddle), Ok(skeleton), Ok(body), Ok(head)) => match oblivion_assets::horse::measure(&saddle, &skeleton, &body, &head, &oblivion_assets::horse::MANES.iter().filter_map(|m| read(m).ok()).collect::<Vec<_>>(), tack.as_ref()) {
                Some(s) => {
                    oblivion_assets::horse::save(&out, &s)?;
                    println!("the horse's saddle on {}: pommel {:?}, seat {:?}, cantle {:?}, a rider's knee {:?} and ankle {:?}, the bit {:?} on {}, the tack's colour {:?}", s.bone, s.pommel, s.seat, s.cantle, s.knee, s.ankle, s.bit, s.head_bone, s.tack);
                }
                None => eprintln!("the horse's saddle: not measured"),
            },
            (Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => eprintln!("the horse's saddle: {e:#}"),
        }
    }
    index.items = items.into_values().collect();
    std::fs::write(&index_path, toml::to_string(&index)?)?;
    println!("{models} models ({failed} unreadable, {faces} with face morphs), {wrote} new textures ({missing} missing) in {}", out.display());
    Ok(())
}

/// A rigid model the game hangs on the head bone: a part of FaceGen's head (with an `.egm`: the
/// ears, eyes, mouth, teeth, tongue, hair), or one naming the head as its parent (`Prn`: the hair,
/// helmets, hoods, circlets).
fn head_part(archives: &Archives, nif: &Nif, path: &str) -> bool {
    let skinned = nif.blocks.iter().any(|b| matches!(b, oblivion_assets::nif::Block::Shape { skin, .. } if *skin >= 0));
    let on_head = nif.blocks.iter().any(|b| matches!(b, oblivion_assets::nif::Block::StringExtra { name, value } if name == "Prn" && value.eq_ignore_ascii_case("Bip01 Head")));
    let egm = format!("{}.egm", path.strip_suffix(".nif").unwrap_or(path));
    !skinned && (on_head || (export::normalise(path, "meshes\\").starts_with("characters\\") && archives.contains(&egm)))
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
            Block::RigidBody(r) => println!("{i:3} rigid body{} shape {} mass {} centre {:?} translation {:?} layer {} motion {}", if r.transformed { " (T)" } else { "" }, r.shape, r.mass, r.center, r.translation, r.layer, r.motion),
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
            Block::MorphData { morphs } => {
                for (name, offsets) in morphs {
                    let moved = offsets.iter().filter(|o| o.length() > 1e-3).count();
                    let most = offsets.iter().map(|o| o.length()).fold(0.0f32, f32::max);
                    println!("{i:3} morph '{name}': {moved} of {} vertices move, at most {most:.2}", offsets.len());
                }
            }
            other => println!("{i:3} {}", format!("{other:?}").chars().take(120).collect::<String>()),
        }
    }
    if let Some(k) = &nif.stopped_at {
        println!("(stopped at {k})");
    }
}
