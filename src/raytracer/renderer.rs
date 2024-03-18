
use image::DynamicImage;
use wgpu::{BufferBinding, BufferUsages, Sampler, TextureView};
use winit::{
    dpi::PhysicalSize, 
    window::Window
};

use std::path::Path;
use image::io::Reader as ImageReader;

use super::{CubeMapMaterial, Scene};

pub struct State<'a> {
    // Device/Context objects
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: &'a Window,

    // Assets
    color_buffer: wgpu::Texture,
    color_buffer_view: TextureView,
    sampler: wgpu::Sampler,
    scene_parameters: wgpu::Buffer,
    object_buffer: wgpu::Buffer,
    node_buffer: wgpu::Buffer,
    object_index_buffer: wgpu::Buffer,
    sky_material: CubeMapMaterial,

    // Pipeline Objects
    ray_tracing_pipeline: wgpu::ComputePipeline,
    ray_tracing_bind_group: wgpu::BindGroup,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,

    // Scene to render
    pub scene: Scene,
}

impl<'a> State<'a> {

    pub async fn new(window: &'a Window, scene: Scene) -> Self {

        let size = window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window)
            .unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor)
            .await.unwrap();

        let (device, queue) = init_device_and_queue(&adapter).await;

        let config = init_surface_configuration(&adapter, &surface, &size);
        surface.configure(&device, &config);

        // Create assets to be used
        let (color_buffer, 
            color_buffer_view, 
            sampler, 
            scene_parameters, 
            object_buffer, 
            node_buffer, 
            object_index_buffer,
            sky_material) = create_assets(&device, &size, &scene, &queue).await;
        
        // create bind group layouts
        let (ray_tracing_bind_group_layout, 
            screen_bind_group_layout) = make_bind_group_layouts(&device).await;
        
        // Create render pipeline
        let (ray_tracing_pipeline, 
            screen_pipeline) = make_pipeline(&device, &ray_tracing_bind_group_layout, &screen_bind_group_layout).await;
        
        // Create bind groups
        let (ray_tracing_bind_group, 
            screen_bind_group) = make_bind_groups(&device, &color_buffer_view, &sampler, &scene_parameters, &object_buffer, &node_buffer, &object_index_buffer, &ray_tracing_bind_group_layout, &screen_bind_group_layout, &sky_material).await;

        Self {
            // Device/Context objects
            surface,
            device,
            queue,
            config,
            size,
            window,
            // Assets
            color_buffer,
            color_buffer_view,
            sampler,
            scene_parameters,
            object_buffer,
            node_buffer,
            object_index_buffer,
            sky_material,
            // Pipeline Objects
            ray_tracing_pipeline,
            ray_tracing_bind_group,
            screen_pipeline,
            screen_bind_group,
            // Scene to render
            scene,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>{
        
        self.prepare_scene();
        
        let start_time = std::time::Instant::now();
        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);
        
        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        };
        let mut command_encoder = self.device.create_command_encoder(&command_encoder_descriptor);
        
        let ray_trace_pass_descriptor = wgpu::ComputePassDescriptor {
            label: Some("Ray Pass Descriptor"),
            timestamp_writes: None,
        };
        let mut ray_trace_pass = command_encoder.begin_compute_pass(&ray_trace_pass_descriptor);
        ray_trace_pass.set_pipeline(&self.ray_tracing_pipeline);
        ray_trace_pass.set_bind_group(0, &self.ray_tracing_bind_group, &[]);
        ray_trace_pass.dispatch_workgroups(self.size.width/8, self.size.height/8, 1);
        drop(ray_trace_pass);
        
        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.75,
                    g: 0.5,
                    b: 0.25,
                    a: 1.0
                }),
                store: wgpu::StoreOp::Store,
            },
        };
        
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        // Begin the render pass and set up for drawing
        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.screen_pipeline); // Set the screen rendering pipeline
            render_pass.set_bind_group(0, &self.screen_bind_group, &[]); // Set the bind group
            render_pass.draw(0..6, 0..1);
        }
        
        self.queue.submit(std::iter::once(command_encoder.finish()));
        
        drawable.present();
        
        let object_count = self.scene.objects.len();
        let duration = start_time.elapsed(); // Calculate how long the rendering took
        println!("Rendered in {:?}, object count: {}", duration, object_count);
        
        Ok(())
    }

    fn prepare_scene(&self) {
        // Convert the f32 array to bytes
        let scene_data_bytes = self.scene.flatten_scene_data();

        // Write to the buffer
        self.queue.write_buffer(
            &self.scene_parameters, 
            0,
            &scene_data_bytes,
        );

        // Get object data in bytes
        let object_data_bytes = self.scene.flatten_object_data();

        // Write to the buffer
        self.queue.write_buffer(
            &self.object_buffer, // The wgpu::Buffer for object data
            0, // Offset within the buffer
            &object_data_bytes, // The byte slice containing the object data
        );
        
        // Get node data in bytes
        let node_data_bytes = self.scene.flatten_node_data();
        // Write to the buffer
        self.queue.write_buffer(
            &self.node_buffer, // The wgpu::Buffer for node data
            0, // Offset within the buffer
            &node_data_bytes, // The byte slice containing the node data
        );

        // Get node data in bytes
        let object_index_data_bytes = self.scene.flatten_object_index_data();
        
        // Write to the buffer
        self.queue.write_buffer(
            &self.object_index_buffer, // The wgpu::Buffer for object_index data
            0, // Offset within the buffer
            &object_index_data_bytes, // The byte slice containing the object_index data
        );
    }
}

