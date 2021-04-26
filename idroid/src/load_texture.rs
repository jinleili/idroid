use image::GenericImageView;
use std::{num::NonZeroU32, path::Path, path::PathBuf};
use wgpu::{Extent3d, Sampler, Texture, TextureFormat, TextureView};

#[allow(dead_code)]
pub fn from_path(
    image_path: &str, app_view: &crate::AppView, usage: wgpu::TextureUsage, set_to_grayscale: bool,
) -> (wgpu::Texture, TextureView, wgpu::TextureFormat, Extent3d, Sampler) {
    let path = uni_view::fs::get_texture_file_path(image_path);
    //  let usage = if is_storage {
    //     wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::STORAGE
    // } else {
    //     wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED
    // };
    let (texels, texture_extent, format) = load_from_path(path, set_to_grayscale);
    let pixel_bytes = single_pixel_bytes(format);

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
            bytes_per_row: Some(NonZeroU32::new(pixel_bytes * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );

    (texture, texture_view, format, texture_extent, default_sampler(&app_view.device))
}

#[allow(dead_code)]
pub fn update_by_path(image_path: &str, app_view: &crate::AppView, texture: &wgpu::Texture, set_to_grayscale: bool) {
    let path = uni_view::fs::get_texture_file_path(image_path);

    let (texels, texture_extent, format) = load_from_path(path, set_to_grayscale);
    let pixel_bytes = single_pixel_bytes(format);

    app_view.queue.write_texture(
        wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO },
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(pixel_bytes * texture_extent.width).unwrap()),
            rows_per_image: Some(NonZeroU32::new(texture_extent.height).unwrap()),
        },
        texture_extent,
    );
}

#[allow(dead_code)]
pub fn from_buffer(
    buffer: &wgpu::Buffer, app_view: &crate::AppView, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32,
    pixel_size: u32, format: TextureFormat, usage: wgpu::TextureUsage,
) -> (TextureView, Extent3d, Sampler) {
    let texture_extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
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

fn load_from_path(path: PathBuf, set_to_grayscale: bool) -> (Vec<u8>, wgpu::Extent3d, wgpu::TextureFormat) {
    let img = image::open(&path.as_path()).unwrap();

    // get TextureFormat from image
    let color_type = img.color();
    let (format, texels) = match color_type {
        image::ColorType::L8 => (wgpu::TextureFormat::R8Unorm, img.to_bytes()),
        // no rgb format without alpha channels in the webgpu spec, so, need to convert.
        image::ColorType::Rgb8 => {
            if set_to_grayscale {
                (wgpu::TextureFormat::R8Unorm, img.to_luma8().into_raw())
            } else {
                (wgpu::TextureFormat::Rgba8Unorm, img.to_bgra8().into_raw())
            }
        }
        image::ColorType::Rgba8 => {
            if set_to_grayscale {
                (wgpu::TextureFormat::R8Unorm, img.to_luma8().into_raw())
            } else {
                (wgpu::TextureFormat::Rgba8Unorm, img.to_bytes())
            }
        }

        _ => panic!("unsupported color type"),
    };

    let (width, height) = img.dimensions();
    let texture_extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    (texels, texture_extent, format)
}

// empty texture as a RENDER_ATTACHMENT
pub fn empty(
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
    crate::load_texture::empty(
        device,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        None,
    )
    .1
}

// 32位浮点纹理
#[allow(dead_code)]
pub fn empty_f32_view(device: &wgpu::Device, width: u32, height: u32) -> TextureView {
    crate::load_texture::empty(
        device,
        wgpu::TextureFormat::Rgba32Float,
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        None,
    )
    .1
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

fn single_pixel_bytes(format: wgpu::TextureFormat) -> u32 {
    let format_val = format as u32;
    if format_val < 4 {
        1
    } else if format_val < 11 {
        2
    } else if format_val == 36 {
        3
    } else if format_val < 26 || format_val == 35 || format_val == 37 {
        4
    } else if format_val < 32 {
        8
    } else if format_val < 35 {
        16
    } else {
        // The format that hasn't matched yet
        0
    }
}
