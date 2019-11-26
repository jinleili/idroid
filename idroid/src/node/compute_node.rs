use super::BindingGroupSettingNode;
use std::vec::Vec;

#[allow(dead_code)]
pub struct ComputeNode {
    pub setting_node: BindingGroupSettingNode,
    pub pipeline: wgpu::ComputePipeline,
    pub threadgroup_count: (u32, u32),
}

#[allow(dead_code)]
impl ComputeNode {
    pub fn new(
        device: &mut wgpu::Device, threadgroup_count: (u32, u32), uniforms: Vec<&wgpu::Buffer>,
        uniform_ranges: Vec<wgpu::BufferAddress>, inout_buffer: Vec<&wgpu::Buffer>,
        inout_buffer_range: Vec<wgpu::BufferAddress>, inout_tv: Vec<(&wgpu::TextureView, bool)>,
        shader: (&str, &str),
    ) -> Self {
        let mut visibilitys: Vec<wgpu::ShaderStage> = vec![];
        for _ in 0..(uniforms.len() + inout_buffer.len() + inout_tv.len()) {
            visibilitys.push(wgpu::ShaderStage::COMPUTE);
        }
        let setting_node = BindingGroupSettingNode::new(
            device,
            uniforms,
            uniform_ranges,
            inout_buffer,
            inout_buffer_range,
            inout_tv,
            vec![],
            visibilitys,
        );

        let shader = crate::shader::Shader::new_by_compute(shader.0, device, shader.1);
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &setting_node.pipeline_layout,
            compute_stage: shader.cs_stage(),
        });

        ComputeNode { setting_node, pipeline, threadgroup_count }
    }

    pub fn compute(&mut self, _device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass();
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
        cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);
    }
}
