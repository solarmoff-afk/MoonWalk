// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use moonwalk::{MoonWalk, ObjectId};
use lunar3d::{LunarFactory, LunarScene, LightId, ShadowQuality, ObjectId as LunarObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec3, Vec4};

const CUBE_OBJ: &str = r#"
v -0.5 -0.5  0.5
v  0.5 -0.5  0.5
v -0.5  0.5  0.5
v  0.5  0.5  0.5
v -0.5  0.5 -0.5
v  0.5  0.5 -0.5
v -0.5 -0.5 -0.5
v  0.5 -0.5 -0.5

vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0

vn  0.0  0.0  1.0
vn  0.0  1.0  0.0
vn  0.0  0.0 -1.0
vn  0.0 -1.0  0.0
vn  1.0  0.0  0.0
vn -1.0  0.0  0.0

f 1/1/1 2/2/1 4/4/1
f 4/4/1 3/3/1 1/1/1
f 3/1/2 4/2/2 6/4/2
f 6/4/2 5/3/2 3/1/2
f 5/1/3 6/2/3 8/4/3
f 8/4/3 7/3/3 5/1/3
f 7/1/4 8/2/4 2/4/4
f 2/4/4 1/3/4 7/1/4
f 2/1/5 8/2/5 6/4/5
f 6/4/5 4/3/5 2/1/5
f 7/1/6 1/2/6 3/4/6
f 3/4/6 5/3/6 7/1/6
"#;

struct PbrDebug {
    factory: Option<LunarFactory>,
    scene: Option<LunarScene>,
    display_id: Option<ObjectId>,
    time: f32,
    object_ids: Vec<LunarObjectId>,
    light_ids: Vec<LightId>,
    test_phase: u32,
    phase_timer: f32,
    initialized: bool,
}

impl PbrDebug {
    fn new() -> Self {
        Self {
            factory: None,
            scene: None,
            display_id: None,
            time: 0.0,
            object_ids: Vec::new(),
            light_ids: Vec::new(),
            test_phase: 0,
            phase_timer: 0.0,
            initialized: false,
        }
    }
}

impl Application for PbrDebug {
    fn on_start(&mut self, mw: &mut MoonWalk, _viewport: Vec2) { 
        let mut factory = LunarFactory::new(mw);
        let cube_meshes = factory.load_obj(mw, CUBE_OBJ.as_bytes());
        
        if cube_meshes.is_empty() {
            println!("Failed to load cube");
            return;
        }
        
        let cube_mesh_id = cube_meshes[0];
        let mut scene = factory.new_scene(mw, 1280, 720);

        scene.set_shadow_quality(mw, ShadowQuality::Off);
        
        scene.ambient_color = Vec3::new(0.3, 0.3, 0.4);
        
        let floor = scene.new_object(cube_mesh_id);
        scene.set_position(floor, Vec3::new(0.0, -1.5, 0.0));
        scene.set_scale(floor, Vec3::new(10.0, 0.1, 10.0));
        scene.set_color(floor, Vec4::new(0.7, 0.7, 0.7, 1.0));
        scene.set_metallic(floor, 0.0);
        scene.set_roughness(floor, 0.5);
        self.object_ids.push(floor);
        
        let test_cube = scene.new_object(cube_mesh_id);
        scene.set_position(test_cube, Vec3::new(0.0, 0.0, 0.0));
        scene.set_scale(test_cube, Vec3::splat(1.5));
        scene.set_color(test_cube, Vec4::new(0.8, 0.2, 0.2, 1.0));
        scene.set_metallic(test_cube, 0.0);
        scene.set_roughness(test_cube, 0.5);
        self.object_ids.push(test_cube);
        
        let metal_cube = scene.new_object(cube_mesh_id);
        scene.set_position(metal_cube, Vec3::new(3.0, 0.0, 0.0));
        scene.set_scale(metal_cube, Vec3::splat(1.5));
        scene.set_color(metal_cube, Vec4::new(0.9, 0.9, 0.9, 1.0));
        scene.set_metallic(metal_cube, 1.0);
        scene.set_roughness(metal_cube, 0.3);
        self.object_ids.push(metal_cube);
        
        self.factory = Some(factory);
        self.scene = Some(scene);
        
        let display_id = mw.new_rect();
        let win_size = mw.get_window_size();
        mw.set_size(display_id, win_size);
        self.display_id = Some(display_id);
        self.initialized = true; 
    }
    
