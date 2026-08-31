mod data;
mod lsystem;
mod render;

use crate::lsystem::generator::build_tree_sequence;
use crate::lsystem::turtle3d::{create_water_plane, Turtle3D};
use crate::render::renderer::run_app;

fn main() {
    let domains = data::parser::parse_domains("assets/data.json", 2);
    let sequence = build_tree_sequence(3);
    //TODO: uzyj funkcji build_tree form domains

    let mut turtle = Turtle3D::new();
    let (mut vertices, mut indices) = turtle.generate_tree_mesh(
        &sequence,
        &domains,
        35.0_f32.to_radians(),
    );
    let (water_verts, water_indices) = create_water_plane(100.0, -0.2);
    let offset = vertices.len() as u32;
    vertices.extend(water_verts);
    indices.extend(water_indices.into_iter().map(|i| offset + i));

    pollster::block_on(run_app(vertices, indices));
}