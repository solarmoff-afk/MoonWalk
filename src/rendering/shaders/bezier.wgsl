struct Uniforms {
    resolution: vec2<f32>,
    thickness: f32,
    smoothing: f32,
    curve_color: vec4<f32>,
    point_count: u32,
};

@group(0) @binding(0) var<uniform> uni: Uniforms;
@group(0) @binding(1) var<storage, read> points: array<vec2<f32>>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    var pos: vec2<f32>;
    switch (vertex_index) {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>(3.0, -1.0); }
        case 2u: { pos = vec2<f32>(-1.0, 3.0); }
        default: { pos = vec2<f32>(0.0, 0.0); }
    }
    
    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = pos * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return output;
}

fn bezier_point(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {
    let u = 1.0 - t;
    let uu = u * u;
    let uuu = uu * u;
    let tt = t * t;
    let ttt = tt * t;
    
    return uuu * p0 + 
           3.0 * uu * t * p1 + 
           3.0 * u * tt * p2 + 
           ttt * p3;
}

fn bezier_derivative(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {
    let u = 1.0 - t;
    return 3.0 * u * u * (p1 - p0) + 
           6.0 * u * t * (p2 - p1) + 
           3.0 * t * t * (p3 - p2);
}

fn bezier_second_derivative(p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>, t: f32) -> vec2<f32> {
    return 6.0 * (1.0 - t) * (p2 - 2.0 * p1 + p0) + 
           6.0 * t * (p3 - 2.0 * p2 + p1);
}

fn bezier_distance(point: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> f32 {
    var min_distance: f32 = 1000000.0;
    let initial_guesses = 8u;
    
    for (var i: u32 = 0u; i <= initial_guesses; i++) {
        var t = f32(i) / f32(initial_guesses);
        
        for (var iter: u32 = 0u; iter < 2u; iter++) {
            let curve_point = bezier_point(p0, p1, p2, p3, t);
            let derivative = bezier_derivative(p0, p1, p2, p3, t);
            let diff = curve_point - point;
            
            let f = dot(diff, derivative);
            let second_deriv = bezier_second_derivative(p0, p1, p2, p3, t);
            let df = dot(derivative, derivative) + dot(diff, second_deriv);
            
            if (abs(df) > 0.00000001) {
                t = clamp(t - f / df, 0.0, 1.0);
            }
        }
        
        let closest_point = bezier_point(p0, p1, p2, p3, t);
        let dist = length(closest_point - point);
        min_distance = min(min_distance, dist);
    }
    
    return min_distance;
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let frag_coord = uv * uni.resolution;
    
    if (uni.point_count < 4u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    var min_dist: f32 = 1000000.0;
    let segment_count = uni.point_count / 4u;
    
    for (var i: u32 = 0u; i < segment_count; i++) {
        let base_idx = i * 4u;
        let p0 = points[base_idx];
        let p1 = points[base_idx + 1u];
        let p2 = points[base_idx + 2u];
        let p3 = points[base_idx + 3u];
        
        let dist = bezier_distance(frag_coord, p0, p1, p2, p3);
        min_dist = min(min_dist, dist);
    }
    
    let alpha = 1.0 - smoothstep(uni.thickness - uni.smoothing, uni.thickness + uni.smoothing, min_dist);
    return vec4<f32>(uni.curve_color.rgb, uni.curve_color.a * alpha);
}