use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

pub struct BufferObj {
    pub buffer: wgpu::Buffer,
    pub size: wgpu::BufferAddress,
    pub has_dynamic_offset: bool,
    pub read_only: bool,
}

#[allow(dead_code)]
impl BufferObj {
    pub fn create_storage_buffer<T>(device: &wgpu::Device, slice: &[T], label: Option<&'static str>) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        BufferObj::create_buffer(device, Some(slice), None, wgpu::BufferUsages::STORAGE, label)
    }

    pub fn create_empty_storage_buffer(
        device: &wgpu::Device, size: wgpu::BufferAddress, can_read_back: bool, label: Option<&'static str>,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            usage: if can_read_back {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
            } else {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
            },
            label,
            mapped_at_creation: false,
        });
        BufferObj { buffer, size, has_dynamic_offset: false, read_only: false }
    }

    pub fn create_uniform_buffer<T>(device: &wgpu::Device, uniform: &T, label: Option<&'static str>) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        BufferObj::create_buffer(device, None, Some(uniform), wgpu::BufferUsages::UNIFORM, label)
    }

    pub fn create_uniforms_buffer<T>(device: &wgpu::Device, slice: &[T], label: Option<&'static str>) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        BufferObj::create_buffer(device, Some(slice), None, wgpu::BufferUsages::UNIFORM, label)
    }

    pub fn update_buffer<T>(&self, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device, data: &T)
    where
        T: 'static + AsBytes + Copy,
    {
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Temp Buffer"),
            contents: data.as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
    }

    pub fn update_buffers_immediately<T>(&self, device: &wgpu::Device, queue: &wgpu::Queue, slice: &[T])
    where
        T: 'static + AsBytes + Copy,
    {
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Temp Buffer"),
            contents: slice.as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
        queue.submit(Some(encoder.finish()));
    }

    pub fn update_buffers<T>(&self, encoder: &mut wgpu::CommandEncoder, device: &wgpu::Device, slice: &[T])
    where
        T: 'static + AsBytes + Copy,
    {
        // 此处想要省掉 staging_buffer, 只能使用 map_write 这个 future 接口：
        // You can also map buffers but that requires polling the device
        // 但是专家 @kvark 说不可行: (2020/04/30)Writing to buffers directly is not currently feasible since you can't have part of a buffer used by GPU when changing it on CPU
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Temp Buffer"),
            contents: slice.as_bytes(),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(&temp_buf, 0, &self.buffer, 0, self.size);
    }

    pub fn create_buffer<T>(
        device: &wgpu::Device, slice: Option<&[T]>, item: Option<&T>, usage: wgpu::BufferUsages,
        label: Option<&'static str>,
    ) -> Self
    where
        T: 'static + AsBytes + Copy,
    {
        let mut size = std::mem::size_of::<T>() as wgpu::BufferAddress;
        let data: &[u8] = if let Some(slice) = slice {
            size *= slice.len() as wgpu::BufferAddress;
            slice.as_bytes()
        } else {
            item.unwrap().as_bytes()
        };
        // 移除staging buffer
        // 移动GPU通常是统一内存架构。这一内存架构下，CPU可以直接访问GPU所使用的内存
        // if cfg!(any(target_os = "ios", target_os = "android")) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: data,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        BufferObj { buffer, size, has_dynamic_offset: false, read_only: false }
    }
}
