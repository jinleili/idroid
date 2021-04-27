use crate::node::BindingGroupSettingNode;
use wgpu::{PrimitiveTopology, ShaderModule, ShaderStage, StorageTextureAccess, TextureFormat};

#[allow(dead_code)]
pub struct BufferlessFullscreenNode {
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,
}

impl BufferlessFullscreenNode {
    pub fn new(
        app_view: &uni_view::AppView, textures: Vec<(&wgpu::TextureView, TextureFormat, Option<StorageTextureAccess>)>,
        samplers: Vec<&wgpu::Sampler>, shader_module: &ShaderModule,
    ) -> Self {
        let mut stages: Vec<ShaderStage> = vec![];
        for _ in 0..(textures.len() + samplers.len()) {
            stages.push(ShaderStage::FRAGMENT);
        }
        let setting_node = BindingGroupSettingNode::new(&app_view.device, vec![], vec![], textures, samplers, stages);
        let pipeline_layout = app_view.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&setting_node.bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline_vertex_buffers = [];
        let pipeline = app_view.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bufferless fullscreen pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: shader_module, entry_point: "main", buffers: &pipeline_vertex_buffers },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: app_view.sc_desc.format,
                    blend: Some(crate::utils::default_blend()),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            // the bufferless vertices are in clock-wise order
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        Self { setting_node, pipeline }
    }

    pub fn draw(
        &self, frame_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, load_op: wgpu::LoadOp<wgpu::Color>,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations { load: load_op, store: true },
            }],
            depth_stencil_attachment: None,
        });
        self.draw_rpass(&mut rpass);
    }

    pub fn draw_rpass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}
