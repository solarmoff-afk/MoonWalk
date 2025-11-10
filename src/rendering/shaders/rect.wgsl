struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) local_pos: vec2<f32>,
    @location(3) rect_size: vec2<f32>,
    @location(4) radii: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) radii: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@vertex
fn vs_rect_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.local_pos = model.local_pos;
    out.rect_size = model.rect_size;
    out.radii = model.radii;
    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r_in: vec4<f32>) -> f32 {
    var r = r_in;
    if (p.x > 0.0) {
        r.x = r.x;
        r.y = r.y;
    } else {
        r.x = r.w;
        r.y = r.z;
    }
    if (p.y > 0.0) {
        r.x = r.x;
    } else {
        r.x = r.y;
    }
    
    let q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r.x;
}

@fragment
fn fs_rect_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = in.rect_size * 0.5;
    let p = in.local_pos - half_size;

    let min_half_dim = min(half_size.x, half_size.y);
    let clamped_radii = min(in.radii, vec4<f32>(min_half_dim));

    let dist = sd_rounded_box(p, half_size - vec2(1.0, 1.0), clamped_radii);
    
    let screen_pixel = fwidth(dist);
    let alpha = clamp(0.5 - dist / screen_pixel, 0.0, 1.0);
    
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}