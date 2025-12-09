// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};

use crate::easy_gpu::{Buffer, Context};

/// Трейт, который должна реализовать любая структура инстанса
/// чтобы её можно было сортировать
pub trait SortableInstance: Pod + Zeroable {
    fn get_z_index(&self) -> f32;
}

/// Контейнер для батчинга
pub struct BatchBuffer<T: SortableInstance> {
    pub cpu_buffer: Vec<T>,
    pub gpu_buffer: Option<Buffer<T>>,
}

impl<T: SortableInstance> BatchBuffer<T> {
    pub fn new() -> Self {
        Self {
            cpu_buffer: Vec::with_capacity(1024),
            gpu_buffer: None,
        }
    }

    /// Эта функция нужна чтобы очистить cpu буфер перед новым кадром.
    pub fn clear(&mut self) {
        self.cpu_buffer.clear();
    }

    /// Эта функция добавляет элемент в буфер
    #[inline]
    pub fn push(&mut self, instance: T) {
        self.cpu_buffer.push(instance);
    }

    /// Отсортировать объекты по z индексу
    pub fn sort(&mut self) {
        // Здесь используется unstable сортировка так как она просто
        // быстрее и не требует дополнительной памяти. Возможна нестабильность, но
        // тесты показали жизнеспособность этого метода сортировки
        self.cpu_buffer.sort_unstable_by(|a, b| {
            a.get_z_index().total_cmp(&b.get_z_index())
        });
    }

    // Заливаем процессорный буфер на видеокарту создавая вершинные буферы. Функция
    // вернёт true если в буфере есть данные для создания буферов gpu
    pub fn upload(&mut self, ctx: &Context) -> bool {
        if self.cpu_buffer.is_empty() {
            return false;
        }

        if let Some(buf) = &mut self.gpu_buffer {
            buf.update(ctx, &self.cpu_buffer);
        } else {
            self.gpu_buffer = Some(Buffer::vertex(ctx, &self.cpu_buffer));
        }

        true
    }
}