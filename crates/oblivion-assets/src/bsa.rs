//! Oblivion's archives (`.bsa`, version 103): folders of files, each file stored as is or zlib
//! compressed (archive flag 4 makes compressed the default; bit 30 of a file's size flips it).

use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Bsa {
    pub path: PathBuf,
    file: Mutex<File>,
    compressed: bool,
    /// Lower-case `folder\name` -> (size field, offset).
    files: BTreeMap<String, (u32, u32)>,
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

impl Bsa {
    pub fn open(path: &Path) -> Result<Bsa> {
        let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let mut h = [0u8; 36];
        f.read_exact(&mut h)?;
        if &h[0..4] != b"BSA\0" || u32_at(&h, 4) != 103 {
            bail!("{}: not an Oblivion BSA", path.display());
        }
        let (flags, folders, file_names_len) = (u32_at(&h, 12), u32_at(&h, 16) as usize, u32_at(&h, 28) as usize);
        let mut folder_records = vec![0u8; folders * 16];
        f.read_exact(&mut folder_records)?;
        let mut records = Vec::new();
        for i in 0..folders {
            let count = u32_at(&folder_records, i * 16 + 8) as usize;
            let mut len = [0u8; 1];
            f.read_exact(&mut len)?;
            let mut name = vec![0u8; len[0] as usize];
            f.read_exact(&mut name)?;
            let folder = String::from_utf8_lossy(&name[..name.len().saturating_sub(1)]).to_lowercase();
            let mut recs = vec![0u8; count * 16];
            f.read_exact(&mut recs)?;
            for j in 0..count {
                records.push((folder.clone(), u32_at(&recs, j * 16 + 8), u32_at(&recs, j * 16 + 12)));
            }
        }
        let mut names = vec![0u8; file_names_len];
        f.read_exact(&mut names)?;
        let mut files = BTreeMap::new();
        for ((folder, size, offset), name) in records.into_iter().zip(names.split(|&c| c == 0)) {
            files.insert(format!("{folder}\\{}", String::from_utf8_lossy(name).to_lowercase()), (size, offset));
        }
        Ok(Bsa { path: path.to_path_buf(), file: Mutex::new(f), compressed: flags & 4 != 0, files })
    }

    /// Every file's path (lower case, backslashes).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(|k| k.as_str())
    }

    pub fn contains(&self, path: &str) -> bool {
        self.files.contains_key(&normalise(path))
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let key = normalise(path);
        let &(size, offset) = self.files.get(&key).with_context(|| format!("{key} isn't in {}", self.path.display()))?;
        let compressed = self.compressed ^ (size & 0x4000_0000 != 0);
        let size = (size & 0x3FFF_FFFF) as usize;
        let mut data = vec![0u8; size];
        {
            let mut f = self.file.lock().unwrap();
            f.seek(SeekFrom::Start(offset as u64))?;
            f.read_exact(&mut data)?;
        }
        if !compressed {
            return Ok(data);
        }
        let original = u32_at(&data, 0) as usize;
        let mut out = Vec::with_capacity(original);
        flate2::read::ZlibDecoder::new(&data[4..]).read_to_end(&mut out).with_context(|| format!("inflate {key}"))?;
        Ok(out)
    }
}

pub fn normalise(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

/// All of a game's archives, searched in order (loose files in `Data` first).
pub struct Archives {
    pub data: PathBuf,
    pub bsas: Vec<Bsa>,
}

impl Archives {
    /// Every `.bsa` in the game's `Data` folder.
    pub fn open(game: &Path) -> Result<Archives> {
        let data = game.join("Data");
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&data)?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("bsa"))).collect();
        paths.sort();
        let bsas = paths.iter().map(|p| Bsa::open(p)).collect::<Result<Vec<_>>>()?;
        Ok(Archives { data, bsas })
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let key = normalise(path);
        let loose = self.data.join(key.replace('\\', "/"));
        if loose.is_file() {
            return Ok(std::fs::read(loose)?);
        }
        for b in &self.bsas {
            if b.contains(&key) {
                return b.read(&key);
            }
        }
        bail!("{key}: not in Data or any archive")
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.bsas.iter().flat_map(|b| b.names())
    }

    /// Whether the game has the file (loose or in an archive).
    pub fn contains(&self, path: &str) -> bool {
        let key = normalise(path);
        self.data.join(key.replace('\\', "/")).is_file() || self.bsas.iter().any(|b| b.contains(&key))
    }
}
