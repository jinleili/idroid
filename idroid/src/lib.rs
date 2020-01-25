extern crate libc;
pub use uni_view::*;

pub mod geometry;
pub mod math;
pub mod texture;
pub mod utils;
pub use utils::{depth_stencil, matrix_helper};

mod buffer;
pub use buffer::BufferObj;

mod mvp_uniform_obj;
pub use mvp_uniform_obj::{MVPUniform, MVPUniformObj};
// mod dynamic_buffer;
// pub use dynamic_buffer::DynamicBufferObj;

pub mod node;
pub mod shader;
pub mod vertex;

use math::TouchPoint;

pub trait SurfaceView {
    fn resize(&mut self);
    fn pintch_start(&mut self, location: TouchPoint, scale: f32);
    fn pintch_changed(&mut self, location: TouchPoint, scale: f32);
    fn touch_start(&mut self, point: TouchPoint);
    fn touch_moved(&mut self, point: TouchPoint);
    fn touch_end(&mut self, point: TouchPoint);

    fn enter_frame(&mut self);
}

#[cfg(not(target_os = "macos"))]
pub fn box_obj(obj: impl SurfaceView) -> *mut libc::c_void {
    let boxed_trait: Box<dyn SurfaceView> = Box::new(obj);
    let boxed_boxed_trait = Box::new(boxed_trait);
    let heap_pointer = Box::into_raw(boxed_boxed_trait);
    // let boxed_boxed_trait = Box::new(v);
    // let heap_pointer = Box::into_raw(boxed_boxed_trait);
    heap_pointer as *mut libc::c_void
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn enter_frame(obj: *mut libc::c_void) -> *mut libc::c_void {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.enter_frame();

    // 重新将所有权移出
    Box::into_raw(obj) as *mut libc::c_void
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn touch_move(obj: *mut libc::c_void, p: TouchPoint) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.touch_moved(p);
    // 重新将所有权移出
    let _ = Box::into_raw(obj) as *mut libc::c_void;
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn touch_start(obj: *mut libc::c_void, p: TouchPoint) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.touch_start(p);
    let _ = Box::into_raw(obj) as *mut libc::c_void;
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn touch_end(obj: *mut libc::c_void, p: TouchPoint) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.touch_end(p);
    let _ = Box::into_raw(obj) as *mut libc::c_void;
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn resize(obj: *mut libc::c_void, _p: TouchPoint) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.resize();
    let _ = Box::into_raw(obj) as *mut libc::c_void;
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn pintch_start(obj: *mut libc::c_void, location: TouchPoint, scale: f32) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.pintch_start(location, scale);
    let _ = Box::into_raw(obj) as *mut libc::c_void;
}

#[cfg(not(target_os = "macos"))]
#[no_mangle]
pub unsafe extern "C" fn pintch_changed(obj: *mut libc::c_void, location: TouchPoint, scale: f32) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.pintch_changed(location, scale);
    let _ = Box::into_raw(obj) as *mut libc::c_void;
}
