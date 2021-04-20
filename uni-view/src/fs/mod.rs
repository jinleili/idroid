use std::path::PathBuf;
use std::env;
use std::fs;

#[cfg(target_os = "ios")]
#[path = "ios_fs.rs"]
mod file_sys;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
#[path = "mac_fs.rs"]
mod file_sys;

pub use file_sys::FileSystem;

pub fn get_texture_file_path(name: &str) -> PathBuf {
    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let f = FileSystem::new(&&base_dir);
    f.get_texture_file_path(name)
}


// Returns the cargo manifest directory when running the executable with cargo
// or the directory in which the executable resides otherwise, 
// traversing symlinks if necessary.
pub fn application_root_dir() -> String {
    match env::var("PROFILE") {
        Ok(_) => String::from(env!("CARGO_MANIFEST_DIR")),
        Err(_) => {
            let mut path = env::current_exe().expect("Failed to find executable path.");
            while let Ok(target) = fs::read_link(path.clone()) {
                path = target;
            }
            String::from(path.parent().expect("Failed to get parent directory of the executable.").to_str().unwrap())
        }
    }
}