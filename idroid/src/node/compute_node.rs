use std::vec::Vec;

#[allow(dead_code)]
pub struct ComputeNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::ComputePipeline,
    pub threadgroup_count: (u32, u32),
}

#[allow(dead_code)]
impl ComputeNode {
    pub fn new(
        device: &mut wgpu::Device, threadgroup_count: (u32, u32), uniform_buffer: &wgpu::Buffer,
        buffer_range: wgpu::BufferAddress, inout_buffer: Vec<&wgpu::Buffer>,
        inout_buffer_range: Vec<wgpu::BufferAddress>, inout_tv: Vec<&wgpu::TextureView>,
        shader: (&str, &str),
    ) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutBinding> = vec![wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
        }];

        let mut bingdings: Vec<wgpu::Binding> = vec![wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: uniform_buffer,
                range: 0..buffer_range,
            },
        }];

        let mut b_index = 0_u32;

        for i in 0..inout_buffer.len() {
            b_index += 1;
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::StorageBuffer { dynamic: false, readonly: false },
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::Buffer {
                    buffer: inout_buffer[i],
                    range: 0..inout_buffer_range[i],
                },
            });
        }

        for i in 0..inout_tv.len() {
            b_index += 1;
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::TextureView(inout_tv[i]),
            });
        }
        let bind_group_layout = device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &layouts });
        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &bingdings,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let shader =
            crate::shader::Shader::new_by_compute(shader.0, device, shader.1);
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: shader.cs_stage(),
        });

        ComputeNode { bind_group_layout, bind_group, pipeline, threadgroup_count }
    }

    pub fn compute(&mut self, _device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);
    }
}
