// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

pub mod shadows;
pub mod renderer;
pub mod uniforms;

use glam::{Vec3, Vec4};
use moonwalk::MoonWalk;
use self::shadows::ShadowSystem;
use self::renderer::SceneRenderer;

use crate::core::types::*;
use crate::factory::LunarFactory;
use crate::internal::store::ObjectStore;

pub struct LunarScene {
    pub(crate) store: ObjectStore,
    pub(crate) shadows: ShadowSystem,
    pub(crate) renderer: SceneRenderer,
    pub camera_pos: Vec3,
    pub camera_target: Vec3,
    pub ambient_color: Vec3,
    pub lights: Vec<Light>,
    pub active_lights: Vec<bool>,
    pub width: u32,
    pub height: u32,
}

impl LunarScene {
    pub(crate) fn new(
        mw: &mut MoonWalk, 
        width: u32, 
        height: u32, 
        factory: &LunarFactory
    ) -> Self {
        Self {
            store: ObjectStore::new(mw),
            shadows: ShadowSystem::new(mw, factory),
            renderer: SceneRenderer::new(mw, width, height),
            camera_pos: Vec3::new(0.0, 5.0, 10.0),
            camera_target: Vec3::ZERO,
            ambient_color: Vec3::splat(0.03),
            lights: vec![Light::default(); crate::core::config::MAX_LIGHTS],
            active_lights: vec![false; crate::core::config::MAX_LIGHTS],
            width, 
            height,
        }
    }

    pub fn render(
        &mut self, 
        mw: &mut MoonWalk, 
        factory: &LunarFactory
    ) -> u32 {
        let (global_u, shadow_mats) = uniforms::calculate(self);
        
        mw.update_buffer(
            &self.renderer.global_buf, 
            bytemuck::bytes_of(&global_u)
        );
        
        let count = self.renderer.update_instances(mw, &mut self.store);
        
        if count == 0 {
            return 0;
        }

        self.shadows.render(
            mw, 
            factory, 
            &self.store, 
            &shadow_mats,
            global_u.num_lights as usize
        );

        self.renderer.render_main_pass(
            mw, 
            factory, 
            &self.store, 
            self.shadows.get_texture_id()
        )
    }

    /// Этот метод устанавливает качество теней
    pub fn set_shadow_quality(&mut self, mw: &mut MoonWalk, quality: ShadowQuality) {
        self.shadows.set_quality(mw, quality);
    }

    /// Этот метод останавливает обновление теней, можно использовать для оптимизации
    pub fn set_shadow_pause(
        &mut self, 
        paused: bool
    ) {
        self.shadows.paused = paused;
    }

    /// Этот метод создаёт объект на сцене по id 3д модели, её можно загрузить через
    /// методы в структуре LunarFactory
    pub fn new_object(&mut self, mesh_id: MeshId) -> ObjectId {
        self.store.new_object(mesh_id)
    }

    /// Этот метод помечает объект мертвый. Как и в движке MoonWalk в Lunar3d
    /// используется система реинкарнации, что позволяет новым объектам 
    /// занимать id старым экономя ресурсы на аллокации
    pub fn remove_object(&mut self, id: ObjectId) {
        self.store.remove(id);
    }

    /// Этот метод помечает всё объекты мёртвыми
    pub fn remove_all(&mut self) {
        self.store.clear();
    }

    /// Этот метод устанавливает позицию объекта, принимает его айди и
    /// Vec3 из glam (x, y, z)
    pub fn set_position(&mut self, id: ObjectId, position: Vec3) { 
        self.store.set_pos(id, position);
    }

    /// Этот метод устанавливает вращение для объекта. Принимает его id 
    /// и для вращения использует Vec3 из glam. В отличии от 2д рендера
    /// в MoonWalk вращение в Lunar3d происходит в трёх координатах,
    /// x, y и z
    pub fn set_rotation(&mut self, id: ObjectId, rotation: Vec3) { 
        self.store.set_rot(id, rotation); 
    }

    /// Позволяет установить размер объекта относительно оригинала, принимает
    /// id и размер (Vec3 из glam)
    pub fn set_scale(&mut self, id: ObjectId, scale: Vec3) { 
        self.store.set_scale(id, scale); 
    }

    /// Установить цвет объекта
    pub fn set_color(&mut self, id: ObjectId, color: Vec4) { 
        self.store.set_color(id, color); 
    }

    pub fn set_texture(&mut self, id: ObjectId, texture_id: u32) { 
        self.store.set_albedo(id, texture_id); 
    }

    pub fn set_normal_map(&mut self, id: ObjectId, normal_map: u32) { 
        self.store.set_normal(id, normal_map); 
    }

    pub fn set_unlit(&mut self, id: ObjectId, unlit: bool) { 
        self.store.set_unlit(id, unlit); 
    }
    
    pub fn set_layer(&mut self, id: ObjectId, name: &str) { 
        self.store.set_layer(id, name); 
    }

    pub fn set_all_layers(&mut self, name: &str) {
        self.store.set_all_layers(name);
    }

    pub fn set_metallic(&mut self, id: ObjectId, metallic: f32) {
        self.store.set_metallic(id, metallic);
    }

    pub fn set_roughness(&mut self, id: ObjectId, roughness: f32) {
        self.store.set_roughness(id, roughness);
    }

    pub fn get_position(&self, id: ObjectId) -> Vec3 {
        self.store.get_pos(id)
    }

    pub fn get_rotation(&self, id: ObjectId) -> Vec3 {
        self.store.get_rot(id)
    }

    pub fn get_scale(&self, id: ObjectId) -> Vec3 {
        self.store.get_scale(id)
    }

    pub fn get_color(&self, id: ObjectId) -> Vec4 {
        self.store.get_color(id)
    }

    pub fn new_light(&mut self) -> LightId {
        for (i, active) in self.active_lights.iter_mut().enumerate() {
            if !*active {
                *active = true;
                self.lights[i] = Light::default();
                return LightId(i);
            }
        }

        eprintln!("Lunar3D: max lights reached");
        LightId(0)
    }

    pub fn remove_light(&mut self, id: LightId) {
        if id.0 < self.active_lights.len() {
            self.active_lights[id.0] = false;
        }
    }

    pub fn remove_all_lights(&mut self) {
        for i in 0..self.active_lights.len() {
            self.active_lights[i] = false;
        }
    }

    pub fn set_light_position(&mut self, id: LightId, position: Vec3) { 
        if id.0 < self.lights.len() { 
            self.lights[id.0].position = position.to_array(); 
        } 
    }

    pub fn set_light_color(&mut self, id: LightId, color: Vec3) { 
        if id.0 < self.lights.len() { 
            self.lights[id.0].color = color.to_array(); 
        } 
    }

    pub fn set_light_intensity(&mut self, id: LightId, intensity: f32) { 
        if id.0 < self.lights.len() { 
            self.lights[id.0].intensity = intensity; 
        } 
    }

    pub fn resize(&mut self, mw: &mut MoonWalk, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.renderer.resize(mw, width, height); 
    }
}
