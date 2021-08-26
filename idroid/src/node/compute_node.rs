use wgpu::{PushConstantRange, ShaderModule, StorageTextureAccess};

use super::BindingGroupSettingNode;
use crate::{buffer::BufferObj, AnyTexture};

use core::ops::Range;
use std::vec::Vec;

#[allow(dead_code)]
pub struct ComputeNode {
    pub setting_node: BindingGroupSettingNode,
    pub pipeline: wgpu::ComputePipeline,
    pub threadgroup_count: (u32, u32, u32),
}

#[allow(dead_code)]
impl ComputeNode {
    pub fn new(
        device: &wgpu::Device, threadgroup_count: (u32, u32, u32), uniforms: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>, inout_tv: Vec<(&AnyTexture, Option<StorageTextureAccess>)>,
        shader_module: &ShaderModule,
    ) -> Self {
        ComputeNode::new_with_push_constants(
            device,
            threadgroup_count,
            uniforms,
            storage_buffers,
            inout_tv,
            shader_module,
            None,
        )
    }

    pub fn new_with_push_constants(
        device: &wgpu::Device, threadgroup_count: (u32, u32, u32), uniforms: Vec<&BufferObj>,
        storage_buffers: Vec<&BufferObj>, inout_tv: Vec<(&AnyTexture, Option<StorageTextureAccess>)>,
        shader_module: &ShaderModule, push_constants: Option<Vec<(wgpu::ShaderStages, Range<u32>)>>,
    ) -> Self {
        let mut visibilitys: Vec<wgpu::ShaderStages> = vec![];
        for _ in 0..(uniforms.len() + storage_buffers.len() + inout_tv.len()) {
            visibilitys.push(wgpu::ShaderStages::COMPUTE);
        }
        let setting_node =
            BindingGroupSettingNode::new(device, uniforms, storage_buffers, inout_tv, vec![], visibilitys);

        let mut ranges: Vec<PushConstantRange> = vec![];
        if let Some(constants) = push_constants {
            for (stage, range) in constants.iter() {
                ranges.push(wgpu::PushConstantRange { stages: stage.clone(), range: range.clone() })
            }
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&setting_node.bind_group_layout],
            push_constant_ranges: &ranges,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: shader_module,
            entry_point: "main",
        });

        ComputeNode { setting_node, pipeline, threadgroup_count }
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        self.dispatch(&mut cpass);
    }

    pub fn dispatch<'a, 'b: 'a>(&'b self, cpass: &mut wgpu::ComputePass<'a>) {
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, self.threadgroup_count.2);
    }
}
