// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::collections::HashMap;
use moonwalk::MoonWalk;
use moonwalk::rendering::custom::{MoonBindGroupLayout, CustomPipeline};
use moonwalk::{MoonPipeline, CullMode, BindGroup, ShaderStage, TextureType, SamplerType};

use crate::core::types::*;
use crate::resources::MeshData;
use crate::scene::LunarScene;

/// Фабрика которая создаёт сцены. Модели общие для всех сцен и загружаются
/// в фабрику
pub struct LunarFactory {
    pub(crate) pipelines: HashMap<String, CustomPipeline>,
    pub(crate) meshes: Vec<MeshData>,
    pub(crate) global_layout: MoonBindGroupLayout,
    pub(crate) shadow_global_layout: MoonBindGroupLayout,
    pub(crate) material_layout: MoonBindGroupLayout,
    pub(crate) default_white: u32,
    pub(crate) default_normal: u32,
}

impl LunarFactory {
    pub fn new(mw: &mut MoonWalk) -> Self {
        let global_layout = mw.create_bind_group_layout(
            BindGroup::new()
                .add_uniform(0, ShaderStage::Both)
                .add_texture(1, TextureType::Depth)
                .add_sampler(2, SamplerType::Comparison)
        ).unwrap();

        let shadow_global_layout = mw.create_bind_group_layout(
            BindGroup::new().add_uniform(0, ShaderStage::Vertex)
        ).unwrap();
        
        let material_layout = mw.create_bind_group_layout(
            BindGroup::new()
                .add_texture(0, TextureType::Float)
                .add_sampler(1, SamplerType::Linear)
                .add_texture(2, TextureType::Float)
                .add_texture(3, TextureType::Float)
                .add_uniform(4, ShaderStage::Fragment)
        ).unwrap();

        let norm_px = vec![128, 128, 255, 255]; 
        let default_normal = mw.renderer.state.add_texture(moonwalk::rendering::texture::Texture::from_raw(
            &mw.renderer.context, &norm_px, 1, 1, "DefNorm").unwrap());
        
        let white_px = vec![255, 255, 255, 255];
        let default_white = mw.renderer.state.add_texture(moonwalk::rendering::texture::Texture::from_raw(
            &mw.renderer.context, &white_px, 1, 1, "DefWhite").unwrap());

        let mut factory = Self {
            pipelines: HashMap::new(),
            meshes: Vec::new(),
            global_layout,
            shadow_global_layout,
            material_layout,
            default_white,
            default_normal,
        };

        factory.register_pipeline(mw, "pbr", include_str!("shaders/pbr.wgsl"));
        factory.register_pipeline(mw, "phong", include_str!("shaders/phong.wgsl"));
        
        factory
    }

    pub fn register_pipeline(&mut self, mw: &MoonWalk, name: &str, src: &str) {
        let desc = MoonPipeline::new(src)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(crate::resources::vertex_layout())
            .add_vertex_layout(crate::resources::instance_layout())
            .cull(CullMode::Back)
            .depth_test(true)
            .depth_write(true)
            .label(name);

        if let Ok(pipe) = mw.compile_pipeline(desc, &[&self.global_layout, &self.material_layout]) {
            self.pipelines.insert(name.to_string(), pipe);
        } else {
            eprintln!("Lunar3D: Failed to compile pipeline '{}'", name);
        }
    }

    pub(crate) fn create_shadow_pipeline(&self, mw: &MoonWalk, src: &str) -> CustomPipeline { 
        let desc = MoonPipeline::new(src)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(crate::resources::vertex_layout())
            .add_vertex_layout(crate::resources::instance_layout())
            .cull(CullMode::Front) 
            .depth_test(true)
            .depth_write(true)
            .label("Shadow Pipeline");
        
        mw.compile_pipeline(desc, &[&self.shadow_global_layout]).unwrap()
    }

    pub fn new_scene(&self, mw: &mut MoonWalk, width: u32, height: u32) -> LunarScene {
        LunarScene::new(mw, width, height, self)
    }

    pub fn load_obj(&mut self, mw: &MoonWalk, bytes: &[u8]) -> Vec<MeshId> {
        let loaded = crate::resources::load_obj(mw, bytes);
        let start_id = self.meshes.len();
        
        let mut ids = Vec::new();
        
        for (i, mesh) in loaded.into_iter().enumerate() {
            self.meshes.push(mesh);
            ids.push(MeshId(start_id + i));
        }

        ids
    }

    pub fn load_gltf(&mut self, mw: &mut MoonWalk, bytes: &[u8]) -> Vec<MeshId> {
        let loaded = crate::resources::load_gltf_bytes(mw, bytes);
        let start_id = self.meshes.len();
        
        let mut ids = Vec::new();
        
        for (i, mesh) in loaded.into_iter().enumerate() {
            self.meshes.push(mesh);
            ids.push(MeshId(start_id + i));
        }
        
        ids
    }
    
    pub fn add_mesh(&mut self, mesh: MeshData) -> MeshId {
        let id = MeshId(self.meshes.len());
        self.meshes.push(mesh);
        
        id
    }

    pub fn get_mesh(&self, id: MeshId) -> Option<&MeshData> {
        self.meshes.get(id.0)
    }
}
