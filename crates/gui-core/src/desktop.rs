use alloc::{string::String, vec::Vec};

use crate::{
    geometry::{Point, Rect},
    input::{InputEvent, Key, MouseButton},
    render::{Color, Renderer, TextStyle},
};

const TOP_BAR_HEIGHT: i32 = 30;
const TASKBAR_HEIGHT: i32 = 44;
const TITLE_HEIGHT: i32 = 30;
const CONTROL_SIZE: i32 = 22;
const WINDOW_MIN_VISIBLE: i32 = 80;

const DESKTOP: Color = Color::rgb(24, 31, 42);
const TOP_BAR: Color = Color::rgb(18, 24, 33);
const TASKBAR: Color = Color::rgb(20, 27, 37);
const WINDOW_BG: Color = Color::rgb(35, 44, 57);
const TITLE_ACTIVE: Color = Color::rgb(42, 107, 160);
const TITLE_INACTIVE: Color = Color::rgb(55, 66, 82);
const BORDER: Color = Color::rgb(88, 105, 126);
const TEXT: Color = Color::rgb(232, 238, 246);
const MUTED: Color = Color::rgb(161, 174, 191);
const CLOSE: Color = Color::rgb(176, 63, 63);
const BUTTON: Color = Color::rgb(54, 67, 84);

pub type WindowId = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub rect: Rect,
    pub minimized: bool,
    pub closed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DragState {
    id: WindowId,
    offset_x: i32,
    offset_y: i32,
}

pub struct Desktop {
    width: i32,
    height: i32,
    cursor: Point,
    left_pressed: bool,
    windows: Vec<Window>,
    z_order: Vec<WindowId>,
    focused: Option<WindowId>,
    dragging: Option<DragState>,
    next_id: WindowId,
}

impl Desktop {
    pub fn new(width: i32, height: i32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            width,
            height,
            cursor: Point::new(width / 2, height / 2),
            left_pressed: false,
            windows: Vec::new(),
            z_order: Vec::new(),
            focused: None,
            dragging: None,
            next_id: 1,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn cursor(&self) -> Point {
        self.cursor
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focused
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|window| window.id == id)
    }

