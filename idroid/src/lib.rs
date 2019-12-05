extern crate libc;
pub use uni_view::*;

pub mod geometry;
pub mod math;
pub mod texture;
pub mod utils;
pub use utils::{depth_stencil, matrix_helper};

pub mod node;
pub mod shader;
pub mod vertex;
pub mod buffer;

use math::Position;

pub trait SurfaceView {
    fn resize(&mut self);
    fn scale(&mut self, scale: f32);
    fn touch_moved(&mut self, position: Position);

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
    obj.touch_moved(crate::math::Position::new(p.x, p.y));

    // 重新将所有权移出
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
pub unsafe extern "C" fn scale(obj: *mut libc::c_void, scale: f32) {
    let mut obj: Box<Box<dyn SurfaceView>> = Box::from_raw(obj as *mut _);
    obj.scale(scale);

    let _ = Box::into_raw(obj) as *mut libc::c_void;
}
