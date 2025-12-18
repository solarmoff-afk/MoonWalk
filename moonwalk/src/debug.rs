// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions))]
        {
            println!($($arg)*);
        }
    }
}

#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*) => {
        #[cfg(all(debug_assertions, feature = "verbose"))]
        {
            println!($($arg)*);
        }
    }
}