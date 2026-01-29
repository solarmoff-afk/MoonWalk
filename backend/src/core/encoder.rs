// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use crate::core::context::BackendContext;
use crate::error::MoonBackendError;

struct RawEncoder {
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
}

pub struct BackendEncoder {
    encoder: Option<RawEncoder>,
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
                    encoder: Some(encoder),
                })
            },
            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }
}