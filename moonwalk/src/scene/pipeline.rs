// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use crate::MoonWalk;
use crate::rendering::custom::*;
use crate::r#abstract::*;
use crate::scene::types::*;

pub(crate) fn create_layouts(mw: &MoonWalk) -> (MoonBindGroupLayout, MoonBindGroupLayout, MoonBindGroupLayout) {
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
    
    (global_layout, shadow_global_layout, material_layout)
}

pub(crate) fn create_pipelines(
    mw: &MoonWalk, 
    global: &MoonBindGroupLayout, 
    shadow_global: &MoonBindGroupLayout,
    material: &MoonBindGroupLayout
) -> (CustomPipeline, CustomPipeline, CustomPipeline) {
    let pbr_src = include_str!("shaders/pbr.wgsl");
    let phong_src = include_str!("shaders/phong.wgsl");
    let shadow_src = include_str!("shaders/shadow.wgsl");

    let pbr_desc = MoonPipeline::new(pbr_src)
        .vertex_shader("vs_main").fragment_shader("fs_main")
        .add_vertex_layout(vertex_layout()).add_vertex_layout(instance_layout())
        .cull(CullMode::Back).depth_test(true).depth_write(true).label("Scene3D PBR");
    let pipeline_pbr = mw.compile_pipeline(pbr_desc, &[global, material]).unwrap();

    let phong_desc = MoonPipeline::new(phong_src)
        .vertex_shader("vs_main").fragment_shader("fs_main")
        .add_vertex_layout(vertex_layout()).add_vertex_layout(instance_layout())
        .cull(CullMode::Back).depth_test(true).depth_write(true).label("Scene3D Phong");
    let pipeline_phong = mw.compile_pipeline(phong_desc, &[global, material]).unwrap();

    let shadow_desc = MoonPipeline::new(shadow_src)
        .vertex_shader("vs_main").fragment_shader("fs_main") 
        .add_vertex_layout(vertex_layout()).add_vertex_layout(instance_layout())
        .cull(CullMode::Back).depth_test(true).depth_write(true).label("Scene3D Shadow");
    let shadow_pipeline = mw.compile_pipeline(shadow_desc, &[shadow_global]).unwrap();
    
    (pipeline_pbr, pipeline_phong, shadow_pipeline)
}

fn vertex_layout() -> VertexLayout {
    VertexLayout::new()
        .stride(std::mem::size_of::<Vertex3D>() as u32)
        .step_mode(StepMode::Vertex)
        .add_attr(VertexAttr::new().format(Format::Float32x3).location(0).offset(0))  
        .add_attr(VertexAttr::new().format(Format::Float32x3).location(1).offset(12)) 
        .add_attr(VertexAttr::new().format(Format::Float32x2).location(2).offset(24)) 
        .add_attr(VertexAttr::new().format(Format::Float32x3).location(3).offset(32)) 
}

fn instance_layout() -> VertexLayout {
    VertexLayout::new()
        .stride(std::mem::size_of::<InstanceRaw>() as u32)
        .step_mode(StepMode::Instance)
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(4).offset(0))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(5).offset(16))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(6).offset(32))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(7).offset(48))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(8).offset(64))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(9).offset(80))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(10).offset(96))
        .add_attr(VertexAttr::new().format(Format::Float32x4).location(11).offset(112))
        .add_attr(VertexAttr::new().format(Format::Float32).location(12).offset(128))
        .add_attr(VertexAttr::new().format(Format::Float32).location(13).offset(132))
}
