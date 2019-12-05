use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
pub struct MVPUniform {
    pub mvp_matrix: [[f32; 4]; 4],
}

pub struct BufferObj {
    pub buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
}

#[allow(dead_code)]
impl BufferObj {
    pub fn create_storage_buffer<T>(device: &mut wgpu::Device, slice: &[T]) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        let buffer = device.create_buffer_with_data(
            slice.as_bytes(),
            wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        );

        BufferObj { buffer, size: (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress }
    }

    pub fn create_uniform_buffer<T>(device: &mut wgpu::Device, uniforms: &T) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        let buffer = device.create_buffer_with_data(
            uniforms.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        BufferObj { buffer, size: std::mem::size_of::<T>() as wgpu::BufferAddress }
    }

    pub fn create_uniforms_buffer<T>(device: &mut wgpu::Device, slice: &[T]) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        let buffer = device.create_buffer_with_data(
            slice.as_bytes(),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        BufferObj { buffer, size: (std::mem::size_of::<T>() * slice.len()) as wgpu::BufferAddress }
    }

    pub fn update_buffer_immediately<T>(
        &mut self, device: &mut wgpu::Device, queue: &mut wgpu::Queue, data: &T,
    ) where
        T: 'static + AsBytes + Copy,
    {
        let temp_buf = device.create_buffer_with_data(data.as_bytes(), wgpu::BufferUsage::COPY_SRC);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
        queue.submit(&[encoder.finish()]);
    }

    pub fn update_buffer<T>(
        &mut self, encoder: &mut wgpu::CommandEncoder, device: &mut wgpu::Device, data: &T,
    ) where
        T: 'static + AsBytes + Copy,
    {
        let temp_buf = device.create_buffer_with_data(data.as_bytes(), wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
    }
}
