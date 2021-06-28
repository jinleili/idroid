extern crate raw_window_handle;

pub struct AppView {
    pub view: winit::window::Window,
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
    pub callback_to_app: Option<extern "C" fn(arg: i32)>,
    pub maximum_frames: i32,
    pub temporary_directory: &'static str,
    pub library_directory: &'static str,
}

impl AppView {
    pub async fn new(view: winit::window::Window) -> Self {
        let scale_factor = view.scale_factor();
        let backend = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
            match backend.to_lowercase().as_str() {
                "vulkan" => wgpu::BackendBit::VULKAN,
                "metal" => wgpu::BackendBit::METAL,
                "dx12" => wgpu::BackendBit::DX12,
                "dx11" => wgpu::BackendBit::DX11,
                "gl" => wgpu::BackendBit::GL,
                "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
                other => panic!("Unknown backend: {}", other),
            }
        } else {
            wgpu::BackendBit::PRIMARY
        };
        let instance = wgpu::Instance::new(backend);
        let (physical, surface) = unsafe { (view.inner_size(), instance.create_surface(&view)) };

        let power_preference = if let Ok(power_preference) = std::env::var("WGPU_POWER_PREF") {
            match power_preference.to_lowercase().as_str() {
                "low" => wgpu::PowerPreference::LowPower,
                "high" => wgpu::PowerPreference::HighPerformance,
                other => panic!("Unknown power preference: {}", other),
            }
        } else {
            wgpu::PowerPreference::default()
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { power_preference, compatible_surface: Some(&surface) })
            .await
            .expect("No suitable GPU adapters found on the system!");
        let adapter_features = adapter.features();

        // 使用 Xcode 调试时，配置 trace_path 会 crash (2021/4/12)
        // let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        // let trace_path = PathBuf::from(&base_dir).join("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: adapter_features,
                    limits: wgpu::Limits {
                        max_dynamic_storage_buffers_per_pipeline_layout: 16,
                        max_storage_buffers_per_shader_stage: 16,
                        max_storage_textures_per_shader_stage: 8,
                        max_push_constant_size: 16,
                        ..Default::default()
                    },
                },
                // Some(&trace_path)
                None,
            )
            .await
            .expect("Unable to find a suitable GPU device!");

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            // supported list: [Bgra8Unorm, Bgra8Srgb, Rgba16Sfloat]
            // format: device.get_swap_chain_preferred_format(),
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width as u32,
            height: physical.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

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
            sc_desc,
            swap_chain,
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
    fn update_swap_chain(&mut self) {
        let size = self.get_view_size();
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
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
}
