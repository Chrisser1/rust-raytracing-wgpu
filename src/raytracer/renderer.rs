
use wgpu::{naga::back::msl::sampler, util::DeviceExt, BufferBinding, BufferUsages, Extent3d, MultisampleState, Sampler, TextureView};
use winit::{
    dpi::PhysicalSize, 
    event::{ElementState, Event, KeyEvent, WindowEvent}, 
    event_loop::EventLoopBuilder, keyboard::{KeyCode, PhysicalKey}, 
    window::{Window, WindowBuilder}
};

use std::fs;
use std::path::Path;

use super::{scene, Scene};

struct State<'a> {
    // Device/Context objects
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: &'a Window,

    // Assets
    color_buffer: wgpu::Texture,
    color_buffer_view: TextureView,
    sampler: wgpu::Sampler,
    scene_parameters: wgpu::Buffer,
    sphere_buffer: wgpu::Buffer,
    node_buffer: wgpu::Buffer,
    sphere_index_buffer: wgpu::Buffer,

    // Pipeline Objects
    ray_tracing_pipeline: wgpu::ComputePipeline,
    ray_tracing_bind_group: wgpu::BindGroup,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,

    // Scene to render
    scene: Scene,
}

impl<'a> State<'a> {

    async fn new(window: &'a Window, scene: Scene) -> Self {

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

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f | f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        surface.configure(&device, &config);

        let (
            color_buffer, 
            color_buffer_view, 
            sampler, 
            scene_parameters, 
            sphere_buffer, 
            node_buffer, 
            sphere_index_buffer) = Self::create_assets(&device, &size, &scene).await;

        let (
            ray_tracing_pipeline, 
            ray_tracing_bind_group, 
            screen_pipeline, 
            screen_bind_group) = Self::make_pipeline(&device, &color_buffer_view, &sampler, &scene_parameters, &sphere_buffer, &node_buffer, &sphere_index_buffer).await;
        
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
            sphere_buffer,
            node_buffer,
            sphere_index_buffer,
            // Pipeline Objects
            ray_tracing_pipeline,
            ray_tracing_bind_group,
            screen_pipeline,
            screen_bind_group,
            // Scene to render
            scene,
        }
    }

    async fn create_assets(
        device: &wgpu::Device,
        size: &winit::dpi::PhysicalSize<u32>,
        scene: &Scene,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
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

        let parameter_buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("Parameter Buffer Descriptor"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let scene_parameters = device.create_buffer(&parameter_buffer_descriptor);

        let sphere_buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("Sphere Buffer Descriptor"),
            size: 32 * scene.spheres.len() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let sphere_buffer = device.create_buffer(&sphere_buffer_descriptor);

        let node_buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("Node Buffer Descriptor"),
            size: 32u64 * (scene.nodes_used as u64),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let node_buffer = device.create_buffer(&node_buffer_descriptor);

        let sphere_index_buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("Sphere Buffer Descriptor"),
            size: 4 * scene.spheres.len() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let sphere_index_buffer = device.create_buffer(&sphere_index_buffer_descriptor);

        // Return the created resources
        (color_buffer, color_buffer_view, sampler, scene_parameters, sphere_buffer, node_buffer, sphere_index_buffer)
    }    

    async fn make_pipeline(
        device: &wgpu::Device,
        color_buffer_view: &wgpu::TextureView,
        sampler: &Sampler,
        scene_parameters: &wgpu::Buffer,
        sphere_buffer: &wgpu::Buffer,
        node_buffer: &wgpu::Buffer,
        sphere_index_buffer: &wgpu::Buffer,
    ) -> (wgpu::ComputePipeline, wgpu::BindGroup, wgpu::RenderPipeline, wgpu::BindGroup) {
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
            ],
        };
        let ray_tracing_bind_group_layout = device.create_bind_group_layout(&ray_tracing_bind_group_layout_descriptor);

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
                        buffer: sphere_buffer,
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
                        buffer: sphere_index_buffer,
                        offset: 0,
                        size: None, // Use the entire buffer
                    }),
                },
            ],
        };
        let ray_tracing_bind_group = device.create_bind_group(&ray_tracing_bind_group_descriptor);

        let ray_tracing_pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Tracing Pipeline Layout"),
            bind_group_layouts: &[&ray_tracing_bind_group_layout],
            push_constant_ranges: &[], // No push constants used,
        };
        let ray_tracing_pipeline_layout = device.create_pipeline_layout(&ray_tracing_pipeline_layout_descriptor);

            // Create the shader module
            let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Ray Tracing Shader Module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/raytracer_kernel.wgsl").into()),
            });

        // Define the compute pipeline descriptor with the shader module and entry point
        let ray_tracing_pipeline_descriptor = wgpu::ComputePipelineDescriptor {
            label: Some("Ray Pipeline Descriptor"),
            layout: Some(&ray_tracing_pipeline_layout),
            module: &shader_module,
            entry_point: "main", // Entry point in the shader
        };

        // Create the compute pipeline
        let ray_tracing_pipeline = device.create_compute_pipeline(&ray_tracing_pipeline_descriptor);

        // ----------Screen pipeline---------- //
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
        

        let screen_pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pipeline Layout"),
            bind_group_layouts: &[&screen_bind_group_layout],
            push_constant_ranges: &[], // No push constants used,
        };
        let screen_pipeline_layout = device.create_pipeline_layout(&screen_pipeline_layout_descriptor);

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
        let screen_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some("Screen Pipeline Descriptor"),
            layout: Some(&screen_pipeline_layout),
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

        // Create the render pipeline
        let screen_pipeline = device.create_render_pipeline(&screen_pipeline_descriptor);

        // Return the created resources
        (ray_tracing_pipeline, ray_tracing_bind_group, screen_pipeline, screen_bind_group)
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>{
        let start_time = std::time::Instant::now();
        self.prepare_scene();

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
        ray_trace_pass.dispatch_workgroups(self.size.width, self.size.height, 1);
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

        let duration = start_time.elapsed(); // Calculate how long the rendering took

        let sphere_count = self.scene.spheres.len();

        println!("Rendered in {:?}, Sphere count: {}", duration, sphere_count);
        Ok(())
    }

    fn prepare_scene(&self) {
        let scene_data = self.scene.to_scene_data();
        let scene_data_flat: [f32; 16] = [
            scene_data.camera_pos.0,
            scene_data.camera_pos.1,
            scene_data.camera_pos.2,
            0.0, // Padding for alignment
            scene_data.camera_forwards.0,
            scene_data.camera_forwards.1,
            scene_data.camera_forwards.2,
            0.0, // Padding for alignment
            scene_data.camera_right.0,
            scene_data.camera_right.1,
            scene_data.camera_right.2,
            0.0, // Padding for alignment
            scene_data.camera_up.0,
            scene_data.camera_up.1,
            scene_data.camera_up.2,
            scene_data.sphere_count
        ];

        // Convert the f32 array to bytes
        let byte_data = bytemuck::cast_slice(&scene_data_flat);

        // Write to the buffer
        self.queue.write_buffer(
            &self.scene_parameters, 
            0,
            byte_data,
        );

        // Get sphere data in bytes
        let sphere_data_bytes = self.scene.flatten_sphere_data();

        // Write to the buffer
        self.queue.write_buffer(
            &self.sphere_buffer, // The wgpu::Buffer for sphere data
            0, // Offset within the buffer
            &sphere_data_bytes, // The byte slice containing the sphere data
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
        let sphere_index_data_bytes = self.scene.flatten_sphere_index_data();
        
        // Write to the buffer
        self.queue.write_buffer(
            &self.sphere_index_buffer, // The wgpu::Buffer for sphere_index data
            0, // Offset within the buffer
            &sphere_index_data_bytes, // The byte slice containing the sphere_index data
        );
    }
}

