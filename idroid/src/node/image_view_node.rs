use crate::buffer::{BufferObj, MVPUniform};
use crate::geometry::plane::Plane;
use crate::node::BindingGroupSettingNode;
use crate::vertex::{Pos, PosTex};

use zerocopy::AsBytes;

#[allow(dead_code)]
pub struct ImageViewNode {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,
}

#[allow(dead_code)]
impl ImageViewNode {
    pub fn new(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        src_views: Vec<(&wgpu::TextureView, bool)>,
        samplers: Vec<&wgpu::Sampler>,
        mvp: MVPUniform,
        shader: (&str, &str),
    ) -> Self {
        let mvp_buf = BufferObj::create_uniform_buffer(device, encoder, &mvp);

        let mut stages: Vec<wgpu::ShaderStage> = vec![wgpu::ShaderStage::VERTEX];
        for _ in 0..src_views.len() {
            stages.push(wgpu::ShaderStage::FRAGMENT)
        }
        let sampler = crate::texture::default_sampler(device);
        let new_samplers: Vec<&wgpu::Sampler> = if samplers.len() > 0 {
            for _ in 0..samplers.len() {
                stages.push(wgpu::ShaderStage::FRAGMENT);
            }
            samplers
        } else {
            stages.push(wgpu::ShaderStage::FRAGMENT);
            vec![&sampler]
        };
        let setting_node = BindingGroupSettingNode::new(
            device,
            vec![&mvp_buf],
            vec![],
            src_views,
            new_samplers,
            stages,
        );

        // Create the vertex and index buffers
        let (vertex_data, index_data) = Plane::new(1, 1).generate_vertices();
        let vertex_buf =
            device.create_buffer_with_data(&vertex_data.as_bytes(), wgpu::BufferUsage::VERTEX);
        let index_buf =
            device.create_buffer_with_data(&index_data.as_bytes(), wgpu::BufferUsage::INDEX);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&setting_node.bind_group_layout],
        });
        // Create the render pipeline
        let shader = crate::shader::Shader::new(shader.0, device, shader.1);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: shader.vertex_stage(),
            fragment_stage: shader.fragment_stage(),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            // primitive_topology: wgpu::PrimitiveTopology::LineList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: crate::utils::color_blend(),
                alpha_blend: crate::utils::alpha_blend(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            // ??????
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<PosTex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &PosTex::attri_descriptor(0),
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        ImageViewNode {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            setting_node,
            pipeline,
        }
    }

    pub fn begin_render_pass(
        &self,
        frame_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: frame_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: crate::utils::clear_color(),
            }],
            depth_stencil_attachment: None,
        });
        self.draw_render_pass(&mut rpass);
    }

    pub fn draw_render_pass(&self, rpass: &mut wgpu::RenderPass) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        rpass.set_index_buffer(&self.index_buf, 0);
        rpass.set_vertex_buffers(0, &[(&self.vertex_buf, 0)]);
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
