use crate::buffer::BufferObj;
use crate::math::Size;
use nalgebra_glm as glm;
use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct MVPUniform {
    pub mvp_matrix: [[f32; 4]; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct MVPUniform2 {
    pub p_matrix: [[f32; 4]; 4],
    pub mv_matrix: [[f32; 4]; 4],
}

impl MVPUniform {
    pub fn zero() -> Self {
        MVPUniform { mvp_matrix: [[0.0; 4]; 4] }
    }
}

pub struct MVPUniformObj {
    pub buffer: BufferObj,
    // 实现绽放与拖拽
    scale: f32,
    pintch_start_location: Option<(f32, f32)>,
    p_matrix: glm::TMat4<f32>,
    base_mv_matrix: glm::TMat4<f32>,
}

impl MVPUniformObj {
    pub fn new(viewport_size: Size<f32>, device: &wgpu::Device) -> Self {
        let (p_matrix, base_mv_matrix, _factor) = crate::utils::matrix_helper::perspective_mvp(viewport_size);
        let buffer = BufferObj::create_uniform_buffer(
            device,
            &MVPUniform { mvp_matrix: (p_matrix * base_mv_matrix).into() },
            Some("MVPUniformObj"),
        );
        MVPUniformObj {
            buffer,
            p_matrix,
            base_mv_matrix,
            scale: 1.0,
            pintch_start_location: None,
        }
    }

    pub fn pintch_start(&mut self, location: (f32, f32), _scale: f32) {
        // 缩放并拖拽始终是以 start 为中心的
        // 可以计算出 start 相对中心点的偏移坐标，无论如何缩放，其偏移坐标是不变的;
        // change 时，直接计算 changed 相对中心点的偏移，缩放完成后，再执行些偏移就能得到正确的位置
        self.pintch_start_location = Some(location);
    }
    // 缩放并拖拽：
    // 先将缩放质心移动到视图中心，执行缩放
    // 再将质心移到到实际位置
    // scale 小于 0 时，只按中心缩放
    pub fn pintch_changed(
        &mut self, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device, location: (f32, f32), scale: f32,
    ) {
        if let Some(start_location) = self.pintch_start_location {
            let mut vm_matrix = self.base_mv_matrix;
            self.scale *= scale;
            if self.scale < 0.7 {
                self.scale = 0.7;
                vm_matrix = glm::scale(&vm_matrix, &glm::vec3(self.scale, self.scale, 1.0));
            } else {
                let (offset_x, offset_y, target_x, target_y) = if self.scale < 1.0 {
                    println!("scale 0: {}, {}", self.scale, scale);
                    (0.0, 0.0, 0.0, 0.0)
                } else {
                    (
                        (0.5 - start_location.0) * 2.0,
                        (0.5 - start_location.1) * 2.0,
                        location.0 - start_location.0,
                        location.1 - start_location.1,
                    )
                };
                // 以 pintch start 为中心点缩放
                vm_matrix = glm::translate(&vm_matrix, &glm::vec3(-offset_x, -offset_y, 0.0));
                vm_matrix = glm::scale(&vm_matrix, &glm::vec3(self.scale, self.scale, 1.0));
                // 平移到 pintch changed 质心
                println!("translate x: {}, y: {}, scale: {}", offset_x + target_x, offset_y + target_y, self.scale,);

                vm_matrix = glm::translate(&vm_matrix, &glm::vec3(offset_x + target_x, offset_y + target_y, 0.0));
            }
            self.buffer.update_buffer(encoder, device, &MVPUniform { mvp_matrix: (self.p_matrix * vm_matrix).into() });
        }
    }
}
