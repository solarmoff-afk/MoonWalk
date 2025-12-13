use moonwalk::{MoonWalk, ObjectId};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use glam::{Vec2, Vec4};

struct TextureApp {
    sprite_id: Option<ObjectId>,
    texture_id: u32,
    screen_size: Vec2,
    angle: f32,
}

impl TextureApp {
    fn new() -> Self {
        Self {
            sprite_id: None,
            texture_id: 0,
            screen_size: Vec2::new(800.0, 600.0),
            angle: 0.0,
        }
    }
}

impl Application for TextureApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        println!("🚀 Loading Texture App...");
        self.screen_size = viewport;

        // 1. Загружаем текстуру
        // ВАЖНО: Убедись, что файл test.png существует там, откуда запускаешь (в корне проекта)
        match mw.load_texture("test.png") {
            Ok(id) => {
                println!("✅ Texture loaded with ID: {}", id);
                self.texture_id = id;
            },
            Err(e) => {
                eprintln!("❌ Failed to load texture: {}", e);
                // Продолжим с белой текстурой (ID=0), но в консоли будет ошибка
            }
        }

        // 2. Создаем фон (для контраста)
        let bg = mw.new_rect();
        mw.set_position(bg, Vec2::ZERO);
        mw.set_size(bg, viewport * 2.0); // С запасом
        mw.set_color(bg, Vec4::new(0.1, 0.1, 0.1, 1.0)); // Темно-серый
        mw.set_z_index(bg, 0.0);

        // 3. Создаем Спрайт (Прямоугольник с текстурой)
        let sprite = mw.new_rect(); // Или new_sprite(tex_id), если ты его добавил
        self.sprite_id = Some(sprite);

        // Центрируем
        let size = 300.0;
        let pos = (viewport - size) * 0.5;

        mw.set_position(sprite, pos);
        mw.set_size(sprite, Vec2::splat(size));
        mw.set_color(sprite, Vec4::ONE); // Белый цвет, чтобы текстура была оригинальной
        mw.set_z_index(sprite, 10.0);
        
        // 4. Применяем текстуру
        if self.texture_id > 0 {
            mw.set_texture(sprite, self.texture_id);
        }

        // 5. Включаем скругление! (Проверка UberShader)
        // 50 пикселей радиус на всех углах
        mw.set_rounded(sprite, Vec4::splat(50.0)); 
    }

    fn on_update(&mut self, dt: f32) {
        // Просто крутим спрайт, чтобы было весело
        self.angle += dt * 1.0;
        println!("Update dt: {}", dt);
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        if let Some(id) = self.sprite_id {
            println!("Rotation: {}", self.angle);
            // mw.set_rotation(id, self.angle);
            
            // Можно еще пульсировать размер или цвет для теста
            // let scale = 1.0 + self.angle.sin() * 0.2;
            // mw.set_size(id, Vec2::splat(300.0 * scale));
        }
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        self.screen_size = viewport;
        // Пересчитываем центр при ресайзе
        if let Some(id) = self.sprite_id {
             let size = 300.0;
             let pos = (viewport - size) * 0.5;
             mw.set_position(id, pos);
        }
    }
}

// ... Boilerplate запуска (как у тебя был) ...
#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = TextureApp::new();
    let settings = WindowSettings::new("MoonWalk Texture Test", 800.0, 600.0).resizable(true);
    Runner::run(app, settings)
}
// ... Android main ...
