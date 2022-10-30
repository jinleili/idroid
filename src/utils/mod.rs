pub mod depth_stencil;
pub mod matrix_helper;

mod hud;
pub use hud::HUD;

#[allow(dead_code)]
pub fn clear_color() -> wgpu::Color {
    wgpu::Color { r: 0.25, g: 0.25, b: 0.3, a: 1.0 }
}

#[allow(dead_code)]
pub fn alpha_color() -> wgpu::Color {
    wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
}

#[allow(dead_code)]
pub fn black_color() -> wgpu::Color {
    wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
}

// 混合：https://vulkan.lunarg.com/doc/view/1.0.26.0/linux/vkspec.chunked/ch26s01.html
// alpha_blend(), color_blend() 的配置为 @kvark 在 gitter 中推荐
#[allow(dead_code)]
pub fn default_blend() -> wgpu::BlendState {
    wgpu::BlendState { color: color_blend(), alpha: alpha_blend() }
}

fn alpha_blend() -> wgpu::BlendComponent {
    wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Max,
    }
}

fn color_blend() -> wgpu::BlendComponent {
    wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    }
}

#[allow(dead_code)]
pub fn replace_blend() -> wgpu::BlendState {
    wgpu::BlendState { color: wgpu::BlendComponent::REPLACE, alpha: wgpu::BlendComponent::REPLACE }
}

// alpha 颜色混合的一种常用设置
// https://www.cnblogs.com/heitao/p/6974203.html
// src_factor：表示的是当前值的因子，dst_factor：表示缓冲区旧值的混合因子
// finalColor.rgb = newAlpha * newColor + (1 - newAlpha) * oldColor;
// finalColor.a = newAlpha.a;
#[allow(dead_code)]
pub fn color_alpha_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: // 下边的 alpha_blend 能兼容 iOS
        wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        }
    }
}

// 简单的颜色叠加
// 原理：https://www.jianshu.com/p/6d9a3f39bb53
#[allow(dead_code)]
pub fn color_blend_over() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    }
}

// 颜色减法：灰色可叠加成黑色
#[allow(dead_code)]
pub fn color_blend_subtract() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::ReverseSubtract,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    }
}
