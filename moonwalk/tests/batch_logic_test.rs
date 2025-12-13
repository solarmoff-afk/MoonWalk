// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

#[derive(Debug, Clone, Copy, PartialEq)]
struct MockObj {
    id: u32,
    z: f32,
    tex: u32,
}

#[derive(Debug, PartialEq)]
struct DrawCall {
    texture_id: u32,
    count: u32,
}

#[test]
fn test_dynamic_batching_split() {
    let mut objects = vec![
        MockObj { id: 1, z: 10.0, tex: 0 },
        MockObj { id: 3, z: 30.0, tex: 0 },
        MockObj { id: 2, z: 20.0, tex: 1 },
        MockObj { id: 4, z: 30.1, tex: 1 },
    ];

    objects.sort_by(|a, b| a.z.total_cmp(&b.z));
    
    let mut draw_calls = Vec::new();
    
    if !objects.is_empty() {
        let mut current_tex = objects[0].tex;
        let mut count = 0;

        for obj in &objects {
            if obj.tex != current_tex {
                draw_calls.push(DrawCall { texture_id: current_tex, count });
                
                current_tex = obj.tex;
                count = 0;
            }
            count += 1;
        }
        draw_calls.push(DrawCall { texture_id: current_tex, count });
    }
    
    assert_eq!(draw_calls.len(), 4);
    
    assert_eq!(draw_calls[0], DrawCall { texture_id: 0, count: 1 });
    assert_eq!(draw_calls[1], DrawCall { texture_id: 1, count: 1 });
    assert_eq!(draw_calls[2], DrawCall { texture_id: 0, count: 1 });
    assert_eq!(draw_calls[3], DrawCall { texture_id: 1, count: 1 });
}

#[test]
fn test_dynamic_batching_optimal() {
    let mut objects = vec![
        MockObj { id: 1, z: 1.0, tex: 0 },
        MockObj { id: 2, z: 2.0, tex: 0 },
        MockObj { id: 3, z: 10.0, tex: 1 },
        MockObj { id: 4, z: 11.0, tex: 1 },
    ];

    objects.sort_by(|a, b| a.z.total_cmp(&b.z));

    let mut draw_calls = Vec::new();
    if !objects.is_empty() {
        let mut current_tex = objects[0].tex;
        let mut count = 0;
        for  obj in &objects {
            if obj.tex != current_tex {
                draw_calls.push(DrawCall { texture_id: current_tex, count });
                current_tex = obj.tex;
                count = 0;
            }
            
            count += 1;
        }
        
        draw_calls.push(DrawCall { texture_id: current_tex, count });
    }

    assert_eq!(draw_calls.len(), 2);
    assert_eq!(draw_calls[0], DrawCall { texture_id: 0, count: 2 });
    assert_eq!(draw_calls[1], DrawCall { texture_id: 1, count: 2 });
}