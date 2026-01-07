// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::ops::Range;
use glam::Vec4;
use wgpu::util::DeviceExt;

use crate::gpu::Context;
use crate::rendering::texture::Texture;
use crate::MoonWalk;

/// Скомпилированный пайплайн который готов к использованию
pub struct CustomPipeline {
    pub(crate) raw: wgpu::RenderPipeline,
}

pub struct MoonBuffer {
    pub(crate) raw: wgpu::Buffer,
    pub(crate) size: u64,
    pub(crate) index_format: Option<wgpu::IndexFormat>,
}

pub struct MoonBindGroup {
    pub(crate) raw: wgpu::BindGroup,
}

pub struct MoonBindGroupLayout {
    pub(crate) raw: wgpu::BindGroupLayout,
}

/// Конфигурация прохода рендеринга, содержит в себе цвет в формате Vec4 из glam
/// и булевое значение очищать ли буфер глубины (обязательно для 3д)
#[derive(Clone, Debug)]
pub struct MoonRenderPass {
    pub clear_color: Option<Vec4>,
    pub clear_depth: bool,
}

impl MoonRenderPass {
    pub fn new() -> Self {
        Self {
            clear_color: None,
            clear_depth: false,
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

/// Конфигурация CustomPaint
pub struct CustomPaint {
    pub target: Texture,
    pub depth: Texture,
    pub width: u32,
    pub height: u32,

    // Состояние
    current_pipeline: Option<wgpu::RenderPipeline>,
    bind_groups: Vec<Option<wgpu::BindGroup>>,
    vertex_buffers: Vec<Option<(wgpu::Buffer, u64)>>, 
    index_buffer: Option<(wgpu::Buffer, wgpu::IndexFormat)>,
    
    // Конфиг
    pass_config: MoonRenderPass,
}

impl CustomPaint {
    pub fn new(ctx: &Context, width: u32, height: u32, label: &str) -> Self {
        let target_label = format!("{} (Target)", label);
        let depth_label = format!("{} (Depth)", label);

        let target = Texture::create_render_target(
            ctx, width, height, wgpu::TextureFormat::Rgba8UnormSrgb
        );
        
        let depth = Texture::create_depth_texture(ctx, width, height, &depth_label);

        Self {
            target,
            depth,
            width,
            height, 
            current_pipeline: None,
            bind_groups: (0..4).map(|_| None).collect(),
            vertex_buffers: (0..8).map(|_| None).collect(),
            index_buffer: None,
            pass_config: MoonRenderPass::default(),
        }
    }

    /// Устанавливает настройки очистки для следующего вызова отрисовки
    pub fn set_render_pass(&mut self, pass: MoonRenderPass) {
        self.pass_config = pass;
    }

    /// Устанавливает скомпилированный пайплайн
    pub fn set_pipeline(&mut self, pipeline: &CustomPipeline) {
        self.current_pipeline = Some(pipeline.raw.clone());
    }

    pub fn set_bind_group(&mut self, index: u32, bg: &MoonBindGroup) {
        if index as usize >= self.bind_groups.len() {
            self.bind_groups.resize_with(index as usize + 1, || None);
        }

        self.bind_groups[index as usize] = Some(bg.raw.clone());
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &MoonBuffer) {
        if slot as usize >= self.vertex_buffers.len() {
            self.vertex_buffers.resize_with(slot as usize + 1, || None);
        }

        self.vertex_buffers[slot as usize] = Some((buffer.raw.clone(), 0));
    }

    pub fn set_index_buffer(&mut self, buffer: &MoonBuffer) {
        if let Some(format) = buffer.index_format {
            self.index_buffer = Some((buffer.raw.clone(), format));
        } else {
            eprintln!("MoonWalk error: Buffer provided to set_index_buffer is not an index buffer");
        }
    }

    /// Рисует геометрию без индексов
    pub fn draw(&mut self, mw: &mut MoonWalk, vertices: Range<u32>, instances: Range<u32>) {
        // base_vertex ноль потому что это не indexed draw
        self.execute_draw(mw, vertices, instances, false, 0);
    }
    
    /// Рисует индексированную геометрию
    pub fn draw_indexed(&mut self, mw: &mut MoonWalk, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.execute_draw(mw, indices, instances, true, base_vertex);
    }

    // Внутренняя реализация отрисовки
    fn execute_draw(&mut self, mw: &mut MoonWalk, range: Range<u32>, instances: Range<u32>, indexed: bool, base_vertex: i32) {
        if self.current_pipeline.is_none() {
            eprintln!("MoonWalk error: pipeline not set in CustomPaint");
            return;
        }

        let ctx = &mw.renderer.context;
        let mut encoder = ctx.create_encoder();

        let color_load = if let Some(c) = self.pass_config.clear_color {
            wgpu::LoadOp::Clear(wgpu::Color { r: c.x as f64, g: c.y as f64, b: c.z as f64, a: c.w as f64 })
        } else {
            wgpu::LoadOp::Load
        };

        let depth_load = if self.pass_config.clear_depth {
            wgpu::LoadOp::Clear(1.0)
        } else { 
            wgpu::LoadOp::Load
        };
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CustomPaint render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.target.view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: color_load, store: wgpu::StoreOp::Store },
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations { load: depth_load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),

                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(pipe) = &self.current_pipeline {
                pass.set_pipeline(pipe);
            }

            for (i, bg) in self.bind_groups.iter().enumerate() {
                if let Some(b) = bg {
                    pass.set_bind_group(i as u32, b, &[]);
                }
            }

            for (i, vb) in self.vertex_buffers.iter().enumerate() {
                if let Some((b, _)) = vb {
                    pass.set_vertex_buffer(i as u32, b.slice(..));
                }
            }

            if indexed {
                if let Some((buf, fmt)) = &self.index_buffer {
                    pass.set_index_buffer(buf.slice(..), *fmt);
                    pass.draw_indexed(range, base_vertex, instances); 
                } else {
                    eprintln!("MoonWalk error: indexed draw call without index buffer");
                }
            } else {
                pass.draw(range, instances);
            }
        }

        ctx.submit(encoder);
    }

    /// Этот метод создаёт текстуру из всего кастом пеинта и возвращает её айди
    pub fn snapshot(&mut self, mw: &mut MoonWalk) -> u32 {
        let renderer = &mut mw.renderer;
        let result = Texture::create_render_target(
            &renderer.context, self.width, self.height, self.target.texture.format()
        );

        let id = renderer.state.add_texture(result);
        self.copy_to_texture(mw, id);
        id
    }

    /// Этот метод похож на snaoshot, но не создаёт новую текстуру, а записывает
    /// содержимое кастом пеинта в уже существующую
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
