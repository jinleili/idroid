use crate::geometry::plane::Plane;
use crate::math::{Position, Rect, Size};
use crate::node::BindingGroupSettingNode;
use crate::shader::Shader;
use crate::vertex::Pos;
use crate::{BufferObj, MVPUniform};
use wgpu::util::DeviceExt;

use std::ops::{Deref, DerefMut};
use zerocopy::AsBytes;

pub struct NodeAttributes<'a, T: Pos> {
    pub view_size: Size<f32>,
    pub vertices_and_indices: Option<(Vec<T>, Vec<u32>)>,
    pub uniform_buffers: Vec<&'a BufferObj>,
    pub storage_buffers: Vec<&'a BufferObj>,
    pub tex_views: Vec<(&'a wgpu::TextureView, bool)>,
    pub samplers: Vec<&'a wgpu::Sampler>,
    // 动态 uniform
    pub dynamic_uniforms: Vec<(&'a BufferObj, wgpu::ShaderStage)>,

    pub tex_rect: Option<crate::math::Rect>,
    pub corlor_format: Option<wgpu::TextureFormat>,
    pub use_depth_stencil: bool,
    pub shader: &'a Shader,
    pub shader_stages: Vec<wgpu::ShaderStage>,
}

pub struct ImageNodeBuilder<'a, T: Pos + AsBytes> {
    pub attributes: NodeAttributes<'a, T>,
}

impl<'a, T: Pos + AsBytes> Deref for ImageNodeBuilder<'a, T> {
    type Target = NodeAttributes<'a, T>;
    fn deref(&self) -> &NodeAttributes<'a, T> {
        &self.attributes
    }
}

impl<'a, T: Pos + AsBytes> DerefMut for ImageNodeBuilder<'a, T> {
    fn deref_mut(&mut self) -> &mut NodeAttributes<'a, T> {
        &mut self.attributes
    }
}

