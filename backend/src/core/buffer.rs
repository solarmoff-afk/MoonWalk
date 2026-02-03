// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use std::marker::PhantomData;
use wgpu::util::DeviceExt;
use bytemuck::Pod;

use crate::core::context::BackendContext;
use crate::error::MoonBackendError;

pub struct BackendBuffer<T: Pod> {
    // Сырой буфер wgpu
    pub raw: wgpu::Buffer,

    pub count: u32,
    _marker: PhantomData<T>,
}

impl<T: Pod> BackendBuffer<T> {
    pub fn vertex(context: &mut BackendContext, data: &[T]) -> Result<Self, MoonBackendError> {
        Self::create(
            context,
            data,
            wgpu::BufferUsages::VERTEX,
            "Vertex Buffer",
        ).map_err(|_e| MoonBackendError::ContextNotFoundError)
    }

    pub fn uniform(context: &mut BackendContext, data: &T) -> Result<Self, MoonBackendError> {
        Self::create(
            context,
            &[*data],
            wgpu::BufferUsages::UNIFORM,
            "Uniform Buffer",
        ).map_err(|_e| MoonBackendError::ContextNotFoundError)
    }
    
    pub fn storage(context: &mut BackendContext, data: &[T]) -> Result<Self, MoonBackendError> {
        Self::create(
            context,
            data,
            wgpu::BufferUsages::STORAGE,
            "Storage Buffer",
        ).map_err(|_e| MoonBackendError::ContextNotFoundError)
    }

    pub fn index(context: &mut BackendContext, data: &[u32]) -> Result<BackendBuffer<u32>, MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                let raw = raw_context.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(data),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    }
                );

                Ok(BackendBuffer {
                    raw,
                    count: data.len() as u32,
                    _marker: PhantomData,
                })
            },

            None => {
                Err(MoonBackendError::ContextNotFoundError)
            }
        }
    }

    // Приватня функция создания wgpu буфера
    fn create(
        context: &mut BackendContext,
        data: &[T],
        usage: wgpu::BufferUsages,
        label: &str
    ) -> Result<Self, MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                let raw = raw_context.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(label),
                        contents: bytemuck::cast_slice(data),
                        usage: usage | wgpu::BufferUsages::COPY_DST,
                    }
                );

                Ok(Self {
                    raw,
                    count:
                    data.len() as u32,
                    _marker: PhantomData,
                })
            },

            None => {
                Err(MoonBackendError::ContextNotFoundError)
            }
        }
    }

    pub fn update(&mut self, context: &mut BackendContext, data: &[T]) -> Result<(), MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                // [MAYBE]
                // Если длина data больше чем размер сырого wgpu буфера пересоздаём сырой
                // wgpu буфер

                if data.len() as u64 > self.raw.size() / std::mem::size_of::<T>() as u64 {
                    let usage = self.raw.usage();
                    
                    self.raw = raw_context.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Resized Buffer"),
                            contents: bytemuck::cast_slice(data),
                            usage,
                        }
                    );

                    // [REFACTORME]
                    // Дубляю кода чтобы предотвратить куда более большой дубляж
                    // кода
                    self.count = data.len() as u32;

                    Ok(())
                } else {
                    raw_context.queue.write_buffer(
                        &self.raw,
                        0,
                        bytemuck::cast_slice(data),
                    );

                    // [REFACTORME]
                    // Дубляю кода чтобы предотвратить куда более большой дубляж
                    // кода
                    self.count = data.len() as u32;

                    Ok(())
                }
            },

            None => Err(MoonBackendError::ContextNotFoundError)
        }
    }
    
    pub fn update_one(&self, context: &mut BackendContext, data: &T) -> Result<(), MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                Ok(raw_context.queue.write_buffer(
                    &self.raw,
                    0,
                    bytemuck::cast_slice(&[*data]),
                ))
            },

            None => {
                Err(MoonBackendError::ContextNotFoundError)
            }
        }
    }

    pub fn instance(context: &mut BackendContext, data: &[T]) -> Result<Self, MoonBackendError> {
        Self::create(
            context,
            data,
            wgpu::BufferUsages::VERTEX,
            "Instance Buffer",
        ).map_err(|_e| MoonBackendError::ContextNotFoundError)
    }
}