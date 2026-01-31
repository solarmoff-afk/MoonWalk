// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use super::types::*;

use crate::error::MoonBackendError;
use crate::core::context::BackendContext;

/// Обёртка для wgpu типа чтобы импортировать и хранить его без подключения
/// wgpu в основном крейте
pub struct RawBindGroupLayout {
    pub raw: wgpu::BindGroupLayout,
}

impl RawBindGroupLayout {
    pub fn new(layout: wgpu::BindGroupLayout) -> Self {
        Self {
            raw: layout
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindGroupEntry {
    /// Индекс binding
    pub binding: u32,
    /// Тип ресурса
    pub entry_type: BindEntryType,
    /// В каких шейдерах доступен
    pub visibility: ShaderStage,
    /// Тип текстуры (только для Texture)
    pub sample_type: Option<TextureType>,
    /// Тип сэмплера (только для Sampler)
    pub sampler_type: Option<SamplerType>,
}

/// Описание bind group
#[derive(Debug, Clone)]
pub struct BindGroup {
    /// Список элементов в bind group
    pub entries: Vec<BindGroupEntry>,
}

impl BindGroup {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_uniform(mut self, binding: u32, visibility: ShaderStage) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            entry_type: BindEntryType::Uniform,
            visibility,
            sample_type: None,
            sampler_type: None,
        });

        self
    }

    pub fn add_texture(mut self, binding: u32, sample_type: TextureType) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            entry_type: BindEntryType::Texture,
            visibility: ShaderStage::Fragment,
            sample_type: Some(sample_type),
            sampler_type: None,
        });

        self
    }

    pub fn add_sampler(mut self, binding: u32, sampler_type: SamplerType) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            entry_type: BindEntryType::Sampler,
            visibility: ShaderStage::Fragment,
            sample_type: None,
            sampler_type: Some(sampler_type),
        });

        self
    }

    pub fn add_storage(mut self, binding: u32, read_only: bool, visibility: ShaderStage) -> Self {
        self.entries.push(BindGroupEntry {
            binding,
            
            entry_type: if read_only {
                BindEntryType::StorageRead
            } else {
                BindEntryType::StorageWrite
            },

            visibility,
            sample_type: None,
            sampler_type: None,
        });

        self
    }

    // TODO: Создать build_v2 который возвращает кастомный тип, а не wgpu
    pub(crate) fn build(&self, context: &mut BackendContext) -> Result<RawBindGroupLayout, MoonBackendError> {
        match &mut context.get_raw() {
            Some(raw_context) => {
                let entries: Vec<wgpu::BindGroupLayoutEntry> = self.entries
                    .iter()
                    .map(|entry| {
                        let visibility = match entry.visibility {
                            ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
                            ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
                            ShaderStage::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
                        };

                        let ty = match entry.entry_type {
                            BindEntryType::Uniform => wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },

                            BindEntryType::Texture => wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: match entry.sample_type.unwrap_or(TextureType::Float) {
                                    TextureType::Float => wgpu::TextureSampleType::Float {
                                        filterable: true
                                    },

                                    TextureType::Depth => wgpu::TextureSampleType::Depth,
                                    TextureType::Uint => wgpu::TextureSampleType::Uint,
                                },
                            },

                            BindEntryType::Sampler => wgpu::BindingType::Sampler(
                                match entry.sampler_type.unwrap_or(SamplerType::Linear) {
                                    SamplerType::Linear => wgpu::SamplerBindingType::Filtering,
                                    SamplerType::Nearest => wgpu::SamplerBindingType::NonFiltering,
                                    SamplerType::Comparison => wgpu::SamplerBindingType::Comparison,
                                }
                            ),

                            BindEntryType::StorageRead => wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: true
                                },

                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },

                            BindEntryType::StorageWrite => wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: false
                                },

                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },

                        };

                        wgpu::BindGroupLayoutEntry {
                            binding: entry.binding,
                            visibility,
                            ty,
                            count: None,
                        }
                    })
                    .collect();

                let layout = raw_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &entries,
                });

                let raw = RawBindGroupLayout::new(layout);
                Ok(raw)
            },

            None => Err(MoonBackendError::ContextNotFoundError),
        }
    }
}

impl Default for BindGroup {
    fn default() -> Self {
        Self::new()
    }
}