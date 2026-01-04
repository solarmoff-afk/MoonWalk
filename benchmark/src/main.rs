// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2025 MoonWalk

use moonwalk::{MoonWalk, FontAsset, ObjectId, TextAlign};
use moonwalk::rendering::container::RenderContainer; 
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

const WARMUP_FRAMES: usize = 30;   
const MEASURE_FRAMES: usize = 90;
const BENCHMARK_FILE: &str = "BENCHMARK.md";

#[derive(Debug, Clone, Copy)]
enum RectStyle {
    Solid,
    Textured,
    Rounded,
    Border,
    GradientLinear,
    GradientRadial,
}

#[derive(Debug, Clone, Copy)]
enum TextMode {
    Short,
    Paragraph,
    Justified,
}

#[derive(Debug, Clone)]
enum Scenario {
    Batching { count: usize, size: f32, style: RectStyle },
    FillRate { layers: usize }, 
    Text { count: usize, mode: TextMode },
    Animation { count: usize, is_text: bool },
    Effects { blur_radius: f32, color_matrix: bool },
    VectorStatic { count: usize },
    VectorDynamic { points: usize },
    RealScene,
}

struct BenchResult {
    category: String,
    name: String,
    avg_fps: f64,
    min_1_fps: f64,
    avg_frame_time_ms: f64,
    ram_usage_mb: u64,
}

struct BenchmarkApp {
    scenarios: VecDeque<Scenario>,
    current_scenario: Option<Scenario>,
    
    frame_count: usize,
    frame_times: Vec<Duration>,
    last_frame: Instant,
    
    font_id: Option<FontAsset>,
    working_texture_id: u32,
    loaded_texture_id: u32,
    
    display_rect_id: Option<ObjectId>,
    active_ids: Vec<ObjectId>,
    results: Vec<BenchResult>,
    sys: System,
    container: Option<RenderContainer>,
    
    screen_size: Vec2,
}

impl BenchmarkApp {
    fn new() -> Self {
        let mut scenarios = VecDeque::new();
        
        scenarios.push_back(Scenario::Batching { count: 10_000, size: 2.0, style: RectStyle::Solid });
        scenarios.push_back(Scenario::Batching { count: 50_000, size: 2.0, style: RectStyle::Solid });
        scenarios.push_back(Scenario::Batching { count: 100_000, size: 2.0, style: RectStyle::Solid });
        scenarios.push_back(Scenario::Batching { count: 200_000, size: 2.0, style: RectStyle::Solid });

        scenarios.push_back(Scenario::Batching { count: 5_000, size: 50.0, style: RectStyle::Solid });
        scenarios.push_back(Scenario::Batching { count: 5_000, size: 50.0, style: RectStyle::Rounded });
        scenarios.push_back(Scenario::Batching { count: 5_000, size: 50.0, style: RectStyle::Border });
        scenarios.push_back(Scenario::Batching { count: 5_000, size: 50.0, style: RectStyle::Textured });

        scenarios.push_back(Scenario::Batching { count: 5_000, size: 100.0, style: RectStyle::GradientLinear });
        scenarios.push_back(Scenario::Batching { count: 5_000, size: 100.0, style: RectStyle::GradientRadial });

        scenarios.push_back(Scenario::FillRate { layers: 10 });
        scenarios.push_back(Scenario::FillRate { layers: 50 });
        scenarios.push_back(Scenario::FillRate { layers: 100 });

        scenarios.push_back(Scenario::Text { count: 1_000, mode: TextMode::Short });
        scenarios.push_back(Scenario::Text { count: 10_000, mode: TextMode::Short });
        scenarios.push_back(Scenario::Text { count: 500, mode: TextMode::Paragraph });
        scenarios.push_back(Scenario::Text { count: 500, mode: TextMode::Justified });

        scenarios.push_back(Scenario::Animation { count: 10_000, is_text: false });
        scenarios.push_back(Scenario::Animation { count: 50_000, is_text: false });
        scenarios.push_back(Scenario::Animation { count: 2_000, is_text: true });

        scenarios.push_back(Scenario::VectorStatic { count: 5_000 });
        scenarios.push_back(Scenario::VectorDynamic { points: 10 });
        scenarios.push_back(Scenario::VectorDynamic { points: 100 });
        scenarios.push_back(Scenario::VectorDynamic { points: 300 });

        scenarios.push_back(Scenario::Effects { blur_radius: 0.0, color_matrix: true });
        scenarios.push_back(Scenario::Effects { blur_radius: 5.0, color_matrix: false });
        scenarios.push_back(Scenario::Effects { blur_radius: 15.0, color_matrix: false });
        scenarios.push_back(Scenario::Effects { blur_radius: 30.0, color_matrix: false });

        scenarios.push_back(Scenario::RealScene);

        let r = RefreshKind::new()
            .with_cpu(CpuRefreshKind::new())
            .with_memory(MemoryRefreshKind::new());

        Self {
            scenarios,
            current_scenario: None,
            frame_count: 0,
            frame_times: Vec::with_capacity(MEASURE_FRAMES),
            last_frame: Instant::now(),
            font_id: None,
            working_texture_id: 0,
            loaded_texture_id: 0,
            display_rect_id: None,
            active_ids: Vec::new(),
            results: Vec::new(),
            container: None,
            sys: System::new_with_specifics(r),
            screen_size: Vec2::new(1280.0, 720.0),
        }
    }

