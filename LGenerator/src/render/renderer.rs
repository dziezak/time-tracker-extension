use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::lsystem::turtle3d::Vertex;
use crate::render::camera::Camera3D;
use crate::render::petals::PetalSystem;
use crate::render::state::State;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformsGPU {
    mvp: [[f32; 4]; 4],
    time: f32,
    _padding: [f32; 3], 
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