// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use lyon::extra::parser::{PathParser, ParserOptions, Source};

pub fn parse_svg_path(inner_builder: &mut crate::path::PathBuilder, data: &str) -> Result<(), String> {
    let mut parser = PathParser::new();
    let options = ParserOptions::DEFAULT;
    let mut source = Source::new(data.chars());

    match parser.parse(&options, &mut source, &mut inner_builder.builder) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}", e)),
    }
}
