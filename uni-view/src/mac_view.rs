extern crate raw_window_handle;

pub struct AppView {
    pub view: winit::window::Window,
    pub scale_factor: f32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    //  一个像素在标准设备坐标中对应的量
    pub pixel_on_ndc_x: f32,
    pub pixel_on_ndc_y: f32,
    pub callback_to_app: Option<extern "C" fn(arg: i32)>,
    pub maximum_frames: i32,
    pub temporary_directory: &'static str,
    pub library_directory: &'static str,
}

impl AppView {
    pub fn new(view: winit::window::Window) -> Self {
        let scale_factor = view.scale_factor();
        // let physical = view.inner_size().to_physical(scale_factor);
        let physical = view.inner_size();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // supported list: [Bgra8Unorm, Bgra8Srgb, Rgba16Sfloat]
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width as u32,
            height: physical.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let instance = wgpu::Instance::new();
        let surface = unsafe { instance.create_surface(&view) };
        let (device, queue) = futures::executor::block_on(request_device(&instance, &surface));
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let pixel_on_ndc_x = 2.0 / physical.width as f32;
        let pixel_on_ndc_y = 2.0 / physical.height as f32;

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
            maximum_frames: 60,
            callback_to_app: None,
            temporary_directory: "",
            library_directory: "",
        }
    }
}

async fn request_device(instance: &wgpu::Instance, surface: &wgpu::Surface) -> (wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(
            &wgpu::RequestAdapterOptions {
                // wgpu::PowerPreference::Lowpower 会获取到电脑上的集成显示
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();
    adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions { anisotropic_filtering: false },
            limits: wgpu::Limits::default(),
        }, None)
        .await
        .unwrap()
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