// ----------Initialization Functions---------- //
async fn init_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let device_descriptor = wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        label: Some("Device"),
    };
    adapter.request_device(&device_descriptor, None).await.unwrap()
}

fn init_surface_configuration(adapter: &wgpu::Adapter, surface: &wgpu::Surface, size: &PhysicalSize<u32>) -> wgpu::SurfaceConfiguration {
    let surface_capabilities = surface.get_capabilities(adapter);
    
    let present_mode = if surface_capabilities.present_modes.contains(&wgpu::PresentMode::Mailbox) {
        wgpu::PresentMode::Mailbox // Triple buffering if available
    } else if surface_capabilities.present_modes.contains(&wgpu::PresentMode::Fifo) {
        wgpu::PresentMode::Fifo // V-Sync (Double Buffering)
    } else {
        wgpu::PresentMode::Immediate // For the lowest latency, might introduce tearing
    };

    let surface_format = surface_capabilities
        .formats
        .iter()
        .copied()
        .filter(|f | f.is_srgb())
        .next()
        .unwrap_or(surface_capabilities.formats[0]);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: present_mode,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2
    }
}

// ----------Asset Creation Functions---------- //
async fn create_assets(
    device: &wgpu::Device,
    size: &winit::dpi::PhysicalSize<u32>,
    scene: &Scene,
    queue: &wgpu::Queue,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, CubeMapMaterial) {

    let (color_buffer, color_buffer_view) = create_color_buffer(device, size);

    let sampler_descriptor = wgpu::SamplerDescriptor {
        label: Some("Sampler Descriptor"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: f32::MAX,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    };

    let sampler = device.create_sampler(&sampler_descriptor);

    let scene_parameters = create_scene_parameters(device).await;

    let object_buffer = create_object_buffer(device, scene).await;

    let node_buffer = create_node_buffer(device, scene).await;

    let object_index_buffer = create_object_index_buffer(device, scene).await;

    let paths = vec![
        "assets/gfx/sky_right.png",
        "assets/gfx/sky_left.png",
        "assets/gfx/sky_bottom.png", // 3 is bottom
        "assets/gfx/sky_top.png",
        "assets/gfx/sky_back.png",
        "assets/gfx/sky_front.png",
    ];
    let images:Vec<DynamicImage> = load_cube_map_images(paths);
    let sky_material: CubeMapMaterial = CubeMapMaterial::new(device, queue, images);
    // Return the created resources
    (color_buffer, color_buffer_view, sampler, scene_parameters, object_buffer, node_buffer, object_index_buffer, sky_material)
} 

fn create_color_buffer(device: &wgpu::Device, size: &PhysicalSize<u32>) -> (wgpu::Texture, wgpu::TextureView) {
    let color_buffer_description = wgpu::TextureDescriptor {
        label: Some("Color Buffer Description"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    };
    let color_buffer = device.create_texture(&color_buffer_description);

    let color_buffer_view_description = wgpu::TextureViewDescriptor {
        label: Some("Color Buffer View Description"),
        format: None,
        dimension: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    };

    let color_buffer_view = color_buffer.create_view(&color_buffer_view_description);
    (color_buffer, color_buffer_view)
}

async fn create_scene_parameters(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Scene Parameters Buffer"),
        size: 80,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

async fn create_object_buffer(device: &wgpu::Device, scene: &Scene) -> wgpu::Buffer {
    let object_buffer_descriptor = wgpu::BufferDescriptor {
        label: Some("Object Buffer Descriptor"),
        size: 84 * scene.objects.len() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    device.create_buffer(&object_buffer_descriptor)
}

async fn create_node_buffer(device: &wgpu::Device, scene: &Scene) -> wgpu::Buffer {
    let node_buffer_descriptor = wgpu::BufferDescriptor {
        label: Some("Node Buffer Descriptor"),
        size: 32 * (scene.nodes_used as u64),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    device.create_buffer(&node_buffer_descriptor)
}

async fn create_object_index_buffer(device: &wgpu::Device, scene: &Scene) -> wgpu::Buffer {
    let object_index_buffer_descriptor = wgpu::BufferDescriptor {
        label: Some("Object Buffer Descriptor"),
        size: 4 * scene.objects.len() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    device.create_buffer(&object_index_buffer_descriptor)
}

// ----------Pipeline and bind group Creation Functions---------- //
async fn make_bind_group_layouts(device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroupLayout) {
    // ----------Ray tracing bind group---------- //
    let ray_tracing_bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Ray Bind Group Layout Descriptor"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None, // Not an arrayed binding
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None,
                },
                count: None, // Not an arrayed binding
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                    has_dynamic_offset: false, 
                    min_binding_size: None,
                },
                count: None, // Not an arrayed binding
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                    has_dynamic_offset: false, 
                    min_binding_size: None,
                },
                count: None, // Not an arrayed binding
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Storage { read_only: true }, 
                    has_dynamic_offset: false, 
                    min_binding_size: None,
                },
                count: None, // Not an arrayed binding
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };
    let ray_tracing_bind_group_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&ray_tracing_bind_group_layout_descriptor);

    // ----------Screen bind group---------- //
    let screen_bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Screen Bind Group Layout Descriptor"),
        entries: &[
            // Sampler entry
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // Texture entry
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    };
    let screen_bind_group_layout = device.create_bind_group_layout(&screen_bind_group_layout_descriptor);

    (ray_tracing_bind_group_layout, screen_bind_group_layout)
}

