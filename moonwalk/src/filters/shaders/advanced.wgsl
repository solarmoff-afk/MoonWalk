// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

struct Uniforms {
    key_color: vec3<f32>,
    tolerance: f32,
    params: vec4<f32>, // x: mode (1=chromakey, 2=stencil), y: invert_stencil
};

@group(0) @binding(0) var<uniform> ubo: Uniforms;

// Основная текстура
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

// Текстура маски (для трафарета)
@group(1) @binding(2) var t_mask: texture_2d<f32>;

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
    var color = textureSample(t_diffuse, s_diffuse, in.uv);
    let mode = u32(ubo.params.x);

    if (mode == 1u) {
        // Дистанция считается от текущего цвета до ключевого
        let diff = color.rgb - ubo.key_color;
        let dist = length(diff);
        
        // Немного сглаживания чтобы края не были рваными
        let edge_softness = 0.05; 
        let alpha_factor = smoothstep(ubo.tolerance, ubo.tolerance + edge_softness, dist);
        
        color.a *= alpha_factor;
    }  else if (mode == 2u) {
        let mask_color = textureSample(t_mask, s_diffuse, in.uv);
        let invert = ubo.params.y > 0.5;

        var mask_alpha = mask_color.a; // альфа-канал маски
        
        // Если invert=false, то прозрачный пиксель маски = видимый основной
        // Если invert=true, то Прозрачный пиксель маски = невидимый основной
        
        if (!invert) {
            mask_alpha = 1.0 - mask_alpha;
        }

        color.a *= mask_alpha;
    }

    return color;
}
