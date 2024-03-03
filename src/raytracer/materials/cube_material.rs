use wgpu::{core::device::queue, util::DeviceExt, TextureDescriptor};
use image::{DynamicImage, Rgba};

pub struct CubeMapMaterial {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl CubeMapMaterial {
    // Initialize the CubeMapMaterial with device and image data
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, images: Vec<DynamicImage>) -> Self {
        assert_eq!(images.len(), 6, "There must be exactly 6 images for a cube map");
        
        // Assuming all images are of the same size
        let img_width = images[0].width();
        let img_height = images[0].height();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: img_width,
                height: img_height,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("CubeMapTexture"),
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        for (i, image) in images.into_iter().enumerate() {
            let rgba = image.to_rgba8();
            let bytes = rgba.as_flat_samples();

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32, // Layer index for each face of the cube map
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes.as_slice(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some((4 * img_width) as u32),
                    rows_per_image: Some(img_height as u32),
                },
                wgpu::Extent3d {
                    width: img_width,
                    height: img_height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(6),
            label: Some("Texture View"),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }
}