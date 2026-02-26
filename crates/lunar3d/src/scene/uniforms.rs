// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use glam::{Vec3, Mat4};
use crate::scene::LunarScene;
use crate::core::config::{MAX_LIGHTS, MAX_SHADOWS};
use crate::core::types::{GlobalUniform, Light, ShadowQuality};

pub(crate) fn calculate(scene: &LunarScene) -> (GlobalUniform, [[[f32; 4]; 4]; MAX_SHADOWS]) {
    let aspect = scene.width as f32 / scene.height as f32;
    let proj = Mat4::perspective_rh(45.0f32.to_radians(), aspect, 0.1, 200.0);
    let view = Mat4::look_at_rh(scene.camera_pos, scene.camera_target, Vec3::Y);
    
    let mut raw_lights = [Light::default(); MAX_LIGHTS];
    let mut count = 0;
    let mut shadow_mats = [[[0.0; 4]; 4]; MAX_SHADOWS];

    for (i, active) in scene.active_lights.iter().enumerate() {
        if *active {
            if count >= MAX_LIGHTS {
                break;
            }
            
            raw_lights[count] = scene.lights[i];
            
            if count < MAX_SHADOWS {
                let light = &scene.lights[i];
                
                if count == 0 {
                    let light_dir = if light.position == [0.0; 3] {
                        Vec3::Y
                    } else {
                        Vec3::from_array(light.position).normalize()
                    };
                    
                    let light_pos = scene.camera_target + light_dir * 50.0; 
                    let shadow_view = Mat4::look_at_rh(light_pos, scene.camera_target, Vec3::Y);
                    
                    let s = scene.shadows.ortho_size; 
                    let shadow_proj = Mat4::orthographic_rh(-s, s, -s, s, 0.1, 300.0);
                    
                    shadow_mats[count] = (shadow_proj * shadow_view).to_cols_array_2d();
                } 
                else {
                    let light_pos = Vec3::from_array(light.position);
                    let shadow_view = Mat4::look_at_rh(light_pos, scene.camera_target, Vec3::Y);
                    let shadow_proj = Mat4::perspective_rh(90.0f32.to_radians(), 1.0, 0.1, 100.0);
                    
                    shadow_mats[count] = (shadow_proj * shadow_view).to_cols_array_2d();
                }
            }
            
            count += 1;
        }
    }

    let enabled = if scene.shadows.quality != ShadowQuality::Off {
        1.0
    } else {
        0.0
    };

    let u = GlobalUniform {
        view_proj: (proj * view).to_cols_array_2d(),
        camera_pos: scene.camera_pos.to_array(),
        num_lights: count as u32,
        lights: raw_lights,
        ambient_color: scene.ambient_color.to_array(),
        shadows_enabled: enabled,
        light_view_projs: shadow_mats,
    };
    
    (u, shadow_mats)
}
