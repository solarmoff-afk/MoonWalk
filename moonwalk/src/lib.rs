// MoonWalk это высокопроизводительный движок основанный на WGPU и предназначенный для
// рендеринга пользовательского интерфейса и игровых 2D сцен. MoonWalk распространяется
// свободно под лицензией EPL 2.0 (Eclipse public license). Подробнее про лицензию
// сказано в файле LICENSE (Корень репозитория). Copyright (с) 2025 MoonWalk
//
// Данный файл предоставляет публичный API рендер движка (В том числе и FFI) для
// использования в других проектах. В этом файле не должна содержаться какая-либо
// логика кроме подключения модулей и объявления публичных функций.
//
// Смотрите подробную документацию здесь: [ССЫЛКА]

// Этот модуль публичный так как используется в тестах
pub mod gpu;

pub mod error;
pub mod rendering;
pub mod objects;
pub mod resource_manager;
pub mod path;
mod batching;
mod textware;
mod fallback;
mod debug;
mod filters;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use glam::{Vec2, Vec4};
use resource_manager::ResourceManager;
use path::PathBuilder;

pub use crate::objects::ObjectId;
use crate::rendering::container::RenderContainer;
use crate::rendering::renderer::MoonRenderer;
use crate::error::MoonWalkError;

/// Основная структура движка которая содержит рендерер. Конструктор new
/// принимает окно (Которое можно получить через winit), ширину окна и
/// высоту окна. 
/// Пример (new возвращает result, необходимо обработать результат): 
/// let moonwalk = MoonWalk::new(static_window, 1280, 720).unwrap();
/// 
/// Совет: Вы можете получить статичное окно с помощью такого кода
/// let window = event_loop.create_window( ... ).unwrap();
/// let static_window: &'static Window = Box::leak(Box::new(window));
pub struct MoonWalk {
    renderer: MoonRenderer,
    pub resources: ResourceManager,
}

/// Обёртка над u64 для хранения айди шрифта (FontId из модуля textware)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontAsset(pub u64);

/// Типы выранивания текста. Влево, вправо, по центру и по ширине (строки растягиваются так, чтобы
/// касаться и левого и правого края блока)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justified,
}