    fn start_next_scenario(&mut self, mw: &mut MoonWalk) {
        mw.remove_all();
        self.container = None; 
        self.display_rect_id = None;
        self.active_ids.clear();
        self.frame_count = 0;
        self.frame_times.clear();

        if let Some(scen) = self.scenarios.pop_front() {
            println!("Running test: {:?}", scen);
            self.setup_scene(mw, &scen);
            self.current_scenario = Some(scen);
        } else {
            self.generate_report(mw);
            println!("Audit finished. Results PREPENDED to {}", BENCHMARK_FILE);
            std::process::exit(0);
        }
    }

    fn setup_scene(&mut self, mw: &mut MoonWalk, scenario: &Scenario) {
        match scenario {
            Scenario::Batching { count, size, style } => {
                self.spawn_rects(mw, *count, *size, *style, false);
            },
            
            Scenario::FillRate { layers } => {
                for _ in 0..*layers {
                    let id = mw.new_rect();
                    mw.set_position(id, Vec2::ZERO);
                    mw.set_size(id, self.screen_size); 
                    mw.set_color(id, Vec4::new(1.0, 0.0, 0.0, 0.01)); 
                }
            },
            
            Scenario::Text { count, mode } => {
                self.spawn_text(mw, *count, *mode, false);
            },

            Scenario::Animation { count, is_text } => {
                if *is_text {
                    self.spawn_text(mw, *count, TextMode::Short, true);
                } else {
                    self.spawn_rects(mw, *count, 10.0, RectStyle::Solid, true);
                }
            },

            Scenario::RealScene => {
                let bg = mw.new_rect();
                mw.set_position(bg, Vec2::ZERO);
                mw.set_size(bg, self.screen_size);
                mw.linear_gradient(bg, Vec2::new(0.0, 1.0));
                mw.set_color(bg, Vec4::new(0.1, 0.1, 0.2, 1.0));
                mw.set_color2(bg, Vec4::new(0.0, 0.0, 0.0, 1.0));

                self.spawn_rects(mw, 200, 100.0, RectStyle::Rounded, true);
                self.spawn_text(mw, 50, TextMode::Short, true);
            },
            
            Scenario::VectorStatic { count } => {
                let mut pb = mw.new_path_builder();
                pb.set_color(Vec4::new(0.0, 1.0, 0.0, 1.0));
                mw.parse_svg_path(&mut pb, "M 50 0 L 100 100 L 0 100 Z M 50 20 L 20 80 L 80 80 Z").unwrap();
                let tex_id = pb.tessellate(mw, 100, 100);
                
                let cols = (*count as f32).sqrt() as u32;
                for i in 0..*count {
                    let id = mw.new_rect();
                    let x = (i as u32 % cols) as f32 * 20.0;
                    let y = (i as u32 / cols) as f32 * 20.0;
                    mw.set_position(id, Vec2::new(x, y));
                    mw.set_size(id, Vec2::new(16.0, 16.0));
                    mw.set_texture(id, tex_id);
                }
            },

            Scenario::Effects { blur_radius: _, color_matrix: _ } => {
                self.setup_effect_scene(mw, 1280, 720);
            },
            
            Scenario::VectorDynamic { .. } => {
                let mut container = mw.new_render_container(500, 500);
                container.draw(mw, Some(Vec4::ZERO));
                self.working_texture_id = container.snapshot(mw, 0, 0, 500, 500);
                
                let id = mw.new_rect();
                mw.set_position(id, Vec2::new(100.0, 100.0));
                mw.set_size(id, Vec2::new(500.0, 500.0));
                mw.set_texture(id, self.working_texture_id);
                self.container = Some(container);
            }
        }
    }

