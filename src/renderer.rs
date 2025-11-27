use std::collections::HashMap;
use std::sync::Arc;
use glam::{Mat4, Vec2, Vec4};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use wgpu::util::DeviceExt;

use crate::error::MoonWalkError;
use crate::font::FontSystem;
use crate::objects::{ObjectId, ObjectStore, ShaderId, UniformValue};
use crate::rendering::batch::{rebuild_batch_groups, BatchGroup, release_batch_groups, RenderPass};
use crate::rendering::glyph_cache::GlyphCache;
use crate::rendering::shader::ShaderStore;

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    object_store: ObjectStore,
    shader_store: ShaderStore,
    glyph_cache: GlyphCache,
    batch_groups: HashMap<RenderPass, Vec<BatchGroup>>,
    projection: Mat4,
    width: u32,
    height: u32,
    projection_buffer: wgpu::Buffer,
}

impl<'a> Renderer<'a> {
    pub async fn new<W>(window: &'a W) -> Result<Self, MoonWalkError>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(MoonWalkError::AdapterRequestError)?;
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let device = Arc::new(device);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Bgra8UnormSrgb)
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 1,
            height: 1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        
        let mut shader_store = ShaderStore::new(device.clone());
        let default_rect_shader = shader_store.create_default_rect_shader()?;
        let default_text_shader = shader_store.create_default_text_shader()?;
        let default_bezier_shader = shader_store.create_default_bezier_shader()?;

        let object_store = ObjectStore::new(default_rect_shader, default_text_shader, default_bezier_shader); 
        let glyph_cache = GlyphCache::new(&device, &queue);
        
