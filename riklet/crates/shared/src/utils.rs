use flate2::read::GzDecoder;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tar::Archive;

/// Find a binary in the host PATH
pub fn find_binary(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(binary);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

/// Generate a hash
pub fn generate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Unpack a .tar.gz archive into the provided location
pub fn unpack(archive: &str, dest: &Path) -> std::io::Result<()> {
    let tar_gz =
        File::open(archive).expect(&format!("Unable to unzip the archive {}", archive)[..]);
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dest).unwrap();
    Ok(())
}

pub fn create_file_with_parent_folders(path: &Path) -> std::io::Result<File> {
    let parent = path.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;

    log::debug!("File {} created.", path.display());

    Ok(file)
}

pub fn create_directory_if_not_exists(dir: &Option<PathBuf>) -> std::io::Result<()> {
    if let Some(dir) = dir {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }

        log::debug!("Directory {} created.", dir.display());
    }

    Ok(())
}

pub fn get_random_hash(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}
