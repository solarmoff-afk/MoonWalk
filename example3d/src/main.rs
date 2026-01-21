use moonwalk::{MoonWalk, ObjectId};
use moonwalk::scene::{
    Scene3D, LightId, InstanceId, LightingModel, ShadowQuality
};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec3, Vec4};

const CUBE_OBJ: &str = r#"
v -0.5 -0.5  0.5
v  0.5 -0.5  0.5
v  0.5  0.5  0.5
v -0.5  0.5  0.5
v -0.5 -0.5 -0.5
v  0.5 -0.5 -0.5
v  0.5  0.5 -0.5
v -0.5  0.5 -0.5
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vn  0.0  0.0  1.0
vn  0.0  0.0 -1.0
vn -1.0  0.0  0.0
vn  1.0  0.0  0.0
vn  0.0  1.0  0.0
vn  0.0 -1.0  0.0
f 1/1/1 2/2/1 3/3/1
f 3/3/1 4/4/1 1/1/1
f 6/1/2 5/2/2 8/3/2
f 8/3/2 7/4/2 6/1/2
f 5/1/3 1/2/3 4/3/3
f 4/3/3 8/4/3 5/1/3
f 2/1/4 6/2/4 7/3/4
f 7/3/4 3/4/4 2/1/4
f 4/1/5 3/2/5 7/3/5
f 7/3/5 8/4/5 4/1/5
f 5/1/6 6/2/6 2/3/6
f 2/3/6 1/4/6 5/1/6
"#;

const CUBE_COUNT: usize = 150;

struct App3D {
    scene: Option<Scene3D>,
    display_id: Option<ObjectId>,
    cubes: Vec<InstanceId>,
    light_id: Option<LightId>,
    time: f32,
    frame_timer: f32,
    frames: u32,
}

impl App3D {
    fn new() -> Self {
        Self {
            scene: None,
            display_id: None,
            cubes: Vec::with_capacity(CUBE_COUNT),
            light_id: None,
            time: 0.0,
            frame_timer: 0.0,
            frames: 0,
        }
    }
}

impl Application for App3D {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        mw.set_vsync(false);

        let scale = mw.get_scale_factor();
        let width = (viewport.x * scale) as u32;
        let height = (viewport.y * scale) as u32;
        
        let mut scene = Scene3D::new(mw, width, height);
        
        scene.set_lighting_model(LightingModel::Phong); // Phong быстрее PBК
        scene.set_shadow_quality(mw, ShadowQuality::Off); // Тени выключены
        
        let mesh_ids = scene.load_obj(mw, CUBE_OBJ.as_bytes());
        let cube_mesh = mesh_ids[0];

        let cols = 15;
        let rows = 10;
        
        for i in 0..CUBE_COUNT {
            let instance = scene.new_instance(cube_mesh);
            
            let x = (i % cols) as f32 * 1.5 - (cols as f32 * 1.5 / 2.0);
            let z = (i / cols) as f32 * 1.5 - (rows as f32 * 1.5 / 2.0);
            
            scene.set_position(instance, Vec3::new(x, 0.0, z));
            
            let r = (i as f32 * 0.1).sin().abs();
            let g = (i as f32 * 0.2).cos().abs();
            let b = (i as f32 * 0.3).sin().abs();
            scene.set_color(instance, Vec4::new(r, g, b, 1.0));
            
            scene.set_roughness(instance, 0.1); 

            self.cubes.push(instance);
        }

        let sun = scene.new_light();
        scene.set_light_position(sun, Vec3::new(10.0, 20.0, 10.0));
        scene.set_light_color(sun, Vec3::new(1.0, 1.0, 1.0));
        scene.set_light_intensity(sun, 100.0);
        self.light_id = Some(sun);

        scene.camera_pos = Vec3::new(0.0, 10.0, 15.0);
        scene.camera_target = Vec3::new(0.0, 0.0, 0.0);

        self.scene = Some(scene);

        let id = mw.new_rect();
        mw.set_size(id, viewport);
        mw.set_position(id, Vec2::ZERO);
        mw.set_color(id, Vec4::ONE);
        
        self.display_id = Some(id);
    }

    fn on_update(&mut self, dt: f32) {
        self.time += dt;
        
        self.frame_timer += dt;
        self.frames += 1;
        if self.frame_timer >= 1.0 {
            println!("FPS: {}", self.frames);
            self.frames = 0;
            self.frame_timer = 0.0;
        }

        if let Some(scene) = &mut self.scene {
            for (i, id) in self.cubes.iter().enumerate() {
                let speed = 1.0 + (i as f32 * 0.01);
                let rot = Vec3::new(
                    self.time * speed,
                    self.time * speed * 0.5,
                    0.0
                );
                scene.set_rotation(*id, rot);
                
                let mut pos = scene.get_position(*id);
                pos.y = (self.time * 2.0 + (i as f32 * 0.1)).sin() * 0.5;
                scene.set_position(*id, pos);
            }
        }
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        if let Some(scene) = &mut self.scene {
            let tex_id = scene.render(mw);
            
            if let Some(id) = self.display_id {
                mw.set_texture(id, tex_id);
            }
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        if let Some(id) = self.display_id {
            mw.set_size(id, viewport);
        }
    }
}

fn main() {
    let app = App3D::new();
    let settings = WindowSettings::new("MoonWalk 3D", 1280.0, 720.0);
    Runner::run(app, settings).unwrap();
}