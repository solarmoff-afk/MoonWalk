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
            [sr + sat, sg, sb, 0.0],
            [sr, sg + sat, sb, 0.0],
            [sr, sg, sb + sat, 0.0],
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Тест яркости
    #[test]
    fn test_brightness_matrix() {
        // Оригинал ничего не меняет
        let (matrix, offset) = matrix_brightness(1.0);
        assert_eq!(matrix[0][0], 1.0);
        assert_eq!(matrix[1][1], 1.0);
        assert_eq!(matrix[2][2], 1.0);
        assert_eq!(offset, [0.0; 4]);
        
        // Удвоенная яркость
        let (matrix, _) = matrix_brightness(2.0);
        assert_eq!(matrix[0][0], 2.0);
        assert_eq!(matrix[1][1], 2.0);
        assert_eq!(matrix[2][2], 2.0);
        
        // Черный
        let (matrix, _) = matrix_brightness(0.0);
        assert_eq!(matrix[0][0], 0.0);
        assert_eq!(matrix[1][1], 0.0);
        assert_eq!(matrix[2][2], 0.0);
    }

    /// Тест контраста
    #[test]
    fn test_contrast_matrix() {
        // Оригинал
        let (matrix, offset) = matrix_contrast(1.0);
        assert_eq!(matrix[0][0], 1.0);
        assert_eq!(offset, [0.0, 0.0, 0.0, 0.0]);
        
        // Серый
        let (matrix, offset) = matrix_contrast(0.5);
        // t = 0.5 * (1.0 - 0.5) = 0.25
        assert_eq!(offset[0], 0.25);
        assert_eq!(offset[1], 0.25);
        assert_eq!(offset[2], 0.25);
        assert_eq!(matrix[0][0], 0.5);
        
        // Высокий контраст
        let (matrix, offset) = matrix_contrast(2.0);
        // t = 0.5 * (1.0 - 2.0) = -0.5
        assert_eq!(offset[0], -0.5);
        assert_eq!(matrix[0][0], 2.0);
    }

    /// Тест насыщенности
    #[test]
    fn test_saturation_matrix_black_and_white() {
        let (matrix, _) = matrix_saturation(0.0);
        
        let lum_r = 0.2126;
        let lum_g = 0.7152;
        let lum_b = 0.0722;
        
        assert_relative_eq!(matrix[0][0], lum_r, epsilon = 1e-6);
        assert_relative_eq!(matrix[0][1], lum_g, epsilon = 1e-6);
        assert_relative_eq!(matrix[0][2], lum_b, epsilon = 1e-6);
        
        assert_relative_eq!(matrix[1][0], lum_r, epsilon = 1e-6);
        assert_relative_eq!(matrix[1][1], lum_g, epsilon = 1e-6);
        assert_relative_eq!(matrix[1][2], lum_b, epsilon = 1e-6);
        
        assert_relative_eq!(matrix[2][0], lum_r, epsilon = 1e-6);
        assert_relative_eq!(matrix[2][1], lum_g, epsilon = 1e-6);
        assert_relative_eq!(matrix[2][2], lum_b, epsilon = 1e-6);
    }

    /// Тест оттенка
    #[test]
    fn test_hue_matrix() {
        // angle = 0.0 - без изменений
        let (matrix, offset) = matrix_hue(0.0);
        assert_relative_eq!(matrix[0][0], 1.0, epsilon = 1e-6);
        assert_relative_eq!(matrix[0][1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(matrix[0][2], 0.0, epsilon = 1e-6);
        assert_eq!(offset, [0.0; 4]);
        
        // angle = 360.0 - полный круг, должно вернуться к оригиналу
        let (matrix, _) = matrix_hue(360.0);
        assert_relative_eq!(matrix[0][0], 1.0, epsilon = 1e-6);
        assert_relative_eq!(matrix[0][1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(matrix[0][2], 0.0, epsilon = 1e-6);
        
        // angle = 180.0 - дополнительный цвет
        let (matrix, _) = matrix_hue(180.0);
        // Не должно быть NaN
        for row in matrix.iter() {
            for val in row.iter() {
                assert!(!val.is_nan());
            }
        }
    }

    /// Тест применения матрицы к цвету (если есть функция применения)
    #[test]
    fn test_apply_color_matrix() {
        // Тест яркости
        let (matrix, offset) = matrix_brightness(2.0);
        
        // Функция применения матрицы к RGB цвету
        fn apply(color: [f32; 4], matrix: [[f32; 4]; 4], offset: [f32; 4]) -> [f32; 4] {
            let mut result = [0.0; 4];
            for i in 0..4 {
                for j in 0..4 {
                    result[i] += matrix[i][j] * color[j];
                }
                result[i] += offset[i];
            }
            result
        }
        
        let white = [1.0, 1.0, 1.0, 1.0];
        let result = apply(white, matrix, offset);
        
        // Яркость 2.0: белый должен стать [2.0, 2.0, 2.0, 1.0]
        assert_relative_eq!(result[0], 2.0);
        assert_relative_eq!(result[1], 2.0);
        assert_relative_eq!(result[2], 2.0);
        assert_relative_eq!(result[3], 1.0);
    }

    /// Тест крайних значений
    #[test]
    fn test_edge_cases() {
        // Отрицательная яркость
        let (matrix, _) = matrix_brightness(-1.0);
        assert_eq!(matrix[0][0], -1.0);
        
        // Огромный контраст
        let (matrix, offset) = matrix_contrast(100.0);
        // t = 0.5 * (1.0 - 100.0) = -49.5
        assert_eq!(offset[0], -49.5);
        assert_eq!(matrix[0][0], 100.0);
        
        // Огромная насыщенность
        let (matrix, _) = matrix_saturation(100.0);
        assert!(matrix[0][0] > 50.0);
    }

    /// Тест что offset всегда правильной длины
    #[test]
    fn test_offset_length() {
        let (_, offset) = matrix_brightness(1.0);
        assert_eq!(offset.len(), 4);
        
        let (_, offset) = matrix_contrast(1.0);
        assert_eq!(offset.len(), 4);
        
        let (_, offset) = matrix_saturation(1.0);
        assert_eq!(offset.len(), 4);
        
        let (_, offset) = matrix_hue(0.0);
        assert_eq!(offset.len(), 4);
    }
}