impl<'a, T: Pos + AsBytes> ImageNodeBuilder<'a, T> {
    pub fn new(tex_views: Vec<(&'a wgpu::TextureView, bool)>, shader: &'a Shader) -> Self {
        ImageNodeBuilder {
            attributes: NodeAttributes {
                view_size: (0.0, 0.0).into(),
                vertices_and_indices: None,
                uniform_buffers: vec![],
                storage_buffers: vec![],
                tex_views,
                samplers: vec![],
                dynamic_uniforms: vec![],
                tex_rect: None,
                corlor_format: None,
                use_depth_stencil: false,
                shader,
                shader_stages: vec![],
            },
        }
    }

    pub fn with_vertices_and_indices(mut self, vertices_and_indices: (Vec<T>, Vec<u32>)) -> Self {
        self.vertices_and_indices = Some(vertices_and_indices);
        self
    }

    pub fn with_view_size(mut self, size: Size<f32>) -> Self {
        self.view_size = size;
        self
    }

    pub fn with_uniform_buffers(mut self, buffers: Vec<&'a BufferObj>) -> Self {
        self.uniform_buffers = buffers;
        self
    }

    pub fn with_dynamic_uniforms(mut self, uniforms: Vec<(&'a BufferObj, wgpu::ShaderStage)>) -> Self {
        self.dynamic_uniforms = uniforms;
        self
    }

    pub fn with_storage_buffers(mut self, buffers: Vec<&'a BufferObj>) -> Self {
        self.storage_buffers = buffers;
        self
    }

    pub fn with_tex_views_and_samplers(mut self, views: Vec<(&'a wgpu::TextureView, bool)>) -> Self {
        self.tex_views = views;
        self
    }

    pub fn with_samplers(mut self, samplers: Vec<&'a wgpu::Sampler>) -> Self {
        self.samplers = samplers;
        self
    }

    pub fn with_tex_rect(mut self, rect: Rect) -> Self {
        self.tex_rect = Some(rect);
        self
    }

    pub fn with_color_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.corlor_format = Some(format);
        self
    }

    pub fn with_use_depth_stencil(mut self, bl: bool) -> Self {
        self.use_depth_stencil = bl;
        self
    }

    pub fn with_shader_states(mut self, states: Vec<wgpu::ShaderStage>) -> Self {
        self.shader_stages = states;
        self
    }

    pub fn build(self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> ImageViewNode {
        ImageViewNode::frome_attributes::<T>(self.attributes, device, encoder)
    }
}

#[allow(dead_code)]
pub struct ImageViewNode {
    pub vertex_buf: BufferObj,
    pub index_buf: wgpu::Buffer,
    index_count: usize,
    setting_node: BindingGroupSettingNode,
    dynamic_node: Option<super::DynamicBindingGroupNode>,
    pipeline: wgpu::RenderPipeline,
    view_width: f32,
    view_height: f32,
    pub clear_color: wgpu::Color,
}

#[allow(dead_code)]
impl ImageViewNode {
    fn frome_attributes<T: Pos + AsBytes>(
        attributes: NodeAttributes<T>, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let corlor_format =
            if let Some(format) = attributes.corlor_format { format } else { wgpu::TextureFormat::Bgra8Unorm };

        let stages: Vec<wgpu::ShaderStage> = if attributes.shader_stages.len() > 0 {
            attributes.shader_stages
        } else {
            let mut stages: Vec<wgpu::ShaderStage> = vec![wgpu::ShaderStage::VERTEX];
            let uniform_buffers_len =
                if attributes.uniform_buffers.len() > 0 { attributes.uniform_buffers.len() } else { 1 };
            for _ in 0..(uniform_buffers_len
                + attributes.storage_buffers.len()
                + attributes.tex_views.len()
                + attributes.samplers.len())
            {
                stages.push(wgpu::ShaderStage::FRAGMENT);
            }
            stages
        };

        let sampler = crate::texture::default_sampler(device);
        let new_samplers: Vec<&wgpu::Sampler> = if attributes.tex_views.len() > 0 {
            if attributes.samplers.len() > 0 {
                attributes.samplers
            } else {
                vec![&sampler]
            }
        } else {
            vec![]
        };
        // 如果没有设置 mvp, 且设置了 view_size, 则设置一个全屏的 mvp
        let mut mvp_buf = BufferObj::create_uniform_buffer(device, &MVPUniform::zero());
        let uniform_buffers = if attributes.uniform_buffers.len() == 0 && attributes.view_size.width > 0.0 {
            let (p_matrix, vm_matrix, _factor) = crate::matrix_helper::perspective_mvp(attributes.view_size);
            let mvp = MVPUniform { mvp_matrix: (p_matrix * vm_matrix).into() };
            mvp_buf = BufferObj::create_uniform_buffer(device, &mvp);
            vec![&mvp_buf]
        } else {
            attributes.uniform_buffers
        };
        let setting_node = BindingGroupSettingNode::new(
            device,
            uniform_buffers,
            attributes.storage_buffers,
            attributes.tex_views,
            new_samplers,
            stages,
        );

        // Create the vertex and index buffers
        let (vertex_buf, index_data) = if let Some(vi) = attributes.vertices_and_indices {
            let vertex_buf = BufferObj::create_buffer(
                device,
                Some(&vi.0.as_bytes()),
                None,
                wgpu::BufferUsage::VERTEX,
                Some("vertex_buf"),
            );
            (vertex_buf, vi.1)
        } else {
            let factor = crate::utils::matrix_helper::fullscreen_factor(attributes.view_size);
            let rect = Rect::new(2.0 * factor.1, 2.0 * factor.2, Position::zero());
            let plane = Plane::new_by_rect(rect, 1, 1);
            let vi: (BufferObj, Vec<u32>) = if let Some(rect) = attributes.tex_rect {
                let (vertex_data, index_data) = plane.generate_vertices_by_texcoord2(rect, None);
                let vertex_buf = BufferObj::create_buffer(
                    device,
                    Some(&vertex_data.as_bytes()),
                    None,
                    wgpu::BufferUsage::VERTEX,
                    Some("vertex_buf"),
                );

                (vertex_buf, index_data)
            } else {
                let (vertex_data, index_data) = plane.generate_vertices();
                let vertex_buf = BufferObj::create_buffer(
                    device,
                    Some(&vertex_data.as_bytes()),
                    None,
                    wgpu::BufferUsage::VERTEX,
                    Some("vertex_buf"),
                );
                (vertex_buf, index_data)
            };
            vi
        };
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &index_data.as_bytes(),
            usage: wgpu::BufferUsage::INDEX,
        });

        let attri_descriptor = T::attri_descriptor(0);
        let pipeline_vertex_buffers = [wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &attri_descriptor,
        }];
        let (dynamic_node, pipeline_layout) = if attributes.dynamic_uniforms.len() > 0 {
            let node = super::DynamicBindingGroupNode::new(device, attributes.dynamic_uniforms);
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&setting_node.bind_group_layout, &node.bind_group_layout],
                push_constant_ranges: &[],
            });
            (Some(node), pipeline_layout)
        } else {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&setting_node.bind_group_layout],
                push_constant_ranges: &[],
            });
            (None, pipeline_layout)
        };
        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image_view pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: attributes.shader.vertex_stage(),
            fragment_stage: attributes.shader.fragment_stage(),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            // primitive_topology: wgpu::PrimitiveTopology::LineList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: corlor_format,
                color_blend: crate::utils::color_blend(),
                alpha_blend: crate::utils::alpha_blend(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            // ??????
            depth_stencil_state: if attributes.use_depth_stencil {
                Some(crate::depth_stencil::create_state_descriptor())
            } else {
                None
            },
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &pipeline_vertex_buffers,
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        ImageViewNode {
            view_width: attributes.view_size.width,
            view_height: attributes.view_size.height,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            setting_node,
            dynamic_node,
            pipeline,
            clear_color: crate::utils::alpha_color(),
        }
    }

    // 视口的宽高发生变化
    pub fn resize(
        &mut self, _sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        tex_rect: Option<crate::math::Rect>,
    ) {
        if let Some(rect) = tex_rect {
            let (vertex_data, _) = Plane::new(1, 1).generate_vertices_by_texcoord(rect);
            self.vertex_buf.update_buffers(encoder, device, &vertex_data);
        } else {
            let (vertex_data, _) = Plane::new(1, 1).generate_vertices();
            self.vertex_buf.update_buffers(encoder, device, &vertex_data);
        };
    }

    pub fn begin_render_pass(
        &self, frame_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, load_op: wgpu::LoadOp<wgpu::Color>,
    ) {
        self.begin_rpass_by_offset(frame_view, encoder, load_op, 0);
    }

    pub fn draw_render_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        self.draw_rpass_by_offset(rpass, 0, 1);
    }

    pub fn draw_by_instance_count<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>, instance_count: u32) {
        self.draw_rpass_by_offset(rpass, 0, instance_count);
    }

    pub fn begin_rpass_by_offset(
        &self, frame_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, load_op: wgpu::LoadOp<wgpu::Color>,
        offset_index: u32,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: frame_view,
                resolve_target: None,
                ops: wgpu::Operations { load: load_op, store: true },
            }],
            depth_stencil_attachment: None,
        });
        self.draw_rpass_by_offset(&mut rpass, offset_index, 1);
    }

    pub fn draw_rpass_by_offset<'a, 'b: 'a>(
        &'b self, rpass: &mut wgpu::RenderPass<'b>, offset_index: u32, instance_count: u32,
    ) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        rpass.set_index_buffer(self.index_buf.slice(..));
        rpass.set_vertex_buffer(0, self.vertex_buf.buffer.slice(..));
        if let Some(node) = &self.dynamic_node {
            rpass.set_bind_group(1, &node.bind_group, &[256 * offset_index as wgpu::DynamicOffset]);
        }
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..instance_count);
    }
}
