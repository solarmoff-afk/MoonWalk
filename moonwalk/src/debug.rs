// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use std::time::Instant;
use std::cell::RefCell;

thread_local! {
    static PERF_STACK: RefCell<Vec<(&'static str, Instant)>> = RefCell::new(Vec::new());
}

/// Выводит сообщение в консоль только если это debug сборка
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        {
            println!($($arg)*);
        }
    }
}

/// Выводит сообщение в консоль только если это debug сборка и в добавок
/// крейт был собран с фичей verbose
#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, feature = "verbose"))]
        {
            println!($($arg)*);
        }
    }
}

/// Эти два макроса позволяют изменить производительность. Делается
/// perf_start!("сообщение");
///  // Блок кода
/// perf_end!("сообщение"); // Выводит время в консоль с меткой [PERF]
/// Работает только в debug (dev) сборке и только если крейт был собран с
/// фичей verbose
#[macro_export]
macro_rules! perf_start {
    ($name:expr) => {
        #[cfg(all(debug_assertions, feature = "verbose"))]
        {
            $crate::debug::perf_start($name);
        }
    };
}

#[macro_export]
macro_rules! perf_end {
    ($name:expr) => {
        #[cfg(all(debug_assertions, feature = "verbose"))]
        {
            $crate::debug::perf_end($name);
        }
    };
}

#[macro_export]
macro_rules! perf_scope {
    ($name:expr) => {
        #[cfg(all(debug_assertions, feature = "verbose"))]
        let _perf_scope = $crate::debug::PerfScope::new($name);
    };
}

#[cfg(all(debug_assertions, feature = "verbose"))]
pub fn perf_start(name: &'static str) {
    PERF_STACK.with(|stack| {
        stack.borrow_mut().push((name, Instant::now()));
    });
}

#[cfg(all(debug_assertions, feature = "verbose"))]
pub fn perf_end(name: &'static str) {
    PERF_STACK.with(|stack| {
        if let Some((saved_name, start)) = stack.borrow_mut().pop() {
            assert_eq!(saved_name, name, "perf_start/perf_end mismatch");
            let elapsed = start.elapsed();
            println!("[PERF] {}: {:?}", name, elapsed);
        }
    });
}

#[cfg(all(debug_assertions, feature = "verbose"))]
pub struct PerfScope {
    name: &'static str,
}

#[cfg(all(debug_assertions, feature = "verbose"))]
impl PerfScope {
    pub fn new(name: &'static str) -> Self {
        perf_start!(name);

        Self {
            name,
        }
    }
}

#[cfg(all(debug_assertions, feature = "verbose"))]
impl Drop for PerfScope {
    fn drop(&mut self) {
        perf_end!(self.name);
    }
}
