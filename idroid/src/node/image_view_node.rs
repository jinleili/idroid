use crate::geometry::plane::Plane;
use crate::math::{Position, Rect, Size};
use crate::node::BindingGroupSettingNode;
use crate::shader::Shader;
use crate::vertex::{Pos, PosTex, PosTex2};
use crate::{BufferObj, MVPUniform, MVPUniformObj};
use nalgebra_glm as glm;

use std::ops::{Deref, DerefMut};
use zerocopy::AsBytes;

pub struct NodeAttributes<'a> {
    pub view_size: Size<f32>,
    pub uniform_buffers: Vec<&'a BufferObj>,
    pub storage_buffers: Vec<&'a BufferObj>,
    pub tex_views: Vec<(&'a wgpu::TextureView, bool)>,
    pub samplers: Vec<&'a wgpu::Sampler>,
    pub tex_rect: Option<crate::math::Rect>,
    pub corlor_format: Option<wgpu::TextureFormat>,
    pub shader: &'a Shader,
    pub shader_stages: Vec<wgpu::ShaderStage>,
}

pub struct ImageNodeBuilder<'a> {
    pub attributes: NodeAttributes<'a>,
}

impl<'a> Deref for ImageNodeBuilder<'a> {
    type Target = NodeAttributes<'a>;
    fn deref(&self) -> &NodeAttributes<'a> {
        &self.attributes
    }
}

impl<'a> DerefMut for ImageNodeBuilder<'a> {
    fn deref_mut(&mut self) -> &mut NodeAttributes<'a> {
        &mut self.attributes
    }
}

impl<'a> ImageNodeBuilder<'a> {
    pub fn new(tex_views: Vec<(&'a wgpu::TextureView, bool)>, shader: &'a Shader) -> Self {
        ImageNodeBuilder {
            attributes: NodeAttributes {
                view_size: (0.0, 0.0).into(),
                uniform_buffers: vec![],
                storage_buffers: vec![],
                tex_views,
                samplers: vec![],
                tex_rect: None,
                corlor_format: None,
                shader,
                shader_stages: vec![],
            },
        }
    }

    pub fn with_view_size(mut self, size: Size<f32>) -> Self {
        self.view_size = size;
        self
    }

    pub fn with_uniform_buffers(mut self, buffers: Vec<&'a BufferObj>) -> Self {
        self.uniform_buffers = buffers;
        self
    }

    pub fn with_storage_buffers(mut self, buffers: Vec<&'a BufferObj>) -> Self {
        self.storage_buffers = buffers;
        self
    }

    pub fn with_tex_views(mut self, views: Vec<(&'a wgpu::TextureView, bool)>) -> Self {
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

    pub fn with_shader_states(mut self, states: Vec<wgpu::ShaderStage>) -> Self {
        self.shader_stages = states;
        self
    }

    pub fn build<T: Pos>(self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> ImageViewNode {
        ImageViewNode::frome_attributes::<T>(self.attributes, device, encoder)
    }
}

#[allow(dead_code)]
pub struct ImageViewNode {
    pub vertex_buf: BufferObj,
    index_buf: wgpu::Buffer,
    index_count: usize,
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,
    view_width: f32,
    view_height: f32,
    pub clear_color: wgpu::Color,
}

#[allow(dead_code)]
impl ImageViewNode {
    fn frome_attributes<T: Pos>(
        attributes: NodeAttributes, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let corlor_format =
            if let Some(format) = attributes.corlor_format { format } else { wgpu::TextureFormat::Bgra8Unorm };

        let stages: Vec<wgpu::ShaderStage> = if attributes.shader_stages.len() > 0 {
            attributes.shader_stages
        } else {
            let mut stages: Vec<wgpu::ShaderStage> = vec![wgpu::ShaderStage::VERTEX];
            for _ in 0..(attributes.uniform_buffers.len()
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
        let setting_node = BindingGroupSettingNode::new(
            device,
            attributes.uniform_buffers,
            attributes.storage_buffers,
            attributes.tex_views,
            new_samplers,
            stages,
        );

        // Create the vertex and index buffers
        let factor = crate::utils::matrix_helper::fullscreen_factor(attributes.view_size);
        let rect = Rect::new(2.0 * factor.1, 2.0 * factor.2, Position::zero());
        let plane = Plane::new_by_rect(rect, 1, 1);
        let (vertex_buf, index_data) = if let Some(rect) = attributes.tex_rect {
            let (vertex_data, index_data) = plane.generate_vertices_by_texcoord2(rect, None);
            let vertex_buf = BufferObj::create_buffer(
                device,
                encoder,
                Some(&vertex_data.as_bytes()),
                None,
                wgpu::BufferUsage::VERTEX,
            );
            (vertex_buf, index_data)
        } else {
            let (vertex_data, index_data) = plane.generate_vertices();
            let vertex_buf = BufferObj::create_buffer(
                device,
                encoder,
                Some(&vertex_data.as_bytes()),
                None,
                wgpu::BufferUsage::VERTEX,
            );
            (vertex_buf, index_data)
        };
        let index_buf = device.create_buffer_with_data(&index_data.as_bytes(), wgpu::BufferUsage::INDEX);

        // let attri_descriptor1 = PosTex2::attri_descriptor(0);
        // let attri_descriptor0 = PosTex::attri_descriptor(0);
        // let pipeline_vertex_buffers = if let Some(_) = attributes.tex_rect {
        //     [wgpu::VertexBufferDescriptor {
        //         stride: std::mem::size_of::<PosTex2>() as wgpu::BufferAddress,
        //         step_mode: wgpu::InputStepMode::Vertex,
        //         attributes: &attri_descriptor1,
        //     }]
        // } else {
        //     [wgpu::VertexBufferDescriptor {
        //         stride: std::mem::size_of::<PosTex>() as wgpu::BufferAddress,
        //         step_mode: wgpu::InputStepMode::Vertex,
        //         attributes: &attri_descriptor0,
        //     }]
        // };
        let attri_descriptor = T::attri_descriptor(0);
        let pipeline_vertex_buffers = [wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &attri_descriptor,
        }];

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&setting_node.bind_group_layout],
        });
        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: attributes.shader.vertex_stage(),
            fragment_stage: attributes.shader.fragment_stage(),
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
                format: corlor_format,
                color_blend: crate::utils::color_blend(),
                alpha_blend: crate::utils::alpha_blend(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            // ??????
            depth_stencil_state: None,
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

    pub fn begin_render_pass(&self, frame_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: frame_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: self.clear_color,
            }],
            depth_stencil_attachment: None,
        });
        self.draw_render_pass(&mut rpass);
    }

    pub fn draw_render_pass<'a, 'b: 'a>(&'b self, rpass: &mut wgpu::RenderPass<'b>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        rpass.set_index_buffer(self.index_buf.slice(..));
        rpass.set_vertex_buffer(0, self.vertex_buf.buffer.slice(..));
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
