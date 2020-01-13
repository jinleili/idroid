use image::GenericImageView;
use std::path::PathBuf;
use wgpu::CreateBufferMapped;
use zerocopy::{AsBytes, FromBytes};


pub struct BufferObj {
    pub buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
}

#[allow(dead_code)]
impl BufferObj {
    pub fn create_storage_buffer<T>(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        slice: &[T],
    ) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        BufferObj::create_buffer(
            device,
            encoder,
            Some(slice),
            None,
            wgpu::BufferUsage::STORAGE,
        )
    }

    pub fn create_empty_storage_buffer(
        device: &mut wgpu::Device,
        size: wgpu::BufferAddress,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        });
        BufferObj { buffer, size }
    }

    pub fn create_uniform_buffer<T>(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uniform: &T,
    ) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        BufferObj::create_buffer(
            device,
            encoder,
            None,
            Some(uniform),
            wgpu::BufferUsage::UNIFORM,
        )
    }

    pub fn create_uniforms_buffer<T>(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        slice: &[T],
    ) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        BufferObj::create_buffer(
            device,
            encoder,
            Some(slice),
            None,
            wgpu::BufferUsage::UNIFORM,
        )
    }

    pub fn update_buffer_immediately<T>(
        &mut self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        data: &T,
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
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &mut wgpu::Device,
        data: &T,
    ) where
        T: 'static + AsBytes + Copy,
    {
        let temp_buf = device.create_buffer_with_data(data.as_bytes(), wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
    }

    pub fn update_buffers<T>(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &mut wgpu::Device,
        slice: &[T],
    ) where
        T: 'static + AsBytes + Copy,
    {
        /**
         * if need to update an existing buffer every frame, ```map_write``` or ```copy_buffer_to_buffer``` which is the best choice?
         * as [gpuweb docs mentioned](https://github.com/gpuweb/gpuweb/blob/a43fbad0f01fbc122b97b005a57b0f5c27d03dc6/design/BufferOperations.md),
         * user can reusing upload buffers to reduces overhead, is it possible or appropriate in wgpu-rs?
         * 
         * kvark:  re-using upload buffers is pretty much blocked on #9, so creating a new upload buffer and copying from it is the way to go, for now
         */
        let temp_buf =
            device.create_buffer_with_data(slice.as_bytes(), wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
    }

    fn create_buffer<T>(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        slice: Option<&[T]>,
        item: Option<&T>,
        usage: wgpu::BufferUsage,
    ) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        let mut data: &[u8] = &[0];
        let mut size = std::mem::size_of::<T>() as wgpu::BufferAddress;
        if let Some(slice) = slice {
            size *= slice.len() as wgpu::BufferAddress;
            data = slice.as_bytes();
        } else {
            data = item.unwrap().as_bytes();
        }
        let temp_buffer = device.create_buffer_with_data(data, wgpu::BufferUsage::COPY_SRC);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: usage | wgpu::BufferUsage::COPY_DST,
        });
        encoder.copy_buffer_to_buffer(&temp_buffer, 0, &buffer, 0, size);
        BufferObj { buffer, size }
    }
}