#[derive(Debug, Clone, Copy)]
enum CustomEvent {
    Timer,
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event()
        .build()
        .unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(17));
        event_loop_proxy.send_event(CustomEvent::Timer).ok();
    });

    let mut program_state: State<'_> = State::new(&window, Scene::new(81400)).await;

    event_loop.run(move | event, elwt | match event {
        Event::UserEvent(..) => {
            program_state.window.request_redraw();
            program_state.scene.update();
        },

        Event::WindowEvent { window_id, ref event } if window_id == program_state.window.id() => match event {
            WindowEvent::Resized(physical_size) => program_state.resize(*physical_size),

            WindowEvent::CloseRequested 
            | WindowEvent::KeyboardInput { 
                event: 
                    KeyEvent { 
                        physical_key: PhysicalKey::Code(KeyCode::Escape), 
                        state: ElementState::Pressed, repeat: false, .. }, .. } => {
                println!("Goodbye see you!");
                elwt.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                let key_code = match physical_key {
                    PhysicalKey::Code(code) => Some(code),
                    _ => None,
                };
                match state {
                    ElementState::Pressed => {
                        if let Some(code) = key_code {
                            program_state.scene.keys_pressed.insert(*code);
                        }
                    },
                    ElementState::Released => {
                        if let Some(code) = key_code {
                            program_state.scene.keys_pressed.remove(&code);
                        }
                    },
                }
            },                    

            WindowEvent::RedrawRequested => match program_state.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => program_state.resize(program_state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                Err(e) => eprintln!("{:?}", e),
            }

            _ => (),

        },

        _ => {},
    }).expect("Error!");
}
