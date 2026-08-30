mod data;
mod lsystem;
mod render;

use crate::lsystem::generator::build_tree_sequence;
use crate::lsystem::turtle3d::Turtle3D;
use crate::render::renderer::run_app;

fn main() {
    let domains = data::parser::parse_domains("assets/data.json", 8);
    let sequence = build_tree_sequence(3);
    //TODO: uzyj funkcji build_tree form domains

    let mut turtle = Turtle3D::new();
    let (vertices, indices) = turtle.generate_tree_mesh(
        &sequence,
        &domains,
        35.0_f32.to_radians(),
    );

    pollster::block_on(run_app(vertices, indices));
}