    fn on_update(&mut self, dt: f32) {
        if !self.initialized {
            return;
        }
        
        self.time += dt;
        self.phase_timer += dt;
        
        if self.phase_timer > 5.0 {
            self.phase_timer = 0.0;
            self.test_phase = (self.test_phase + 1) % 4;
            
            if let Some(scene) = &mut self.scene {
                match self.test_phase {
                    0 => {
                        while !self.light_ids.is_empty() {
                            if let Some(id) = self.light_ids.pop() {
                                scene.remove_light(id);
                            }
                        }

                        scene.ambient_color = Vec3::new(0.3, 0.3, 0.4);
                    }
                    1 => {
                        let sun = scene.new_light();
                        
                        scene.set_light_position(sun, Vec3::new(10.0, 20.0, 10.0));
                        scene.set_light_color(sun, Vec3::new(1.0, 0.95, 0.9));
                        scene.set_light_intensity(sun, 2.0);
                        self.light_ids.push(sun);
                        scene.ambient_color = Vec3::new(0.1, 0.1, 0.15);
                    }
                    2 => {
                        let fill = scene.new_light();
                        
                        scene.set_light_position(fill, Vec3::new(-5.0, 10.0, -5.0));
                        scene.set_light_color(fill, Vec3::new(0.8, 0.85, 1.0));
                        scene.set_light_intensity(fill, 1.0);
                        self.light_ids.push(fill);
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn on_draw(&mut self, mw: &mut MoonWalk) {
        if !self.initialized {
            return;
        }
        
        let scene = match &mut self.scene {
            Some(s) => s,
            None => return,
        };
        
        let factory = match &self.factory {
            Some(f) => f,
            None => return,
        };
        
        scene.camera_pos = Vec3::new(0.0, 3.0, 8.0);
        scene.camera_target = Vec3::new(0.0, 0.0, 0.0);
        
        if self.object_ids.len() >= 2 {
            scene.set_rotation(self.object_ids[1], Vec3::new(0.0, self.time * 0.3, 0.0));
        }

        if self.object_ids.len() >= 3 {
            scene.set_rotation(self.object_ids[2], Vec3::new(0.0, -self.time * 0.2, 0.0));
        }
        
        if self.test_phase >= 1 && !self.light_ids.is_empty() {
            let sun_x = 10.0 + (self.time * 0.2).sin() * 5.0;
            let sun_y = 20.0;
            let sun_z = 10.0 + (self.time * 0.2).cos() * 5.0;

            scene.set_light_position(self.light_ids[0], Vec3::new(sun_x, sun_y, sun_z));
        }
        
        if self.test_phase == 3 {
            scene.set_shadow_quality(mw, ShadowQuality::High);
        } else {
            scene.set_shadow_quality(mw, ShadowQuality::Off);
        }
        
        let tex_id = scene.render(mw, factory);
        
        if let Some(display_id) = self.display_id {
            mw.set_texture(display_id, tex_id);
        } 
    }
    
    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        if let Some(display_id) = self.display_id {
            mw.set_size(display_id, viewport);
        }

        if let Some(scene) = &mut self.scene {
            if viewport.x > 0.0 && viewport.y > 0.0 {
                scene.width = viewport.x as u32;
                scene.height = viewport.y as u32;
            }
        }
    }
}

fn main() {
    let app = PbrDebug::new();
    let settings = WindowSettings::new("Lunar3D test", 1280.0, 720.0);
    Runner::run(app, settings).unwrap();
}

