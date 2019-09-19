use std::vec::Vec;

#[allow(dead_code)]
pub struct BindingGroupSettingNode {
    bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub bind_group: wgpu::BindGroup,
}

#[allow(dead_code)]
impl BindingGroupSettingNode {
    pub fn new(
        device: &mut wgpu::Device, uniform_buffer: &wgpu::Buffer,
        buffer_range: wgpu::BufferAddress, inout_buffer: Vec<&wgpu::Buffer>,
        inout_buffer_range: Vec<wgpu::BufferAddress>, textures: Vec<&wgpu::TextureView>,
        samplers: Vec<&wgpu::Sampler>, visibilitys: Vec<wgpu::ShaderStage>,
    ) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutBinding> = vec![wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: visibilitys[0],
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
                visibility: visibilitys[b_index as usize],
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

        for i in 0..textures.len() {
            b_index += 1;
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                },
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::TextureView(textures[i]),
            });
        }

        for i in 0..samplers.len() {
            b_index += 1;
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::Sampler,
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::Sampler(samplers[i]),
            });
        }
        println!("{:?}", bingdings);

        let bind_group_layout = device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &layouts });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &bingdings,
        });

        BindingGroupSettingNode { bind_group_layout, bind_group, pipeline_layout }
    }
}
