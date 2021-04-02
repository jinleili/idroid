use std::{borrow::Cow, fs::read_to_string, path::PathBuf};
use wgpu::util::DeviceExt;
use wgpu::{ShaderFlags, ShaderModule, ShaderModuleDescriptor, ShaderSource};

pub fn create_shader_module(device: &wgpu::Device, shader_name: &'static str, lable: Option<&str>) -> ShaderModule {
    let flags = ShaderFlags::VALIDATION | ShaderFlags::EXPERIMENTAL_TRANSLATION;
    // env!("CARGO_MANIFEST_DIR") 是编译时执行的，得到的是当前所编辑的库的所在路径，而不是项目的路径
    let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = PathBuf::from(base_dir).join("shader-wgsl").join(format!("{}.wgsl", shader_name));
    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            panic!("Unable to read {:?}: {:?}", path, e)
        }
    };
    device.create_shader_module(&ShaderModuleDescriptor {
        label: lable,
        source: ShaderSource::Wgsl(Cow::Borrowed(&code)),
        flags,
    })
}
