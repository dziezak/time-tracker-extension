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
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut domain_idx = 0;

        for ch in axiom_sequence.chars() {
            match ch {
                'F' => {
                    let forward = self.state.rotation * Vec3::Y;
                    let next_pos = self.state.position + forward * self.state.length;

                    // Budujemy segment pnia/gałęzi
                    self.append_cylinder(&mut vertices, &mut indices, self.state.position, next_pos, self.state.thickness);
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
                    self.state.thickness *= 0.65;
                    self.state.length *= 0.58;
                }
                ']' => {
                    if let Some(saved) = self.stack.pop() {
                        self.state = saved;
                    }
                }
                'J' => {
                    // Utwórz domyślną domenę, jeśli lista z parsera jest pusta
                    let fallback = DomainData {
                        name: "localhost".to_string(),
                        seconds: 3600,
                        hue: 120.0,
                        weight: 1.0,
                    };

                    let domain = if domains.is_empty() {
                        &fallback
                    } else {
                        &domains[domain_idx % domains.len()]
                    };

                    self.append_leaf(&mut vertices, &mut indices, self.state.position, domain);
                    domain_idx += 1;
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
    ) {
        let base_index = vertices.len() as u32;
        let color = domain_to_rgb(&domain.name);

        // Zabezpieczenie skali: bezpieczny log1p, aby dla seconds >= 0 otrzymywać wartości > 0
        let scale = ((domain.seconds as f32 + 1.0).ln() * 0.15).max(0.12);

        let up = self.state.rotation * glam::Vec3::Y * scale;
        let right = self.state.rotation * glam::Vec3::X * scale;

        let normal = (self.state.rotation * glam::Vec3::Z).normalize().into();

        // Wierzchołki płaszczyzny liścia
        vertices.push(Vertex { position: (pos - right).into(), normal, color }); // 0
        vertices.push(Vertex { position: (pos + up).into(), normal, color });    // 1
        vertices.push(Vertex { position: (pos + right).into(), normal, color }); // 2
        vertices.push(Vertex { position: (pos - up).into(), normal, color });    // 3

        // Strona przednia (Front Face - CCW)
        indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ]);

        // Strona tylna (Back Face - CW) — rysuje liść widoczny z drugiej strony bez wyłączania cull_mode w pipeline
        indices.extend_from_slice(&[
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