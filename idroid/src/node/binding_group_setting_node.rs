use crate::buffer::BufferObj;
use std::vec::Vec;

#[allow(dead_code)]
pub struct BindingGroupSettingNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

#[allow(dead_code)]
impl BindingGroupSettingNode {
    pub fn new(
        device: &mut wgpu::Device, uniforms: Vec<&BufferObj>, inout_buffers: Vec<&BufferObj>,
        textures: Vec<(&wgpu::TextureView, bool)>, samplers: Vec<&wgpu::Sampler>, visibilitys: Vec<wgpu::ShaderStage>,
    ) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutBinding> = vec![];

        let mut bingdings: Vec<wgpu::Binding> = vec![];

        let mut b_index = 0_u32;
        for i in 0..uniforms.len() {
            let buffer_obj = uniforms[i];
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::Buffer { buffer: &buffer_obj.buffer, range: 0..buffer_obj.size },
            });
            b_index += 1;
        }

        for i in 0..inout_buffers.len() {
            let buffer_obj = inout_buffers[i];
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::StorageBuffer { dynamic: false, readonly: false },
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::Buffer { buffer: &buffer_obj.buffer, range: 0..buffer_obj.size },
            });
            b_index += 1;
        }

        for i in 0..textures.len() {
            println!("tex id: {}", b_index);
            let is_storage_texture = textures[i].1;
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: if is_storage_texture {
                    wgpu::BindingType::StorageTexture { dimension: wgpu::TextureViewDimension::D2 }
                } else {
                    wgpu::BindingType::SampledTexture { multisampled: false, dimension: wgpu::TextureViewDimension::D2 }
                },
            });
            bingdings
                .push(wgpu::Binding { binding: b_index, resource: wgpu::BindingResource::TextureView(textures[i].0) });
            b_index += 1;
        }

        for i in 0..samplers.len() {
            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::Sampler,
            });
            bingdings.push(wgpu::Binding { binding: b_index, resource: wgpu::BindingResource::Sampler(samplers[i]) });
            b_index += 1;
        }

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &layouts });

        let bind_group: wgpu::BindGroup =
            device.create_bind_group(&wgpu::BindGroupDescriptor { layout: &bind_group_layout, bindings: &bingdings });

        BindingGroupSettingNode { bind_group_layout, bind_group }
    }
}
