struct Uniforms {
    mvp_matrix: mat4x4<f32>,
    time: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var pos = model.position;

    if (pos.y > 1.5) {
        let wind = sin(uniforms.time * 2.0 + pos.y * 0.8) * 0.12 * (pos.y * 0.1);
        pos.x += wind;
        pos.z += wind * 0.4;
    }

    out.clip_position = uniforms.mvp_matrix * vec4<f32>(pos, 1.0);
    out.color = model.color;
    out.normal = model.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.4, 1.0, 0.5));
    let ambient = 0.25;
    let diffuse = max(dot(normalize(in.normal), light_dir), 0.0);

    let light_factor = ambient + diffuse;
    let final_rgb = in.color.rgb * light_factor;

    return vec4<f32>(final_rgb, in.color.a);
}