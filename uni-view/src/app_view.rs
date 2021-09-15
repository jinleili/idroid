extern crate raw_window_handle;

pub struct AppView {
    pub view: winit::window::Window,
    pub scale_factor: f32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    //  一个像素在[-1, 1]缩放为满屏的设备空间中对应的量
    pub pixel_on_ndc_x: f32,
    pub pixel_on_ndc_y: f32,
    // 一个像素在标准的设备空间中对应的量
    pub pixel_on_normal_ndc: f32,
    pub callback_to_app: Option<extern "C" fn(arg: i32)>,
    pub maximum_frames: i32,
    pub temporary_directory: &'static str,
    pub library_directory: &'static str,
}

impl AppView {
    pub async fn new(view: winit::window::Window, native_only: bool) -> Self {
        let scale_factor = view.scale_factor();
        let backend = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY);
        let instance = wgpu::Instance::new(backend);
        let (physical, surface) = unsafe { (view.inner_size(), instance.create_surface(&view)) };

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

        let all_features = adapter.features();
        let required_features = wgpu::Features::empty();
        let optional_features = wgpu::Features::TEXTURE_COMPRESSION_BC
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2
            | wgpu::Features::TEXTURE_COMPRESSION_ASTC_LDR;
        // let adapter_features = wgpu::Features::DEPTH_CLAMPING;

        // 使用 Xcode 调试时，配置 trace_path 会 crash (2021/4/12)
        // let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        // let trace_path = std::path::PathBuf::from(&base_dir).join("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: (optional_features & all_features) | required_features,
                    limits: wgpu::Limits {
                        // Error in Adapter::request_device: Limit 'max_dynamic_storage_buffers_per_pipeline_layout' value 16 is better than allowed 4
                        max_dynamic_storage_buffers_per_pipeline_layout: 4,
                        max_storage_buffers_per_shader_stage: 8,
                        max_storage_textures_per_shader_stage: 8,
                        max_push_constant_size: 16,
                        ..Default::default()
                    },
                },
                // Some(&trace_path),
                None,
            )
            .await
            .expect("Unable to find a suitable GPU device!");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // supported list: [Bgra8Unorm, Bgra8Srgb, Rgba16Sfloat]
            // format: surface.get_preferred_format(&adapter).unwrap(),
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width as u32,
            height: physical.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &config);

        let pixel_on_ndc_x = 2.0 / physical.width as f32;
        let pixel_on_ndc_y = 2.0 / physical.height as f32;
        let pixel_on_normal_ndc =
            if physical.width < physical.height { 2.0 / physical.width as f32 } else { 2.0 / physical.height as f32 };
        AppView {
            view,
            scale_factor: scale_factor as f32,
            device,
            queue,
            surface,
            config,
            pixel_on_ndc_x,
            pixel_on_ndc_y,
            pixel_on_normal_ndc,
            maximum_frames: 60,
            callback_to_app: None,
            temporary_directory: "",
            library_directory: "",
        }
    }
}

impl crate::GPUContext for AppView {
    fn resize_surface(&mut self) {
        let size = self.get_view_size();
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.pixel_on_ndc_x = 2.0 / size.width as f32;
        self.pixel_on_ndc_y = 2.0 / size.height as f32;
    }

    fn set_view_size(&mut self, size: (f64, f64)) {
        let inner_size = winit::dpi::Size::Logical(winit::dpi::LogicalSize { width: size.0, height: size.1 });
        self.view.set_inner_size(inner_size);
    }

    fn get_view_size(&self) -> crate::ViewSize {
        // let scale_factor = self.view.hidpi_factor();
        // let physical = size.to_physical(scale_factor);
        let physical = self.view.inner_size();
        crate::ViewSize { width: physical.width as u32, height: physical.height as u32 }
    }

    fn normalize_touch_point(&self, touch_point_x: f32, touch_point_y: f32) -> (f32, f32) {
        let size = self.get_view_size();
        (touch_point_x * self.scale_factor / size.width as f32, touch_point_y * self.scale_factor / size.height as f32)
    }

    fn get_current_frame_view(&self) -> (wgpu::SurfaceFrame, wgpu::TextureView) {
        self.create_current_frame_view(&self.device, &self.surface, &self.config)
    }
}