    pub fn add_window(&mut self, title: impl Into<String>, rect: Rect) -> WindowId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1).max(1);

        let rect = self.clamp_window(rect);
        self.windows.push(Window {
            id,
            title: title.into(),
            rect,
            minimized: false,
            closed: false,
        });
        self.z_order.push(id);
        self.focused = Some(id);
        id
    }

    pub fn show(&mut self, id: WindowId) {
        if let Some(window) = self.window_mut(id) {
            window.closed = false;
            window.minimized = false;
            self.bring_front(id);
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::PointerMove { dx, dy } => {
                self.cursor.x = self.cursor.x.saturating_add(dx).clamp(0, self.width - 1);
                self.cursor.y = self.cursor.y.saturating_add(dy).clamp(0, self.height - 1);
                if self.left_pressed {
                    self.apply_drag();
                }
            }
            InputEvent::PointerButton {
                button: MouseButton::Left,
                pressed,
            } => {
                let was_pressed = self.left_pressed;
                self.left_pressed = pressed;
                if pressed && !was_pressed {
                    self.pointer_down();
                } else if !pressed {
                    self.dragging = None;
                }
            }
            InputEvent::Key {
                key: Key::Tab,
                pressed: true,
            } => self.focus_next(),
            _ => {}
        }
    }

    pub fn render<R: Renderer>(&self, renderer: &mut R) {
        renderer.clear(DESKTOP);
        renderer.fill_rect(Rect::new(0, 0, self.width, TOP_BAR_HEIGHT), TOP_BAR);
        renderer.text(12, 7, "Generic", TEXT, TextStyle::Bold);
        renderer.fill_rect(
            Rect::new(0, self.height - TASKBAR_HEIGHT, self.width, TASKBAR_HEIGHT),
            TASKBAR,
        );

        for id in &self.z_order {
            let Some(window) = self.window(*id) else {
                continue;
            };
            if window.closed || window.minimized {
                continue;
            }
            self.render_window(renderer, window);
        }

        self.render_taskbar(renderer);
        self.render_cursor(renderer);
    }

    fn render_window<R: Renderer>(&self, renderer: &mut R, window: &Window) {
        renderer.fill_rect(window.rect, WINDOW_BG);
        renderer.stroke_rect(window.rect, BORDER, 1);

        let title = self.title_rect(window.rect);
        renderer.fill_rect(
            title,
            if self.focused == Some(window.id) {
                TITLE_ACTIVE
            } else {
                TITLE_INACTIVE
            },
        );
        renderer.text(
            title.x + 10,
            title.y + 7,
            &window.title,
            TEXT,
            TextStyle::Bold,
        );

        let minimize = self.minimize_button(window.rect);
        renderer.fill_rect(minimize, BUTTON);
        renderer.text(minimize.x + 7, minimize.y + 2, "-", TEXT, TextStyle::Bold);

        let close = self.close_button(window.rect);
        renderer.fill_rect(close, CLOSE);
        renderer.text(close.x + 7, close.y + 3, "x", TEXT, TextStyle::Bold);

        renderer.text(
            window.rect.x + 16,
            window.rect.y + TITLE_HEIGHT + 18,
            "Application surface",
            MUTED,
            TextStyle::Regular,
        );
    }

    fn render_taskbar<R: Renderer>(&self, renderer: &mut R) {
        let mut x = 8;
        for id in &self.z_order {
            let Some(window) = self.window(*id) else {
                continue;
            };
            if window.closed {
                continue;
            }
            let button = Rect::new(x, self.height - TASKBAR_HEIGHT + 7, 140, 30);
            renderer.fill_rect(button, BUTTON);
            renderer.text(
                button.x + 8,
                button.y + 7,
                &window.title,
                TEXT,
                TextStyle::Regular,
            );
            x += 146;
            if x >= self.width - 140 {
                break;
            }
        }
    }

    fn render_cursor<R: Renderer>(&self, renderer: &mut R) {
        renderer.fill_rect(Rect::new(self.cursor.x, self.cursor.y, 2, 16), Color::WHITE);
        renderer.fill_rect(Rect::new(self.cursor.x, self.cursor.y, 10, 2), Color::WHITE);
        renderer.fill_rect(
            Rect::new(self.cursor.x + 2, self.cursor.y + 2, 2, 10),
            Color::WHITE,
        );
    }

    fn pointer_down(&mut self) {
        if let Some(id) = self.taskbar_hit(self.cursor) {
            self.show(id);
            return;
        }

        let hit = self.z_order.iter().rev().copied().find(|id| {
            self.window(*id)
                .map(|window| {
                    !window.closed && !window.minimized && window.rect.contains(self.cursor)
                })
                .unwrap_or(false)
        });

        let Some(id) = hit else {
            self.focused = None;
            return;
        };

        self.bring_front(id);
        let Some(rect) = self.window(id).map(|window| window.rect) else {
            return;
        };

        if self.close_button(rect).contains(self.cursor) {
            if let Some(window) = self.window_mut(id) {
                window.closed = true;
            }
            self.dragging = None;
            self.focused = self.top_visible_window();
            return;
        }

        if self.minimize_button(rect).contains(self.cursor) {
            if let Some(window) = self.window_mut(id) {
                window.minimized = true;
            }
            self.dragging = None;
            self.focused = self.top_visible_window();
            return;
        }

        if self.title_rect(rect).contains(self.cursor) {
            self.dragging = Some(DragState {
                id,
                offset_x: self.cursor.x - rect.x,
                offset_y: self.cursor.y - rect.y,
            });
        }
    }

    fn apply_drag(&mut self) {
        let Some(drag) = self.dragging else {
            return;
        };
        let Some(rect) = self.window(drag.id).map(|window| window.rect) else {
            self.dragging = None;
            return;
        };

        let new_rect = Rect::new(
            self.cursor.x - drag.offset_x,
            self.cursor.y - drag.offset_y,
            rect.width,
            rect.height,
        );
        let clamped = self.clamp_window(new_rect);
        if let Some(window) = self.window_mut(drag.id) {
            window.rect = clamped;
        }
    }

    fn focus_next(&mut self) {
        let visible: Vec<WindowId> = self
            .z_order
            .iter()
            .copied()
            .filter(|id| {
                self.window(*id)
                    .map(|window| !window.closed && !window.minimized)
                    .unwrap_or(false)
            })
            .collect();

        if visible.is_empty() {
            self.focused = None;
            return;
        }

        let next = match self.focused {
            Some(current) => visible
                .iter()
                .position(|id| *id == current)
                .map(|index| visible[(index + 1) % visible.len()])
                .unwrap_or(visible[0]),
            None => visible[0],
        };
        self.bring_front(next);
    }

    fn taskbar_hit(&self, point: Point) -> Option<WindowId> {
        let mut x = 8;
        for id in &self.z_order {
            let window = self.window(*id)?;
            if window.closed {
                continue;
            }
            let button = Rect::new(x, self.height - TASKBAR_HEIGHT + 7, 140, 30);
            if button.contains(point) {
                return Some(*id);
            }
            x += 146;
            if x >= self.width - 140 {
                break;
            }
        }
        None
    }

    fn bring_front(&mut self, id: WindowId) {
        if let Some(position) = self.z_order.iter().position(|candidate| *candidate == id) {
            self.z_order.remove(position);
            self.z_order.push(id);
            self.focused = Some(id);
        }
    }

    fn top_visible_window(&self) -> Option<WindowId> {
        self.z_order.iter().rev().copied().find(|id| {
            self.window(*id)
                .map(|window| !window.closed && !window.minimized)
                .unwrap_or(false)
        })
    }

    fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == id)
    }

    fn clamp_window(&self, mut rect: Rect) -> Rect {
        rect.width = rect.width.clamp(120, self.width.max(120));
        rect.height = rect
            .height
            .clamp(TITLE_HEIGHT + 40, self.height.max(TITLE_HEIGHT + 40));

        let min_x = -rect.width + WINDOW_MIN_VISIBLE;
        let max_x = self.width - WINDOW_MIN_VISIBLE;
        rect.x = rect.x.clamp(min_x, max_x);

        let min_y = TOP_BAR_HEIGHT;
        let max_y = (self.height - TASKBAR_HEIGHT - TITLE_HEIGHT).max(min_y);
        rect.y = rect.y.clamp(min_y, max_y);
        rect
    }

    fn title_rect(&self, rect: Rect) -> Rect {
        Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, TITLE_HEIGHT)
    }

    fn close_button(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + rect.width - CONTROL_SIZE - 6,
            rect.y + 4,
            CONTROL_SIZE,
            CONTROL_SIZE,
        )
    }

    fn minimize_button(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + rect.width - CONTROL_SIZE * 2 - 10,
            rect.y + 4,
            CONTROL_SIZE,
            CONTROL_SIZE,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Renderer;

    struct MockRenderer {
        width: i32,
        height: i32,
        operations: usize,
    }

    impl Renderer for MockRenderer {
        fn width(&self) -> i32 {
            self.width
        }

        fn height(&self) -> i32 {
            self.height
        }

        fn clear(&mut self, _color: Color) {
            self.operations += 1;
        }

        fn fill_rect(&mut self, _rect: Rect, _color: Color) {
            self.operations += 1;
        }

        fn stroke_rect(&mut self, _rect: Rect, _color: Color, _thickness: i32) {
            self.operations += 1;
        }

        fn text(&mut self, _x: i32, _y: i32, _text: &str, _color: Color, _style: TextStyle) {
            self.operations += 1;
        }
    }

    fn move_cursor_to(desktop: &mut Desktop, point: Point) {
        let current = desktop.cursor();
        desktop.handle_event(InputEvent::PointerMove {
            dx: point.x - current.x,
            dy: point.y - current.y,
        });
    }

    #[test]
    fn drag_moves_focused_window() {
        let mut desktop = Desktop::new(800, 600);
        let id = desktop.add_window("Test", Rect::new(100, 100, 300, 200));

        move_cursor_to(&mut desktop, Point::new(120, 110));
        desktop.handle_event(InputEvent::PointerButton {
            button: MouseButton::Left,
            pressed: true,
        });
        desktop.handle_event(InputEvent::PointerMove { dx: 50, dy: 20 });
        desktop.handle_event(InputEvent::PointerButton {
            button: MouseButton::Left,
            pressed: false,
        });

        assert_eq!(desktop.window(id).unwrap().rect.x, 150);
        assert_eq!(desktop.window(id).unwrap().rect.y, 120);
        assert_eq!(desktop.focused(), Some(id));
    }

    #[test]
    fn tab_cycles_focus() {
        let mut desktop = Desktop::new(800, 600);
        let first = desktop.add_window("First", Rect::new(80, 80, 260, 180));
        let second = desktop.add_window("Second", Rect::new(120, 120, 260, 180));
        assert_eq!(desktop.focused(), Some(second));

        desktop.handle_event(InputEvent::Key {
            key: Key::Tab,
            pressed: true,
        });
        assert_eq!(desktop.focused(), Some(first));
    }

    #[test]
    fn render_is_platform_independent() {
        let mut desktop = Desktop::new(640, 480);
        desktop.add_window("Welcome", Rect::new(60, 60, 320, 220));
        let mut renderer = MockRenderer {
            width: 640,
            height: 480,
            operations: 0,
        };

        desktop.render(&mut renderer);
        assert!(renderer.operations > 10);
        assert_eq!(renderer.width(), 640);
        assert_eq!(renderer.height(), 480);
    }
}
