#[allow(dead_code)]
use zerocopy::{AsBytes, FromBytes};

pub trait Pos {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor>;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosOnly {
    pub pos: [f32; 3],
}

impl PosOnly {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3]) -> PosOnly {
        PosOnly { pos }
    }
}

impl Pos for PosOnly {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![wgpu::VertexAttributeDescriptor {
            shader_location: offset + 0,
            format: wgpu::VertexFormat::Float3,
            offset: 0,
        }]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosColor {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

impl PosColor {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3], color: [f32; 4]) -> PosColor {
        PosColor { pos, color }
    }

    pub fn color_offset() -> wgpu::BufferAddress {
        4 * 3
    }
}

impl Pos for PosColor {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 0,
                format: wgpu::VertexFormat::Float3,
                offset: 0,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float4,
                offset: PosColor::color_offset(),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosTex {
    pub pos: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl PosTex {
    #[allow(dead_code)]
    pub fn vertex_i(pos: [i8; 3], tc: [i8; 2]) -> PosTex {
        PosTex { pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32], tex_coord: [tc[0] as f32, tc[1] as f32] }
    }

    pub fn vertex_f32(pos: [f32; 3], tex_coord: [f32; 2]) -> PosTex {
        PosTex { pos, tex_coord }
    }

    pub fn tex_offset() -> wgpu::BufferAddress {
        4 * 3
    }

    // 移动顶点位置到
    // step_rate: step_index / step_count
    pub fn move_to(&self, to: &[f32; 3], step_rate: f32) -> PosTex {
        PosTex {
            pos: [
                self.pos[0] + (to[0] - self.pos[0]) * step_rate,
                self.pos[1] + (to[1] - self.pos[1]) * step_rate,
                self.pos[2] + (to[2] - self.pos[2]) * step_rate,
            ],
            tex_coord: self.tex_coord,
        }
    }

    // pub fn vb_descriptor<'a>(offset: u32) -> wgpu::VertexBufferDescriptor<'a> {

    //     wgpu::VertexBufferDescriptor {
    //             stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
    //             step_mode: wgpu::InputStepMode::Vertex,
    //             attributes: &PosTex::attri_descriptor(offset)
    //         }
    // }
}

impl Pos for PosTex {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 0,
                format: wgpu::VertexFormat::Float3,
                offset: 0,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float2,
                offset: PosTex::tex_offset(),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosTex2 {
    pos: [f32; 3],
    tex_coord0: [f32; 2],
    tex_coord1: [f32; 2],
}

impl PosTex2 {
    #[allow(dead_code)]
    pub fn vertex_i(pos: [i8; 3], tc0: [i8; 2], tc1: [i8; 2]) -> PosTex2 {
        PosTex2 {
            pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
            tex_coord0: [tc0[0] as f32, tc0[1] as f32],
            tex_coord1: [tc1[0] as f32, tc1[1] as f32],
        }
    }

    pub fn vertex_f32(pos: [f32; 3], tex_coord0: [f32; 2], tex_coord1: [f32; 2]) -> PosTex2 {
        PosTex2 { pos, tex_coord0, tex_coord1 }
    }

    pub fn tex_offset() -> wgpu::BufferAddress {
        4 * 3
    }
}

impl Pos for PosTex2 {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 0,
                format: wgpu::VertexFormat::Float3,
                offset: 0,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float2,
                offset: PosTex2::tex_offset(),
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 2,
                format: wgpu::VertexFormat::Float2,
                offset: PosTex2::tex_offset() + (4 * 2),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosWeight {
    pub pos: [f32; 3],
    // 离数学中心位置的权重
    pub weight: f32,
}

#[allow(dead_code)]
impl PosWeight {
    pub fn new(pos: [f32; 3], weight: f32) -> Self {
        PosWeight { pos, weight }
    }

    pub fn slope_ridian(&self, last: &PosWeight) -> f32 {
        (self.pos[1] - last.pos[1]).atan2(self.pos[0] - last.pos[0])
    }
}

impl Pos for PosWeight {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 0,
                format: wgpu::VertexFormat::Float3,
                offset: 0,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float,
                offset: 4 * 3,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosBrush {
    pos: [f32; 3],
    uv: [f32; 2],
    // weight, time_interval, pressure
    params: [f32; 3],
}

#[allow(dead_code)]
impl PosBrush {
    pub fn new(pos: [f32; 3], uv: [f32; 2], params: [f32; 3]) -> Self {
        PosBrush { pos, uv, params }
    }
}

impl Pos for PosBrush {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 0,
                format: wgpu::VertexFormat::Float3,
                offset: 0,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 1,
                format: wgpu::VertexFormat::Float2,
                offset: 4 * 3,
            },
            wgpu::VertexAttributeDescriptor {
                shader_location: offset + 2,
                format: wgpu::VertexFormat::Float3,
                offset: 4 * (3 + 2),
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosParticleIndex {
    pos: [u32; 3],
}

#[allow(dead_code)]
impl PosParticleIndex {
    pub fn new(pos: [u32; 3]) -> Self {
        PosParticleIndex { pos }
    }
}

impl Pos for PosParticleIndex {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![wgpu::VertexAttributeDescriptor {
            shader_location: offset + 0,
            format: wgpu::VertexFormat::Uint3,
            offset: 0,
        }]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct PosParticle {
    pos: [f32; 3],
}

#[allow(dead_code)]
impl PosParticle {
    pub fn new(pos: [f32; 3]) -> Self {
        PosParticle { pos }
    }
}

impl Pos for PosParticle {
    fn attri_descriptor(offset: u32) -> Vec<wgpu::VertexAttributeDescriptor> {
        vec![wgpu::VertexAttributeDescriptor {
            shader_location: offset + 0,
            format: wgpu::VertexFormat::Float3,
            offset: 0,
        }]
    }
}
