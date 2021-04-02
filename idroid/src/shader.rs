use std::fs::read_to_string;
use std::path::PathBuf;

#[cfg(not(target_os = "ios"))]
use shaderc::ShaderKind;

#[cfg(target_os = "ios")]
use std::io::Read;

use wgpu::util::make_spirv;

// 所有 GL_ 打头的宏名称都是 glsl 保留的，不能自定义
const SHADER_VERSION_GL: &str = "#version 450\n";
const SHADER_IMPORT: &str = "#include ";

#[allow(dead_code)]
pub enum ShaderStage {
    General,
    Compute,
}

pub struct Shader {
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: Option<wgpu::ShaderModule>,
}

#[allow(dead_code)]
impl Shader {
    pub fn new(name: &str, device: &wgpu::Device, base_path: &str) -> Self {
        let (vs_module, fs_module) = load_general_glsl(name, device, base_path);
        Shader { vs_module, fs_module: Some(fs_module) }
    }

    // 计算着色
    #[cfg(target_os = "ios")]
    pub fn new_by_compute(name: &str, device: &wgpu::Device, _base_path: &str) -> Self {
        let bytes = generate_shader_source(name, "comp");
        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: make_spirv(&bytes),
            flags: wgpu::ShaderFlags::VALIDATION,
        });
        Shader { vs_module: module, fs_module: None }
    }

    #[cfg(not(target_os = "ios"))]
    pub fn new_by_compute(name: &str, device: &wgpu::Device, base_path: &str) -> Self {
        let binary_result = generate_shader_source(name, ShaderKind::Compute, &base_path);
        Shader::shader_by_bytes(binary_result.as_binary_u8(), device)
    }

    fn shader_by_bytes(bytes: &[u8], device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: make_spirv(bytes),
            flags: wgpu::ShaderFlags::VALIDATION,
        });
        Shader { vs_module: module, fs_module: None }
    }
}

#[cfg(target_os = "ios")]
#[allow(dead_code)]
pub fn load_general_glsl(
    name: &str, device: &wgpu::Device, _base_path: &str,
) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_bytes = generate_shader_source(name, "vs");
    let fs_bytes = generate_shader_source(name, "fs");

    let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: make_spirv(&vs_bytes),
        flags: wgpu::ShaderFlags::VALIDATION,
    });
    let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: make_spirv(&fs_bytes),
        flags: wgpu::ShaderFlags::VALIDATION,
    });
    (vs_module, fs_module)
}

#[cfg(target_os = "ios")]
#[allow(dead_code)]
fn generate_shader_source(name: &str, suffix: &str) -> Vec<u8> {
    let p = uni_view::fs::FileSystem::get_shader_path(name, suffix);
    println!("spv path: {:?}", &p);
    let mut f = std::fs::File::open(p).unwrap();
    let mut spv = Vec::new();
    // read the whole file
    f.read_to_end(&mut spv).unwrap();
    spv
}

// wgpu-rs 0.7， glsl 着色器 ShaderFlags 只能设置为 default，否则无法通过 naga 的验证
#[cfg(not(target_os = "ios"))]
#[allow(dead_code)]
pub fn load_general_glsl(
    name: &str, device: &wgpu::Device, base_path: &str,
) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_binary = generate_shader_source(name, ShaderKind::Vertex, &base_path);
    let fs_binary = generate_shader_source(name, ShaderKind::Fragment, &base_path);
    let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: make_spirv(vs_binary.as_binary_u8()),
        flags: wgpu::ShaderFlags::default(),
    });
    let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: make_spirv(fs_binary.as_binary_u8()),
        flags: wgpu::ShaderFlags::default(),
    });

    (vs_module, fs_module)
}

#[cfg(not(target_os = "ios"))]
fn generate_shader_source(name: &str, ty: ShaderKind, base_path: &str) -> shaderc::CompilationArtifact {
    let suffix = match ty {
        ShaderKind::Vertex => ".vs.glsl",
        ShaderKind::Fragment => ".fs.glsl",
        _ => ".comp.glsl",
    };

    let path = PathBuf::from(base_path).join("shader").join(format!("{}{}", name, suffix));
    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            if cfg!(target_os = "macos") && ty == ShaderKind::Vertex {
                load_common_vertex_shader(base_path)
            } else {
                panic!("Unable to read {:?}: {:?}", path, e)
            }
        }
    };
    let mut shader_source = String::new();
    shader_source.push_str(SHADER_VERSION_GL);
    parse_shader_source(&code, &mut shader_source, base_path);

    let mut compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();
    let binary_result = compiler.compile_into_spirv(&shader_source, ty, "shader.glsl", "main", Some(&options)).unwrap();

    binary_result
}

#[allow(dead_code)]
fn load_common_vertex_shader(base_path: &str) -> String {
    let path = PathBuf::from(base_path).join("shader").join("common.vs.glsl");

    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };

    code
}

// Parse a shader string for imports. Imports are recursively processed, and
// prepended to the list of outputs.
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
            output.push_str("\n");
        }
        // 移除注释
        // match line.find("//") {
        //     Some(_) => (),
        //     None => {

        //     }
        // }
    }
}

// 获取通用 shader function
// 将着色器代码预先静态加载进程序，避免打包成 .a 静态库时找不到文件
#[allow(dead_code)]
fn get_shader_funcs(key: &str, base_path: &str) -> Option<String> {
    let path = PathBuf::from(base_path).join("shader").join(key.replace('"', ""));
    let shader = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };
    Some(shader)
}
