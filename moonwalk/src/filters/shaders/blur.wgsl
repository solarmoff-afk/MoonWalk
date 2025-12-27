// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

struct Uniforms {
    direction: vec2<f32>,
    radius: f32, // sigma
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
    out.uv.y = 1.0 - out.uv.y; 
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sigma = max(params.radius, 1.0); 
    let k_size = min(i32(ceil(sigma * 3.0)), 40);

    let tex_one = 1.0 / params.resolution;
    
    var color = vec4<f32>(0.0);
    var total_weight = 0.0;

    for (var i = -k_size; i <= k_size; i = i + 1) {
        let offset = f32(i);
        
        let weight = exp(-(offset * offset) / (2.0 * sigma * sigma));
        
        let sample_uv = in.uv + (params.direction * offset * tex_one);
        
        color += textureSample(t_diffuse, s_diffuse, sample_uv) * weight;
        total_weight += weight;
    }
    
    return color / total_weight;
}
