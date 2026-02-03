// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use crate::core::context::BackendContext;
use crate::render::texture::RawTexture;
use crate::error::MoonBackendError;

pub struct RawEncoder {
    encoder: Option<wgpu::CommandEncoder>,
}

impl RawEncoder {
    pub fn new() -> Self {
        Self {
            encoder: None
        }
    }

    pub fn create_encoder(&mut self, device: &wgpu::Device, label: &str) {
        self.encoder = Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(label),
        }));
    }

    pub fn finish(&mut self) -> Result<wgpu::CommandBuffer, MoonBackendError> {
        match self.encoder.take() {
            Some(raw_encoder) => {
                Ok(raw_encoder.finish())
            },

            None => {
                Err(MoonBackendError::EncoderSubmitError)
            }
        }
    }
}

pub struct BackendEncoder {
    encoder: RawEncoder,
}

impl BackendEncoder {
    pub fn new(&self, context: &mut BackendContext, label: &str) -> Result<Self, MoonBackendError> {
        match &mut context.get_raw().as_mut() {
            // Берём сырой контекст через метод get_raw, проверяет что Option не None
            // (Требование компилятора раста), создаём новый RawEncoder (Сырая обёртка над wgpu
            // которая нужна для возможной смены абстрации), передаём туда ссылку наdevice
            // из сырого контекста и название для енкодера, после чего кладём сырой энкодер
            // в наш Backend encoder и тут возвращается Ok, а еслп сырой контекст в
            // BackendContext это None то значит контекста нет и нужно вернуть
            // ContextNotFoundError

            Some(raw_context) => {
                let mut encoder = RawEncoder::new();
                encoder.create_encoder(&raw_context.device, label);

                Ok(Self {
                    encoder,
                })
            },
            None => {
                Err(MoonBackendError::ContextNotFoundError)
            }
        }
    }

    /// Этот метод нужен для отправки кадра рендера перед презентацией на экране.
    /// Принимает контекст (BackendContext, а не RawContext) чтобы извлечь из
    /// RawContext очередь, взять RawEncoder и провести отправку. Напрямую
    /// передавать queue из RawContext не нужно для простоты использования api
    /// бэкенда
    pub fn submit_frame(&mut self, context: &mut BackendContext) -> Result<(), MoonBackendError> {
        match &mut context.get_raw().as_mut() {
            Some(raw_context) => {
                raw_context.queue.submit(
                    std::iter::once(self.encoder.finish().unwrap())
                );

                Ok(())
            },
            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }

    // Метод для получения сырого wgpu команд энкодера, используется в
    // BackendRenderPass, но может также использоваться за пределами крейта
    pub fn get_raw(&mut self) -> Option<&mut wgpu::CommandEncoder> {
        match &mut self.encoder.encoder {
            Some(raw) => Some(raw),
            None => None,
        }
    }

    /// Метод чтобы копировать текстуру в текстуру
    pub fn copy_texture_to_texture(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        texture: &RawTexture,
        target_texture: &RawTexture,
    ) -> Result<(), MoonBackendError> {
        match &mut self.encoder.encoder {
            Some(raw_encoder) => {
                raw_encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: x,
                            y: y,
                            z: 0
                        },
                        aspect: wgpu::TextureAspect::All,
                    },

                    wgpu::TexelCopyTextureInfo {
                        texture: &target_texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },

                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    }
                );

                Ok(())
            },

            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }
}