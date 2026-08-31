use glam::Vec3;
use rand::{Rng};
use crate::lsystem::turtle3d::Vertex;

pub struct Petal{
    pub position: Vec3,
    pub velocity: Vec3, 
    pub rotation_speed: f32,
    pub touch_phase: f32,
    pub size: f32,
    pub phase: f32,
    pub rotation: Vec3,
    pub is_on_ground: bool,
    pub ground_timer: f32,
}

pub  struct PetalSystem {
    pub petals: Vec<Petal>,
    pub static_petals: Vec<Petal>,
    pub ground_height: f32,
    pub stay_duration: f32,
}

impl PetalSystem {
    pub fn new(count: usize, static_count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut petals = Vec::with_capacity(count);

        let ground_count = (count as f32 * 0.4) as usize;
        let falling_count = count - ground_count;

        let mut static_petals = Vec::with_capacity(static_count);

        for _ in 0..static_count {
            let radius = rng.gen_range(0.8..14.0);
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);

            static_petals.push(Petal {
                position: Vec3::new(
                    radius * angle.cos(),
                    0.0,
                    radius * angle.sin(),
                ),
                velocity: Vec3::ZERO,
                rotation: Vec3::new(
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                ),
                rotation_speed: 0.0,
                touch_phase: 0.0,
                size: rng.gen_range(0.18..0.35),
                phase: 0.0,
                is_on_ground: true,
                ground_timer: 0.0,
            });
        }

        for _ in 0..count {
            let radius = rng.gen_range(1.5..12.0);
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);


            petals.push(Petal {
                position: Vec3::new(
                    radius * angle.cos(),
                    0.0,
                    radius * angle.sin(),
                ),
                velocity: Vec3::ZERO,
                rotation: Vec3::new(
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                ),
                rotation_speed: 0.0, // rng.gen_range(1.0..3.0),
                touch_phase: 0.0,
                size: rng.gen_range(0.15..0.3),
                phase: 0.0,
                is_on_ground: true,
                ground_timer: rng.gen_range(0.0..8.0),
            });
        }

        for _ in 0..falling_count {
            let radius = rng.gen_range(1.0..15.0);
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);

            petals.push(Petal {
                position: Vec3::new(
                    radius * angle.cos(),
                    rng.gen_range(2.0..25.0),
                    radius * angle.sin(),
                ),
                velocity: Vec3::new(
                    rng.gen_range(-0.3..0.3),
                    rng.gen_range(-1.2..-0.4),
                    rng.gen_range(-0.3..0.3),
                ),
                rotation: Vec3::new(
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                    rng.gen_range(0.0..std::f32::consts::TAU),
                ),
                rotation_speed: rng.gen_range(1.0..3.0),
                touch_phase: 0.0,
                size: rng.gen_range(0.15..0.3),
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
                is_on_ground: false,
                ground_timer: 0.0,
            });
        }

        Self
        {
            petals,
            static_petals,
            ground_height: 0.0,
            stay_duration: 10.0,
        }
    }

    pub fn update(&mut self, dt: f32, time: f32) {
        let mut rng = rand::thread_rng();

        for petal in &mut self.petals {
            if petal.is_on_ground {
                petal.ground_timer += dt;

                if petal.ground_timer >= self.stay_duration {
                    let radius = rng.gen_range(1.0..15.0);
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);

                    petal.position = Vec3::new(
                        radius * angle.cos(),
                        rng.gen_range(25.0..35.0),
                        radius * angle.sin(),
                    );
                    petal.velocity = Vec3::new(
                        rng.gen_range(-0.3..0.3),
                        rng.gen_range(-1.2..-0.4),
                        rng.gen_range(-0.3..0.3),
                    );
                    petal.is_on_ground = false;
                    petal.ground_timer = 0.0;
                }
            } else {
                petal.position.y += petal.velocity.y * dt;
                petal.position.x += (time * petal.rotation_speed + petal.phase).sin() * 0.8 * dt;
                petal.position.z += (time * petal.rotation_speed * 0.7 + petal.phase).cos() * 0.5 * dt;

                if petal.position.y <= self.ground_height {
                    petal.position.y = self.ground_height;
                    petal.is_on_ground = true;
                    petal.ground_timer = 0.0;
                }
            }
        }
    }

    pub fn to_vertices(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(self.petals.len() * 3);
        let mut indices = Vec::with_capacity(self.petals.len() * 3);

        let all_petals = self.petals.iter().chain(self.static_petals.iter());

        for (i, petal) in all_petals.enumerate() {
            let base_idx = (i * 3) as u32;
            let p = petal.position;
            let s = petal.size;

            let alpha = if petal.is_on_ground && petal.ground_timer > (self.stay_duration - 2.0){
                ((self.stay_duration - petal.ground_timer) * 0.5).clamp(0.0, 1.0) * 0.9
            } else {
                0.9
            };

            let pink_color = [0.98, 0.65, 0.78, alpha];
            let normal = [0.0, 1.0, 0.0];
            let object_type = 1.0;
            let tangent = [1.0, 0.0, 0.0];

            vertices.push(Vertex {
                position: [p.x, p.y + s, p.z],
                normal: [0.0, 1.0, 0.0],
                color: pink_color,
                uv: [0.5, 1.0],
                object_type,
                tangent,
            });
            vertices.push(Vertex {
                position: [p.x - s, p.y - s, p.z + s * 0.5],
                normal: [0.0, 1.0, 0.0],
                color: pink_color,
                uv: [0.0, 0.0],
                object_type,
                tangent,
            });
            vertices.push(Vertex {
                position: [p.x + s, p.y - s, p.z - s * 0.5],
                normal: [0.0, 1.0, 0.0],
                color: pink_color,
                uv: [1.0, 0.0],
                object_type,
                tangent,
            });

            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        }

        (vertices, indices)
    }
}