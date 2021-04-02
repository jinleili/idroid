use image::GenericImageView;
use std::{num::NonZeroU32, path::PathBuf};
use wgpu::{Extent3d, Sampler, Texture, TextureView};

#[allow(dead_code)]
pub fn from_img_name(image_path: &str, app_view: &crate::AppView) -> (Texture, TextureView, Extent3d, Sampler) {
    self::from_img_name_and_usage_write(image_path, app_view, false, false)
}

// is_gray_pic: 是否为单通道灰度纹理
#[allow(dead_code)]
pub fn from_img_name_and_usage_write(
    image_path: &str, app_view: &crate::AppView, usage_write: bool, is_gray_pic: bool,
) -> (Texture, TextureView, Extent3d, Sampler) {
    // 动态加载本地文件
    let path = PathBuf::from(image_path);
    crate::texture::from_path(path, app_view, usage_write, is_gray_pic)
}

#[allow(dead_code)]
pub fn from_path_for_usage(
    path: PathBuf, app_view: &crate::AppView, usage: wgpu::TextureUsage, is_gray_pic: bool,
) -> (wgpu::Texture, TextureView, Extent3d, Sampler) {
    let (texels, texture_extent) = load_from_path(path, is_gray_pic);
    let (format, channel_count) =
        if is_gray_pic { (wgpu::TextureFormat::R8Unorm, 1) } else { (wgpu::TextureFormat::Rgba8UnormSrgb, 4) };

    let texture = app_view.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        label: None,
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    app_view.queue.write_texture(
        wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO },
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(channel_count * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );

    (texture, texture_view, texture_extent, default_sampler(&app_view.device))
}

#[allow(dead_code)]
pub fn from_path(
    path: PathBuf, app_view: &crate::AppView, is_storage: bool, is_gray_pic: bool,
) -> (wgpu::Texture, TextureView, Extent3d, Sampler) {
    let usage = if is_storage {
        wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::STORAGE
    } else {
        wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED
    };
    crate::texture::from_path_for_usage(path, app_view, usage, is_gray_pic)
}

#[allow(dead_code)]
pub fn update_by_path(path: PathBuf, app_view: &crate::AppView, texture: &wgpu::Texture, is_gray_pic: bool) {
    let (texels, texture_extent) = load_from_path(path, is_gray_pic);

    app_view.queue.write_texture(
        wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO },
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(if is_gray_pic { 1 } else { 4 } * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );
}

fn load_from_path(path: PathBuf, is_gray_pic: bool) -> (Vec<u8>, wgpu::Extent3d) {
    let image_bytes = match std::fs::read(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };

    let img_load = image::load_from_memory(&image_bytes).expect("Failed to load image.");
    let img_raw = if is_gray_pic { img_load.to_luma8().into_raw() } else { img_load.to_rgba8().into_raw() };
    let texels: Vec<u8> = img_raw;

    let (width, height) = img_load.dimensions();
    let texture_extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    (texels, texture_extent)
}

#[allow(dead_code)]
pub fn from_buffer_and_usage_write(
    buffer: &wgpu::Buffer, app_view: &crate::AppView, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32,
    pixel_size: u32, usage_write: bool,
) -> (TextureView, Extent3d, Sampler) {
    let texture_extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let usage = if usage_write {
        wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE
    } else {
        wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED
    };
    let texture = app_view.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage,
        label: None,
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // BufferCopyView 必须 >= TextureCopyView
    encoder.copy_buffer_to_texture(
        wgpu::ImageCopyBuffer {
            buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(NonZeroU32::new(pixel_size * width).unwrap()),
                rows_per_image: Some(NonZeroU32::new(height).unwrap()),
            },
        },
        wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO },
        texture_extent,
    );

    (texture_view, texture_extent, default_sampler(&app_view.device))
}

// empty texture as a RENDER_ATTACHMENT
#[allow(dead_code)]
pub fn empty(
    device: &wgpu::Device, format: wgpu::TextureFormat, extent: Extent3d, usage: Option<wgpu::TextureUsage>,
) -> TextureView {
    self::empty2(device, format, extent, usage).1
}

pub fn empty2(
    device: &wgpu::Device, format: wgpu::TextureFormat, extent: Extent3d, usage: Option<wgpu::TextureUsage>,
) -> (Texture, TextureView) {
    let usage = if let Some(u) = usage {
        u
    } else {
        wgpu::TextureUsage::RENDER_ATTACHMENT
            | wgpu::TextureUsage::COPY_DST
            | wgpu::TextureUsage::SAMPLED
            | wgpu::TextureUsage::STORAGE
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        label: None,
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, texture_view)
}

#[allow(dead_code)]
pub fn empty_view(device: &wgpu::Device, width: u32, height: u32) -> TextureView {
    crate::texture::empty(
        device,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        None,
    )
}

// 32位浮点纹理
#[allow(dead_code)]
pub fn empty_f32_view(device: &wgpu::Device, width: u32, height: u32) -> TextureView {
    crate::texture::empty(
        device,
        wgpu::TextureFormat::Rgba32Float,
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        None,
    )
}

#[allow(dead_code)]
pub fn default_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

// 瓦片式平铺采样
#[allow(dead_code)]
pub fn tile_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

// 双线性插值
// https://vulkan-tutorial.com/Texture_mapping/Image_view_and_sampler
#[allow(dead_code)]
pub fn bilinear_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        // iOS 上设置了 compare 值会 crash
        // compare: Some(wgpu::CompareFunction::LessEqual),
        // compare: wgpu::CompareFunction::Undefined,
        ..Default::default()
    })
}
