// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use bytemuck::{Pod, Zeroable};

use moonwalk_backend::{core::buffer::BackendBuffer, error::MoonBackendError};
use moonwalk_backend::core::context::BackendContext;

/// Трейт, который должна реализовать любая структура инстанса
/// чтобы её можно было сортировать
pub trait SortableInstance: Pod + Zeroable {
    fn get_z_index(&self) -> f32;
}

/// Контейнер для батчинга
pub struct BatchBuffer<T: SortableInstance> {
    pub cpu_buffer: Vec<T>,
    pub gpu_buffer: Option<BackendBuffer<T>>,
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
    pub fn upload(&mut self, context: &mut BackendContext) -> Result<bool, MoonBackendError> {
        if self.cpu_buffer.is_empty() {
            return Ok(false);
        }

        if let Some(buf) = &mut self.gpu_buffer {
            buf.update(context, &self.cpu_buffer);
        } else {
            self.gpu_buffer = Some(BackendBuffer::vertex(context, &self.cpu_buffer)?);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::{Pod, Zeroable};

    // Тестовая структура для инстансов
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Pod, Zeroable)]
    struct TestInstance {
        z: f32,
        _padding: [u8; 12],
    }

    impl SortableInstance for TestInstance {
        fn get_z_index(&self) -> f32 {
            self.z
        }
    }

    impl TestInstance {
        fn new(z: f32) -> Self {
            Self {
                z,
                _padding: [0; 12],
            }
        }
    }

    /// Тест создания и очистки буфера
    #[test]
    fn test_buffer_clear() {
        let mut buffer = BatchBuffer::<TestInstance>::new();
        
        // Изначально буфер пустой
        assert!(buffer.cpu_buffer.is_empty());
        assert!(buffer.gpu_buffer.is_none());
        
        // Добавляем инстансы
        buffer.cpu_buffer.push(TestInstance::new(1.0));
        buffer.cpu_buffer.push(TestInstance::new(2.0));
        assert_eq!(buffer.cpu_buffer.len(), 2);
        
        // Очищаем
        buffer.clear();
        assert!(buffer.cpu_buffer.is_empty());

        // Gpu буфер не должен измениться при clear
        assert!(buffer.gpu_buffer.is_none());
    }

    /// Тест создания с капасити
    #[test]
    fn test_buffer_creation() {
        let buffer = BatchBuffer::<TestInstance>::new();
        
        // Проверяем что капасити установлен
        assert!(buffer.cpu_buffer.capacity() >= 1024);
        assert!(buffer.cpu_buffer.is_empty());
    }

    /// Тест что инстансы правильно хранят z индексом
    #[test]
    fn test_z_index_storage() {
        let mut buffer = BatchBuffer::<TestInstance>::new();
        
        buffer.cpu_buffer.push(TestInstance::new(42.0));
        buffer.cpu_buffer.push(TestInstance::new(24.0));
        
        assert_eq!(buffer.cpu_buffer[0].get_z_index(), 42.0);
        assert_eq!(buffer.cpu_buffer[1].get_z_index(), 24.0);
    }

    /// Тест множественных операций
    #[test]
    fn test_multiple_clear_cycles() {
        let mut buffer = BatchBuffer::<TestInstance>::new();
        
        for _cycle in 0..3 {
            // Добавляем инстансы в этом цикле
            for i in 0..10 {
                buffer.cpu_buffer.push(TestInstance::new(i as f32));
            }

            assert_eq!(buffer.cpu_buffer.len(), 10);
            
            // Очищаем для следующего кадра
            buffer.clear();
            assert!(buffer.cpu_buffer.is_empty());
        }
    }

    /// Тест работы с большим количеством инстансов
    #[test]
    fn test_large_batch() {
        let mut buffer = BatchBuffer::<TestInstance>::new();
        
        // Добавляем 10 тысяч инстансов
        for i in 0..10_000 {
            buffer.cpu_buffer.push(TestInstance::new(i as f32));
        }
        
        assert_eq!(buffer.cpu_buffer.len(), 10_000);
        assert_eq!(buffer.cpu_buffer[5000].get_z_index(), 5000.0);
        
        buffer.clear();
        assert!(buffer.cpu_buffer.is_empty());

        // Проверяем что капасити сохранилось (не переаллоцировалось)
        assert!(buffer.cpu_buffer.capacity() >= 10_000);
    }
}
