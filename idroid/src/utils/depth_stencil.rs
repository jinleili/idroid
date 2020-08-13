#[allow(dead_code)]

// 获取 depth_stencil 状态描述符
pub fn create_state_descriptor() -> wgpu::DepthStencilStateDescriptor {
    wgpu::DepthStencilStateDescriptor {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilStateDescriptor::default(),
    }
}

#[allow(dead_code)]
pub fn create_depth_texture_view(size: wgpu::Extent3d, device: &wgpu::Device) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        label: Some("depth buffer"),
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

#[allow(dead_code)]
// 创建 render_pass 的 depth_stencil_attachment 描述符
pub fn create_attachment_descriptor<'a>(
    depth_textue_view: &'a wgpu::TextureView,
) -> wgpu::RenderPassDepthStencilAttachmentDescriptor<'a> {
    // VK_ATTACHMENT_STORE_OP_DONT_CARE should be used in case the application is not expecting to read the data rendered to the attachment
    // this is commonly the case for depth buffers and MSAA targets.
    wgpu::RenderPassDepthStencilAttachmentDescriptor {
        attachment: depth_textue_view,
        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: false }),
        stencil_ops: None,
    }
}
