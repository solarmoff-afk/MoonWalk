// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::collections::HashMap;
use glam::Mat4;
use bytemuck::bytes_of;

use crate::MoonWalk;
use crate::rendering::custom::*;
use crate::public::custom::BindResource;
use crate::scene::types::*;

pub(crate) struct DrawCommand {
    pub(crate) mesh_id: usize,
    pub(crate) bind_group: MoonBindGroup,
    pub(crate) offset: usize,
    pub(crate) count: usize,
}

pub(crate) fn update_uniforms(_mw: &MoonWalk, scene: &crate::scene::Scene3D, width: u32, height: u32) -> (GlobalUniform, ShadowUniform) {
    let aspect = width as f32 / height as f32;
    let proj = Mat4::perspective_rh(45.0f32.to_radians(), aspect, 0.1, 100.0);
    let view = Mat4::look_at_rh(scene.camera_pos, scene.camera_target, glam::Vec3::Y);
    
    let mut raw_lights = [LightRaw::default(); MAX_LIGHTS];
    
    let mut active_count = 0;
    for (i, alive) in scene.lights_active.iter().enumerate() {
        if *alive && active_count < MAX_LIGHTS {
            raw_lights[active_count] = scene.lights_data[i];
            active_count += 1;
        }
    }
    
    let main_light = raw_lights[0];
    let light_dir = if main_light.position == [0.0; 3] {
        glam::Vec3::Y
    } else {
        glam::Vec3::from_array(main_light.position).normalize()
    };

    let light_pos = scene.camera_target + light_dir * 20.0; 
    
    let shadow_view = Mat4::look_at_rh(light_pos, scene.camera_target, glam::Vec3::Y);
    let s = scene.shadow_ortho_size;
    let shadow_proj = Mat4::orthographic_rh(-s, s, -s, s, 0.1, 100.0);
    let light_space_matrix = shadow_proj * shadow_view;

    let shadows_active = if scene.shadow_quality == ShadowQuality::Off {
        0.0
    } else {
        1.0
    };

    let global_data = GlobalUniform {
        view_proj: (proj * view).to_cols_array_2d(),
        camera_pos: scene.camera_pos.to_array(),
        num_lights: active_count as u32,
        lights: raw_lights,
        ambient_color: scene.ambient_color.to_array(),
        shadows_enabled: shadows_active,
        light_view_proj: light_space_matrix.to_cols_array_2d(),
    };

    let shadow_data = ShadowUniform {
        light_view_proj: light_space_matrix.to_cols_array_2d(),
    };
    
    (global_data, shadow_data)
}

pub(crate) fn prepare_draw_commands(
    mw: &MoonWalk,
    batches: &HashMap<BatchKey, Vec<InstanceRaw>>,
    offsets: &HashMap<BatchKey, (usize, usize)>,
    layout: &MoonBindGroupLayout,
    cache: &mut HashMap<(u32, u32, u32), MoonBindGroup>,
    default_normal: u32,
    default_white: u32,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();

    for (key, _) in batches {
        let mat_key = (key.albedo_id, key.normal_id, key.mr_id);
        
        if !cache.contains_key(&mat_key) {
            let flags_data = MaterialFlags {
                use_albedo_map: 1, 
                use_normal_map: if key.normal_id != default_normal {
                    1
                } else {
                    0
                },

                use_mr_map: if key.mr_id != default_white {
                    1
                } else {
                    0
                },

                _pad: 0,
            };

            let flags_buf = mw.create_uniform_buffer(bytes_of(&flags_data));

            let bg = mw.create_bind_group(layout, &[
                BindResource::Texture(key.albedo_id),
                BindResource::Sampler(key.albedo_id),
                BindResource::Texture(key.normal_id),
                BindResource::Texture(key.mr_id),
                BindResource::Uniform(&flags_buf), 
            ]);
            
            if let Ok(g) = bg {
                cache.insert(mat_key, g);
            }
        }

        if let Some(bg) = cache.get(&mat_key) {
            let (start, count) = offsets[key];
            commands.push(DrawCommand {
                mesh_id: key.mesh_id,
                bind_group: bg.clone(),
                offset: start,
                count,
            });
        }
    }
    
    commands
}