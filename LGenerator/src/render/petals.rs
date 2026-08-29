use bytemuck::__core;
use glam::Vec3;
use rand::{Rng};
use serde::de::Unexpected::Str;
use wgpu::core::validation::FilteringError::Integer;
use wgpu::naga::Expression::ArrayLength;
use crate::lsystem::turtle3d::Vertex;

pub struct Petal{
    pub position: Vec3,
    pub velocity: Vec3, 
    pub rotation_speed: f32,
    pub touch_phase: f32,
    pub size: f32,
    pub phase: f32,
}

pub  struct PetalSystem {
    pub petals: Vec<Petal>,
}

impl PetalSystem {
    pub fn new(count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut petals = Vec::with_capacity(count);

        for _ in 0..count {
            petals.push(Petal {
                position: Vec3::new(
                    rng.gen_range(-25.0..25.0),
                    rng.gen_range(5.0..40.0),
                    rng.gen_range(-25.0..25.0),
                ),
                velocity: Vec3::new(
                    rng.gen_range(-0.5..5.0),
                    rng.gen_range(-1.5..-0.5),
                    rng.gen_range(-0.5..0.5),
                ),
                rotation_speed: rng.gen_range(1.0..3.0),
                touch_phase: rng.gen_range(0.0..std::f32::consts::TAU),
                size: rng.gen_range(0.15..0.3),
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
            });
        }
       Self { petals } 
    }

    pub fn update(&mut self, dt: f32, time: f32) {
        let mut rng = rand::thread_rng();

        for petal in &mut self.petals {
            petal.position.y += petal.velocity.y * dt;
            petal.position.x += (time * petal.rotation_speed + petal.phase).sin() * 0.8 * dt;
            petal.position.z += (time * petal.rotation_speed * 0.7 + petal.phase).cos() * 0.5 * dt;

            // Gdy płatek spadnie poniżej podłoża (Y < 0.0), responuje się na górze
            if petal.position.y < 0.0 {
                petal.position.y = rng.gen_range(30.0..40.0);
                petal.position.x = rng.gen_range(-25.0..25.0);
                petal.position.z = rng.gen_range(-25.0..25.0);
            }
        }
    }

    pub fn to_vertices(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(self.petals.len() * 3);
        let mut indices = Vec::with_capacity(self.petals.len() * 3);

        let pink_color = [0.98, 0.65, 0.78, 0.9];

        for (i, petal) in self.petals.iter().enumerate() {
            let base_idx = (i * 3) as u32;
            let p = petal.position;
            let s = petal.size;

            vertices.push(Vertex {
                position: [p.x, p.y + s, p.z],
                normal: [0.0, 1.0, 0.0],
                color: pink_color,
            });
            vertices.push(Vertex {
                position: [p.x - s, p.y - s, p.z + s * 0.5],
                normal: [0.0, 1.0, 0.0],
                color: pink_color,
            });
            vertices.push(Vertex {
                position: [p.x + s, p.y - s, p.z - s * 0.5],
                normal: [0.0, 1.0, 0.0],
                color: pink_color,
            });

            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        }

        (vertices, indices)
    }
}