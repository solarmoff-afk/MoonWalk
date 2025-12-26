// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

/// Возвращает матрицу и смещение для изменения яркости factor: 1.0 = оригинал,
/// 0.0 = черный, 2.0 = в 2 раза ярче
pub fn matrix_brightness(factor: f32) -> ([[f32; 4]; 4], [f32; 4]) {
    (
        [
            [factor, 0.0, 0.0, 0.0],
            [0.0, factor, 0.0, 0.0],
            [0.0, 0.0, factor, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],

        [0.0, 0.0, 0.0, 0.0]
    )
}

/// Возвращает матрицу и смещение для изменения контраста contrast: 1.0 = оригинал,
/// 0.5 = серый, 2.0 = высокий контраст
pub fn matrix_contrast(contrast: f32) -> ([[f32; 4]; 4], [f32; 4]) {
    let t = 0.5 * (1.0 - contrast);
    (
        [
            [contrast, 0.0, 0.0, 0.0],
            [0.0, contrast, 0.0, 0.0],
            [0.0, 0.0, contrast, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],

        [t, t, t, 0.0]
    )
}

/// Возвращает матрицу и смещение для изменения насыщенности sat: 1.0 = оригинал,
/// 0.0 = черно-белый, >1.0 = перенасыщенный
pub fn matrix_saturation(sat: f32) -> ([[f32; 4]; 4], [f32; 4]) {
    let lum_r = 0.2126;
    let lum_g = 0.7152;
    let lum_b = 0.0722;
    
    let sr = (1.0 - sat) * lum_r;
    let sg = (1.0 - sat) * lum_g;
    let sb = (1.0 - sat) * lum_b;

    (
        [
            [sr + sat, sr, sr, 0.0],
            [sg, sg + sat, sg, 0.0],
            [sb, sb, sb + sat, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        
        [0.0; 4]
    )
}

/// Возвращает матрицу для изменения оттенка angle_degrees: угол поворота
/// цветового колеса (0..360).
pub fn matrix_hue(angle_degrees: f32) -> ([[f32; 4]; 4], [f32; 4]) {
    let rad = angle_degrees.to_radians();
    let c = rad.cos();
    let s = rad.sin();
    
    // Коэффициенты для вращения в пространстве YIQ/RGB
    let lum_r = 0.213;
    let lum_g = 0.715;
    let lum_b = 0.072;
    
    (
        [
            [
                lum_r + c * (1.0 - lum_r) + s * (-lum_r),
                lum_g + c * (-lum_g) + s * (-lum_g),
                lum_b + c * (-lum_b) + s * (1.0 - lum_b),
                0.0,
            ],
            [
                lum_r + c * (-lum_r) + s * 0.143,
                lum_g + c * (1.0 - lum_g) + s * 0.140,
                lum_b + c * (-lum_b) + s * (-0.283),
                0.0,
            ],
            [
                lum_r + c * (-lum_r) + s * (-(1.0 - lum_r)),
                lum_g + c * (-lum_g) + s * lum_g,
                lum_b + c * (1.0 - lum_b) + s * lum_b,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ],
        
        [0.0; 4]
    )
}