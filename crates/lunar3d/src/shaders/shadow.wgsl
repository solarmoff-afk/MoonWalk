// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct ShadowGlobal {
    light_view_proj: mat4x4<f32>,
    atlas_offset: vec2<f32>,
    atlas_scale: f32,
    _pad: f32,
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
    var clip_pos = global.light_view_proj * world_pos;
    
    let scale = global.atlas_scale;
    let offset_x = global.atlas_offset.x;
    let offset_y = global.atlas_offset.y;
    
    clip_pos.x = clip_pos.x * scale + (offset_x - 0.5 + scale * 0.5) * 2.0 * clip_pos.w;
    clip_pos.y = clip_pos.y * scale - (offset_y - 0.5 + scale * 0.5) * 2.0 * clip_pos.w;
    
    return clip_pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
