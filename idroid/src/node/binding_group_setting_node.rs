use crate::buffer::BufferObj;
use std::vec::Vec;
use wgpu::{StorageTextureAccess, TextureFormat};

#[allow(dead_code)]
pub struct BindingGroupSettingNode {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

#[allow(dead_code)]
impl BindingGroupSettingNode {
    pub fn new(
        device: &wgpu::Device, uniforms: Vec<&BufferObj>, inout_buffers: Vec<&BufferObj>,
        textures: Vec<(&wgpu::TextureView, TextureFormat, Option<StorageTextureAccess>)>,
        samplers: Vec<&wgpu::Sampler>, visibilitys: Vec<wgpu::ShaderStage>,
    ) -> Self {
        let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![];

        // 关于 min_binding_size
        // https://gpuweb.github.io/gpuweb/#dom-gpubindgrouplayoutentry-minbufferbindingsize
        let mut b_index = 0_u32;
        for i in 0..uniforms.len() {
            let buffer_obj = uniforms[i];
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(0),
                },
                count: None,
            });
            entries.push(wgpu::BindGroupEntry { binding: b_index, resource: buffer_obj.buffer.as_entire_binding() });
            b_index += 1;
        }

        for i in 0..inout_buffers.len() {
            let buffer_obj = inout_buffers[i];
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: buffer_obj.read_only },
                    has_dynamic_offset: buffer_obj.has_dynamic_offset,
                    min_binding_size: wgpu::BufferSize::new(0),
                },
                count: None,
            });
            entries.push(wgpu::BindGroupEntry { binding: b_index, resource: buffer_obj.buffer.as_entire_binding() });
            b_index += 1;
        }

        for i in 0..textures.len() {
            let storage_access = textures[i].2;
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: if let Some(access) = storage_access {
                    wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        access,
                        format: textures[i].1,
                    }
                } else {
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: texture_sample_filterable(textures[i].1),
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    }
                },
                count: None,
            });
            entries.push(wgpu::BindGroupEntry {
                binding: b_index,
                resource: wgpu::BindingResource::TextureView(textures[i].0),
            });
            b_index += 1;
        }

        for i in 0..samplers.len() {
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: b_index,
                visibility: visibilitys[b_index as usize],
                ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
                count: None,
            });
            entries
                .push(wgpu::BindGroupEntry { binding: b_index, resource: wgpu::BindingResource::Sampler(samplers[i]) });
            b_index += 1;
        }

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { entries: &layouts, label: None });

        let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: None,
        });

        BindingGroupSettingNode { bind_group_layout, bind_group }
    }
}

fn texture_sample_filterable(format: TextureFormat) -> bool {
    match format {
        // on iOS: texture binding 1 expects sample type = Float { filterable: true }, but given a view with format = R32Float
        TextureFormat::R32Float => false,
        _ => true,
    }
}
