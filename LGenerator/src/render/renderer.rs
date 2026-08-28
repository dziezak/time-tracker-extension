use std::sync::Arc;
use winit::window::Window;
use crate::lsystem::turtle3d::Vertex;
use crate::render::camera::Camera3D;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformsGPU {
    mvp: [[f32; 4]; 4],
    time: f32,
    _padding: [f32; 3], // Wygładzenie pod std140 ułożenie pamięci w GPU
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
    camera: Camera3D,
    start_time: std::time::Instant,
}

impl State {
    pub async fn new(window: Arc<Window>, vertices: &[Vertex], indices: &[u32]) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Bufory Geometrii
        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Kamera i Uniformy
        let camera = Camera3D {
            eye: (0.0, 10.0, 45.0).into(),
            target: (0.0, 12.0, 0.0).into(),
            up: glam::Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fov: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };

        let initial_mvp = camera.build_view_projection_matrix();
        let uniforms_gpu = UniformsGPU {
            mvp: initial_mvp.to_cols_array_2d(),
            time: 0.0,
            _padding: [0.0; 3],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms_gpu]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        // Shader i Pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/tree.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniform_buffer,
            uniform_bind_group,
            depth_texture_view,
            camera,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let time = self.start_time.elapsed().as_secs_f32();

        // Aktualizacja pozycji kamery (obrót wokół sceny)
        self.camera.update_orbit(time * 0.2, 45.0);
        let mvp = self.camera.build_view_projection_matrix();

        let updated_uniforms = UniformsGPU {
            mvp: mvp.to_cols_array_2d(),
            time,
            _padding: [0.0; 3],
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[updated_uniforms]));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.03, g: 0.04, b: 0.07, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self) {
        // Logika aktualizacji klatki (np. obrót kamery), jeśli potrzebna
    }
}

pub async fn run_app(vertices: Vec<Vertex>, indices: Vec<u32>) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = std::sync::Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("L-System 3D Tree")
            .build(&event_loop)
            .unwrap(),
    );

    let mut state = State::new(window.clone(), &vertices, &indices).await;

    event_loop
        .run(move |event, elwt| match event {
            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                winit::event::WindowEvent::CloseRequested => elwt.exit(),
                winit::event::WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = window.inner_size();
                            state.resize(size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            },
            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        })
        .unwrap();
}