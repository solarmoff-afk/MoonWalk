use moonwalk::{MoonWalk, ObjectId, MoonVideo, VideoPreset, VideoFormat};
use moonwalk::rendering::container::RenderContainer;
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

const VIDEO_WIDTH: u32 = 1920;
const VIDEO_HEIGHT: u32 = 1080;
const FPS: usize = 24;
const DURATION_SEC: usize = 5;
const TOTAL_FRAMES: usize = FPS * DURATION_SEC;

struct SceneObjects {
    container: RenderContainer,
    snapshot_id: u32,
    bg_id: ObjectId,
    rect_id: ObjectId,
    text_id: Option<ObjectId>,
}

struct VideoApp {
    recorder: Option<MoonVideo>,
    frame_count: usize,
    scene: Option<SceneObjects>,
    display_id: Option<ObjectId>,
}

impl VideoApp {
    fn new() -> Self {
        Self {
            recorder: None,
            frame_count: 0,
            scene: None,
            display_id: None,
        }
    }
}

impl Application for VideoApp {
    fn on_start(&mut self, mw: &mut MoonWalk, _viewport: Vec2) { 
        match mw.new_video_recorder(
            VIDEO_WIDTH, VIDEO_HEIGHT, FPS, "example-video/output.mp4", VideoFormat::Mp4,
            VideoPreset::Balanced,
        ) {
            Ok(rec) => self.recorder = Some(rec),
            Err(e) => eprintln!("Failed to init video recorder: {}", e),
        }

        let mut container = mw.new_render_container(VIDEO_WIDTH, VIDEO_HEIGHT);
        
        let bg_id = container.new_rect();
        container.config_size(bg_id, Vec2::new(VIDEO_WIDTH as f32, VIDEO_HEIGHT as f32));
        container.config_position(bg_id, Vec2::ZERO);
        
        let rect_id = container.new_rect();
        container.config_size(rect_id, Vec2::splat(200.0));
        container.config_color(rect_id, Vec4::new(1.0, 0.5, 0.0, 1.0));
        container.set_rounded(rect_id, Vec4::splat(20.0));

        let text_id = if let Ok(font) = mw.load_font("assets/Hundo.ttf", "Hundo") {
            let tid = container.new_text("MoonWalk Video", font, 100.0);
            container.config_color(tid, Vec4::ONE);
            Some(tid)
        } else {
            None
        };

        container.draw(mw, Some(Vec4::new(0.0, 0.0, 0.0, 1.0)));
        let snapshot_id = container.snapshot(mw, 0, 0, VIDEO_WIDTH, VIDEO_HEIGHT);

        self.scene = Some(SceneObjects {
            container,
            snapshot_id,
            bg_id,
            rect_id,
            text_id,
        });

        let display = mw.new_rect();
        let win_size = mw.get_window_size();
        mw.set_size(display, win_size);
        mw.set_position(display, Vec2::ZERO);
        mw.set_texture(display, snapshot_id);
        
        self.display_id = Some(display);
    }

    fn on_update(&mut self, _dt: f32) {

    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        let scene = match &mut self.scene {
            Some(s) => s,
            None => return,
        };

        let container = &mut scene.container;
        let t = self.frame_count as f32 * (1.0 / FPS as f32);

        let bg_r = 0.1 + (t * 0.5).sin().abs() * 0.1;
        container.config_color(scene.bg_id, Vec4::new(bg_r, 0.1, 0.2, 1.0));

        let rect_pos = Vec2::new(
            VIDEO_WIDTH as f32 / 2.0 + (t * 2.0).cos() * 300.0 - 100.0,
            VIDEO_HEIGHT as f32 / 2.0 + (t * 2.0).sin() * 300.0 - 100.0
        );
        container.config_position(scene.rect_id, rect_pos);
        container.config_rotation(scene.rect_id, t * 2.0);

        if let Some(tid) = scene.text_id {
            let scale_val = 1.0 + (t * 5.0).sin() * 0.2;
            container.set_font_size(tid, 100.0 * scale_val);
            container.config_position(tid, Vec2::new(100.0, 100.0)); 
        }

        container.draw(mw, None);
        container.update_snapshot(mw, 0, 0, VIDEO_WIDTH, VIDEO_HEIGHT, scene.snapshot_id);

        if let Some(recorder) = &mut self.recorder {
            if self.frame_count < TOTAL_FRAMES {
                if self.frame_count % 60 == 0 {
                    println!("Recording: {:.1}%", (self.frame_count as f32 / TOTAL_FRAMES as f32) * 100.0);
                }
                
                if let Err(e) = recorder.add_frame(mw, scene.snapshot_id) {
                    eprintln!("Error encoding frame: {}", e);
                }
            } else {
                if let Some(final_rec) = self.recorder.take() {
                    match final_rec.finish() {
                        Ok(_) => println!("Video saved to example-video/output.mp4"),
                        Err(e) => eprintln!("Failed to save video: {}", e),
                    }

                    std::process::exit(0);
                }
            }
        }

        self.frame_count += 1;
    }

    fn on_resize(&mut self, _mw: &mut MoonWalk, _viewport: Vec2) {

    }

    fn on_touch(&mut self, _mw: &mut MoonWalk, _phase: moonwalk_bootstrap::TouchPhase, _pos: Vec2) {

    }
}

fn main() {
    let app = VideoApp::new();
    let settings = WindowSettings::new("MoonWalk video", 960.0, 540.0);
    Runner::run(app, settings).unwrap();
}
