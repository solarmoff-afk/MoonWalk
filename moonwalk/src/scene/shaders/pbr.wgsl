// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

const PI: f32 = 3.14159265359;

struct Light {
    position: vec3<f32>,
    _pad1: f32,
    color: vec3<f32>,
    intensity: f32,
};

struct Global {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    num_lights: u32,
    lights: array<Light, 4>, 
    ambient_color: vec3<f32>,
    shadows_enabled: f32,
    light_view_proj: mat4x4<f32>, 
};

struct MaterialFlags {
    use_albedo_map: u32,
    use_normal_map: u32,
    use_mr_map: u32,
    _pad: u32,
};

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_sampler: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var t_mr: texture_2d<f32>;
@group(1) @binding(4) var<uniform> flags: MaterialFlags;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>, 
};

struct InstanceInput {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) norm_0: vec4<f32>,
    @location(9) norm_1: vec4<f32>,
    @location(10) norm_2: vec4<f32>,
    @location(11) color: vec4<f32>,
    @location(12) metallic: f32,
    @location(13) roughness: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) tbn_0: vec3<f32>,
    @location(3) tbn_1: vec3<f32>,
    @location(4) tbn_2: vec3<f32>,
    @location(5) color: vec4<f32>,
    @location(6) metallic: f32,
    @location(7) roughness: f32,
    @location(8) shadow_pos: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_0, instance.model_1, instance.model_2, instance.model_3
    );
    
    let normal_matrix = mat3x3<f32>(
        instance.norm_0.xyz, instance.norm_1.xyz, instance.norm_2.xyz
    );

    var out: VertexOutput;
    let world_pos_4 = model_matrix * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos_4.xyz;
    out.clip_position = global.view_proj * world_pos_4;
    out.uv = in.uv;

    let T = normalize(normal_matrix * in.tangent);
    let N = normalize(normal_matrix * in.normal);
    let B = cross(N, T); 

    out.tbn_0 = T; out.tbn_1 = B; out.tbn_2 = N;
    
    out.color = instance.color;
    out.metallic = instance.metallic;
    out.roughness = instance.roughness;
    out.shadow_pos = global.light_view_proj * world_pos_4;

    return out;
}

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return num / (PI * denom * denom);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn calculate_shadow(shadow_pos: vec4<f32>, N: vec3<f32>, L: vec3<f32>) -> f32 {
    let proj_coords = shadow_pos.xyz / shadow_pos.w;
    let uv = vec2<f32>(proj_coords.x * 0.5 + 0.5, 0.5 - proj_coords.y * 0.5);
    let current_depth = proj_coords.z;

    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || current_depth > 1.0 || current_depth < 0.0) {
        return 1.0;
    }

    let bias = max(0.0005 * (1.0 - dot(N, L)), 0.00005);
    var shadow = 0.0;
    let size = textureDimensions(t_shadow);
    let texel_size = vec2<f32>(1.0 / f32(size.x), 1.0 / f32(size.y));

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            shadow += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>(f32(x), f32(y)) * texel_size, current_depth - bias);
        }
    }
    
    return shadow / 9.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var base_color = in.color;
    if (flags.use_albedo_map == 1u) {
        let tex = textureSample(t_albedo, s_sampler, in.uv);
        base_color = vec4(pow(tex.rgb, vec3(2.2)), tex.a) * base_color;
    }
    
    if (base_color.a < 0.1) { discard; }
    let albedo = base_color.rgb;

    var N = normalize(in.tbn_2);
    if (flags.use_normal_map == 1u) {
        let normal_map = textureSample(t_normal, s_sampler, in.uv).rgb;
        let tangent_normal = normalize(normal_map * 2.0 - 1.0);
        let tbn = mat3x3<f32>(normalize(in.tbn_0), normalize(in.tbn_1), normalize(in.tbn_2));
        N = normalize(tbn * tangent_normal);
    }

    var roughness = in.roughness;
    var metallic = in.metallic;

    if (flags.use_mr_map == 1u) {
        let mr_sample = textureSample(t_mr, s_sampler, in.uv);
        roughness = mr_sample.g * roughness;
        metallic = mr_sample.b * metallic;
    }

    let V = normalize(global.camera_pos - in.world_pos);
    var F0 = vec3(0.04); 
    F0 = mix(F0, albedo, metallic);

    var Lo = vec3(0.0);

    for (var i = 0u; i < global.num_lights; i = i + 1u) {
        let light = global.lights[i];
        let L = normalize(light.position - in.world_pos);
        let H = normalize(V + L);
        
        let dist = distance(light.position, in.world_pos);
        let attenuation = 1.0 / (1.0 + 0.09 * dist + 0.032 * dist * dist);
        let radiance = light.color * light.intensity * attenuation;

        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let kS = F;
        var kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        let NdotL = max(dot(N, L), 0.0);
        
        var shadow = 1.0;
        if (i == 0u) {
            shadow = calculate_shadow(in.shadow_pos, N, L);
            shadow = mix(1.0, shadow, global.shadows_enabled);
        }

        Lo += (kD * albedo / PI + specular) * radiance * NdotL * shadow;
    }

    let ambient = global.ambient_color * albedo; 
    var color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    return vec4<f32>(color, base_color.a);
}