    fn setup_effect_scene(&mut self, mw: &mut MoonWalk, w: u32, h: u32) {
        let mut container = mw.new_render_container(w, h);
        
        let bg = container.new_rect();
        container.config_size(bg, Vec2::new(w as f32, h as f32));
        container.config_gradient_data(bg, [1.0, 1.0, 0.0, 0.0]);
        container.config_color(bg, Vec4::new(0.8, 0.2, 0.2, 1.0));
        container.config_color2(bg, Vec4::new(0.2, 0.2, 0.8, 1.0));
        
        for i in 0..100 {
            let id = container.new_rect();
            let x = (i % 20) as f32 * 60.0;
            let y = (i / 20) as f32 * 60.0;
            container.config_position(id, Vec2::new(x, y));
            container.config_size(id, Vec2::new(40.0, 40.0));
            container.config_color(id, Vec4::new(1.0, 1.0, 1.0, 1.0));
            container.set_rounded(id, Vec4::splat(10.0));
        }
        
        container.draw(mw, Some(Vec4::ZERO));
        self.working_texture_id = container.snapshot(mw, 0, 0, w, h);
        
        let id = mw.new_rect();
        mw.set_position(id, Vec2::ZERO);
        mw.set_size(id, Vec2::new(w as f32, h as f32));
        mw.set_texture(id, self.working_texture_id);
        
        self.display_rect_id = Some(id);
        self.container = Some(container);
    }

