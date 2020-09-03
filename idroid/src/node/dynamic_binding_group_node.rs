use crate::buffer::BufferObj;
use std::vec::Vec;

#[allow(dead_code)]
pub struct DynamicBindingGroupNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl DynamicBindingGroupNode {
    pub fn new(device: &wgpu::Device, uniforms: Vec<(&BufferObj, wgpu::ShaderStage)>) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![];

        let mut b_index = 0;
        for i in 0..uniforms.len() {
            let buffer_obj = uniforms[i];

            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: buffer_obj.1,
                ty: wgpu::BindingType::UniformBuffer { dynamic: true, min_binding_size: wgpu::BufferSize::new(0) },
                count: None,
            });
            // 对于动态 uniform buffer, 必须指定 buffer slice 大小
            // make sure that in your BindingResource::Buffer, you're slicing with .slice(..size_of::<Whatever>() as BufferAddress)
            // and not .slice(..)
            // for dynamic uniform buffers, BindingResource::Buffer specifies a "window" into the buffer that is then offset by your dynamic offset value
            entries.push(wgpu::BindGroupEntry {
                binding: b_index,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &buffer_obj.0.buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(256),
                },
            });
            b_index += 1;
        }
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { entries: &layouts, label: None });

        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: None,
        });

        DynamicBindingGroupNode { bind_group_layout, bind_group }
    }
}
