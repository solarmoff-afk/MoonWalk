// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::Pod;

use crate::easy_gpu::pipeline::Pipeline;
use crate::easy_gpu::buffer::Buffer;

pub struct RenderPass<'a> {
    raw: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    pub fn new(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView, clear_color: Option<wgpu::Color>) -> Self {
        let load_op = if let Some(color) = clear_color {
            wgpu::LoadOp::Clear(color)
        } else {
            wgpu::LoadOp::Load
        };

        let raw = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EasyGPU Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        Self {
            raw
        }
    }

    pub fn set_pipeline(&mut self, pipeline: &'a Pipeline) {
        self.raw.set_pipeline(&pipeline.raw);
    }

    pub fn set_bind_group(&mut self, index: u32, group: &'a wgpu::BindGroup) {
        self.raw.set_bind_group(index, group, &[]);
    }

    pub fn set_vertex_buffer<T: Pod>(&mut self, slot: u32, buffer: &'a Buffer<T>) {
        self.raw.set_vertex_buffer(slot, buffer.raw.slice(..));
    }

    pub fn set_index_buffer(&mut self, buffer: &'a Buffer<u32>) {
        self.raw.set_index_buffer(buffer.raw.slice(..), wgpu::IndexFormat::Uint32);
    }

    pub fn set_scissor(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.raw.set_scissor_rect(x, y, w, h);
    }

    pub fn draw(&mut self, vertex_count: u32) {
        self.raw.draw(0..vertex_count, 0..1);
    }

    pub fn draw_indexed(&mut self, index_count: u32) {
        self.raw.draw_indexed(0..index_count, 0, 0..1);
    }

    pub fn draw_instanced(&mut self, vertex_count: u32, instance_count: u32) {
        self.raw.draw(0..vertex_count, 0..instance_count);
    }

    pub fn draw_indexed_instanced(&mut self, index_count: u32, instance_count: u32) {
        self.raw.draw_indexed(0..index_count, 0, 0..instance_count);
    }
}