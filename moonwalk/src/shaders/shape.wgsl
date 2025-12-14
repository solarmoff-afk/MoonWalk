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
    @location(2) uv: vec4<f32>,
    @location(3) radii_packed: vec4<u32>,
    @location(4) gradient_data: vec4<f32>,
    @location(5) extra: vec2<f32>,
    @location(6) color2_packed: u32,
    @location(7) color_packed: u32,
    @location(8) type_id: u32,
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
    
    out.radii = vec4<f32>(instance.radii_packed) / 16.0;
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
    let uv = local_pos / size;
    let uv_centered = uv - 0.5; 
    let direction = normalize(dir + vec2<f32>(0.0001, 0.0001));
    let t_projected = dot(uv_centered, direction);
    let t = clamp(t_projected + 0.5, 0.0, 1.0);
    
    return mix(color, color2, t);
}

fn radial_gradient(
    local_pos: vec2<f32>,
    size: vec2<f32>,
    center_offset: vec2<f32>,
    inner_radius: f32,
    outer_radius: f32,
    color: vec4<f32>,
    color2: vec4<f32>,
) -> vec4<f32> {
    let uv_centered = (local_pos / size) - 0.5;
    
    let delta = uv_centered - center_offset;
    let dist = length(delta);

    if (outer_radius <= inner_radius) {
        return color;
    }

    let t = clamp((dist - inner_radius) / (outer_radius - inner_radius), 0.0, 1.0);

    return mix(color, color2, t);
}

fn is_gradient(gradient_data: vec4<f32>) -> bool {
    return gradient_data.z >= 0;
}

fn get_gradient_color(
    local_pos: vec2<f32>,
    size: vec2<f32>,
    gradient_data: vec4<f32>,
    color: vec4<f32>,
    color2: vec4<f32>,
) -> vec4<f32> {
    if (is_gradient(gradient_data)) {
        if (gradient_data.z == 0.0 && gradient_data.w == 0.0) {
            return linear_gradient(local_pos, size, gradient_data.xy, color, color2);
            // return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        } else {
            return radial_gradient(local_pos, size, gradient_data.xy, abs(gradient_data.z), 
                abs(gradient_data.w), color, color2);    
            // return vec4<f32>(0.0, 0.0, 1.0, 1.0);
        }
    } else {
        return color;
        // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
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

    let final_color = get_gradient_color(
        in.local_pos,
        in.size,
        in.gradient_data,
        in.color,
        in.color2
    );

    // Цвет
    if (in.type_id == 0u) {
        return vec4<f32>(final_color.rgb, final_color.a * alpha);
    } else {
        // Текстура
        let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
        
        return vec4<f32>(
            tex_color.rgb * final_color.rgb, 
            tex_color.a * final_color.a * alpha
        );
    }
}