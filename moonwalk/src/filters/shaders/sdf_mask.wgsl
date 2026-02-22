// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct SdfUniforms {
    size: vec2<f32>, // Размер текстуры
    radius: f32,
    hardness: f32,  // Кривизна ската (1.0 это линейный, больше 1.0 это выпуклый)
    threshold: f32, // Что считать границей
};

@group(0) @binding(0) var<uniform> params: SdfUniforms;

@group(1) @binding(0) var t_mask: texture_2d<f32>;
@group(1) @binding(1) var s_mask: sampler;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Прозрачные пиксели не трогаем
    let center_alpha = textureSample(t_mask, s_mask, in.uv).a;
    if (center_alpha < params.threshold) {
        return vec4<f32>(0.0);
    }

    let texel_size = 1.0 / params.size;
    let max_dist = params.radius;
    var min_dist = max_dist;

    // Здесь используется алгоритм ray scan который позволяет сканировать текстуру лучами
    // Количество лучей (12 достаточно для качественной фаски)
    let rays = 12.0; 
    let step_size = 2.0; // Оптимизация
    
    for (var i = 0.0; i < rays; i = i + 1.0) {
        let angle = (i / rays) * 6.28318;
        let dir = vec2<f32>(cos(angle), sin(angle)) * texel_size;
        
        var current_dist = max_dist;
        
        for (var d = 1.0; d <= max_dist; d = d + step_size) {
            let sample_uv = in.uv + dir * d;
            let a = textureSample(t_mask, s_mask, sample_uv).a;
            
            // Если наткнулись на прозрачность то это значит что это край
            if (a < params.threshold) {
                current_dist = d;
                break;
            }
        }
        
        // Самое короткое расстяние до краёв
        min_dist = min(min_dist, current_dist);
    }

    let normalized = min_dist / max_dist; 
    let alpha = pow(normalized, params.hardness);

    // Маска по дефолту возвращается белая
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}
