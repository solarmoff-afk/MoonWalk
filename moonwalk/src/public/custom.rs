// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use moonwalk_backend::{
    core::{
        context::BackendContext,
        buffer::BackendBuffer,
    },
    pipeline::{
        BackendPipeline,
        RawPipeline,
        
        bind::{
            RawBindGroup,
            RawBindGroupLayout,
            BindGroup,
            BindGroupEntry,
        },

        types::{
            ShaderStage,
            BindEntryType,
            TextureType,
            SamplerType,
        }
    },
    render::texture::BackendTextureFormat,
    error::MoonBackendError,
};

use crate::MoonWalk;
use crate::CustomPaint;

pub struct MoonBindGroupLayout {
    pub raw: moonwalk_backend::pipeline::bind::RawBindGroupLayout,
}

use crate::MoonWalkError;
use crate::MoonBindGroup;
use crate::CustomPipeline;
use crate::rendering::custom::MoonBuffer;

/// Ресурсы которые можно передать в шейдер
pub enum BindResource<'a> {
    /// Uniform буфер с данными
    Uniform(&'a MoonBuffer),
    /// Текстура MoonWalk по id (биндится как texture_2d)
    Texture(u32),
    /// Сэмплер текстуры MoonWalk по айди (биндится как sampler)
    Sampler(u32),
}

impl MoonWalk {
    pub fn new_custom_paint(&mut self, width: u32, height: u32, label: &str) -> CustomPaint {
        CustomPaint::new(&mut self.renderer.context, width, height, label)
            .expect("Failed to create CustomPaint")
    }

    pub fn create_vertex_buffer(&mut self, data: &[u8]) -> MoonBuffer {
        let buffer = BackendBuffer::vertex(&mut self.renderer.context, bytemuck::cast_slice(data))
            .expect("Failed to create vertex buffer");

        MoonBuffer {
            raw_buffer: buffer
        }
    }

    pub fn create_index_buffer_u32(&mut self, data: &[u8]) -> MoonBuffer {
        let buffer = BackendBuffer::<u32>::index(
            &mut self.renderer.context, 
            bytemuck::cast_slice::<u8, u32>(data)
        ).expect("Failed to create u32 index buffer");

        MoonBuffer {
            raw_buffer: buffer
        }
    }

    pub fn create_uniform_buffer(&mut self, data: &[u32]) -> MoonBuffer {
        let buffer = BackendBuffer::storage(&mut self.renderer.context, data)
            .expect("Failed to create uniform buffer (using storage)");

        MoonBuffer {
            raw_buffer: buffer
        }
    }

    pub fn update_buffer(&mut self, buffer: &mut MoonBuffer, data: &[u8]) {
        buffer.raw_buffer.update(&mut self.renderer.context, bytemuck::cast_slice(data))
            .expect("Failed to update buffer");
    }

    pub fn create_bind_group_layout(
        &mut self,
        desc: moonwalk_backend::pipeline::bind::BindGroup
    ) -> Result<MoonBindGroupLayout, MoonWalkError> {
        let layout = desc.build(&mut self.renderer.context)
            .map_err(|e| MoonWalkError::BackendError(e.to_string()))?;

        Ok(MoonBindGroupLayout {
            raw: layout
        })
    }

    pub fn compile_pipeline(
        &mut self,
        mut pipeline_desc: moonwalk_backend::pipeline::BackendPipeline,
        layouts: &[&MoonBindGroupLayout]
    ) -> Result<CustomPipeline, MoonWalkError> {
        let format = self.renderer.context.get_format();

        let raw_layouts: Vec<&RawBindGroupLayout> = layouts.iter().map(|l| &l.raw).collect();

        let result = pipeline_desc.build(
            &mut self.renderer.context,
            format,
            &raw_layouts
        )?;

        let raw_pipeline = result.pipeline
            .ok_or(MoonWalkError::ShaderError("Pipeline creation failed".to_string()))?;

        Ok(CustomPipeline {
            raw_pipeline
        })
    }

    pub fn create_bind_group(
        &mut self,
        layout: &MoonBindGroupLayout,
        resources: &[BindResource]
    ) -> Result<MoonBindGroup, MoonWalkError> {
        let mut textures = Vec::new();
        let mut samplers = Vec::new();

        for (binding_idx, res) in resources.iter().enumerate() {
            let binding = binding_idx as u32;

            match res {
                BindResource::Uniform(buf) => {
                    let bg = BindGroup::create_uniform_bind_group(
                        &layout.raw,
                        &mut self.renderer.context,
                        &buf.raw_buffer,
                        Some("Custom uniform bind group")
                    )?;

                    return Ok(MoonBindGroup { raw_bind_group: bg });
                }
                BindResource::Texture(id) => {
                    let tex = self.renderer.state.textures.get(id)
                        .ok_or(MoonWalkError::ShaderError(format!("Texture {} not found", id)))?;

                    textures.push((tex, binding));
                }
                BindResource::Sampler(id) => {
                    let tex = self.renderer.state.textures.get(id)
                        .ok_or(MoonWalkError::ShaderError(format!("Texture {} not found", id)))?;

                    samplers.push((tex, binding));
                }
            }
        }

        let bg = BindGroup::create_texture_bind_group(
            &layout.raw,
            &mut self.renderer.context,
            &textures,
            &samplers,
            Some("Custom texture/sampler bind group")
        )?;

        Ok(MoonBindGroup {
            raw_bind_group: bg
        })
    }
}