        let projection = Mat4::orthographic_lh(0.0, 1.0, 1.0, 0.0, -1.0, 1.0);
        let projection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projection Buffer"),
            contents: bytemuck::cast_slice(&[projection]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            surface,
            device,
            queue,
            surface_config,
            object_store,
            shader_store,
            glyph_cache,
            batch_groups: HashMap::new(),
            projection,
            width: 1,
            height: 1,
            projection_buffer,
        })
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            
            self.projection = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
            self.object_store.mark_dirty();
            
            self.queue.write_buffer(&self.projection_buffer, 0, bytemuck::cast_slice(&[self.projection]));
        }
    }

    pub fn render_frame(&mut self, font_system: &mut FontSystem, clear_color: Vec4) -> Result<(), wgpu::SurfaceError> {
        self.device.poll(wgpu::Maintain::Wait);
        
        if self.object_store.is_dirty() {
            release_batch_groups(&mut self.batch_groups);
            
            self.batch_groups = rebuild_batch_groups(
                &self.device,
                &self.object_store,
                &mut self.glyph_cache,
                font_system,
                self.width,
                self.height,
            );

            self.glyph_cache.upload_pending(&self.queue);
            self.object_store.reset_dirty();
        }
        
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut simple_pass_bind_groups = Vec::new();
        if let Some(groups) = self.batch_groups.get(&RenderPass::Simple) {
            for group in groups {
                if let Some(pipeline) = self.shader_store.get_pipeline(group.shader_id) {
                    if group.storage_buffers.is_empty() {
                        let proj_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &pipeline.get_bind_group_layout(0),
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.projection_buffer.as_entire_binding(),
                            }],
                            label: Some("Projection Bind Group"),
                        });
                        simple_pass_bind_groups.push(proj_bind_group);
                    } else {
                        let uniform_buffer = group.storage_buffers.get(&0).unwrap();
                        let storage_buffer = group.storage_buffers.get(&1).unwrap();
                        let bezier_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Bezier Bind Group"),
                            layout: &pipeline.get_bind_group_layout(0),
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 1, resource: storage_buffer.as_entire_binding() },
                            ],
                        });
                        simple_pass_bind_groups.push(bezier_bind_group);
                    }
                }
            }
        }

        let mut glyph_pass_proj_bind_groups = Vec::new();
        if let Some(groups) = self.batch_groups.get(&RenderPass::Glyph) {
            for group in groups {
                if let Some(pipeline) = self.shader_store.get_pipeline(group.shader_id) {
                    let proj_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &pipeline.get_bind_group_layout(0),
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.projection_buffer.as_entire_binding(),
                        }],
                        label: Some("Glyph Projection Bind Group"),
                    });
                    glyph_pass_proj_bind_groups.push(proj_bind_group);
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color.x as f64, g: clear_color.y as f64, b: clear_color.z as f64, a: clear_color.w as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if let Some(groups) = self.batch_groups.get(&RenderPass::Simple) {
                for (group, bind_group) in groups.iter().zip(simple_pass_bind_groups.iter()) {
                    if let Some(pipeline) = self.shader_store.get_pipeline(group.shader_id) {
                        render_pass.set_pipeline(pipeline);

                        if let Some(rect) = group.scissor_rect {
                            render_pass.set_scissor_rect(rect[0], rect[1], rect[2], rect[3]);
                        } else {
                            render_pass.set_scissor_rect(0, 0, self.width, self.height);
                        }

                        render_pass.set_bind_group(0, bind_group, &[]);
                        if group.storage_buffers.is_empty() {
                            render_pass.set_vertex_buffer(0, group.vbo.slice(..));
                        }
                        
                        render_pass.draw(0..group.vertex_count as u32, 0..1);
                    }
                }
            }

            if let Some(groups) = self.batch_groups.get(&RenderPass::Glyph) {
                let glyph_texture_bind_group = self.glyph_cache.get_bind_group();
                for (group, proj_bind_group) in groups.iter().zip(glyph_pass_proj_bind_groups.iter()) {
                    if let Some(pipeline) = self.shader_store.get_pipeline(group.shader_id) {
                        render_pass.set_pipeline(pipeline);
                        
                        if let Some(rect) = group.scissor_rect {
                            render_pass.set_scissor_rect(rect[0], rect[1], rect[2], rect[3]);
                        } else {
                            render_pass.set_scissor_rect(0, 0, self.width, self.height);
                        }
                        
                        render_pass.set_bind_group(0, proj_bind_group, &[]);
                        render_pass.set_bind_group(1, glyph_texture_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, group.vbo.slice(..));
                        render_pass.draw(0..group.vertex_count as u32, 0..1);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
        
    pub fn new_rect(&mut self) -> ObjectId {
        self.object_store.new_rect()
    }
    
    pub fn new_text(&mut self, cs: &mut cosmic_text::FontSystem) -> ObjectId {
        self.object_store.new_text(cs)
    }

    pub fn new_bezier(&mut self) -> ObjectId {
        self.object_store.new_bezier()
    }

    pub fn config_position(&mut self, id: ObjectId, pos: Vec2) {
        self.object_store.config_position(id, pos);
    }
    
    pub fn config_size(&mut self, id: ObjectId, size: Vec2, cs: &mut cosmic_text::FontSystem) {
        self.object_store.config_size(id, size, cs);
    }
    
    pub fn config_rotation(&mut self, id: ObjectId, angle_degrees: f32) {
        self.object_store.config_rotation(id, angle_degrees);
    }

    pub fn config_color(&mut self, id: ObjectId, color: Vec4) {
        self.object_store.config_color(id, color);
    }

    pub fn config_z_index(&mut self, id: ObjectId, z: f32) {
        self.object_store.config_z_index(id, z);
    }

    pub fn config_text(&mut self, id: ObjectId, text: &str, font_system: &mut FontSystem) {
        self.object_store.config_text(id, text, font_system);
    }
    
    pub fn config_font(&mut self, id: ObjectId, font_id: crate::font::FontId) {
        self.object_store.config_font(id, font_id);
    }

    pub fn config_rounded(&mut self, id: ObjectId, radii: Vec4) {
        self.object_store.config_rounded(id, radii);
    }

    pub fn set_bezier_points(&mut self, id: ObjectId, points: Vec<Vec2>) {
        self.object_store.set_bezier_points(id, points);
    }

    pub fn config_bezier_thickness(&mut self, id: ObjectId, thickness: f32) {
        self.object_store.config_bezier_thickness(id, thickness);
    }

    pub fn config_bezier_smooth(&mut self, id: ObjectId, smoothing: f32) {
        self.object_store.config_bezier_smooth(id, smoothing);
    }

    pub fn delete_object(&mut self, id: ObjectId) {
        self.object_store.delete_object(id);
    }

    pub fn clear_all_objects(&mut self) {
        self.object_store.clear_all();
    }

    pub fn compile_shader(&mut self, src: &str) -> Result<ShaderId, MoonWalkError> {
        self.shader_store.compile_shader(src)
    }

    pub fn set_object_shader(&mut self, object_id: ObjectId, shader_id: ShaderId) {
        self.object_store.set_object_shader(object_id, shader_id);
    }

    pub fn set_uniform(&mut self, id: ObjectId, name: String, value: UniformValue) {
        self.object_store.set_uniform(id, name, value);
    }

    pub fn set_parent(&mut self, child: ObjectId, parent: ObjectId) {
        self.object_store.set_parent(child, parent);
    }

    pub fn set_masking(&mut self, id: ObjectId, enable: bool) {
        self.object_store.set_masking(id, enable);
    }
}