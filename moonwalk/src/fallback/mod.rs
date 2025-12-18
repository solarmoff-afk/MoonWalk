// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

pub mod batch;
pub mod pipeline;
pub mod check;

use crate::gpu::context::Context;

pub fn check_fallback(ctx: &Context) -> bool {
    check::is_fallback_required(ctx)
}