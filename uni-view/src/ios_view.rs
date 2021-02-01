use crate::ffi::strings::c_char_to_string;
use libc::c_void;
use std::marker::{Send, Sync};
use std::os::raw::c_char;

extern crate objc;
use self::objc::{
    rc::StrongPtr,
    runtime::{Class, Object},
    *,
};
extern crate core_graphics;
use self::core_graphics::{base::CGFloat, geometry::CGRect};

#[repr(C)]
pub struct AppViewObj {
    pub view: *mut Object,
    pub metal_layer: *mut c_void,
    pub maximum_frames: i32,
    pub temporary_directory: *const c_char,
    pub callback_to_swift: extern "C" fn(arg: i32),
}

pub struct AppView {
    pub view: *mut Object,
    pub scale_factor: f32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    //  一个像素在[-1, 1]缩放为满屏的设备空间中对应的量
    pub pixel_on_ndc_x: f32,
    pub pixel_on_ndc_y: f32,
    // 一个像素在标准的设备空间中对应的量
    pub pixel_on_normal_ndc: f32,
    pub maximum_frames: i32,
    pub callback_to_app: Option<extern "C" fn(arg: i32)>,
    pub temporary_directory: &'static str,
    pub library_directory: &'static str,
}

// 标记为可以在线程间安全的传递不可变引用
// 默认情况下， *mut Object 会导致在 async fn 中报错：
// within `uni_view::app_view::AppView`, the trait `std::marker::Sync` is not implemented for `*mut objc::runtime::Object`
unsafe impl Sync for AppView {}

impl AppView {
    pub fn new(obj: AppViewObj) -> Self {
        let scale_factor = get_scale_factor(obj.view);
        let s: CGRect = unsafe { msg_send![obj.view, frame] };
        let physical = crate::ViewSize {
            width: (s.size.width as f32 * scale_factor) as u32,
            height: (s.size.height as f32 * scale_factor) as u32,
        };
        
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface_from_core_animation_layer(obj.metal_layer) };

        let (device, queue) = futures::executor::block_on(request_device(&instance, &surface));
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            // iOS 上这个纹理格式肯定是可以使用的： wgpu::TextureFormat::Bgra8Unorm
            // 使用 get_swap_chain_preferred_format() 渲染的画面是类似过暴的
            // format: device.get_swap_chain_preferred_format(),
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width,
            height: physical.height,
            // 在移动端上，这个呈现模式最高效
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let pixel_on_ndc_x = 2.0 / physical.width as f32;
        let pixel_on_ndc_y = 2.0 / physical.height as f32;
        let pixel_on_normal_ndc =
            if physical.width < physical.height { 2.0 / physical.width as f32 } else { 2.0 / physical.height as f32 };
        // 这样传递过来的字符串为空
        let temporary_directory: &'static str = Box::leak(c_char_to_string(obj.temporary_directory).into_boxed_str());
        AppView {
            view: obj.view,
            scale_factor,
            device,
            queue,
            surface,
            sc_desc,
            swap_chain,
            pixel_on_ndc_x,
            pixel_on_ndc_y,
            pixel_on_normal_ndc,
            callback_to_app: Some(obj.callback_to_swift),
            maximum_frames: obj.maximum_frames,
            temporary_directory,
            library_directory: "",
        }
    }
}

impl crate::GPUContext for AppView {
    fn update_swap_chain(&mut self) {
        let size = self.get_view_size();
        println!("view_size: {:?}", size);
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.pixel_on_ndc_x = 2.0 / size.width as f32;
        self.pixel_on_ndc_y = 2.0 / size.height as f32;
    }

    fn get_view_size(&self) -> crate::ViewSize {
        let s: CGRect = unsafe { msg_send![self.view, frame] };
        crate::ViewSize {
            width: (s.size.width as f32 * self.scale_factor) as u32,
            height: (s.size.height as f32 * self.scale_factor) as u32,
        }
    }

    fn normalize_touch_point(&self, touch_point_x: f32, touch_point_y: f32) -> (f32, f32) {
        let size = self.get_view_size();
        (touch_point_x * self.scale_factor / size.width as f32, touch_point_y * self.scale_factor / size.height as f32)
    }
}

fn get_scale_factor(obj: *mut Object) -> f32 {
    let s: CGFloat = unsafe { msg_send![obj, contentScaleFactor] };
    s as f32
}

async fn request_device(instance: &wgpu::Instance, surface: &wgpu::Surface) -> (wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
        })
        .await
        .unwrap();

    let adapter_features = adapter.features();
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter_features,
                limits: wgpu::Limits {
                    max_bind_groups: 4,
                    max_dynamic_uniform_buffers_per_pipeline_layout: 8,
                    max_dynamic_storage_buffers_per_pipeline_layout: 28,
                    max_sampled_textures_per_shader_stage: 16,
                    max_samplers_per_shader_stage: 16,
                    max_storage_buffers_per_shader_stage: 28,
                    max_storage_textures_per_shader_stage: 8,
                    max_uniform_buffers_per_shader_stage: 12,
                    max_uniform_buffer_binding_size: 16384,
                    max_push_constant_size: 16,
                },
            },
            None,
        )
        .await
        .unwrap()
}
