# MoonWalk Widget
Это легкий и быстрый крейт для верстки интерфейсов через рендер движок MoonWalk. Он связывает графические объекты движка с мощной системой лейаутов taffy для реализации flexbox системы

### Особенности

*   **Flexbox верстка:** Строки, колонки, отступы, выравнивание, растягивание
*   **Императивный подход:** Полный контроль над деревом виджетов
*   **Высокая производительность:** Никаких лишних аллокаций каждый кадр. Лейаут пересчитывается только тогда, когда вы этого захотите
*   **Автоматический Z index:** Элементы, добавленные позже или находящиеся глубже в дереве, автоматически рисуются поверх остальных

### Быстрый старт

Работа строится вокруг WidgetTree это менеджер который знает структуру вашего интерфейса и двигает объекты MoonWalk

```rust
use moonwalk::{MoonWalk, Vec2, Vec4};
use moonwalk_bootstrap::{Application, Runner, WindowSettings};
use moonwalk_widget::{WidgetTree, Layout, AlignItems, JustifyContent};

struct MyApp {
    tree: WidgetTree,
}

impl Application for MyApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        // Корневой элемент на весь экран
        let root_style = Layout::column()
            .width(viewport.x)
            .height(viewport.y)
            .align(AlignItems::Center) // Центрирует детей по горизонтали
            .justify(JustifyContent::Center) // И по вертикали
            .gap(20.0, 20.0); // Отступы между элементами

        self.tree = WidgetTree::new(root_style);

        // Кнопка через стандартные функции MoonWalk
        let btn_id = mw.new_rect();
        mw.set_color(btn_id, Vec4::new(0.2, 0.6, 1.0, 1.0));
        mw.set_rounded(btn_id, Vec4::splat(10.0));

        // Создание лайаута для кнопки
        let btn_node = self.tree.new_node(
            Layout::row()
                .width(200.0)
                .height(50.0)
        );

        // Связывание их и добавляем в дерево
        self.tree.bind(btn_node, btn_id);
        self.tree.add_child(self.tree.root, btn_node);

        // Вычисляем и применяем позиции
        self.tree.compute_and_apply(mw, viewport.x, viewport.y);
    }

    fn on_resize(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        // При изменении окна обновляем размер корня
        self.tree.set_style(self.tree.root, 
            Layout::column().width(viewport.x).height(viewport.y)
        );
        
        // И пересчитываем все дерево
        self.tree.compute_and_apply(mw, viewport.x, viewport.y);
    }
}
```

### Как это работает

#### Layout (Конструктор стилей)
Используйте **Layout** чтобы описать параметры блока

**Контейнеры:**
*   `Layout::column()` Элементы идут сверху вниз
*   `Layout::row()` Элементы идут слева направо
*   `Layout::stack()` Элементы накладываются друг на друга (используйте для слоев и абсолютного позиционирования)

**Выравнивание:**
*   `.justify(JustifyContent::...)` Выравнивание по главной оси (вдоль строки или колонки)
    *   `Start`, `Center`, `End`
    *   `SpaceBetween` (по краям), `SpaceAround`, `SpaceEvenly` (равные отступы)
*   `.align(AlignItems::...)` Выравнивание по поперечной оси
    *   `Start`, `Center`, `End`
    *   `Stretch` (растянуть на всю ширину/высоту)

**Размеры:**
*   `.width(100.0)`, `.height(50.0)` Точные размеры
*   `.width_pct(0.5)` 50% от ширины родителя
*   `.width_auto()` Размер зависит от контента
*   `.grow(1.0)` Растянуться, чтобы заполнить свободное место.

**Отступы:**
*   `.padding(left, top, right, bottom)` Внутренние отступы.
*   `.padding_all(10.0)` Отступ со всех сторон.
*   `.gap(x, y)` Расстояние между детьми.

#### Связывание
Сама по себе нода в WidgetTree невидима. Это просто прямоугольник в математической модели. Чтобы увидеть её, нужно привязать к ней объект MoonWalk (ObjectId)

*   `tree.new_node(layout)` Создает ноду лайаута.
*   `tree.bind(node_id, object_id)` Говорит дереву "Когда двигаешь эту ноду, двигай и этот объект"
*   `tree.unbind(node_id)` Убирает связь

> **Совет:** Вы можете создавать ноды без привязки к объектам. Это полезно для создания невидимых контейнеров, группировок и распорок

#### Обновление
Лайаут не пересчитывается сам по себе. Вы должны вызвать `tree.compute_and_apply(mw, w, h)` когда:
1.  Изменился размер окна
2.  Вы добавили или удалили элементы
3.  Вы изменили стиль какого-то элемента

Этот метод делает всю тяжелую работу, то есть считает flexbox математику и массово обновляет позиции (set_position) и размеры (set_size) в MoonWalk. Также он автоматически выставляет правильный Z index для всех элементов на основе их порядка в дереве

---

### Примеры

**Центрирование элемента:**
```rust
Layout::column()
    .size_pct(1.0, 1.0)      // На весь родительский блок
    .align(AlignItems::Center) // По горизонтали
    .justify(JustifyContent::Center) // По вертикали
```

**Разнести по краям (SpaceBetween):**
```rust
// Полезно для хедеров: логотип слева, кнопка справа
Layout::row()
    .width_pct(1.0)
    .justify(JustifyContent::SpaceBetween)
```

**Список с авто-переносом (Wrap):**
```rust
Layout::row()
    .wrap() // Если не влезает, перенести на новую строку
    .gap(10.0, 10.0)
```

**Абсолютное позиционирование (например, бейдж на иконке):**
```rust
// Родитель
Layout::stack().size(50.0, 50.0);

// Ребенок (бейдж в правом верхнем углу)
Layout::row()
    .absolute()
    .size(15.0, 15.0)
    .inset(f32::NAN, -5.0, -5.0, f32::NAN) // top: -5, right: -5, остальные auto
```

### Лицензия
Распространяется под лицензией EPL 2.0
