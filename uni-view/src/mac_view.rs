extern crate raw_window_handle;

pub struct AppView {
    pub view: winit::window::Window,
    pub scale_factor: f32,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
}

impl AppView {
    pub fn new(view: winit::window::Window) -> Self {
        let scale_factor = view.hidpi_factor();
        let physical = view.inner_size().to_physical(scale_factor);
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // supported list: [Bgra8Unorm, Bgra8Srgb, Rgba16Sfloat]
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: physical.width as u32,
            height: physical.height as u32,
            present_mode: wgpu::PresentMode::Vsync,
        };

        let (device, queue) = request_device();
        let surface = wgpu::Surface::create(&view);
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        AppView { view, scale_factor: scale_factor as f32, device, queue, surface, sc_desc, swap_chain }
    }
}

fn request_device() -> (wgpu::Device, wgpu::Queue) {
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            // wgpu::PowerPreference::Lowpower 会获取到电脑上的集成显示
            power_preference: wgpu::PowerPreference::Default,
        },
        wgpu::BackendBit::PRIMARY,
    )
    .unwrap();
    adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions { anisotropic_filtering: false },
        limits: wgpu::Limits::default(),
    })
}

impl crate::GPUContext for AppView {
    fn update_swap_chain(&mut self) {
        let size = self.get_view_size();
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn get_view_size(&self) -> crate::ViewSize {
        let scale_factor = self.view.hidpi_factor();
        // let physical = size.to_physical(scale_factor);
        let physical = self.view.inner_size().to_physical(scale_factor);

        crate::ViewSize { width: physical.width as u32, height: physical.height as u32 }
    }

    fn normalize_touch_point(&self, touch_point_x: f32, touch_point_y: f32) -> (f32, f32) {
        let size = self.get_view_size();
        (touch_point_x * self.scale_factor / size.width as f32, touch_point_y * self.scale_factor / size.height as f32)
    }
}
