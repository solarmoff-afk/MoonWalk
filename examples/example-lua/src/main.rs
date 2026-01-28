use moonwalk_bootstrap::{Runner, WindowSettings};
use moonwalk_lua::MoonLua;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = MoonLua::new()?;
    app.init()?;

    let script = std::fs::read_to_string("examples/example-lua/test.lua")
        .expect("Failed to load test.lua");
    app.execute(&script)?;

    let settings = WindowSettings::new("MoonWalk lua", 800.0, 600.0)
        .with_min_size(400.0, 300.0)
        .resizable(true);

    Runner::run(app, settings)?;
    Ok(())
}