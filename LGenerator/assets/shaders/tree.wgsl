struct UniformsGPU {
    mvp: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    light_pos: vec3<f32>,
    _padding1: f32,
    light_color: vec3<f32>,
    _padding2: f32,
};

@group(0) @binding(0) var<uniform> uniforms: UniformsGPU;
@group(0) @binding(1) var t_normal: texture_2d<f32>;
@group(0) @binding(2) var s_normal: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) object_type: f32,
    @location(5) tangent: vec4<f32>, // Wektor stycznej przesyłany z Rust (wgpu)
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) object_type: f32,
    @location(5) tangent: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = model.position;
    out.normal = model.normal;
    out.color = model.color;
    out.uv = model.uv;
    out.object_type = model.object_type;
    out.tangent = model.tangent.xyz;
    out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(uniforms.light_pos - in.world_pos);
    let view_dir = normalize(uniforms.camera_pos - in.world_pos);
    let raw_normal: vec4<f32> = textureSample(t_normal, s_normal, in.uv);

    // Domyślna normalna geometryczna wierzchołka
    var N = normalize(in.normal);

    // ==========================================
    // 1. TAFLA WODY (object_type == 2.0)
    // ==========================================
    if (in.object_type > 1.5) {
        let tree_center = vec2<f32>(0.0, 0.0);
        let water_pos = in.world_pos.xz - tree_center;

        let dist = length(water_pos);
        let dir = select(vec2<f32>(0.0, 1.0), normalize(water_pos), dist > 0.0001);

        let freq1 = 4.0;
        let speed1 = 3.5;
        let amp1 = 0.08;

        let freq2 = 8.5;
        let speed2 = 5.0;
        let amp2 = 0.03;

        let attenuation = exp(-dist * 0.25);

        let phase1 = dist * freq1 - uniforms.time * speed1;
        let phase2 = dist * freq2 - uniforms.time * speed2;

        let slope1 = cos(phase1) * freq1 * amp1 * attenuation;
        let slope2 = cos(phase2) * freq2 * amp2 * attenuation;
        let total_slope = slope1 + slope2;

        let wave_normal = normalize(vec3<f32>(-dir.x * total_slope, 1.0, -dir.y * total_slope));

        let half_dir = normalize(light_dir + view_dir);
        let spec = pow(max(dot(wave_normal, half_dir), 0.0), 64.0);
        let fresnel = pow(1.0 - max(dot(view_dir, wave_normal), 0.0), 3.0);

        let deep_water = vec3<f32>(0.03, 0.10, 0.22);
        let sky_color = vec3<f32>(0.45, 0.65, 0.90);

        let water_base = mix(deep_water, sky_color, fresnel * 0.85);
        let specular_light = uniforms.light_color * spec * 3.0;

        return vec4<f32>(water_base + specular_light, 0.88);
    }

    // ==========================================
    // MAPOWANIE NORMALNYCH DLA PŁATKÓW (object_type == 1.0)
    // ==========================================
    if (in.object_type >= 0.5 && in.object_type < 1.5) {
        // Obliczenie przestrzeni TBN (Tangent, Bitangent, Normal)
        let T = normalize(in.tangent);
        let B = normalize(cross(N, T));
        let TBN = mat3x3<f32>(T, B, N);

        // Odczyt z mapy normalnych i przeskalowanie z [0, 1] na [-1, 1]
        let raw_normal = textureSampleLevel(t_normal, s_normal, in.uv, 0.0).rgb * 2.0 - 1.0;
        // Siła uwypuklenia (bump_strength)
        let bump_strength = 2.5;
        let map_normal = normalize(vec3<f32>(raw_normal.xy * bump_strength, raw_normal.z));

        // Transformacja wektora normalnego z przestrzeni stycznej do przestrzeni świata
        N = normalize(TBN * map_normal);
    }

    // Wyznaczenie bazowego koloru materiału
    var base_color: vec4<f32>;

    // ==========================================
    // 2. KORA DREWNA / PIEŃ (object_type == 0.0)
    // ==========================================
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
        let alpha_mask = raw_normal.a;

        if (alpha_mask < 0.1) {
            discard;
        }

        let T = normalize(in.tangent);
        let B = normalize(cross(N, T));
        let TBN = mat3x3<f32>(T, B, N);

        let bump_strength = 2.2;
        let nx = (raw_normal[0] * 2.0 - 1.0) * bump_strength;
        let ny = (raw_normal[1] * 2.0 - 1.0) * bump_strength;
        let nz = raw_normal[2] * 2.0 - 1.0;

        let map_normal = normalize(vec3<f32>(nx, ny, nz));
        N = normalize(TBN * map_normal);

        let center_vein = smoothstep(0.0, 0.05, abs(in.uv.x - 0.5));
        let vein_darkness = mix(0.85f, 1.0, center_vein);

        base_color = vec4<f32>(in.color.rgb * vein_darkness, in.color.a);
    }

    // ==========================================
    // MODEL OŚWIETLENIA DLA DRZEWA I PŁATKÓW
    // ==========================================

    let ambient = 0.25;
    let diff = max(dot(N, light_dir), 0.0);

    let half_dir = normalize(light_dir + view_dir);
    var spec_intensity = 0.0;

    if (in.object_type >= 0.5) {
        // Szeroki refleks świetlny (specular) wyciągający nierówności z mapy normalnych
        spec_intensity = pow(max(dot(N, half_dir), 0.0), 12.0) * 0.8;
    } else {
        // Kora pozostaje matowa
        spec_intensity = pow(max(dot(N, half_dir), 0.0), 4.0) * 0.05;
    }

    let light_contribution = (ambient + diff) * uniforms.light_color;
    let specular_contribution = uniforms.light_color * spec_intensity;

    let final_rgb = base_color.rgb * light_contribution + specular_contribution;

    return vec4<f32>(final_rgb, base_color.a);
}