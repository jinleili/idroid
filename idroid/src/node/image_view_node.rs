use crate::geometry::plane::Plane;
use crate::math::TouchPoint;
use crate::node::BindingGroupSettingNode;
use crate::vertex::{Pos, PosTex};
use crate::{BufferObj, MVPUniform, MVPUniformObj};
use nalgebra_glm as glm;

use zerocopy::AsBytes;

#[allow(dead_code)]
pub struct ImageViewNode {
    vertex_buf: BufferObj,
    index_buf: wgpu::Buffer,
    index_count: usize,
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,
    view_width: f32,
    view_height: f32,
}

#[allow(dead_code)]
impl ImageViewNode {
    pub fn new(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        inout_buffers: Vec<&BufferObj>,
        src_views: Vec<(&wgpu::TextureView, bool)>,
        samplers: Vec<&wgpu::Sampler>,
        mvp_obj: &MVPUniformObj,
        shader: (&str, &str),
        tex_rect: Option<crate::math::Rect>,
    ) -> Self {
        let mut stages: Vec<wgpu::ShaderStage> = vec![wgpu::ShaderStage::VERTEX];
        for _ in 0..(inout_buffers.len() + src_views.len() + samplers.len()) {
            stages.push(wgpu::ShaderStage::FRAGMENT)
        }
        let sampler = crate::texture::default_sampler(device);
        let new_samplers: Vec<&wgpu::Sampler> = if src_views.len() > 0 {
            if samplers.len() > 0 {
                samplers
            } else {
                stages.push(wgpu::ShaderStage::FRAGMENT);
                vec![&sampler]
            }
        } else {
            vec![]
        };
        let setting_node = BindingGroupSettingNode::new(
            device,
            vec![&mvp_obj.buffer],
            inout_buffers,
            src_views,
            new_samplers,
            stages,
        );

        // Create the vertex and index buffers
        let (vertex_data, index_data) = if let Some(rect) = tex_rect {
            Plane::new(1, 1).generate_vertices_by_texcoord(rect)
        } else {
            Plane::new(1, 1).generate_vertices()
        };
        let vertex_buf = BufferObj::create_buffer(
            device,
            encoder,
            Some(&vertex_data.as_bytes()),
            None,
            wgpu::BufferUsage::VERTEX,
        );
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
            view_width: sc_desc.width as f32,
            view_height: sc_desc.height as f32,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            setting_node,
            pipeline,
        }
    }

    // 视口的宽高发生变化
    pub fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        tex_rect: Option<crate::math::Rect>,
    ) {
        let (vertex_data, _) = if let Some(rect) = tex_rect {
            Plane::new(1, 1).generate_vertices_by_texcoord(rect)
        } else {
            Plane::new(1, 1).generate_vertices()
        };
        self.vertex_buf.update_buffers(encoder, device, &vertex_data);
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

    pub fn draw_render_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        rpass.set_index_buffer(&self.index_buf, 0);
        rpass.set_vertex_buffers(0, &[(&self.vertex_buf.buffer, 0)]);
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
