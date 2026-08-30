use glam::{Vec3, Quat};
use crate::data::parser::DomainData;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: 24, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}

#[derive(Clone)]
struct TurtleState {
    position: Vec3,
    rotation: Quat,
    thickness: f32,
    length: f32,
}

pub struct Turtle3D {
    state: TurtleState,
    stack: Vec<TurtleState>,
}

impl Turtle3D {
    pub fn new() -> Self {
        Self {
            state: TurtleState {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                thickness: 0.35,
                length: 1.5,
            },
            stack: Vec::new(),
        }
    }

    pub fn generate_tree_mesh(
        &mut self,
        axiom_sequence: &str,
        domains: &[DomainData],
        base_angle_rad: f32,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(axiom_sequence.len() * 8);
        let mut indices = Vec::with_capacity(axiom_sequence.len() * 12);

        self.state = TurtleState {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            thickness: 0.35,
            length: 1.0,
        };
        self.stack.clear();

        let mut current_domain_idx: usize = 0;
        let mut domain_stack: Vec<usize> = Vec::new();
        let mut branch_counter: usize = 0;
        let mut leaf_counter: usize = 0; // <-- Dodany licznik wariacji liści

        for ch in axiom_sequence.chars() {
            match ch {
                'F' => {
                    let forward = self.state.rotation * Vec3::Y;
                    let next_pos = self.state.position + forward * self.state.length;

                    self.append_cylinder(
                        &mut vertices,
                        &mut indices,
                        self.state.position,
                        next_pos,
                        self.state.thickness,
                    );
                    self.state.position = next_pos;
                }
                '+' => self.state.rotation = self.state.rotation * Quat::from_rotation_z(base_angle_rad),
                '-' => self.state.rotation = self.state.rotation * Quat::from_rotation_z(-base_angle_rad),
                '&' => self.state.rotation = self.state.rotation * Quat::from_rotation_x(base_angle_rad),
                '^' => self.state.rotation = self.state.rotation * Quat::from_rotation_x(-base_angle_rad),
                '/' => self.state.rotation = self.state.rotation * Quat::from_rotation_y(base_angle_rad),
                '\\' => self.state.rotation = self.state.rotation * Quat::from_rotation_y(-base_angle_rad),

                '[' => {
                    self.stack.push(self.state.clone());
                    domain_stack.push(current_domain_idx);

                    branch_counter += 1;
                    current_domain_idx = branch_counter;

                    self.state.thickness *= 0.68;
                    self.state.length *= 0.72;
                }
                ']' => {
                    if let Some(saved) = self.stack.pop() {
                        self.state = saved;
                    }
                    if let Some(prev_domain) = domain_stack.pop() {
                        current_domain_idx = prev_domain;
                    }
                }
                'J' => {
                    let fallback = DomainData {
                        name: format!("domain_{}", current_domain_idx),
                        seconds: 600,
                        weight: 0.5,
                        color: [0.2, 0.8, 0.3, 1.0],
                    };

                    let domain = if domains.is_empty() {
                        &fallback
                    } else {
                        &domains[current_domain_idx % domains.len()]
                    };

                    // Przekazujemy leaf_counter jako 5. argument
                    self.append_leaf(&mut vertices, &mut indices, self.state.position, domain, leaf_counter);
                    leaf_counter += 1;
                }
                _ => {}
            }
        }

        (vertices, indices)
    }

    fn append_cylinder(&self, verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, start: Vec3, end: Vec3, radius: f32) {
        let base_idx = verts.len() as u32;
        let sides = 8;
        let dir = (end - start).normalize();
        let up = if dir.y.abs() > 0.99 { Vec3::X } else { Vec3::Y };
        let right = dir.cross(up).normalize();
        let forward = right.cross(dir).normalize();

        for i in 0..sides {
            let a = (i as f32 / sides as f32) * std::f32::consts::TAU;
            let norm = right * a.cos() + forward * a.sin();
            let color = [0.35, 0.22, 0.12, 1.0]; // Kolor pnia

            verts.push(Vertex { position: (start + norm * radius).to_array(), normal: norm.to_array(), color });
            verts.push(Vertex { position: (end + norm * (radius * 0.75)).to_array(), normal: norm.to_array(), color });

            let curr = base_idx + i * 2;
            let next = base_idx + ((i + 1) % sides) * 2;
            inds.extend_from_slice(&[curr, next, curr + 1, next, next + 1, curr + 1]);
        }
    }

    pub fn append_leaf(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        pos: glam::Vec3,
        domain: &DomainData,
        leaf_variant: usize,
    ) {
        let base_index = vertices.len() as u32;

        let scale = 0.08 + domain.weight * 0.12;

        let mut color = domain.color;
        let brightness_offset = ((leaf_variant % 5) as f32 - 2.0) * 0.04;
        color[0] = (color[0] + brightness_offset).clamp(0.0, 1.0);
        color[1] = (color[1] + brightness_offset).clamp(0.0, 1.0);
        color[2] = (color[2] + brightness_offset).clamp(0.0, 1.0);

        let up = self.state.rotation * glam::Vec3::Y * scale;
        let right = self.state.rotation * glam::Vec3::X * (scale * 0.6);
        let normal = (self.state.rotation * glam::Vec3::Z).normalize().into();

        let p0 = pos - right;
        let p1 = pos + up;
        let p2 = pos + right;
        let p3 = pos - up * 0.4;

        vertices.push(Vertex { position: p0.into(), normal, color });
        vertices.push(Vertex { position: p1.into(), normal, color });
        vertices.push(Vertex { position: p2.into(), normal, color });
        vertices.push(Vertex { position: p3.into(), normal, color });

        indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
            base_index, base_index + 2, base_index + 1,
            base_index, base_index + 3, base_index + 2,
        ]);
    }
}

pub fn domain_to_rgb(domain_name: &str) -> [f32; 4] {
    let mut hash: u32 = 0;
    for byte in domain_name.bytes() {
        let b = byte as u32;
        hash = b.wrapping_add(hash.wrapping_shl(5).wrapping_sub(hash));
    }

    let hue = (hash % 360) as f32;
    let [r, g, b] = hsl_to_rgb(hue, 0.75, 0.55);
    [r, g, b, 1.0] // Zwracamy tablicę 4-elementową (RGBA)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    [r + m, g + m, b + m]
}