// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct Uniforms {
    view_proj: mat4x4<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> ubo: Uniforms;
@group(1) @binding(0) var t_brush: texture_2d<f32>;
@group(1) @binding(1) var s_brush: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>, // ширина и высота
    @location(2) rotation: f32, // в радианах
    @location(3) opacity: f32, // per-instance opacity (для jitter/flow)
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha_mod: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Генерация квада 0..1
    var pos = vec2<f32>(0.0);
    var uv = vec2<f32>(0.0);
    
    switch (i32(vertex_index)) {
        case 0: { pos = vec2(-0.5, -0.5); uv = vec2(0.0, 1.0); }
        case 1: { pos = vec2( 0.5, -0.5); uv = vec2(1.0, 1.0); }
        case 2: { pos = vec2(-0.5,  0.5); uv = vec2(0.0, 0.0); }
        case 3: { pos = vec2(-0.5,  0.5); uv = vec2(0.0, 0.0); }
        case 4: { pos = vec2( 0.5, -0.5); uv = vec2(1.0, 1.0); }
        default: { pos = vec2( 0.5,  0.5); uv = vec2(1.0, 0.0); }
    }

    // Размер + сплбщивание
    var local_pos = pos * instance.size;

    // Вращение
    let c = cos(instance.rotation);
    let s = sin(instance.rotation);
    let rotated_x = local_pos.x * c - local_pos.y * s;
    let rotated_y = local_pos.x * s + local_pos.y * c;
    local_pos = vec2(rotated_x, rotated_y);

    // Позиция
    let world_pos = instance.position + local_pos;

    out.clip_position = ubo.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.uv = uv;
    out.alpha_mod = instance.opacity;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let brush_alpha = textureSample(t_brush, s_brush, in.uv).a;
    
    let dist = distance(in.uv, vec2(0.5));
    let hardness = ubo.params.x;
    let edge = mix(0.5, 0.0, hardness); 
    let sdf_alpha = 1.0 - smoothstep(0.5 - edge, 0.5, dist);

    let final_alpha = brush_alpha * sdf_alpha * in.alpha_mod;

    return vec4<f32>(ubo.color.rgb, final_alpha);
}
