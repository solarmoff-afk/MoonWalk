// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

#![allow(dead_code)]

use std::ops::Range;
use glam::Vec4;

use crate::gpu::Context;
use crate::rendering::texture::Texture;
use crate::MoonWalk;

#[derive(Clone)]
pub struct CustomPipeline {
    pub(crate) raw: wgpu::RenderPipeline,
}

#[derive(Clone)]
pub struct MoonBuffer {
    pub(crate) raw: wgpu::Buffer,
    pub(crate) size: u64,
    pub(crate) index_format: Option<wgpu::IndexFormat>,
}

#[derive(Clone)]
pub struct MoonBindGroup {
    pub(crate) raw: wgpu::BindGroup,
}

pub struct MoonBindGroupLayout {
    pub(crate) raw: wgpu::BindGroupLayout,
}

#[derive(Clone, Debug)]
pub struct MoonRenderPass {
    pub clear_color: Option<Vec4>,
    pub clear_depth: bool,
}

impl MoonRenderPass {
    pub fn new() -> Self {
        Self { clear_color: None, clear_depth: false }
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
    pass: wgpu::RenderPass<'a>,
}

impl<'a> ActiveMoonRenderPass<'a> {
    pub fn set_pipeline(&mut self, pipeline: &'a CustomPipeline) {
        self.pass.set_pipeline(&pipeline.raw);
    }

    pub fn set_bind_group(&mut self, index: u32, bg: &'a MoonBindGroup) {
        self.pass.set_bind_group(index, &bg.raw, &[]);
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &'a MoonBuffer, offset: u64, size: Option<u64>) {
        if let Some(s) = size {
             self.pass.set_vertex_buffer(slot, buffer.raw.slice(offset..offset+s));
        } else {
             self.pass.set_vertex_buffer(slot, buffer.raw.slice(offset..));
        }
    }

    pub fn set_index_buffer(&mut self, buffer: &'a MoonBuffer, offset: u64, size: Option<u64>) {
        if let Some(format) = buffer.index_format {
            let slice = if let Some(s) = size {
                buffer.raw.slice(offset..offset+s)
            } else {
                buffer.raw.slice(offset..)
            };

            self.pass.set_index_buffer(slice, format);
        }
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.pass.draw(vertices, instances);
    }
    
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.pass.draw_indexed(indices, base_vertex, instances);
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.pass.set_viewport(x, y, w, h, 0.0, 1.0);
    }
}

pub struct CustomPaint {
    pub target: Texture,
    pub depth: Texture,
    pub width: u32,
    pub height: u32,
    active_encoder: Option<wgpu::CommandEncoder>,
}

impl CustomPaint {
    pub fn new(ctx: &Context, width: u32, height: u32, label: &str) -> Self {
        let depth_label = format!("{} (Depth)", label);
        let target = Texture::create_render_target(ctx, width, height, wgpu::TextureFormat::Rgba8UnormSrgb);
        let depth = Texture::create_depth_texture(ctx, width, height, &depth_label);

        Self {
            target,
            depth,
            width,
            height,
            active_encoder: None,
        }
    }

    pub fn start_frame(&mut self, ctx: &Context) {
        if self.active_encoder.is_some() {
            self.active_encoder = None; 
        }

        self.active_encoder = Some(ctx.create_encoder());
    }

    pub fn submit_frame(&mut self, ctx: &Context) {
        if let Some(encoder) = self.active_encoder.take() {
            ctx.submit(encoder);
        }
    }

    pub fn render_pass<'a>(&'a mut self, config: MoonRenderPass) -> Option<ActiveMoonRenderPass<'a>> {
        let encoder = self.active_encoder.as_mut()?;

        let color_load = if let Some(c) = config.clear_color {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: c.x as f64,
                g: c.y as f64,
                b: c.z as f64,
                a: c.w as f64
            })
        } else {
            wgpu::LoadOp::Load
        };

        let depth_load = if config.clear_depth {
            wgpu::LoadOp::Clear(1.0)
        } else {
            wgpu::LoadOp::Load
        };

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("CustomPaint RenderPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: color_load,
                    store: wgpu::StoreOp::Store
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: wgpu::StoreOp::Store
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        Some(ActiveMoonRenderPass {
            pass
        })
    }

    pub fn snapshot(&mut self, mw: &mut MoonWalk) -> u32 {
        let renderer = &mut mw.renderer;
        let result = Texture::create_render_target(
            &renderer.context, self.width, self.height, self.target.texture.format()
        );

        let id = renderer.state.add_texture(result);
        
        self.copy_to_texture(mw, id);
        id
    }

    pub fn update_snapshot(&mut self, mw: &mut MoonWalk, texture_id: u32) {
        self.copy_to_texture(mw, texture_id);
    }

    fn copy_to_texture(&self, mw: &mut MoonWalk, target_id: u32) {
        let renderer = &mut mw.renderer;
        if let Some(target_tex) = renderer.state.textures.get(&target_id) {
            let mut encoder = renderer.context.create_encoder();
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.target.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All
                },

                wgpu::TexelCopyTextureInfo {
                    texture: &target_tex.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All
                },

                wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1
                }
            );
            
            renderer.context.submit(encoder);
        }
    }
}