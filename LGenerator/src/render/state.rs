use std::sync::Arc;
use winit::window::Window;
use crate::lsystem::turtle3d::Vertex;
use crate::render::camera::Camera3D;
use crate::render::depth::DepthTexture;
use crate::render::petals::PetalSystem;
use crate::render::mesh_buffer::GpuMeshBuffer;
use crate::render::pipeline::{RenderContext};
use crate::render::renderer::UniformsGPU;

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_ctx: RenderContext,
    tree_mesh: GpuMeshBuffer,
    petal_mesh: GpuMeshBuffer,
    petal_system: PetalSystem,
    camera: Camera3D,
    start_time: std::time::Instant,
    last_update_time: std::time::Instant,
    rotation: f32,
    depth_texture: DepthTexture,
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

        let camera = Camera3D {
            eye: (0.0, 25.0, 25.0).into(),
            target: (0.0, 10.0, 0.0).into(),
            up: glam::Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fov: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };

        let mvp = camera.build_view_projection_matrix().to_cols_array_2d();
        let render_ctx = RenderContext::new(&device, &queue, config.format, mvp);

        let tree_mesh = GpuMeshBuffer::new(
            &device,
            "Tree",
            vertices,
            indices,
            wgpu::BufferUsages::empty(),
        );

        let petal_system = PetalSystem::new(300, 800);
        let (petal_vertices, petal_indices) = petal_system.to_vertices();

        let petal_mesh = GpuMeshBuffer::new(
            &device,
            "Petal",
            &petal_vertices,
            &petal_indices,
            wgpu::BufferUsages::COPY_DST,
        );

        let depth_texture = DepthTexture::create_depth_texture(&device, &config, "Depth Texture");

        Self {
            surface,
            device,
            queue,
            config,
            render_ctx,
            tree_mesh,
            petal_mesh,
            petal_system,
            camera,
            start_time: std::time::Instant::now(),
            last_update_time: std::time::Instant::now(),
            rotation: 0.0,
            depth_texture,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let time = self.start_time.elapsed().as_secs_f32();
        let mvp = self.camera.build_view_projection_matrix();
        let camera_pos = self.camera.eye.to_array();

        let updated_uniforms = UniformsGPU {
            mvp: mvp.to_cols_array_2d(),
            camera_pos,
            time,
            light_pos: [10.0, 20.0, 10.0],
            _padding1: 0.0,
            light_color: [1.0, 0.95, 0.8],
            _padding2: 0.0,
        };
        self.queue.write_buffer(&self.render_ctx.uniform_buffer, 0, bytemuck::cast_slice(&[updated_uniforms]));

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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_ctx.pipeline);
            render_pass.set_bind_group(0, &self.render_ctx.uniform_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.tree_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.tree_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.tree_mesh.index_count, 0, 0..1);

            render_pass.set_vertex_buffer(0, self.petal_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.petal_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.petal_mesh.index_count, 0, 0..1);
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
            self.depth_texture = DepthTexture::create_depth_texture(&self.device, &self.config, "Depth");
        }
    }

    pub fn update(&mut self) {
        self.rotation += 0.1;
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_update_time).as_secs_f32();
        self.last_update_time = now;

        let time = self.start_time.elapsed().as_secs_f32();
        self.petal_system.update(dt, time);

        let (vertices, indices) = self.petal_system.to_vertices();
        self.petal_mesh.update_data(&self.queue, &vertices, &indices);
    }
}