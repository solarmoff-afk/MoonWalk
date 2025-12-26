// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

struct Uniforms {
    direction: vec2<f32>,
    radius: f32,
    _pad: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> params: Uniforms;

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
    
    // Для правильного отображения нужно инвертировать
    out.uv.y = 1.0 - out.uv.y; 
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 21 tap gaussian (Sigma где-то 7.0), эти веса дают очень мягкий спад
    
    let tex_offset = 1.0 / params.resolution;
    
    // Тут выполняется цикл от -10 до 10. Веса (упрощенная колоколообразная кривая):
    // exp(-x*x / (2 * sigma*sigma))
    
    var color = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Здесь цикл захардкожен. Не очень хорошо, желательно улучшить в будущем
    let steps = 24; 
    let sigma = 5.0; 
    
    for (var i = -steps; i <= steps; i = i + 1) {
        let offset = f32(i);
        
        // Вес Гаусса
        let weight = exp(-(offset * offset) / (2.0 * sigma * sigma));
        
        let sample_uv = in.uv + (params.direction * tex_offset * offset * params.radius);
        
        color += textureSample(t_diffuse, s_diffuse, sample_uv) * weight;
        total_weight += weight;
    }
    
    return color / total_weight;
}