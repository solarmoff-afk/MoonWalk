// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct ShadowGlobal {
    light_view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> global: ShadowGlobal;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> @builtin(position) vec4<f32> {
    let model_matrix = mat4x4<f32>(
        instance.model_0, instance.model_1, instance.model_2, instance.model_3
    );
    
    let world_pos = model_matrix * vec4<f32>(in.position, 1.0);
    
    return global.light_view_proj * world_pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}