use std::path::PathBuf;

pub struct FileSystem {
    base_path: &'static str,
}

impl FileSystem {
    pub fn new(base_path: &'static str) -> Self {
        FileSystem { base_path }
    }

    pub fn get_bundle_url() -> &'static str {
        env!("CARGO_MANIFEST_DIR")
    }

    pub fn get_texture_file_path(&self, name: &str) -> PathBuf {
        PathBuf::from(self.base_path).join("assets").join(name)
    }
}
