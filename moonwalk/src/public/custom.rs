// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use wgpu::util::DeviceExt;

use crate::MoonWalk;
use crate::CustomPaint;
use crate::MoonBindGroupLayout;
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
    /// CustomPaint это второй способ оффскрин рендеринга который отличается от
    /// RenderContainer тем, что не содержит в себе абстраций MoonWalk выского
    /// уровня (батчинг, текстовая система, объекты, стандартный пайплайн) и
    /// позволяет настроить пайплайн (Конвейер рендеринга) и рендер пасс
    /// (проход рендеринга) своими руками что открывает возможности для создания
    /// рендеринга любой сложности, в том числе и трехмерной отрисовки. Принимает
    /// ширину полотна, высоту полотна и название для него. (Например "my scene")
    /// Возвращает структуру CustomPaint которую можно конфигурировать задав
    /// MoonPipeline и MoonRenderPass
    pub fn new_custom_paint(&self, width: u32, height: u32, label: &str) -> CustomPaint {
        CustomPaint::new(&self.renderer.context, width, height, label)
    }

    /// [WAIT DOC]
    pub fn create_vertex_buffer(&self, data: &[u8]) -> MoonBuffer {
        let buffer = self.renderer.context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Vertex Buffer"),
            contents: data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        MoonBuffer { raw: buffer, size: data.len() as u64, index_format: None }
    }

    /// [WAIT DOC]
    pub fn create_index_buffer_u16(&self, data: &[u8]) -> MoonBuffer {
        let buffer = self.renderer.context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Index Buffer u16"),
            contents: data,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        MoonBuffer { raw: buffer, size: data.len() as u64, index_format: Some(wgpu::IndexFormat::Uint16) }
    }

    /// [WAIT DOC]
    pub fn create_index_buffer_u32(&self, data: &[u8]) -> MoonBuffer {
        let buffer = self.renderer.context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Index Buffer u32"),
            contents: data,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        MoonBuffer { raw: buffer, size: data.len() as u64, index_format: Some(wgpu::IndexFormat::Uint32) }
    }

    /// [WAIT DOC]
    pub fn create_uniform_buffer(&self, data: &[u8]) -> MoonBuffer {
        let buffer = self.renderer.context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Uniform Buffer"),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        MoonBuffer { raw: buffer, size: data.len() as u64, index_format: None }
    }

    /// [WAIT DOC]
    pub fn update_buffer(&self, buffer: &MoonBuffer, data: &[u8]) {
        self.renderer.context.queue.write_buffer(&buffer.raw, 0, data);
    }

    /// [WAIT DOC]
    pub fn create_bind_group_layout(
        &self, 
        desc: crate::r#abstract::BindGroup
    ) -> Result<MoonBindGroupLayout, MoonWalkError> {
        let layout = desc.build(&self.renderer.context)?;
        Ok(MoonBindGroupLayout {
            raw: layout
        })
    }

    pub fn compile_pipeline(
        &self, 
        pipeline_desc: crate::r#abstract::MoonPipeline, 
        layouts: &[&MoonBindGroupLayout]
    ) -> Result<CustomPipeline, MoonWalkError> {
        let raw_layouts: Vec<&wgpu::BindGroupLayout> = layouts.iter().map(|l| &l.raw).collect();
        
        let result = pipeline_desc.build(
            &self.renderer.context, 
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &raw_layouts
        )?;
        
        Ok(CustomPipeline {
            raw: result.pipeline.raw
        })
    }

    pub fn create_bind_group(
        &self, 
        layout: &MoonBindGroupLayout, 
        resources: &[BindResource]
    ) -> Result<MoonBindGroup, MoonWalkError> {
        let mut entries = Vec::with_capacity(resources.len());

        for (i, res) in resources.iter().enumerate() {
            let resource = match res {
                BindResource::Uniform(buf) => {
                    wgpu::BindingResource::Buffer(buf.raw.as_entire_buffer_binding())
                },

                BindResource::Texture(id) => {
                    let tex = self.renderer.state.textures.get(id)
                        .ok_or(MoonWalkError::ShaderError(format!("Texture {} not found", id)))?;
                    wgpu::BindingResource::TextureView(&tex.view)
                },

                BindResource::Sampler(id) => {
                    let tex = self.renderer.state.textures.get(id)
                        .ok_or(MoonWalkError::ShaderError(format!("Texture {} not found", id)))?;
                    wgpu::BindingResource::Sampler(&tex.sampler)
                },
            };
            
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource,
            });
        }

        let bg = self.renderer.context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Custom bind group"),
            layout: &layout.raw,
            entries: &entries,
        });

        Ok(MoonBindGroup { raw: bg })
    }
}
