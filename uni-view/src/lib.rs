extern crate libc;
extern crate wgpu;
use std::ops::Deref;

#[cfg(not(target_os = "ios"))]
#[path = "app_view.rs"]
mod app_view;

#[cfg(target_os = "ios")]
#[path = "ios_view.rs"]
mod app_view;

pub use app_view::*;
pub mod ffi;
pub mod fs;
pub use fs::application_root_dir;

#[cfg(target_arch = "wasm32")]
#[macro_use]
pub use ffi::web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use std::println as console_log;

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
    fn set_view_size(&mut self, _size: (f64, f64)) {}
    fn get_view_size(&self) -> ViewSize;
    fn resize_surface(&mut self);
    fn normalize_touch_point(&self, touch_point_x: f32, touch_point_y: f32) -> (f32, f32);
    fn get_current_frame_view(&self) -> (wgpu::SurfaceFrame, wgpu::TextureView);
    fn create_current_frame_view(
        &self, device: &wgpu::Device, surface: &wgpu::Surface, config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::SurfaceFrame, wgpu::TextureView) {
        let frame = match surface.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                surface.configure(&device, &config);
                surface.get_current_frame().expect("Failed to acquire next surface texture!")
            }
        };
        let view = frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        // frame cannot be drop early
        (frame, view)
    }
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
