use std::{borrow::Cow, fs::read_to_string, path::PathBuf};
use wgpu::util::DeviceExt;
use wgpu::{ShaderFlags, ShaderModule, ShaderModuleDescriptor, ShaderSource};

#[cfg(not(target_os = "ios"))]
#[allow(dead_code)]
pub fn create_shader_module(device: &wgpu::Device, shader_name: &'static str, lable: Option<&str>) -> ShaderModule {
    // @Kvark 20210402 ：Please don't use EXPERIMENTAL_TRANSLATION on Metal for this shader for now.
    // let flags = ShaderFlags::VALIDATION | ShaderFlags::EXPERIMENTAL_TRANSLATION;
    let flags = ShaderFlags::VALIDATION;
    // let flags = ShaderFlags::default();

    // env!("CARGO_MANIFEST_DIR") 是编译时执行的，得到的是当前所编辑的库的所在路径，而不是项目的路径
    // std::env::var("CARGO_MANIFEST_DIR") 在 xcode debug 时不存在
    // std::env::current_dir() 在 xcode debug 时只能获得相对路径： “/”

    // let base_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let base_dir = uni_view::fs::application_root_dir();
    let path = PathBuf::from(&base_dir).join("shader-wgsl").join(format!("{}.wgsl", shader_name));
    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            panic!("Unable to read {:?}: {:?}", path, e)
        }
    };

    let mut shader_source = String::new();
    parse_shader_source(&code, &mut shader_source, &base_dir);
    // println!("{:?} \n === \n \n", &shader_source);

    device.create_shader_module(&ShaderModuleDescriptor {
        label: lable,
        source: ShaderSource::Wgsl(Cow::Borrowed(&shader_source)),
        flags,
    })
}

#[cfg(target_os = "ios")]
#[allow(dead_code)]
pub fn create_shader_module(device: &wgpu::Device, shader_name: &'static str, lable: Option<&str>) -> ShaderModule {
    // @Kvark 20210402 ：Please don't use EXPERIMENTAL_TRANSLATION on Metal for this shader for now.
    // let flags = ShaderFlags::VALIDATION | ShaderFlags::EXPERIMENTAL_TRANSLATION;
    let flags = ShaderFlags::VALIDATION;

    // env!("CARGO_MANIFEST_DIR") 是编译时执行的，得到的是当前所编辑的库的所在路径，而不是项目的路径
    let base_dir = uni_view::fs::application_root_dir();
    let path = PathBuf::from(&base_dir).join("shader-preprocessed-wgsl").join(format!("{}.wgsl", shader_name.replace("/", "_")));
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

const SHADER_IMPORT: &str = "#include ";

#[allow(dead_code)]
fn parse_shader_source(source: &str, output: &mut String, base_path: &str) {
    for line in source.lines() {
        if line.starts_with(SHADER_IMPORT) {
            let imports = line[SHADER_IMPORT.len()..].split(',');
            // For each import, get the source, and recurse.
            for import in imports {
                if let Some(include) = get_shader_funcs(import, base_path) {
                    parse_shader_source(&include, output, base_path);
                } else {
                    println!("shader parse error -------");
                    println!("can't find shader functions: {}", import);
                    println!("--------------------------");
                }
            }
        } else {
            output.push_str(line);
            output.push_str("\n ");
            // 移除注释
            // let need_delete = match line.find("//") {
            //     Some(_) => {
            //         let segments: Vec<&str> = line.split("//").collect();
            //         segments.len() > 1 && segments.first().unwrap().trim().is_empty()
            //     }
            //     None => false,
            // };
            // if !need_delete {
            //     output.push_str(line);
            //     output.push_str("\n");
            // }
        }
    }
}

#[allow(dead_code)]
fn get_shader_funcs(key: &str, base_path: &str) -> Option<String> {
    let path = PathBuf::from(base_path).join("shader-wgsl").join(key.replace('"', ""));
    let shader = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };
    Some(shader)
}
