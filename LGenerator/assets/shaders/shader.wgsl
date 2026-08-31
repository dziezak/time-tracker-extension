struct UniformsGPU {
    mvp: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: UniformsGPU;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) object_type: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) object_type: f32,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = model.position;
    out.normal = model.normal;
    out.color = model.color;
    out.uv = model.uv;
    out.object_type = model.object_type;
    out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.3, 0.9, 0.4));
    // Używamy pozycji z Uniforms zamiast wartości na sztywno
    let view_dir = normalize(uniforms.camera_pos - in.world_pos);

    // ==========================================
    // 1. TAFLA WODY (object_type == 2.0)
    // ==========================================
    if (in.object_type > 1.5) {
        let wave1 = sin(in.world_pos.x * 2.5 + uniforms.time * 2.0) * 0.15;
        let wave2 = cos(in.world_pos.z * 3.0 + uniforms.time * 1.5) * 0.15;
        let wave_normal = normalize(vec3<f32>(wave1, 1.0, wave2));

        let half_dir = normalize(light_dir + view_dir);
        let spec_specular = pow(max(dot(wave_normal, half_dir), 0.0), 64.0);
        let fresnel = pow(1.0 - max(dot(view_dir, wave_normal), 0.0), 2.0);

        let deep_water = vec3<f32>(0.05, 0.18, 0.35);
        let sky_reflection = vec3<f32>(0.45, 0.65, 0.85);

        let water_color = mix(deep_water, sky_reflection, fresnel * 0.8);
        let final_water = water_color + vec3<f32>(1.0, 0.95, 0.8) * spec_specular * 2.0;

        return vec4<f32>(final_water, 0.9);
    }

    // ==========================================
    // 2. KORA DREWNA (object_type == 0.0)
    // ==========================================
    var base_color = in.color;
    if (in.object_type < 0.5) {
        let bark_pattern = sin(in.uv.x * 25.0 + in.world_pos.y * 6.0) * 0.1;
        let noise = (hash(floor(in.uv * 40.0)) - 0.5) * 0.06;
        base_color = vec4<f32>(
            clamp(in.color.rgb + vec3<f32>(bark_pattern + noise), vec3<f32>(0.0), vec3<f32>(1.0)),
            in.color.a
        );
    }
    // ==========================================
    // 3. LIŚCIE I PŁATKI (object_type == 1.0)
    // ==========================================
    else {
        let center_vein = smoothstep(0.0f, 0.06f, abs(in.uv.x - 0.5f));
        let vein_darkness = mix(0.75f, 1.0f, center_vein);
        base_color = vec4<f32>(in.color.rgb * vein_darkness, in.color.a);
    }

    let N = normalize(in.normal);
    let diff = max(dot(N, light_dir), 0.35); // Ambient

    return vec4<f32>(base_color.rgb * diff, base_color.a);
}