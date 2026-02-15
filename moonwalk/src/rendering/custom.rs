// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

#![allow(dead_code)]

use std::ops::Range;
use glam::Vec4;
use moonwalk_backend::{
    core::context::BackendContext,
    core::encoder::BackendEncoder,
    core::buffer::BackendBuffer,
    pipeline::RawPipeline,
    render::texture::BackendTexture,
    render::pass::RenderPass,
    error::MoonBackendError,
};

use crate::MoonWalk;

#[derive(Clone)]
pub struct CustomPipeline {
    pub raw_pipeline: RawPipeline,
}

#[derive(Clone)]
pub struct MoonBuffer {
    pub raw_buffer: BackendBuffer<u32>,
}

#[derive(Clone)]
pub struct MoonBindGroup {
    pub raw_bind_group: moonwalk_backend::pipeline::bind::RawBindGroup,
}

#[derive(Clone)]
pub struct MoonRenderPass {
    pub clear_color: Option<Vec4>,
    pub clear_depth: bool,
}

impl MoonRenderPass {
    pub fn new() -> Self {
        Self {
            clear_color: None,
            clear_depth: false
        }
    }

    pub fn set_clear_color(mut self, color: Option<Vec4>) -> Self {
        self.clear_color = color;
        self
    }

    pub fn set_clear_depth(mut self, clear: bool) -> Self {
        self.clear_depth = clear;
        self
    }
}

impl Default for MoonRenderPass {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ActiveMoonRenderPass<'a> {
    pass: RenderPass<'a>,
}

impl<'a> ActiveMoonRenderPass<'a> {
    pub fn set_pipeline(&mut self, pipeline: &'a CustomPipeline) {
        self.pass.set_pipeline(&pipeline.raw_pipeline);
    }

    pub fn set_bind_group(&mut self, index: u32, bind_group: &'a MoonBindGroup) {
        self.pass.set_bind_group(index, &bind_group.raw_bind_group);
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a MoonBuffer, _offset: u64, _size: Option<u64>) {
        self.pass.set_vertex_buffer(slot, &buffer.raw_buffer);
    }

    pub fn set_index_buffer(&mut self, buffer: &'a MoonBuffer, _offset: u64, _size: Option<u64>) {
        self.pass.set_index_buffer(&buffer.raw_buffer);
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, _base_vertex: i32, _instances: Range<u32>) {
        self.pass.draw_indexed(indices.end - indices.start);
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.pass.set_scissor(x as u32, y as u32, width as u32, height as u32);
    }
}

pub struct CustomPaint {
    pub render_target_texture: BackendTexture,
    pub depth_texture: BackendTexture,
    pub width: u32,
    pub height: u32,

    active_command_encoder: Option<BackendEncoder>,
    current_render_pass_config: Option<MoonRenderPass>,
    current_pipeline: Option<CustomPipeline>,
    current_bind_group_slot_zero: Option<MoonBindGroup>,
    current_vertex_buffer_slot_zero: Option<MoonBuffer>,
    current_index_buffer: Option<MoonBuffer>,
}

impl CustomPaint {
    pub fn new(
        backend_context: &mut BackendContext,
        width: u32,
        height: u32,
        _label: &str,
    ) -> Result<Self, MoonBackendError> {
        let mut render_target_texture = BackendTexture::new(width, height);
        render_target_texture.create_render_target(backend_context, width, height)?;

        let mut depth_texture = BackendTexture::new(width, height);
        depth_texture.create_depth_texture(backend_context, width, height)?;

        Ok(Self {
            render_target_texture,
            depth_texture,
            width,
            height,
            active_command_encoder: None,
            current_render_pass_config: None,
            current_pipeline: None,
            current_bind_group_slot_zero: None,
            current_vertex_buffer_slot_zero: None,
            current_index_buffer: None,
        })
    }

    pub fn start_frame(&mut self, backend_context: &mut BackendContext) -> Result<(), MoonBackendError> {
        if self.active_command_encoder.is_some() {
            let _ = self.submit_frame(backend_context);
        }
        
        let command_encoder = BackendEncoder::new(backend_context, "CustomPaint Command Encoder")?;
        self.active_command_encoder = Some(command_encoder);
        
        Ok(())
    }

