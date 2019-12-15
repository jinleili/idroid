use crate::buffer::BufferObj;
use std::vec::Vec;

#[allow(dead_code)]
pub struct DynamicBindingGroupNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DynamicBindingGroupNode {
    pub fn new(device: &mut wgpu::Device, uniforms: Vec<&BufferObj>, visibilitys: Vec<wgpu::ShaderStage>) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutBinding> = vec![];

        let mut bingdings: Vec<wgpu::Binding> = vec![];

        let mut b_index = 0;
        for i in 0..uniforms.len() {
            let buffer_obj = uniforms[i];

            layouts.push(wgpu::BindGroupLayoutBinding {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::UniformBuffer { dynamic: true },
            });
            bingdings.push(wgpu::Binding {
                binding: b_index,
                resource: wgpu::BindingResource::Buffer { buffer: &buffer_obj.buffer, range: 0..buffer_obj.size },
            });
            b_index += 1;
        }
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &layouts });

        let bind_group: wgpu::BindGroup =
            device.create_bind_group(&wgpu::BindGroupDescriptor { layout: &bind_group_layout, bindings: &bingdings });

        DynamicBindingGroupNode { bind_group_layout, bind_group }
    }
}
