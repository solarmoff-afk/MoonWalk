// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct MeshUniforms {
    colors: array<vec4<f32>, 9>,
    positions: array<vec4<f32>, 9>,
    resolution: vec2<f32>,
    noise_intensity: f32,
    warp_strength: f32,
    warp_phase: f32,
    gamma: f32,
    blend_mode: u32,
    pad0: f32,
};

@group(0) @binding(0) var<uniform> params: MeshUniforms;
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

// Генератор зернистости
fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// Фрактальное искажение (3 слоя синусоид)
fn get_warp(p: vec2<f32>) -> vec2<f32> {
    let ph = params.warp_phase;
    var w = p;
    
    // Медленные крупные волны
    w += vec2<f32>(
        sin(p.y * 3.5 + ph * 0.5),
        cos(p.x * 2.8 + ph * 0.4)
    ) * 0.1;
    
    // Средние завихрения
    w += vec2<f32>(
        sin(w.y * 6.2 - ph * 0.8),
        cos(w.x * 5.5 + ph * 0.7)
    ) * 0.05;
    
    // Мелкая рябь
    w += vec2<f32>(
        sin(w.y * 12.0 + ph),
        cos(w.x * 15.0 - ph)
    ) * 0.02;
    
    return w;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let res = params.resolution;
    let uv = in.uv;
    let aspect = res.x / res.y;

    let p = mix(uv, get_warp(uv), params.warp_strength);

    // Сборка цветов
    var total_weight = 0.0;
    var acc_color = vec4<f32>(0.0);
    
    for (var i = 0; i < 9; i++) {
        let pt_data = params.positions[i];
        let pt_pos = pt_data.xy;
        let pt_radius = pt_data.z; 
        let pt_color = params.colors[i];
        
        let dist_vec = (p - pt_pos) * vec2<f32>(aspect, 1.0);
        let dist_sq = dot(dist_vec, dist_vec);
        
        // 4.0 это коэффициент размытости, чем он меньше тем шире текут цвета
        let w = exp(-dist_sq * (6.0 / (pt_radius + 0.001)));
        
        let weighted_w = w * pt_color.a;
        acc_color += pt_color * weighted_w;
        total_weight += weighted_w;
    }
    
    var grad_color = acc_color / max(total_weight, 0.0001);

    // [HACK]
    // Убирает серую муть которая возникает при смешивании противоположных цветов
    
    let luminance = dot(grad_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    grad_color = vec4<f32>(mix(vec3<f32>(luminance), grad_color.rgb, 1.2), grad_color.a);
    
    // Зернистость
    let noise = (rand(in.position.xy) - 0.5) * params.noise_intensity;
    grad_color = vec4<f32>(grad_color.rgb + noise, grad_color.a);

    // Смешивание с фоном
    let bg_color = textureSample(t_diffuse, s_diffuse, uv);
    var final_color = vec4<f32>(0.0);

    if (params.blend_mode == 0u) {
        // Замена
        final_color = vec4<f32>(mix(bg_color.rgb, grad_color.rgb, grad_color.a), max(bg_color.a, grad_color.a));
    } else {
        // Оверлей
        final_color = bg_color + grad_color * grad_color.a;
    }

    // Гамма
    if (params.gamma != 1.0) {
        final_color = vec4<f32>(pow(max(final_color.rgb, vec3<f32>(0.0)), vec3<f32>(params.gamma)), final_color.a);
    }

    return final_color;
}
