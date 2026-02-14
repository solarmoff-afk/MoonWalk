// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

struct Uniforms {
    view_proj: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> @builtin(position) vec4<f32> {
    return params.view_proj * vec4<f32>(in.position, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return params.color;
}
