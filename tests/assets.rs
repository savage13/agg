
use std::env;
use std::path::PathBuf;
// https://www.reddit.com/r/rust/comments/ahsz9q/psa_if_the_examples_for_your_crate_rely_on_media/

pub fn is_base_dir(cwd: &PathBuf) -> Option<(PathBuf, PathBuf)> {
    if ! cwd.join("Cargo.toml").is_file() {
        return None;
    }
    let mut images = cwd.clone();
    let mut test_tmp = cwd.clone();
    images.push("images");
    test_tmp.push("tests");
    test_tmp.push("tmp");
    if images.is_dir() && test_tmp.is_dir() {
        Some((images, test_tmp))
    } else {
        None
    }
}

pub fn find_assets() -> Option<(PathBuf, PathBuf)> {
    // First check currnet directory
    let cwd = env::current_dir().ok()?;
    if let Some(v) = is_base_dir(&cwd) {
        return Some(v);
    }
    // Search backwards from current executable path
    let mut exec = env::current_exe().ok()?;
    while let Some(dir) = exec.parent() {
        if let Some(v) = is_base_dir(&dir.to_path_buf()) {
            return Some(v);
        }
        exec = dir.to_path_buf();
    }
    // Could not find base directory
    None
}
