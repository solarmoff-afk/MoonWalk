// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::marker::PhantomData;
use wgpu::util::DeviceExt;
use bytemuck::Pod;

use crate::easy_gpu::context::Context;

pub struct Buffer<T: Pod> {
    pub raw: wgpu::Buffer,
    pub count: u32,
    _marker: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    pub fn vertex(ctx: &Context, data: &[T]) -> Self {
        Self::create(ctx, data, wgpu::BufferUsages::VERTEX, "Vertex Buffer")
    }

    pub fn index(ctx: &Context, data: &[u32]) -> Buffer<u32> {
        let raw = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        Buffer { raw, count: data.len() as u32, _marker: PhantomData }
    }

    pub fn uniform(ctx: &Context, data: &T) -> Self {
        Self::create(ctx, &[*data], wgpu::BufferUsages::UNIFORM, "Uniform Buffer")
    }
    
    pub fn storage(ctx: &Context, data: &[T]) -> Self {
        Self::create(ctx, data, wgpu::BufferUsages::STORAGE, "Storage Buffer")
    }

    fn create(ctx: &Context, data: &[T], usage: wgpu::BufferUsages, label: &str) -> Self {
        let raw = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        Self { raw, count: data.len() as u32, _marker: PhantomData }
    }

    pub fn update(&mut self, ctx: &Context, data: &[T]) {
        if data.len() as u64 > self.raw.size() / std::mem::size_of::<T>() as u64 {
            let usage = self.raw.usage();
            self.raw = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Resized Buffer"),
                contents: bytemuck::cast_slice(data),
                usage,
            });
        } else {
            ctx.queue.write_buffer(&self.raw, 0, bytemuck::cast_slice(data));
        }
        self.count = data.len() as u32;
    }
    
    pub fn update_one(&self, ctx: &Context, data: &T) {
        ctx.queue.write_buffer(&self.raw, 0, bytemuck::cast_slice(&[*data]));
    }

    pub fn instance(ctx: &Context, data: &[T]) -> Self {
        Self::create(ctx, data, wgpu::BufferUsages::VERTEX, "Instance Buffer")
    }
}