// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct GlassUniforms {
    size: vec2<f32>,
    offset: vec2<f32>,
    resolution: vec2<f32>,
    refraction_amount: f32,
    refraction_height: f32,
    depth_effect: f32,
    chromatic_aberration: f32,
    tolerance: f32,
    gamma: f32,
    pad0: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: GlassUniforms;

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

// Маска
@group(1) @binding(2) var t_mask: texture_2d<f32>;

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

fn get_mask_alpha(uv: vec2<f32>) -> f32 {
    let a = textureSample(t_mask, s_diffuse, uv).a;
    return smoothstep(params.tolerance - 0.005, params.tolerance + 0.005, a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let res = params.resolution;
    let coord = in.uv * res;
    
    let local_p = coord - params.offset;
    
    // Если пиксель за пределами стекла то пустота
    if (local_p.x < 0.0 || local_p.y < 0.0 || local_p.x > params.size.x || local_p.y > params.size.y) {
        return vec4<f32>(0.0);
    }

    let mask_uv = local_p / params.size;
    let alpha = get_mask_alpha(mask_uv);
    
    if (alpha <= 0.0) { 
        return vec4<f32>(0.0); 
    }

    // Вычисление нормали на основе соседних пикселей
    let eps = 1.0 / params.size; 
    let a_l = get_mask_alpha(mask_uv - vec2<f32>(eps.x, 0.0));
    let a_r = get_mask_alpha(mask_uv + vec2<f32>(eps.x, 0.0));
    let a_u = get_mask_alpha(mask_uv - vec2<f32>(0.0, eps.y));
    let a_d = get_mask_alpha(mask_uv + vec2<f32>(0.0, eps.y));

    var normal = vec2<f32>(a_l - a_r, a_u - a_d);
    
    let p_center = mask_uv - 0.5;
    let lens_dir = normalize(p_center + 0.0001);

    // Эффект чёрной дыры на краях маски
    let suction = pow(1.0 - alpha, 2.0); 
    let final_normal = normalize(normal * params.refraction_height + params.depth_effect * lens_dir);
    
    let distortion_power = (params.refraction_amount * alpha) + (suction * params.refraction_amount * 2.5);
    
    let refracted_uv = in.uv + (final_normal * distortion_power / res);

    // Дисперсия из 7 ступеней
    let dispersion = params.chromatic_aberration * (1.0 - alpha * 0.5);
    let disp_vec = (final_normal * distortion_power * dispersion) / res;
    
    var color = vec4<f32>(0.0);
    
    color.r += textureSample(t_diffuse, s_diffuse, refracted_uv + disp_vec).r / 3.5;
    color.a += textureSample(t_diffuse, s_diffuse, refracted_uv + disp_vec).a / 7.0;
    
    let orange = textureSample(t_diffuse, s_diffuse, refracted_uv + disp_vec * 0.66);
    color.r += orange.r / 3.5; color.g += orange.g / 7.0; color.a += orange.a / 7.0;
    
    let yellow = textureSample(t_diffuse, s_diffuse, refracted_uv + disp_vec * 0.33);
    color.r += yellow.r / 3.5; color.g += yellow.g / 3.5; color.a += yellow.a / 7.0;
    
    let green = textureSample(t_diffuse, s_diffuse, refracted_uv);
    color.g += green.g / 3.5; color.a += green.a / 7.0;
    
    let cyan = textureSample(t_diffuse, s_diffuse, refracted_uv - disp_vec * 0.33);
    color.g += cyan.g / 3.5; color.b += cyan.b / 3.0; color.a += cyan.a / 7.0;
    
    let blue = textureSample(t_diffuse, s_diffuse, refracted_uv - disp_vec * 0.66);
    color.b += blue.b / 3.0; color.a += blue.a / 7.0;
    
    let purple = textureSample(t_diffuse, s_diffuse, refracted_uv - disp_vec);
    color.r += purple.r / 7.0; color.b += purple.b / 3.0; color.a += purple.a / 7.0;

    let edge_aa = smoothstep(0.0, 0.02, alpha);
    
    // Гамма
    if (params.gamma != 1.0) {
        color = vec4<f32>(pow(max(color.rgb, vec3<f32>(0.0)), vec3<f32>(params.gamma)), color.a);
    }
    
    return color * edge_aa;
}
