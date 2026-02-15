// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod color_matrix;
pub mod uniforms;
pub mod factory;

use bytemuck::Pod;

use moonwalk_backend::core::context::BackendContext;
use moonwalk_backend::render::texture::BackendTexture;
use moonwalk_backend::core::buffer::BackendBuffer;
use moonwalk_backend::pipeline::PipelineResult;
use moonwalk_backend::pipeline::bind::{BindGroup, RawBindGroupLayout, RawBindGroup};
use moonwalk_backend::pipeline::types::{ShaderStage, TextureType, SamplerType};
use moonwalk_backend::core::encoder::BackendEncoder;
use moonwalk_backend::render::texture::BackendTextureFormat;

use crate::error::MoonWalkError;

use self::uniforms::*;

pub struct FilterSystem {
    swap_texture: Option<BackendTexture>,

    blur_pipeline: PipelineResult,
    color_pipeline: PipelineResult,
    advanced_pipeline: PipelineResult,
    
    uniform_layout: RawBindGroupLayout,
    texture_layout: RawBindGroupLayout,
    advanced_texture_layout: RawBindGroupLayout,

    dummy_vbo: BackendBuffer<DummyVertex>,
}

impl FilterSystem {
    pub fn new(context: &mut BackendContext) -> Result<Self, MoonWalkError> {
        let dummy_vertices = [DummyVertex {
            _dummy: 0.0
        }];

        let dummy_vbo = BackendBuffer::vertex(context, &dummy_vertices);

        let uniform_layout = BindGroup::new()
            .add_uniform(0, ShaderStage::Fragment)
            .build(context)?;

        let texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .build(context)?;

        let advanced_texture_layout = BindGroup::new()
            .add_texture(0, TextureType::Float)
            .add_sampler(1, SamplerType::Linear)
            .add_texture(2, TextureType::Float)
            .build(context)?;

        let blur_pipeline = factory::create_blur_pipeline(context, &uniform_layout, &texture_layout)?;
        let color_pipeline = factory::create_color_pipeline(context, &uniform_layout, &texture_layout)?;
        let advanced_pipeline = factory::create_advanced_pipeline(context, &uniform_layout, &advanced_texture_layout)?;
        
        Ok(Self {
            swap_texture: None,
            blur_pipeline,
            color_pipeline,
            advanced_pipeline,
            uniform_layout,
            texture_layout,
            advanced_texture_layout,
            dummy_vbo: dummy_vbo?,
        })
    }

    pub fn apply_blur(
        &mut self,
        context: &mut BackendContext,
        target_texture: &BackendTexture,
        radius: f32,
        horizontal: bool
    ) -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref()
            .ok_or(MoonWalkError::TextureLoading("Failed to get filters system swap texture ref".to_string()))?;

        let dir = if horizontal {
            [1.0, 0.0]
        } else {
            [0.0, 1.0]
        };
        
        let uniform_data = BlurUniform {
            direction: dir,
            radius,
            _pad: 0.0,
            resolution: [width as f32, height as f32],
        };

        self.execute_pass(
            context,
            &self.blur_pipeline,
            target_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;

        Ok(())
    }

    pub fn apply_color_matrix(
        &mut self,
        context: &mut BackendContext,
        target_texture: &mut BackendTexture,
        matrix: [[f32; 4]; 4],
        offset: [f32; 4]
    )  -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = ColorMatrixUniform {
            matrix, offset
        };

        self.execute_pass(
            context,
            &self.color_pipeline,
            target_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;

        Ok(())
    }

    pub fn apply_chromakey(
        &mut self,
        context: &mut BackendContext,
        target_texture: &mut BackendTexture,
        key_color: [f32; 3],
        tolerance: f32
    ) -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = AdvancedUniform {
            key_color,
            tolerance,
            params: [1.0, 0.0, 0.0, 0.0],
        };

        self.execute_advanced_pass(
            context,
            target_texture,
            target_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;

        Ok(())
    }

    pub fn apply_stencil(
        &mut self,
        context: &mut BackendContext,
        target_texture: &BackendTexture,
        mask_texture: &BackendTexture,
        invert: bool
    ) -> Result<(), MoonWalkError> {
        let width = target_texture.width;
        let height = target_texture.height;
        
        self.ensure_swap_texture(context, width, height, target_texture.config.get_format());
        let swap = self.swap_texture.as_ref().unwrap();

        let uniform_data = AdvancedUniform {
            key_color: [0.0; 3],
            tolerance: 0.0,
            params: [2.0, if invert {
                1.0
            } else {
                0.0
            }, 0.0, 0.0],
        };

        self.execute_advanced_pass(
            context,
            target_texture,
            mask_texture,
            swap,
            &uniform_data,
        );

        self.blit_back(context, target_texture, swap, width, height)?;
    
        Ok(())
    }

