// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

struct Uniforms {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> ubo: Uniforms;

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) pos_size: vec4<f32>,
    @location(2) radii: vec4<f32>,
    @location(3) uv: vec4<f32>,
    @location(4) extra: vec2<f32>,
    @location(5) color_packed: u32,
    @location(6) color2_packed: u32,
    @location(7) type_id: u32,
    @location(8) gradient_data: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radii: vec4<f32>,
    @location(4) uv: vec2<f32>,
    @location(5) type_id: u32,
    @location(6) color2: vec4<f32>,
    @location(7) gradient_data: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let pos = instance.pos_size.xy;
    let size = instance.pos_size.zw;
    let z_index = instance.extra.x;
    let rotation = instance.extra.y;

    let center_offset = size * 0.5;
    let local_unrotated = (in.position * size) - center_offset;

    let c = cos(rotation);
    let s = sin(rotation);
    let rotated_x = local_unrotated.x * c - local_unrotated.y * s;
    let rotated_y = local_unrotated.x * s + local_unrotated.y * c;
    
    let final_x = rotated_x + center_offset.x + pos.x;
    let final_y = rotated_y + center_offset.y + pos.y;

    out.clip_position = ubo.view_proj * vec4<f32>(final_x, final_y, z_index, 1.0);
    
    out.color = unpack4x8unorm(instance.color_packed);
    out.color2 = unpack4x8unorm(instance.color2_packed);
    
    out.radii = instance.radii;
    out.size = size;
    out.local_pos = in.position * size;

    out.uv = instance.uv.xy + (in.position * instance.uv.zw);
    out.type_id = instance.type_id;
    out.gradient_data = instance.gradient_data;

    return out;
}

fn linear_gradient(
    local_pos: vec2<f32>,
    size: vec2<f32>,
    dir: vec2<f32>,
    color: vec4<f32>,
    color2: vec4<f32>
) -> vec4<f32> {
    // Направление обязательно должно быть от 0 до 1 (по x и y) поэтому тут
    // нужна нормализация
    let direction = normalize(dir);
    
    let rel_pos = local_pos / size;

    let proj_length = dot(size, abs(direction)); 
    let proj = dot(rel_pos, direction);

    let t = proj / proj_length;
    
    return mix(color, color2, clamp(t, 0.0, 1.0));
}

fn radial_gradient(
    local_pos: vec2<f32>,
    size: vec2<f32>,
    center: vec2<f32>,
    inner_radius: f32,
    outer_radius: f32,
    color: vec4<f32>,
    color2: vec4<f32>,
) -> vec4<f32> {
    let rel_pos = local_pos / size;
    
    let delta = rel_pos - center;
    let dist = length(delta);

    if(outer_radius <= inner_radius) {
        return color;
    }

    let t = (dist - inner_radius) / (outer_radius - inner_radius);
    return mix(color, color2, clamp(t, 0.0, 1.0));
}

fn is_gradient(gradient_data: vec4<f32>) -> bool {
    return gradient_data.z >= 0;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var radius = r.x; // TL
    if (p.x > 0.0) {
        if (p.y > 0.0) {
            radius = r.z;
        } else {
            radius = r.y;
        }
    } else {
        if (p.y > 0.0) {
            radius = r.w;
        }
    }
    
    let q = abs(p) - b + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = in.size * 0.5;
    let p = in.local_pos - half_size;

    let min_half = min(half_size.x, half_size.y);
    let r = min(in.radii, vec4<f32>(min_half));

    let dist = sd_rounded_box(p, half_size, r);
    
    let alpha = 1.0 - smoothstep(-0.5, 0.5, dist / length(vec2<f32>(dpdx(dist), dpdy(dist))));

    if (alpha <= 0.0) {
        discard;
    }

    // Цвет
    if (in.type_id == 0u) {
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    } else {
        // Текстура
        let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
        
        return vec4<f32>(
            tex_color.rgb * in.color.rgb, 
            tex_color.a * in.color.a * alpha
        );
    }
}