    fn spawn_rects(&mut self, mw: &mut MoonWalk, count: usize, size: f32, style: RectStyle, keep_ids: bool) {
        let cols = (count as f32).sqrt() as u32;
        if cols == 0 { return; } 

        if keep_ids { self.active_ids.reserve(count); }

        let tex_id = if let RectStyle::Textured = style { self.loaded_texture_id } else { 0 };

        for i in 0..count {
            let id = mw.new_rect();
            let step = size * 1.2;
            let x = (i as u32 % cols) as f32 * step; 
            let y = (i as u32 / cols) as f32 * step;
            
            mw.set_position(id, Vec2::new(x % 3000.0, y % 3000.0)); 
            mw.set_size(id, Vec2::new(size, size));
            
            match style {
                RectStyle::Solid => {
                    mw.set_color(id, Vec4::new(0.0, 0.5, 1.0, 1.0));
                },
                RectStyle::Textured => {
                    if tex_id > 0 {
                        mw.set_texture(id, tex_id);
                        mw.set_color(id, Vec4::ONE);
                    } else {
                        mw.set_color(id, Vec4::new(1.0, 0.0, 1.0, 1.0));
                    }
                },
                RectStyle::Rounded => {
                    mw.set_color(id, Vec4::new(0.0, 0.8, 0.5, 1.0));
                    mw.set_rounded(id, Vec4::splat(size / 3.0));
                },
                RectStyle::Border => {
                    mw.set_color(id, Vec4::new(0.0, 0.0, 0.0, 0.0));
                    mw.set_color2(id, Vec4::new(1.0, 1.0, 0.0, 1.0));
                    mw.set_effect(id, size / 10.0, 0.0);
                    mw.set_rounded(id, Vec4::splat(size / 4.0));
                },
                RectStyle::GradientLinear => {
                    mw.set_color(id, Vec4::new(1.0, 0.0, 0.0, 1.0));
                    mw.set_color2(id, Vec4::new(0.0, 0.0, 1.0, 1.0));
                    mw.linear_gradient(id, Vec2::new(1.0, 1.0));
                },
                RectStyle::GradientRadial => {
                    mw.set_color(id, Vec4::new(1.0, 1.0, 1.0, 1.0));
                    mw.set_color2(id, Vec4::new(0.0, 0.0, 0.0, 1.0));
                    mw.radial_gradient(id, Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0));
                }
            }

            if keep_ids { self.active_ids.push(id); }
        }
    }

    fn spawn_text(&mut self, mw: &mut MoonWalk, count: usize, mode: TextMode, keep_ids: bool) {
        if let Some(font) = self.font_id {
            if keep_ids { self.active_ids.reserve(count); }

            let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor.";

            for i in 0..count {
                let (content, size, width) = match mode {
                    TextMode::Short => ("MoonWalk UI", 16.0, 200.0),
                    TextMode::Paragraph | TextMode::Justified => (lorem, 14.0, 300.0),
                };

                let id = mw.new_text(content, font, size);
                
                if let TextMode::Justified = mode {
                    mw.set_text_size(id, width, 500.0);
                    mw.set_text_align(id, TextAlign::Justified);
                } else if let TextMode::Paragraph = mode {
                    mw.set_text_size(id, width, 500.0);
                }

                let x = (i as f32 * 10.0) % 1200.0;
                let y = (i as f32 * 20.0) % 3000.0;
                
                mw.set_position(id, Vec2::new(x, y));
                mw.set_color(id, Vec4::ONE);

                if keep_ids { self.active_ids.push(id); }
            }
        }
    }
    
    fn record_current_result(&mut self) {
        if let Some(scen) = &self.current_scenario {
            let total_time: Duration = self.frame_times.iter().sum();
            let count = self.frame_times.len();
            let avg_time_sec = total_time.as_secs_f64() / count as f64;
            let avg_fps = 1.0 / avg_time_sec;

            let mut sorted = self.frame_times.clone();
            sorted.sort();
            let p99_idx = (count as f64 * 0.99) as usize;
            let p99_time = sorted.get(p99_idx).unwrap_or(&sorted[count-1]);
            let min_1_fps = 1.0 / p99_time.as_secs_f64();

            let pid = sysinfo::Pid::from(std::process::id() as usize);
            self.sys.refresh_process(pid);
            let ram_mb = self.sys.process(pid).map(|p| p.memory() / 1024 / 1024).unwrap_or(0);

            let (category, name) = match scen {
                Scenario::Batching { count, style, .. } => (
                    "Geometry".to_string(),
                    format!("Rects {:?} x{}", style, count)
                ),
                Scenario::FillRate { layers } => (
                    "Fill Rate".to_string(),
                    format!("Fullscreen Layers x{}", layers)
                ),
                Scenario::Text { count, mode } => (
                    "Text".to_string(),
                    format!("Text {:?} x{}", mode, count)
                ),
                Scenario::Animation { count, is_text } => (
                    "Dynamics".to_string(),
                    format!("Moving {} x{}", if *is_text { "Text" } else { "Rects" }, count)
                ),
                Scenario::Effects { blur_radius, color_matrix } => (
                    "Effects".to_string(),
                    if *color_matrix { "Color Matrix".into() } else { format!("Blur (r={})", blur_radius) }
                ),
                Scenario::VectorStatic { count } => ("Vector".into(), format!("Static x{}", count)),
                Scenario::VectorDynamic { points } => ("Vector".into(), format!("Dynamic ({} pts)", points)),
                Scenario::RealScene => ("Simulation".into(), "Real Scene (200 Rects + 50 Text)".into()),
            };

            self.results.push(BenchResult {
                category,
                name,
                avg_fps,
                min_1_fps,
                avg_frame_time_ms: avg_time_sec * 1000.0,
                ram_usage_mb: ram_mb,
            });
        }
    }

    fn generate_report(&self, mw: &MoonWalk) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        
        let mut sys = System::new_all();
        sys.refresh_cpu();
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_cpu();

        let cpu = sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown CPU");
        let ram = sys.total_memory() / 1024 / 1024 / 1024;
        let os = System::name().unwrap_or("Unknown OS".to_string());
        let os_ver = System::os_version().unwrap_or("".to_string());
        
        let info = mw.get_graphics_info(); 
        
        let build_type = if cfg!(debug_assertions) { "DEBUG" } else { "RELEASE" };

        let mut report = String::new();
        report.push_str("--------------------------------------------------------------------------------\n");
        report.push_str(&format!("## MoonWalk Performance Audit [{}]\n\n", now));
        
        report.push_str("## System Configuration\n");
        report.push_str(&format!("* **Build:** {}\n", build_type));
        report.push_str(&format!("* **OS:** {} {}\n", os, os_ver));
        report.push_str(&format!("* **CPU:** {}\n", cpu));
        report.push_str(&format!("* **RAM:** {} GB\n", ram));
        report.push_str(&format!("* **GPU:** {} ({})\n", info.name, info.backend));
        report.push_str(&format!("* **Driver:** {}\n\n", info.driver));

        report.push_str("## Benchmark Results\n\n");
        report.push_str("| Category | Test Scenario | Avg FPS | 1% Low | Frame Time | RAM Usage |\n");
        report.push_str("| :--- | :--- | ---: | ---: | ---: | ---: |\n");

        for res in &self.results {
            report.push_str(&format!("| **{}** | {} | {:.1} | {:.1} | {:.2} ms | {} MB |\n", 
                res.category, res.name, res.avg_fps, res.min_1_fps, res.avg_frame_time_ms, res.ram_usage_mb));
        }
        report.push_str("\n\n");

        let path = Path::new(BENCHMARK_FILE);
        let old_content = if path.exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        };

        let final_content = format!("{}{}", report, old_content);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        
        file.write_all(final_content.as_bytes()).unwrap();
    }
}