    fn ensure_swap_texture(&mut self, context: &mut BackendContext, w: u32, h: u32, format: BackendTextureFormat) {
        let need_create = self.swap_texture.as_ref()
            .map_or(true, |t| t.width != w || t.height != h);

        if need_create {
            let mut texture = BackendTexture::new(w, h);
            texture.config.set_format(format);
            texture.create_render_target(context, w, h);

            self.swap_texture = Some(texture);
        }
    }

    fn blit_back(
        &self,
        context: &mut BackendContext,
        target: &BackendTexture,
        source: &BackendTexture,
        width: u32,
        height: u32
    ) -> Result<(), MoonWalkError> {
        let mut encoder = BackendEncoder::new(
            context, "MoonWalk encoder blit"
        )?;

        let source_raw = match source.get_raw() {
            Some(raw) => raw,
            None => return Err(MoonWalkError::BackendError("Texture is empty".to_string())),
        };

        let target_raw = match target.get_raw() {
            Some(raw) => raw,
            None => return Err(MoonWalkError::BackendError("Texture is empty".to_string())),
        };
        
        encoder.copy_texture_to_texture(0, 0, width, height, source_raw, target_raw);
        encoder.submit_frame(context);

        Ok(())
    }

    fn execute_pass<T: Pod>(
        &self,
        context: &mut BackendContext,
        pipeline: &PipelineResult,
        source: &BackendTexture,
        dest: &BackendTexture,
        uniform_data: &T,
    ) -> Result<(), MoonWalkError> {
        let uniform_buffer = BackendBuffer::<T>::uniform_bytes(
            context, 
            bytemuck::bytes_of(uniform_data)
        )?;

        let uniform_bg = BindGroup::create_uniform_bind_group(
            &self.uniform_layout,
            context,
            &uniform_buffer,
            Some("Filter Uniform BG")
        )?;

        let texture_bg = BindGroup::create_texture_bind_group(
            &self.texture_layout,
            context,
            &[(source, 0)],
            &[(source, 1)],
            Some("Filter Texture BG")
        )?;

        self.run_pipeline(context, pipeline, dest, &uniform_bg, &texture_bg)?;
    
        Ok(())
    }

    fn execute_advanced_pass<T: Pod>(
        &self,
        context: &mut BackendContext,
        source: &BackendTexture,
        mask: &BackendTexture,
        dest: &BackendTexture,
        uniform_data: &T,
    ) -> Result<(), MoonWalkError> {
        let uniform_buffer = BackendBuffer::<T>::uniform_bytes(
            context, 
            bytemuck::bytes_of(uniform_data)
        )?;

        let uniform_bg = BindGroup::create_uniform_bind_group(
            &self.uniform_layout,
            context,
            &uniform_buffer,
            Some("Filter Uniform BG")
        )?;

        let texture_bg = BindGroup::create_texture_bind_group(
            &self.advanced_texture_layout,
            context,
            &[(source, 0), (mask, 2)],
            &[(source, 1)],
            Some("Advanced Texture BG")
        )?;

        self.run_pipeline(context, &self.advanced_pipeline, dest, &uniform_bg, &texture_bg)?;
    
        Ok(())
    }

    fn run_pipeline(
        &self, 
        context: &mut BackendContext, 
        pipeline: &PipelineResult, 
        dest: &BackendTexture, 
        bg0: &RawBindGroup, 
        bg1: &RawBindGroup,
    ) -> Result<(), MoonWalkError> {
        use glam::Vec4;
        use moonwalk_backend::render::pass::RenderPass;

        let mut encoder = BackendEncoder::new(context, "MoonWalk filters encoder")?;
        let mut pass = RenderPass::new(
            &mut encoder,
            dest,

            // Прозрачный цвет для заливки чтобы если фильтр применился к png картинке
            // это wgpu не перекрыл бы прозрачный фон своим каким-то цветом
            Some(Vec4::ZERO),
            "Filter render pass"
        )?;

        pass.set_pipeline(pipeline.get_raw()?);
        pass.set_bind_group(0, bg0);
        pass.set_bind_group(1, bg1);
        pass.set_vertex_buffer(0, &self.dummy_vbo);
        
        // Отрисовка !!! Можно выкидывать в мусор, работа завершена
        pass.draw(3);

        // Боров чекер хочет чтобы я выкинул на свалку экземпляр прохода
        // чтобы вызвать метод submit_frame который работает с &mut self
        // BackendEncoder, поэтому нужно сделать дроп чтобы освободить
        // заимствование, pass.draw вызывается выше
        drop(pass);

        encoder.submit_frame(context);

        Ok(())
    }
}