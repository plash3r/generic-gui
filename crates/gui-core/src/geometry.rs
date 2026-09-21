#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }

    pub fn contains(self, point: Point) -> bool {
        self.width > 0
            && self.height > 0
            && point.x >= self.x
            && point.y >= self.y
            && point.x < self.right()
            && point.y < self.bottom()
    }

    pub fn inset(self, amount: i32) -> Self {
        let doubled = amount.saturating_mul(2);
        Self {
            x: self.x.saturating_add(amount),
            y: self.y.saturating_add(amount),
            width: self.width.saturating_sub(doubled).max(0),
            height: self.height.saturating_sub(doubled).max(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_uses_half_open_bounds() {
        let rect = Rect::new(10, 20, 30, 40);
        assert!(rect.contains(Point::new(10, 20)));
        assert!(rect.contains(Point::new(39, 59)));
        assert!(!rect.contains(Point::new(40, 59)));
        assert!(!rect.contains(Point::new(39, 60)));
    }

    #[test]
    fn inset_never_produces_negative_size() {
        assert_eq!(Rect::new(0, 0, 5, 5).inset(10), Rect::new(10, 10, 0, 0));
    }
}
