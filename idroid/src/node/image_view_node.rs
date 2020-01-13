use crate::buffer::{BufferObj, MVPUniform};
use crate::geometry::plane::Plane;
use crate::math::TouchPoint;
use crate::node::BindingGroupSettingNode;
use crate::vertex::{Pos, PosTex};
use nalgebra_glm as glm;

use zerocopy::AsBytes;

#[allow(dead_code)]
pub struct ImageViewNode {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,
    view_width: f32,
    view_height: f32,
    // 实现绽放与拖拽
    scale: f32,
    pintch_start_location: Option<(f32, f32)>,
    p_matrix: glm::TMat4<f32>,
    base_mv_matrix: glm::TMat4<f32>,
    mvp_buf: BufferObj,
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
        shader: (&str, &str),
    ) -> Self {
        let (p_matrix, base_mv_matrix) =
            crate::utils::matrix_helper::perspective_mvp(sc_desc, true);
        let mvp_buf = BufferObj::create_uniform_buffer(
            device,
            encoder,
            &MVPUniform {
                mvp_matrix: (p_matrix * base_mv_matrix).into(),
            },
        );

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
            vec![&mvp_buf],
            inout_buffers,
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
            view_width: sc_desc.width as f32,
            view_height: sc_desc.height as f32,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            setting_node,
            pipeline,
            scale: 1.0,
            p_matrix,
            base_mv_matrix,
            mvp_buf,
            pintch_start_location: None,
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

    pub fn pintch_start(&mut self, location: (f32, f32), scale: f32) {
        // 缩放并拖拽始终是以 start 为中心的
        // 可以计算出 start 相对中心点的偏移坐标，无论如何缩放，其偏移坐标是不变的;
        // change 时，直接计算 changed 相对中心点的偏移，缩放完成后，再执行些偏移就能得到正确的位置
        self.pintch_start_location = Some(location);
    }
    // 缩放并拖拽：
    // 先将缩放质心移动到视图中心，执行缩放
    // 再将质心移到到实际位置
    // scale 小于 0 时，只按中心缩放
    pub fn pintch_changed(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &mut wgpu::Device,
        location: (f32, f32),
        scale: f32,
    ) {
        if let Some(start_location) = self.pintch_start_location {
            let mut vm_matrix = self.base_mv_matrix;
            self.scale *= scale;
            if self.scale < 0.7 {
                self.scale = 0.7;
                vm_matrix = glm::scale(&vm_matrix, &glm::vec3(self.scale, self.scale, 1.0));
            } else {
                let (offset_x, offset_y, target_x, target_y) = if self.scale < 1.0 {
                    println!("scale 0: {}, {}", self.scale, scale);
                    (0.0, 0.0, 0.0, 0.0)
                } else {
                    println!("scale 1: {}, {}", self.scale, scale);

                    (
                        (0.5 - start_location.0) * 2.0,
                        (0.5 - start_location.1) * 2.0,
                        location.0 - start_location.0,
                        location.1 - start_location.1,
                    )
                };
                // 以 pintch start 为中心点缩放
                vm_matrix = glm::translate(&vm_matrix, &glm::vec3(-offset_x, -offset_y, 0.0));
                vm_matrix = glm::scale(&vm_matrix, &glm::vec3(self.scale, self.scale, 1.0));
                // 平移到 pintch changed 质心
                vm_matrix = glm::translate(
                    &vm_matrix,
                    &glm::vec3(offset_x + target_x, offset_y + target_y, 0.0),
                );
            }
            self.mvp_buf.update_buffer(
                encoder,
                device,
                &MVPUniform {
                    mvp_matrix: (self.p_matrix * vm_matrix).into(),
                },
            );
        }
    }
}
