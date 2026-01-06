// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use moonwalk::path::PathBuilder;
use lyon::path::Event;

#[test]
fn test_path_builder_svg_parsing() {
    let mut pb = PathBuilder::new();
    
    // Треугольник 
    // 1. MoveTo (10, 10)
    // 2. LineTo (50, 10)
    // 3. LineTo (30, 50)
    // 4. Close
    let svg = "M 10 10 L 50 10 L 30 50 Z";
    let res = moonwalk::path::svg::parse_svg_path(&mut pb, svg);
    
    assert!(res.is_ok(), "Failed to parse valid SVG path");
    
    let builder = pb.get_internal_builder();
    let path = builder.clone().build();
    
    let events: Vec<_> = path.iter().collect();
    
    assert_eq!(events.len(), 4, "Path should contain 4 events");

    match events[0] {
        Event::Begin { at } => {
            assert_eq!(at.x, 10.0);
            assert_eq!(at.y, 10.0);
        },
        _ => panic!("Expected Begin event at index 0"),
    }

    match events[1] {
        Event::Line { to, .. } => {
            assert_eq!(to.x, 50.0);
            assert_eq!(to.y, 10.0);
        },
        _ => panic!("Expected Line event at index 1"),
    }

    match events[2] {
        Event::Line { to, .. } => {
            assert_eq!(to.x, 30.0);
            assert_eq!(to.y, 50.0);
        },
        _ => panic!("Expected Line event at index 2"),
    }

    match events[3] {
        Event::End { close, .. } => {
            assert!(close, "Path should be closed (Z command)");
        },
        _ => panic!("Expected End event at index 3"),
    }
}

#[test]
fn test_path_builder_invalid_svg() {
    let mut pb = PathBuilder::new();
    let svg = "INVALID SYNTAX 123";
    
    let res = moonwalk::path::svg::parse_svg_path(&mut pb, svg);
    assert!(res.is_err());
}

#[test]
fn test_path_builder_mixed_commands() {
    let mut pb = PathBuilder::new();
    
    let svg = "M 0 0 C 10 10 20 20 30 30";
    
    let res = moonwalk::path::svg::parse_svg_path(&mut pb, svg);
    assert!(res.is_ok());

    let builder = pb.get_internal_builder();
    let path = builder.clone().build();
    let events: Vec<_> = path.iter().collect();

    assert_eq!(events.len(), 3);

    match events[1] {
        Event::Cubic { to, ctrl1, ctrl2, .. } => {
            assert_eq!(ctrl1.x, 10.0);
            assert_eq!(ctrl2.x, 20.0);
            assert_eq!(to.x, 30.0);
        },
        _ => panic!("Expected Cubic event"),
    }
}
