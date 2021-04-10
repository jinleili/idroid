extern crate libc;
extern crate wgpu;
use std::path::PathBuf;
use std::ops::Deref;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
#[path = "mac_view.rs"]
mod app_view;

#[cfg(target_os = "ios")]
#[path = "ios_view.rs"]
mod app_view;

pub use app_view::*;
pub mod ffi;

#[repr(C)]
#[derive(Debug)]
pub struct ViewSize {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct TouchPoint {
    pub x: f32,
    pub y: f32,
    pub force: f32,
}

pub trait GPUContext {
    fn get_view_size(&self) -> ViewSize;
    fn update_swap_chain(&mut self);
    fn normalize_touch_point(&self, touch_point_x: f32, touch_point_y: f32) -> (f32, f32);
}

// 元组结构类型，默认的构造方法只能在当前模块访问，除非将元组参数添加 pub
pub struct AppViewWrapper(pub AppView);
// `*mut libc::c_void` cannot be sent between threads safely
// 强制 AppView 为线程安全的
unsafe impl Send for AppViewWrapper {}
unsafe impl Sync for AppViewWrapper {}

impl Deref for AppViewWrapper {
    type Target = AppView;

    fn deref(&self) -> &AppView {
        &self.0
    }
}


#[cfg(target_os = "ios")]
#[path = "ios_fs.rs"]
pub mod fs;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
#[path = "mac_fs.rs"]
pub mod fs;

pub fn get_texture_file_path(name: &str) -> PathBuf {
    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let f = fs::FileSystem::new(&&base_dir);
    f.get_texture_file_path(name)
}