async fn make_bind_groups(
    device: &wgpu::Device,
    color_buffer_view: &wgpu::TextureView,
    sampler: &Sampler,
    scene_parameters: &wgpu::Buffer,
    object_buffer: &wgpu::Buffer,
    node_buffer: &wgpu::Buffer,
    object_index_buffer: &wgpu::Buffer,
    ray_tracing_bind_group_layout: &wgpu::BindGroupLayout,
    screen_bind_group_layout: &wgpu::BindGroupLayout,
    sky_material: &CubeMapMaterial) -> (wgpu::BindGroup, wgpu::BindGroup) {
    // ----------Ray tracing bind groups---------- //
    let ray_tracing_bind_group_descriptor = wgpu::BindGroupDescriptor {
        label: Some("Ray bind Group Descriptor"),
        layout: &ray_tracing_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&color_buffer_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: scene_parameters,
                    offset: 0,
                    size: None, // Use the entire buffer
                }),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: object_buffer,
                    offset: 0,
                    size: None, // Use the entire buffer
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: node_buffer,
                    offset: 0,
                    size: None, // Use the entire buffer
                }),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: object_index_buffer,
                    offset: 0,
                    size: None, // Use the entire buffer
                }),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&sky_material.view),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Sampler(&sky_material.sampler),
            },
        ],
    };
    let ray_tracing_bind_group = device.create_bind_group(&ray_tracing_bind_group_descriptor);
    
    // ----------Screen bind groups---------- //
    let screen_bind_group_descriptor = wgpu::BindGroupDescriptor {
        label: Some("Screen bind Group Descriptor"),
        layout: &screen_bind_group_layout,
        entries: &[
            // Sampler
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            // Texture view
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&color_buffer_view),
            },
        ],
    };
    let screen_bind_group = device.create_bind_group(&screen_bind_group_descriptor);

    (ray_tracing_bind_group, screen_bind_group)
}

async fn make_pipeline(
    device: &wgpu::Device,
    ray_tracing_bind_group_layout: &wgpu::BindGroupLayout,
    screen_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::ComputePipeline, wgpu::RenderPipeline) {
    // ----------Ray tracing pipeline---------- //
    let ray_tracing_pipeline = create_ray_compute_pipeline(device, ray_tracing_bind_group_layout);

    // ----------Screen/render pipeline---------- //
    let screen_pipeline = create_screen_pipeline(device, screen_bind_group_layout);

    // Return the created resources
    (ray_tracing_pipeline, screen_pipeline)
}

fn create_ray_compute_pipeline(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline {
    let pipeline_layout = create_pipeline_layout(device, bind_group_layout);

    // Create the shader module
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Tracing Shader Module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/raytracer_kernel.wgsl").into()),
    });

    // Define the compute pipeline descriptor with the shader module and entry point
    let pipeline_descriptor = wgpu::ComputePipelineDescriptor {
        label: Some("Pipeline Descriptor"),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: "main", // Entry point in the shader
    };

    // Create the compute pipeline
    device.create_compute_pipeline(&pipeline_descriptor)
}

fn create_screen_pipeline(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
    let pipeline_layout = create_pipeline_layout(device, bind_group_layout);

    // Vertex shader module
    let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader Module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/screen_shader.wgsl").into()),
    });

    // Fragment shader module
    let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader Module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/screen_shader.wgsl").into()),
    });

    // Define the render pipeline descriptor
    let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
        label: Some("Screen Pipeline Descriptor"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader_module,
            entry_point: "vert_main",
            buffers: &[], // Define your vertex buffers here
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader_module,
            entry_point: "frag_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    };

    device.create_render_pipeline(&pipeline_descriptor)
}

fn create_pipeline_layout(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    })
}

fn load_cube_map_images(paths: Vec<&str>) -> Vec<DynamicImage> {
    paths.into_iter().map(|path| {
        ImageReader::open(Path::new(path))
            .expect("Failed to open image")
            .decode()
            .expect("Failed to decode image")
    }).collect()
}