impl MoonWalk {
    #[cfg(not(target_os = "android"))]
    pub fn new(
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, error::MoonWalkError> {
        let renderer = MoonRenderer::new(window, width, height)?;
        let resources = ResourceManager::new();

        Ok(Self {
            renderer,
            resources,
        })
    }

    /// Для android нужен отдельный new из-за AssetManager который необходим
    /// для загрузки шрифтов и текстур
    #[cfg(target_os = "android")]
    pub fn new(
        window: &'static (impl HasWindowHandle + HasDisplayHandle + Send + Sync),
        width: u32, height: u32,
        asset_manager: ndk::asset::AssetManager,
    ) -> Result<Self, error::MoonWalkError> {
        let renderer = MoonRenderer::new(window, width, height)?;
        let resources = ResourceManager::new(asset_manager);

        Ok(Self {
            renderer,
            resources,
        })
    }

    /// Функция чтобы установить размер viewport'а (Область, куда идёт рисование)
    /// Если пользователь вашего приложения изменит размер окна (Через оконный менеджер) 
    /// то область рисования не уменьшится и не увеличиться.
    /// Решение: слушать событие изменения размеров окна и вызывать mw.set_viewport
    /// передавая туда новую ширину и высоту окна.
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    /// Scale Factor нужно взять у winit либо другой библиотеки
    /// он необходим чтобы преобразовать логические размеры окна
    /// в физические (Иначе полноэкранного режима не будет на телефонах)
    pub fn set_scale_factor(&mut self, scale: f32) {
        self.renderer.set_scale_factor(scale);
    }

    /// Функция для рендеринга всех элементов которые накопил движок.
    /// Вызывать нужно КАЖДЫЙ КАДР, но не делать этого в бесконечном
    /// цикле (While/loop). Вместо этого лучше использовать встроенное
    /// событие в библиотеку для работы с окнами. Пример для winit:
    /// WindowEvent::RedrawRequested => { ... }
    /// Первый аргумент это структура Vec4 из крейта GLAM, сюда нужно
    /// передать цвет которым будет заливаться экран.
    pub fn render_frame(&mut self, _clear_color: Vec4) -> Result<(), MoonWalkError> {
        self.renderer.render()
    }

    /// Функция для создания прямоугольника и получения его ID.
    /// Важное предупреждение: НЕ СОЗДАВАЙТЕ ОБЪЕКТЫ КАЖДЫЙ КАДР
    /// ЕСЛИ ЭТО НЕ ВАША ПРЯМАЯ ЦЕЛЬ. После создания объекта он
    /// существует в кэше рендер движка и просто отправляется на
    /// отрисовку в момент вызове render_frame, вам нужно только
    /// создать объект один раз, получить его ID (структура ObjectId)
    /// и работать с ним используя методы конфигурации
    pub fn new_rect(&mut self) -> ObjectId {
        self.renderer.new_rect()
    }

    /// Функция для изменения позиции любого объекта по его ID
    /// (Структура ObjectId которую можно получить вызвав new_* функцию)
    /// принимает ID объекта и структуру Vec2 для описания 2D позиции
    /// в мировой системе координат, (0, 0) это верхний левый угол.
    pub fn set_position(&mut self, id: ObjectId, pos: Vec2) {
        self.renderer.config_position(id, pos);
    }

    /// Функция для изменения размер любого объекта по его ID
    /// (Структура ObjectId которую можно получить вызвав new_* функцию)
    /// принимает ID объекта и структуру Vec2 для описания ширины и высоты.
    pub fn set_size(&mut self, id: ObjectId, size: Vec2) {
        self.renderer.config_size(id, size);
    }

    /// Функция для конфигурации угла вращения. Принимает ID объекта
    /// и f32 в качестве угла. ИСПОЛЬЗУЕТ РАДИАНЫ, А НЕ ГРАДУСЫ!
    pub fn set_rotation(&mut self, id: ObjectId, radians: f32) {
        self.renderer.config_rotation(id, radians);
    }

    /// Функция для изменения цвета любого объекта по его ID
    /// принимает ObjectId и Vec4 из GLAM. Цвета заполняются
    /// следующим образом:
    ///     1 значение - Красный, 0-1 (Где 0 это 0, а 1 это 255 по RGBa)
    ///     2 значение - Зелёный, 0-1
    ///     3 значение - Синий  , 0-1
    ///     4 значение - Прозрачность (Альфа канал), тоже от 0 до 1
    ///
    /// Прозрачность 0 это объект не видно, 1 полностью видно,
    /// 0.5 это полупрозрачный
    pub fn set_color(&mut self, id: ObjectId, color: Vec4) {
        self.renderer.config_color(id, color);
    }

    /// Эта функция устаналивает второй цвет который нужен для градиента. Принимает
    /// айди объекта и vec4 (Второй цвет поддерживает прозрачность как и первый)
    /// Второй цвет начнёт работать только когда будет установлен линейный или
    /// радиальный градиент
    pub fn set_color2(&mut self, id: ObjectId, color2: Vec4) {
        self.renderer.config_color2(id, color2);
    }

    /// Эта функция устанавливает объекту объекту линейный градиент. Принимает его 
    /// айди и vec2 направления (В x и y). Линейный градиент это градиент, который
    /// направлен в одном конкретном направлении и цвет плавно меняется по этому
    /// направлению
    pub fn linear_gradient(&mut self, id: ObjectId, direction: Vec2) {
        self.renderer.config_gradient_data(id, [direction.x, direction.y, 0.0, 0.0]);
    }

    /// Эта функция устаналивает радиальный градиент объекту. Принимает айди объекта,
    /// центр самого градиента (vec2 из glam) и радиусы (Внутри и снаружи). Тоже vec2
    /// Радиальный градиент это градиент который выглядит как окружность внутри которой
    /// и происходит плавная смена цвета (Чем ближе к центру окружности)
    pub fn radial_gradient(&mut self, id: ObjectId, center: Vec2, radius: Vec2) {
        self.renderer.config_gradient_data(id, [center.x, center.y, radius.x, radius.y]);
    }

    /// Эта функция принимает айди объекта и удаляет градиент у него
    /// Работает и для линейного и для радиального
    pub fn reset_gradient(&mut self, id: ObjectId) {
        self.renderer.config_gradient_data(id, [0.0, 0.0, -1.0, 0.0]);
    }

    /// Функция для конфигурации скругления у прямоугольника.
    ///  - [!] Не работает для каких-либо объектов кроме прямоугольника.
    ///
    /// Принимает ID прямоугольника и Vec4 из GLAM для описания
    /// скругления каждого угла.
    ///    - [*] Скругление рисуется на GPU поэтому
    ///        не стоит переживать насчёт производительности от 
    ///        его использования.
    ///
    /// Описание radii (По часовой стрелке):
    ///     1 параметр - Верхний левый угол
    ///     2 параметр - Верхний праый угол
    ///     3 параметр - Нижний правый угол
    ///     4 параметр - Нижний левый угол
    ///
    ///    - [*] Про оптимизацию скругления - По факту скругление углов
    ///        ялвется чисто визуальным. У любого прямоугольника всегда
    ///        4 вершины и 6 индексов, но шейдер через алгоритм SDF
    ///        отсекает часть пикселей создавая скругление. Это очень
    ///        быстрый подход по сравнению со SKIA
    ///
    /// - [?] Для создания идеального круга создайте квадрат
    ///      (ширина и высота должна быть одинаковой) и установите
    ///      скругление углов на половину ширины/высоты.
    pub fn set_rounded(&mut self, id: ObjectId, radii: Vec4) {
        self.renderer.set_rounded(id, radii);
    }
 
    /// Функция определения Z индекса объекта. Обратите внимание,
    /// z индекс никак не вляяет на размер или координаты объекта.
    /// Он нужен чтобы отсортировать объекты и определить какие
    /// объекты будут перекрывать другие.
    ///     [*] Пример:
    ///         Объект A: Z индекс = 0.1
    ///         Объект B: Z индекс = 0.2
    ///     Объект B будет перекрывать объект A
    /// Принимает Id объекта и z индекс (флоат, может быть отрицательным
    /// Важно, z иднекс должен быть от 0 до 1. Нельзя использовать числа
    /// которые больше 1.0
    pub fn set_z_index(&mut self, id: ObjectId, z: f32) {
        self.renderer.set_z_index(id, z);
    }

    /// Эта функция устаналивает текстуру объекту. Сюда нужно передать айди объекта
    /// и айди текстуры. Айди текстуры млжно получить через mw.load_texture
    pub fn set_texture(&mut self, id: ObjectId, texture_id: u32) {
        self.renderer.state.store.config_texture(id, texture_id);
    }

    /// Эта функция устанавливает UV координаты для текстуры на объекте.
    /// Принимает айди объекта и массив из 4 флоатом (x, y, ширина, высота)
    pub fn set_uv(&mut self, id: ObjectId, uv: [f32; 4]) {
        self.renderer.state.store.config_uv(id, uv);
    }

    /// Эта функция устаналивает эффекты для объекта. Работает идеально только
    /// с прямоугольниками. Принимает айди объекта, ширину обводки и настройку
    /// для размытия углов (box_shadow Так как используется технология как в CSS)
    /// Цвет обводки объекта указывается в color2, обводка у которой есть градиент
    /// не поддерживается. Box_shadow не создаёт тень под объектом, а размывает границы.
    /// box_shadow не может заменить тени в Material design 3 либо в других дизайн
    /// системах где используется блюр по Гауссу, для реализации таких теней
    /// нужно создать рендер контейнер, сделать снапшот и применить блюр к текстуре
    /// два раза (горизонтально и вертикально) используя функцию mw.blur_texture(...)
    pub fn set_effect(&mut self, id: ObjectId, border_width: f32, box_shadow: f32) {
        self.renderer.set_effect(id, [border_width, box_shadow]);
    }

    /// Эта функция пересоздаёт холст для рендеринга. На android
    /// при сворачивании приложение старый холст удаляется поэтому
    /// нам нужен новый
    pub fn recreate_surface(&mut self, window: &(impl HasWindowHandle + HasDisplayHandle), width: u32, height: u32) {
        self.renderer.recreate_surface(window, width, height);
    }

    /// Эта функция делает объект с переданным ID мёртвым. Он сохраняет в ObjectStore,
    /// но перестаёт отрисовываться. Потом при создании другого объекта он занимает
    /// айди любого мёртвого объекта, если мёртвого объекта нет - создаёт новый id
    /// для себя
    pub fn remove(&mut self, id: ObjectId) {
        self.renderer.state.store.remove(id);
    }

    /// Эта функция агружает текстуру из файла через его путь
    ///  [!] Данная функция очень медленная, не рекомендуется подгружать всё
    ///      при старте программы
    /// На windows, linux, macos, bsd и android указывается путь в файловой системе
    /// На android указывается либо путь к файловой системе
    ///   (Определяется по "/" как первый символ)
    /// либо как имя файла в assets.
    ///
    /// [?] Android примеры:
    ///  "test.png" - файл test.png из assets приложения
    ///  "data/data/com.example.package/file/test.png" - файл test.png из файловой системы
    pub fn load_texture(&mut self, path: &str) -> Result<u32, error::MoonWalkError> {
        let texture = self.resources.load_texture(&self.renderer.context, path)?;
        let id = self.renderer.register_texture(texture);
        
        Ok(id)
    }

    /// Снапшот (скриншот) это запекание конкретной области на экране либо в рендер
    /// контейнере в указанных координатах (pos, Vec2 из glam) и с указанным размером
    /// (size, Vec2 из glam). На выходе у функции обычное айди текстуры которое
    /// можно использовать
    pub fn snapshot(&mut self, pos: Vec2, size: Vec2) -> u32 {
        self.renderer.request_snapshot(
            pos.x as u32, 
            pos.y as u32, 
            size.x as u32, 
            size.y as u32,
        )
    }

    /// Эта функция создаёт снапшот не в новой текстуре, а в уже существующей.
    /// Нужно только передать её айди 3 аргументом. Первый и второй аргумент
    /// не отличается от snapshot, также pos (Vec2 из glam) и size (Vec2 из glam)
    /// чтобы указать координаты области и ширину/высоту области для снапшота.
    /// Ничего не возвращает, только обновляет
    pub fn update_snapshot(&mut self, pos: Vec2, size: Vec2, id: u32) {
        self.renderer.update_snapshot(
            pos.x as u32, 
            pos.y as u32, 
            size.x as u32, 
            size.y as u32,
            id,
        );
    }

    /// Рендер контейнер (Либо просто контейнер) это отдельный невидимый рендерер 
    /// (он не отправляет данные куда-либо) в котором можно создавать отдельные 
    /// от основного объекты (со своим айди) и единственный способ получить
    /// изображение из него, это функция .snapshot() внутри которая позволяет
    /// превратить в текстуру участок этого контейнера, указав x/y этого участка
    /// и ширину/высоту участка. После создания контейнера можно использовать почти
    /// все API функции который представлены здесь для создания, изменения и удаления
    /// объектов в контейнере.
    ///  [!] Функций для пересоздания поверхности (surface), создания контейнера
    ///      и так далее нет. Только создание (new_*), изменение (set_*) и удаление
    ///      (remove)
    /// После создания контейнера получается экземпляр структуры RenderContainer.
    /// Чтобы делать снапшоты внутри него нужно в каждый кадр вызывать container.draw()
    ///  в функцию draw нужно передать экземпляр структуры MoonWalk и цвет заливки.
    /// Прозрачный цвет принимается (Vec4::ZERO), если его указать то у текстуры
    /// не будет фона (Она будет прозрачная). Пример:
    ///  container.draw(mw, Some(Vec4::ZERO));
    /// Только после того как была вызвана функция .draw можно делать снапшот, так
    /// как до этого момента данные ещё не готовы
    pub fn new_render_container(&self, width: u32, height: u32) -> RenderContainer {
        RenderContainer::new(&self.renderer.context, width, height)
    }

    /// Функции для фильтров текстуры (пост обработка)
    
    /// Эта функция перезаписывает текстуру применив к ней блюр по гаусу. Для
    /// правильного блюра обязательно применить блюр дважды, первый раз
    /// с horizontal true (горизонтальный проход), второй раз с horizontal false
    /// (вертикальный проход). Принимает айди текстуры, радиус блюра и направление
    pub fn blur_texture(&mut self, texture_id: u32, radius: f32, horizontal: bool) {
        self.renderer.apply_blur(texture_id, radius, horizontal);
    }

    /// Эта функция перезаписывает текстуру обновив её яркость. Принимает айди текстуры
    /// и значение яркости. 0.0 - нулевая яркость, 1.0 - стандартная (как в оригинале),
    /// 2.0 - в два раза ярче
    pub fn brightness(&mut self, texture_id: u32, factor: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_brightness(factor);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
    
    /// Эта функция перезаписывает текстуру обновив её контраст. Принимает айди текстуры
    /// и новое значение контраста
    pub fn contrast(&mut self, texture_id: u32, contrast: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_contrast(contrast);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
    
    /// Эта функция перезаписывает текстуру обновив её насыщеность. Принимает айди текстуры
    /// и новое значение насыщености
    pub fn saturation(&mut self, texture_id: u32, sat: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_saturation(sat);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }
    
    /// Эта функция перезаписывает текстуру обновив её тон. Принимает айди текстуры
    /// и градусы для угла поворота цветового колеса (от 0 до 360)
    pub fn hue_shift(&mut self, texture_id: u32, degrees: f32) {
        let (matrix, offset) = crate::filters::color_matrix::matrix_hue(degrees);
        self.renderer.apply_color_matrix(texture_id, matrix, offset);
    }

    /// Эта функция загружает шрифт во время выполнения программы (Этот шрифт обязательно
    /// должен поставляться с программой) используя путь к шрифту. Возвращает структуру
    /// FontAsset (обёртка для u64) который нужен чтобы не использовать структуру FontId
    /// из TextWare
    pub fn load_font(&mut self, path: &str, name: &str) -> Result<FontAsset, crate::error::MoonWalkError> {
        let bytes = self.resources.read_bytes(path)?;
        
        let internal_id = self.renderer.text_engine.font_system.load_font_from_bytes(&bytes, name)?;

        Ok(FontAsset(internal_id.0))
    }

    /// Эта функция загружает шрифт из набора байт который чаще всего известен уже на этапе
    /// компиляции. Создана для того, чтобы вшивать шрифт в бинарник/разделяемую библиотеку
    /// используя макрос для получения набора байтов из файла во время компиляции
    pub fn load_font_from_bytes(
        &mut self, 
        bytes: &[u8], 
        name: &str
    ) -> Result<FontAsset, crate::error::MoonWalkError> {
        let id = self.renderer.text_engine.font_system.load_font_from_bytes(bytes, name)?;
        Ok(FontAsset(id.0))
    }

    /// Эта функция создаёт текст. Рендеринг текстов менее производительный чем
    /// рендеринг прямоугольников, но чаще всего текстов и меньше чем прямоугольников
    /// (В играх и UI). Это не должно быть критичным, но нужно учитывать.
    /// В будущем могут быть работы по дополнительной оптимизации
    pub fn new_text(&mut self, content: &str, font: FontAsset, size: f32) -> ObjectId {
        // Конвертируем обратно во внутренний тип
        let internal_id = crate::textware::FontId(font.0);
        self.renderer.state.store.new_text(content.to_string(), internal_id, size)
    }

    /// Эта функция меняет уже существующий текст, принимает айди объекта (текста)
    /// у которого нужно изменить контент и новый контент
    pub fn set_text(&mut self, id: ObjectId, content: &str) {
        self.renderer.state.store.set_text(id, content.to_string());
    }

    /// Эта функция меняет размер шрифта у текста. Принимает айди текста и новый
    /// размер шрифта в формате f32
    pub fn set_font_size(&mut self, id: ObjectId, size: f32) {
        self.renderer.state.store.set_font_size(id, size);
    }
    
    /// Эта функция меняет границы текста. Не путать с set_font_size,
    /// она меняет размер шрифта, а эта функция меняет размер границ текста
    /// (как и было указанно ранее). Границы текста необходимы для переноса текста
    /// принимает айди, новую шириную и новую высоту
    pub fn set_text_size(&mut self, id: ObjectId, w: f32, h: f32) {
        self.renderer.state.store.set_text_bounds(id, w, h);
    }

    /// Эта функция нужна для выравнивания текста. Принимает его айди и
    /// направление для выравнивания. В enum TextAlign есть направления:
    /// 1. Left, выравнивания по левой стороне
    /// 2. Center, выравнивание по центру
    /// 3. Right, выравнивание по правой стороне
    /// 4. Justified, текст прижат к сторонам своих границ (они меняются через set_text_size) 
    pub fn set_text_align(&mut self, id: ObjectId, align: TextAlign) {
        let val = match align {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
            TextAlign::Justified => 3,
        };
        self.renderer.state.store.set_text_align(id, val);
    }
    
    /// Эта функция нужна для получения размеров текста. Принимает контент строкой, шрифт
    /// (FontAsset, его можно получить через функцию load_font и load_font_from_bytes)
    /// размер шрифта и максимальную ширину. На выходе идёт Vec2 из крейта glam который
    /// содержит ширину и высоту текста по указанным параметрам
    pub fn measure_text(&mut self, text: &str, font: FontAsset, size: f32, max_width: f32) -> Vec2 {
        let (w, h) = self.renderer.text_engine.measure_text(
            text, 
            crate::textware::FontId(font.0), 
            size, 
            max_width
        );
        Vec2::new(w, h)
    }

    /// [WAIT DOC]
    pub fn new_path_builder(&self) -> PathBuilder {
        PathBuilder::new()
    }

    /// [WAIT DOC]
    pub fn parse_svg_path(&self, pb: &mut crate::path::PathBuilder, data: &str) -> Result<(), String> {
        crate::path::svg::parse_svg_path(pb, data)
    }
}
