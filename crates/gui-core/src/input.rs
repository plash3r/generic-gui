#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Tab,
    Enter,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Character(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    PointerMove { dx: i32, dy: i32 },
    PointerButton { button: MouseButton, pressed: bool },
    Scroll { delta: i32 },
    Key { key: Key, pressed: bool },
}
