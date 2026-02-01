// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use bytemuck::Pod;
use glam::Vec4;

use crate::core::encoder::BackendEncoder;
use crate::pipeline::PipelineResult;
use crate::render::texture::{BackendTexture, RawTexture};
use crate::error::MoonBackendError;

pub struct RenderPass<'a> {
    encoder: &'a mut BackendEncoder,
    raw: Option<wgpu::RenderPass<'a>>,
    label: String,
}

impl<'a> RenderPass<'a> {
    pub fn new(encoder: &'a mut BackendEncoder) -> Self {
        Self {
            encoder,
            raw: None,
            label: "Unname render pass".to_string(),
        }
    }

    pub fn begin_render_pass(
        &'a mut self,
        texture: &BackendTexture,
        clear_color: Option<Vec4>,
    ) -> Result<(), MoonBackendError> {
        let load_op = if let Some(color) = clear_color {
            let wgpu_clear_color: wgpu::Color = wgpu::Color {
                r: color.x as f64,
                g: color.y as f64,
                b: color.z as f64,
                a: color.w as f64,
            };

            wgpu::LoadOp::Clear(wgpu_clear_color)
        } else {
            wgpu::LoadOp::Load
        };

        let view: &wgpu::TextureView = match texture.get_raw() {
            Some(raw) => &raw.view,
            None => {
                return Err(MoonBackendError::RenderPassError("Texture raw not found".into()));
            }
        };

        match self.encoder.get_raw() {
            Some(raw_encoder) => {
                self.raw = Some(raw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(self.label.as_str()),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: load_op,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                }));

                Ok(())
            },

            None => Err(MoonBackendError::EncoderNotFountError),
        }
    }

    pub fn set_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn set_pipeline(&mut self, pipeline: PipelineResult) -> Result<(), MoonBackendError> {
        match &mut self.raw {
            Some(raw) => {
                raw.set_pipeline(&pipeline.get_raw()?.pipeline);
                Ok(())
            },

            None => Err(MoonBackendError::RenderPassError("Render pass not created".into()))
        }
    }

    // pub fn set_bind_group(&mut self, index: u32, group: &'a wgpu::BindGroup) {
    //     self.raw.set_bind_group(index, group, &[]);
    // }

    // pub fn set_vertex_buffer<T: Pod>(&mut self, slot: u32, buffer: &'a Buffer<T>) {
    //     self.raw.set_vertex_buffer(slot, buffer.raw.slice(..));
    // }

    // pub fn set_index_buffer(&mut self, buffer: &'a Buffer<u32>) {
    //     self.raw.set_index_buffer(buffer.raw.slice(..), wgpu::IndexFormat::Uint32);
    // }

    pub fn set_scissor(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), MoonBackendError> {
        match &mut self.raw {
            Some(raw) => {
                raw.set_scissor_rect(x, y, w, h);
                Ok(())
            },

            None => Err(MoonBackendError::RenderPassError("Render pass not created".into()))
        }
    }

    pub fn draw(&mut self, vertex_count: u32) -> Result<(), MoonBackendError> {
        match &mut self.raw {
            Some(raw) => {
                raw.draw(0..vertex_count, 0..1);
                Ok(())
            },

            None => Err(MoonBackendError::RenderPassError("Render pass not created".into()))
        }
    }

    pub fn draw_indexed(&mut self, index_count: u32) -> Result<(), MoonBackendError> {
        match &mut self.raw {
            Some(raw) => {
                raw.draw_indexed(0..index_count, 0, 0..1);
                Ok(())
            },

            None => Err(MoonBackendError::RenderPassError("Render pass not created".into()))
        }
    }

    pub fn draw_indexed_instanced_extended(
        &mut self, 
        index_count: u32, 
        instance_count: u32, 
        base_index: u32, 
        base_vertex: i32, 
        first_instance: u32
    ) -> Result<(), MoonBackendError> {
        match &mut self.raw {
            Some(raw) => {
                raw.draw_indexed(
                    base_index..(base_index + index_count), 
                    base_vertex, 
                    first_instance..(first_instance + instance_count)
                );

                Ok(())
            },

            None => Err(MoonBackendError::RenderPassError("Render pass not created".into()))
        }
    }
}