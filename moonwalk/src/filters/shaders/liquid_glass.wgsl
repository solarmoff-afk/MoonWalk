// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct GlassUniforms {
    // 0 по 16 байт
    size: vec2<f32>,
    offset: vec2<f32>,

    // 16 по 32 байт
    corner_radius: vec4<f32>,

    // 32 по 48 байт
    resolution: vec2<f32>,
    refraction_height: f32,
    refraction_amount: f32,

    // 48..64 bytes
    depth_effect: f32,
    chromatic_aberration: f32,
    rotation: f32,
    gamma: f32,

    // 64 по 80 байт
    unused_color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: GlassUniforms;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index) & 1);
    let y = f32(i32(in_vertex_index >> 1));
    out.uv = vec2<f32>(x * 2.0, y * 2.0);
    out.position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv.y = 1.0 - out.uv.y; 
    return out;
}

fn rotate_vec(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec2<f32>(v.x * c - v.y * s, v.x * s + v.y * c);
}

fn radius_at(p: vec2<f32>, r: vec4<f32>) -> f32 {
    if (p.x >= 0.0) {
        if (p.y >= 0.0) {
          return r.z;
        } else {
          return r.y;
        }
    } else {
        if (p.y >= 0.0) {
          return r.w;
        } else {
          return r.x;
        }
    }
}

fn sd_rounded_rect(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

fn circle_map(x: f32) -> f32 {
    return 1.0 - sqrt(max(0.0, 1.0 - x * x));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let res = params.resolution;
    let coord = in.uv * res;
    let half_size = params.size * 0.5;
    
    let p_unrotated = coord - params.offset - half_size;
    
    let p = rotate_vec(p_unrotated, -params.rotation);
    
    let radius = radius_at(p, params.corner_radius);
    let sd = sd_rounded_rect(p, half_size, radius);
    
    if (sd > 0.0) {
        return vec4<f32>(0.0);
    }

    let ref_h = max(params.refraction_height, 1.0);
    
    let p_dir_world = normalize(p_unrotated + 0.0001);
    
    let t = clamp(-sd / ref_h, 0.0, 1.0);
    let edge_stretch = pow(1.0 - t, 2.5); 
    let distortion = (edge_stretch * params.refraction_amount * 2.0) + (t * params.depth_effect * 10.0);
    
    // Искажаем uv в сторону центра линзы
    let refracted_coord = coord + distortion * p_dir_world;

    // Дисперсия из 7 ступеней
    let dispersion = params.chromatic_aberration * (1.0 - t);
    let disp_vec = p_dir_world * distortion * dispersion;
    
    var color = vec4<f32>(0.0);
    
    // Спектральная выборка из фона
    color.r += textureSample(t_diffuse, s_diffuse, (refracted_coord + disp_vec) / res).r / 3.5;
    
    let orange = textureSample(t_diffuse, s_diffuse, (refracted_coord + disp_vec * 0.66) / res);
    color.r += orange.r / 3.5; color.g += orange.g / 7.0;
    
    let yellow = textureSample(t_diffuse, s_diffuse, (refracted_coord + disp_vec * 0.33) / res);
    color.r += yellow.r / 3.5; color.g += yellow.g / 3.5;
    
    let green = textureSample(t_diffuse, s_diffuse, refracted_coord / res);
    color.g += green.g / 3.5;
    
    let cyan = textureSample(t_diffuse, s_diffuse, (refracted_coord - disp_vec * 0.33) / res);
    color.g += cyan.g / 3.5; color.b += cyan.b / 3.0;
    
    let blue = textureSample(t_diffuse, s_diffuse, (refracted_coord - disp_vec * 0.66) / res);
    color.b += blue.b / 3.0;
    
    let purple = textureSample(t_diffuse, s_diffuse, (refracted_coord - disp_vec) / res);
    color.r += purple.r / 7.0; color.b += purple.b / 3.0;

    color.a = 1.0;

    if (params.gamma != 1.0) {
        color = vec4<f32>(pow(max(color.rgb, vec3<f32>(0.0)), vec3<f32>(params.gamma)), color.a);
    }
    
    // Мягкий антиалиасинг края линзы
    let aa_mask = 1.0 - smoothstep(-2.0, 0.0, sd);
    return color * aa_mask;
}
