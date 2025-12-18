// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use crate::gpu::{Buffer, Context, RenderPass};
use crate::rendering::vertex::{ObjectInstance, InstancePartA, InstancePartB};

/// Хелпер для хранения данных в двух буферах (для старых устройств)

/// Один буфер не влезает из-за ограничений на слабых устройствах,
/// поэтому тут нужна структура для хранения двух буферов 
pub struct SplitStorage {
    buf_a: Option<Buffer<InstancePartA>>,
    buf_b: Option<Buffer<InstancePartB>>,
}

impl SplitStorage {
    pub fn new() -> Self {
        Self {
            buf_a: None,
            buf_b: None
        }
    }

    /// Эта функция берёт массив заполненный полным ObjectInstance и дробит
    /// данные на два буфера по 32 байта чтобы передать их отдельно
    pub fn update(&mut self, ctx: &Context, data: &[ObjectInstance]) {
        if data.is_empty() { return; }

        // Расщепление на CPU (неизбежная плата за совместимость)

        // Неизбежный оверхед для совместимости на устройствах где ограничение
        // данных в 32 бита. Это повлияет на производительность, но избежать
        // этого не выйдет
        let count = data.len();
        let mut part_a = Vec::with_capacity(count);
        let mut part_b = Vec::with_capacity(count);

        for item in data {
            part_a.push(InstancePartA {
                pos_size: item.pos_size,
                uv: item.uv,
                extra: item.extra,
            });

            part_b.push(InstancePartB {
                radii: item.radii,
                gradient_data: item.gradient_data,
                color2: item.color2,
                color: item.color,
                type_id: item.type_id,
                _pad: 0,
            });
        }

        // Заливка первого буфера
        if let Some(buf) = &mut self.buf_a {
            buf.update(ctx, &part_a);
        } else {
            self.buf_a = Some(Buffer::vertex(ctx, &part_a));
        }

        // Заливка второго буфера
        if let Some(buf) = &mut self.buf_b {
            buf.update(ctx, &part_b);
        } else {
            self.buf_b = Some(Buffer::vertex(ctx, &part_b));
        }
    }

    /// Эта функция биндит оба буфера в рендер пасе
    pub fn bind<'a>(&'a self, pass: &mut RenderPass<'a>) {
        if let (Some(a), Some(b)) = (&self.buf_a, &self.buf_b) {
            pass.set_vertex_buffer(1, a);
            pass.set_vertex_buffer(2, b);
        }
    }
    
    pub fn is_ready(&self) -> bool {
        self.buf_a.is_some() && self.buf_b.is_some()
    }
}