impl Application for BenchmarkApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        mw.set_vsync(false);
        self.screen_size = viewport;

        if let Ok(font) = mw.load_font("assets/font.ttf", "BenchFont") {
            self.font_id = Some(font);
        } else {
             println!("Note: Text tests will be skipped (no assets/font.ttf)");
        }

        if let Ok(tex) = mw.load_texture("assets/test.jpg") {
            self.loaded_texture_id = tex;
        } else {
            println!("Note: Texture tests will use fallback (no assets/test.jpg)");
        }
        
        self.start_next_scenario(mw);
    }

    fn on_update(&mut self, _dt: f32) {}

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame);
        self.last_frame = now;

        self.frame_count += 1;

        if let Some(scen) = &self.current_scenario {
            match scen {
                Scenario::RealScene | Scenario::Animation { .. } => {
                    let time = self.frame_count as f32 * 0.05;
                    
                    for (i, id) in self.active_ids.iter().enumerate() { 
                        let offset_x = (time + (i as f32 * 0.1)).sin() * 5.0;
                        let offset_y = (time + (i as f32 * 0.1)).cos() * 5.0;
                        
                        let x = 100.0 + (i as f32 * 13.0) % 1200.0 + offset_x;
                        let y = 100.0 + (i as f32 * 7.0) % 600.0 + offset_y;
                        mw.set_position(*id, Vec2::new(x, y));
                    }
                },
                
                Scenario::VectorDynamic { points } => {
                    let mut pb = mw.new_path_builder();
                    pb.set_color(Vec4::new(1.0, 0.5, 0.0, 1.0));
                    
                    let offset = (self.frame_count as f32 * 0.1).sin() * 50.0;
                    let mut path = format!("M 10 10");
                    
                    for i in 0..*points {
                        let x = 100.0 + (i as f32 * 10.0) + offset;
                        let y = 200.0 + if i % 2 == 0 { 50.0 } else { -50.0 };
                        path.push_str(&format!(" L {} {}", x, y));
                    }
                    path.push_str(" Z");
                    
                    if mw.parse_svg_path(&mut pb, &path).is_ok() {
                        pb.tessellate_to(mw, self.working_texture_id, 500, 500);
                    }
                },

                Scenario::Effects { blur_radius, color_matrix } => {
                    if *color_matrix {
                        mw.hue_shift(self.working_texture_id, (self.frame_count % 360) as f32);
                    } else {
                        mw.blur_texture(self.working_texture_id, *blur_radius, true);
                        mw.blur_texture(self.working_texture_id, *blur_radius, false);
                    }
                },

                _ => {}
            }

            if self.frame_count > WARMUP_FRAMES {
                self.frame_times.push(dt);
            }

            if self.frame_count >= (WARMUP_FRAMES + MEASURE_FRAMES) {
                self.record_current_result();
                self.start_next_scenario(mw);
            }
        }
    }

    fn on_resize(&mut self, _mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    env_logger::init();
    let app = BenchmarkApp::new();
    let settings = WindowSettings::new("MoonWalk Performance Audit", 1280.0, 720.0);
    Runner::run(app, settings).unwrap();
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default());
    let app_logic = BenchmarkApp::new();
    let settings = WindowSettings::new("MoonWalk Audit", 0.0, 0.0);
    Runner::run(app_logic, settings, app).unwrap();
}