    pub fn submit_frame(&mut self, backend_context: &mut BackendContext) -> Result<(), MoonBackendError> {
        if let Some(mut command_encoder) = self.active_command_encoder.take() {
            command_encoder.submit_frame(backend_context)?;
        }

        Ok(())
    }

    pub fn begin_render_pass(&mut self, config: MoonRenderPass) -> Option<ActiveMoonRenderPass<'_>> {
        let command_encoder = self.active_command_encoder.as_mut()?;

        let clear_color = config.clear_color;

        if let Ok(render_pass) = RenderPass::new(
            command_encoder,
            &self.render_target_texture,
            clear_color,
            "CustomPaint Render Pass",
        ) {
            return Some(ActiveMoonRenderPass {
                pass: render_pass
            });
        }

        None
    }

    pub fn set_render_pass_config(&mut self, config: MoonRenderPass) {
        self.current_render_pass_config = Some(config);
    }

    pub fn set_pipeline(&mut self, pipeline: &CustomPipeline) {
        self.current_pipeline = Some(pipeline.clone());
    }

    pub fn set_bind_group(&mut self, index: u32, bind_group: &MoonBindGroup) {
        if index == 0 {
            self.current_bind_group_slot_zero = Some(bind_group.clone());
        }
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &MoonBuffer) {
        if slot == 0 {
            self.current_vertex_buffer_slot_zero = Some(buffer.clone());
        }
    }

    pub fn set_index_buffer(&mut self, buffer: &MoonBuffer) {
        self.current_index_buffer = Some(buffer.clone());
    }

    pub fn draw_indexed(
        &mut self,
        moonwalk: &mut MoonWalk,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) {
        let context = &mut moonwalk.renderer.context;

        if self.active_command_encoder.is_none() {
            let _ = self.start_frame(context);
        }

        let config = self.current_render_pass_config.clone().unwrap_or_default();

        let pipeline = self.current_pipeline.clone();
        let bind_group = self.current_bind_group_slot_zero.clone();
        let vertex_buffer = self.current_vertex_buffer_slot_zero.clone();
        let index_buffer = self.current_index_buffer.clone();

        if let Some(mut active_pass) = self.begin_render_pass(config) {
            if let Some(ref pipeline) = pipeline {
                active_pass.set_pipeline(pipeline);
            }

            if let Some(ref bind_group) = bind_group {
                active_pass.set_bind_group(0, bind_group);
            }

            if let Some(ref vertex_buffer) = vertex_buffer {
                active_pass.set_vertex_buffer(0, vertex_buffer, 0, None);
            }

            if let Some(ref index_buffer) = index_buffer {
                active_pass.set_index_buffer(index_buffer, 0, None);
            }

            active_pass.draw_indexed(indices, base_vertex, instances);
        }

        let _ = self.submit_frame(context);
    }

    pub fn create_snapshot(&mut self, moonwalk: &mut MoonWalk) -> u32 {
        let context = &mut moonwalk.renderer.context;
        let mut snapshot_texture = BackendTexture::new(self.width, self.height);
        let _ = snapshot_texture.create_render_target(context, self.width, self.height);

        let id = moonwalk.renderer.state.add_texture(snapshot_texture);
        self.copy_to_snapshot(moonwalk, id);

        id
    }

    pub fn update_snapshot(&mut self, moonwalk: &mut MoonWalk, snapshot_id: u32) {
        let context = &mut moonwalk.renderer.context;

        if self.active_command_encoder.is_some() {
            let _ = self.submit_frame(context);
        }

        self.copy_to_snapshot(moonwalk, snapshot_id);
    }

    fn copy_to_snapshot(&self, moonwalk: &mut MoonWalk, snapshot_id: u32) {
        let context = &mut moonwalk.renderer.context;

        if let Some(snapshot_texture) = moonwalk.renderer.state.textures.get(&snapshot_id) {
            if let Ok(mut copy_encoder) = BackendEncoder::new(context, "Copy to Snapshot") {
                if let (Some(source_raw), Some(target_raw)) = (
                    self.render_target_texture.get_raw(),
                    snapshot_texture.get_raw(),
                ) {
                    let _ = copy_encoder.copy_texture_to_texture(
                        0, 0, self.width, self.height,
                        source_raw,
                        target_raw,
                    );
                }

                let _ = copy_encoder.submit_frame(context);
            }
        }
    }
}
