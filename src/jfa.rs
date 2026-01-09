use std::sync::Arc;
use wgpu::{
    self,
    util::{DeviceExt, RenderEncoder},
    BindGroup, Buffer, CommandEncoder, Device, Queue, RenderPipeline, Surface,
    SurfaceConfiguration, TextureView,
};
use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode::*, PhysicalKey},
    window::Window,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1.],
    },
    Vertex {
        position: [-1., -3.],
    },
    Vertex { position: [3., 1.] },
];

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MouseUniform {
    pos: [f32; 2],
}

macro_rules! shader {
    ($name:expr) => {
        wgpu::ShaderModuleDescriptor {
            label: Some($name),
            source: wgpu::ShaderSource::Wgsl(include_str!($name).into()),
        }
    };
}

pub struct State {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    mouse_uniform: MouseUniform,
    mouse_buffer: Buffer,
    mouse_bind_group: BindGroup,
    dimensions_buffer: Buffer,
    dimensions_bind_group: BindGroup,
    step_bind_group: BindGroup,
    step_buffer: Buffer,
    // pixel_buffer: Buffer,
    pub size: winit::dpi::PhysicalSize<u32>,
    texture_a: wgpu::Texture,
    texture_b: wgpu::Texture,
    texture_a_view: wgpu::TextureView,
    texture_b_view: wgpu::TextureView,
    texture_a_bind_group: BindGroup,
    texture_b_bind_group: BindGroup,
    initial_render_pipeline: wgpu::RenderPipeline,
    jfa_render_pipeline: wgpu::RenderPipeline,
    final_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Buffer,
    clear_color: wgpu::Color,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Arc<Window>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> State {
        let size = window.inner_size();

        let window = Arc::new(window);

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Some(adapter) => adapter,
            None => {
                instance
                    .enumerate_adapters(wgpu::Backends::all())
                    .into_iter()
                    .filter(|adapter| {
                        // Check if this adapter supports our surface
                        adapter.is_surface_supported(&surface)
                    })
                    .next()
                    .unwrap()
            }
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // Assumes an sRGB surface texture. Using a different one will result in all
        // the colors coming out darker. If you want to support non sRGB surfaces,
        // you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // ------
        // Unfiforms/Buffers and Bind Groups for the initial render
        // ------
        let mouse_uniform = MouseUniform { pos: [0., 0.] };

        let mouse_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mouse Buffer"),
            contents: bytemuck::cast_slice(&[mouse_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mouse_bind_group_layout = create_uniform_bind_group_layout(
            &device,
            "Mouse BGL",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let mouse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mouse_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mouse_buffer.as_entire_binding(),
            }],
            label: Some("mouse_bind_group"),
        });

        let dimensions_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mouse Buffer"),
            contents: bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let dimensions_bind_group_layout = create_uniform_bind_group_layout(
            &device,
            "Dimensions BGL",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );

        let dimensions_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &dimensions_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: dimensions_buffer.as_entire_binding(),
            }],
            label: Some("dimensions_bind_group"),
        });

        // ------
        // Ping Pong Textures
        // ------
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            label: Some("Ping Pong Texture"),
        };

        let texture_a = device.create_texture(&texture_desc);
        let texture_b = device.create_texture(&texture_desc);

        let texture_a_view = texture_a.create_view(&Default::default());
        let texture_b_view = texture_b.create_view(&Default::default());

        let jfa_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge, // NOTE: Could be interesting to repeat
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let jfa_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("sdf_bind_group_layout"),
            });

        let texture_a_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &jfa_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_a_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&jfa_sampler),
                },
            ],
            label: Some("ping_bind_group"),
        });

        let texture_b_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &jfa_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_b_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&jfa_sampler),
                },
            ],
            label: Some("pong_bind_group"),
        });

        // ------
        // Step Bind Group Layout
        // ------
        let step_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Step Buffer"),
            contents: bytemuck::cast_slice(&[0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let step_bind_group_layout = create_uniform_bind_group_layout(
            &device,
            "step_bind_group_layout",
            wgpu::ShaderStages::FRAGMENT,
        );

        let step_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &step_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: step_buffer.as_entire_binding(),
            }],
            label: Some("step_bind_group"),
        });

        // ------
        // Initial Drawing of Seeds
        // ------
        let clear_color = wgpu::Color::BLUE;

        let initial_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Initial Render Pipeline Layout"),
                bind_group_layouts: &[&mouse_bind_group_layout, &dimensions_bind_group_layout],
                push_constant_ranges: &[],
            });

        let initial_render_pipeline = {
            create_render_pipeline(
                "Initial Render Pipeline",
                &device,
                &initial_render_pipeline_layout,
                wgpu::TextureFormat::Rgba8Unorm,
                None,
                &[Vertex::desc()],
                shader!("seed.wgsl"),
                wgpu::PrimitiveTopology::TriangleList,
            )
        };

        // ------
        // JFA Render Pipeline
        // ------

        let jfa_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("JFA Render Pipeline Layout"),
                bind_group_layouts: &[&jfa_bind_group_layout, &step_bind_group_layout],
                push_constant_ranges: &[],
            });

        let jfa_render_pipeline = {
            create_render_pipeline(
                "JFA Render Pipeline",
                &device,
                &jfa_render_pipeline_layout,
                wgpu::TextureFormat::Rgba8Unorm,
                None,
                &[Vertex::desc()],
                shader!("jfa.wgsl"),
                wgpu::PrimitiveTopology::TriangleList,
            )
        };

        // ------
        // Final Drawing to Window
        // ------

        let final_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Final Render Pipeline Layout"),
                bind_group_layouts: &[&jfa_bind_group_layout],
                push_constant_ranges: &[],
            });

        let final_render_pipeline = {
            create_render_pipeline(
                "Final Render Pipeline",
                &device,
                &final_render_pipeline_layout,
                config.format,
                None,
                &[Vertex::desc()],
                shader!("final.wgsl"),
                wgpu::PrimitiveTopology::TriangleList,
            )
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            mouse_uniform,
            mouse_buffer,
            mouse_bind_group,
            dimensions_buffer,
            dimensions_bind_group,
            step_buffer,
            step_bind_group,
            size,
            texture_a,
            texture_b,
            texture_a_view,
            texture_b_view,
            texture_a_bind_group,
            texture_b_bind_group,
            initial_render_pipeline,
            jfa_render_pipeline,
            final_render_pipeline,
            vertex_buffer,
            clear_color,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // BUG: Resize doesn't really work for now
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;

            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.queue.write_buffer(
                &self.dimensions_buffer,
                0,
                bytemuck::cast_slice(&[new_size.width as f32, new_size.height as f32]),
            );

            let mut pixel_coords = Vec::with_capacity((new_size.width * new_size.height) as usize);
            for y in 0..new_size.height {
                for x in 0..new_size.width {
                    pixel_coords.push([x as f32, y as f32]);
                }
            }

            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) {
        // println!("{:?}", event);
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = match event.state == ElementState::Pressed {
                    true => 1.0,
                    false => 0.0,
                };

                match event.physical_key {
                    PhysicalKey::Code(key) => match key {
                        KeyT => {}
                        _ => (),
                    },
                    _ => (),
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {}
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_uniform.pos[0] = position.x as f32;
                self.mouse_uniform.pos[1] = position.y as f32;

                self.queue.write_buffer(
                    &self.mouse_buffer,
                    0,
                    bytemuck::cast_slice(&[self.mouse_uniform]),
                );
                self.update();
            }
            _ => (),
        }
    }

    pub fn update(&mut self) {
        self.window().request_redraw();
    }

    fn new_encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            })
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // ------
        // SEED
        // ------

        let mut encoder = self.new_encoder();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_a_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.initial_render_pipeline);
            render_pass.set_bind_group(0, &self.mouse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.dimensions_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        // ------
        // JFA
        // ------

        // let count = self.config.height.isqrt();
        let mut step = 1f32;

        let mut ping = (&self.texture_a_bind_group, &self.texture_a_view);
        let mut pong = (&self.texture_b_bind_group, &self.texture_b_view);

        let mut i = 0;
        while i < 8 {
            step = step / 2.;
            self.queue
                .write_buffer(&self.step_buffer, 0, bytemuck::cast_slice(&[step]));
            let mut encoder = self.new_encoder();
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("JFA Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: pong.1,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.jfa_render_pipeline);
                render_pass.set_bind_group(0, ping.0, &[]);
                render_pass.set_bind_group(1, &self.step_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..3, 0..1);
            }

            let tmp = ping;
            ping = pong;
            pong = tmp;
            i += 1;
            self.queue.submit(std::iter::once(encoder.finish()));
        }

        // ------
        // Draw to the window
        // ------

        let output = self.surface.get_current_texture()?;
        let view = output // NOTE: Does this need to be recreated every time?
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.new_encoder();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.final_render_pipeline);
            render_pass.set_bind_group(0, ping.0, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn create_render_pipeline(
    label: &str,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        cache: None,
        multiview: None,
    })
}

fn create_uniform_bind_group_layout(
    device: &wgpu::Device,
    label: &str,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(label),
    })
}
