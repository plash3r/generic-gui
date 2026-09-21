#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod abi;

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
use generic_gui_core::{Color, Desktop, InputEvent, Key, MouseButton, Rect, Renderer, TextStyle};

const MAX_WIDTH: usize = 1280;
const MAX_HEIGHT: usize = 1024;
const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;
const HEAP_BYTES: usize = 256 * 1024;
const TEXT_SCALE: i32 = 2;

struct BumpAllocator {
    next: AtomicUsize,
}

#[repr(align(16))]
struct UserHeap([u8; HEAP_BYTES]);

#[repr(align(4096))]
struct PixelBuffer([u32; MAX_PIXELS]);

static mut USER_HEAP: UserHeap = UserHeap([0; HEAP_BYTES]);
static mut PIXELS: PixelBuffer = PixelBuffer([0; MAX_PIXELS]);

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    next: AtomicUsize::new(0),
};

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = core::ptr::addr_of_mut!(USER_HEAP.0) as *mut u8 as usize;
        let align = layout.align().max(1);
        let size = layout.size().max(1);

        loop {
            let current = self.next.load(Ordering::Relaxed);
            let absolute = match base.checked_add(current) {
                Some(value) => value,
                None => return ptr::null_mut(),
            };
            let aligned = match absolute
                .checked_add(align - 1)
                .map(|value| value & !(align - 1))
            {
                Some(value) => value,
                None => return ptr::null_mut(),
            };
            let offset = aligned - base;
            let end = match offset.checked_add(size) {
                Some(value) if value <= HEAP_BYTES => value,
                _ => return ptr::null_mut(),
            };

            if self
                .next
                .compare_exchange(current, end, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return aligned as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

struct FramebufferRenderer {
    pixels: *mut u32,
    width: i32,
    height: i32,
}

impl FramebufferRenderer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: core::ptr::addr_of_mut!(PIXELS.0) as *mut u32,
            width: width as i32,
            height: height as i32,
        }
    }

    fn present(&self) -> bool {
        let count = (self.width as usize).saturating_mul(self.height as usize);
        abi::present(self.pixels.cast_const(), count)
    }

    fn packed(color: Color) -> u32 {
        ((color.red() as u32) << 16) | ((color.green() as u32) << 8) | color.blue() as u32
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let index = y as usize * self.width as usize + x as usize;
        // SAFETY: clipping above keeps the write inside the statically sized
        // backbuffer because startup rejects modes larger than MAX_PIXELS.
        unsafe {
            self.pixels.add(index).write(Self::packed(color));
        }
    }

    fn clipped(&self, rect: Rect) -> Option<(i32, i32, i32, i32)> {
        let x0 = rect.x.max(0).min(self.width);
        let y0 = rect.y.max(0).min(self.height);
        let x1 = rect.right().max(0).min(self.width);
        let y1 = rect.bottom().max(0).min(self.height);
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }

    fn draw_glyph(&mut self, x: i32, y: i32, character: char, color: Color) {
        let glyph = glyph5x7(character.to_ascii_uppercase());
        for (row, bits) in glyph.iter().copied().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) == 0 {
                    continue;
                }
                let pixel_x = x + column * TEXT_SCALE;
                let pixel_y = y + row as i32 * TEXT_SCALE;
                self.fill_rect(Rect::new(pixel_x, pixel_y, TEXT_SCALE, TEXT_SCALE), color);
            }
        }
    }
}

impl Renderer for FramebufferRenderer {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn clear(&mut self, color: Color) {
        let value = Self::packed(color);
        let count = self.width as usize * self.height as usize;
        // SAFETY: the renderer dimensions were validated against MAX_PIXELS.
        unsafe {
            core::slice::from_raw_parts_mut(self.pixels, count).fill(value);
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let Some((x0, y0, x1, y1)) = self.clipped(rect) else {
            return;
        };
        let value = Self::packed(color);
        for y in y0..y1 {
            let start = y as usize * self.width as usize + x0 as usize;
            let len = (x1 - x0) as usize;
            // SAFETY: clipped coordinates are inside the backbuffer.
            unsafe {
                core::slice::from_raw_parts_mut(self.pixels.add(start), len).fill(value);
            }
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, thickness: i32) {
        let thickness = thickness.max(1);
        self.fill_rect(Rect::new(rect.x, rect.y, rect.width, thickness), color);
        self.fill_rect(
            Rect::new(
                rect.x,
                rect.bottom().saturating_sub(thickness),
                rect.width,
                thickness,
            ),
            color,
        );
        self.fill_rect(Rect::new(rect.x, rect.y, thickness, rect.height), color);
        self.fill_rect(
            Rect::new(
                rect.right().saturating_sub(thickness),
                rect.y,
                thickness,
                rect.height,
            ),
            color,
        );
    }

    fn text(&mut self, x: i32, y: i32, text: &str, color: Color, _style: TextStyle) {
        let mut cursor_x = x;
        for character in text.chars() {
            self.draw_glyph(cursor_x, y, character, color);
            cursor_x += 6 * TEXT_SCALE;
            if cursor_x >= self.width {
                break;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    abi::write_stdout("GENERIC GUI: userspace display server starting\n");

    let Some(info) = abi::display_info() else {
        abi::write_stdout("GENERIC GUI: display ABI unavailable\n");
        abi::exit(0x40);
    };

    let width = info.width as usize;
    let height = info.height as usize;
    if width == 0 || height == 0 || width > MAX_WIDTH || height > MAX_HEIGHT {
        abi::write_stdout("GENERIC GUI: unsupported framebuffer dimensions\n");
        abi::exit(0x41);
    }

    let mut renderer = FramebufferRenderer::new(width, height);
    let mut desktop = Desktop::new(width as i32, height as i32);
    desktop.add_window(
        "Welcome",
        Rect::new(70, 70, (width as i32 / 2).max(320), 260),
    );
    desktop.add_window(
        "Terminal",
        Rect::new(
            (width as i32 / 3).max(140),
            (height as i32 / 3).max(150),
            (width as i32 / 2).max(360),
            280,
        ),
    );

    desktop.render(&mut renderer);
    if !renderer.present() {
        abi::write_stdout("GENERIC GUI: initial display_present failed\n");
        abi::exit(0x42);
    }
    abi::write_stdout("GENERIC GUI: desktop presented; input loop active\n");

    let mut previous_buttons = 0u32;
    loop {
        let mut dirty = false;
        if let Some(packet) = abi::poll_input() {
            match packet.kind {
                abi::INPUT_MOUSE => {
                    if packet.x != 0 || packet.y != 0 {
                        desktop.handle_event(InputEvent::PointerMove {
                            dx: packet.x,
                            dy: packet.y,
                        });
                        dirty = true;
                    }
                    if packet.z != 0 {
                        desktop.handle_event(InputEvent::Scroll { delta: packet.z });
                        dirty = true;
                    }

                    for (mask, button) in [
                        (1u32, MouseButton::Left),
                        (2u32, MouseButton::Right),
                        (4u32, MouseButton::Middle),
                    ] {
                        let was_pressed = previous_buttons & mask != 0;
                        let pressed = packet.flags & mask != 0;
                        if was_pressed != pressed {
                            desktop.handle_event(InputEvent::PointerButton { button, pressed });
                            dirty = true;
                        }
                    }
                    previous_buttons = packet.flags & 0x7;
                }
                abi::INPUT_KEY => {
                    if let Some(key) = translate_key(packet.code) {
                        desktop.handle_event(InputEvent::Key { key, pressed: true });
                        dirty = true;
                    }
                }
                _ => {}
            }
        }

        if dirty {
            desktop.render(&mut renderer);
            let _ = renderer.present();
        } else {
            // The first bridge is polling-based. PAUSE keeps the spin loop
            // friendly until scheduler-backed blocking input lands.
            unsafe {
                asm!("pause", options(nomem, nostack, preserves_flags));
            }
        }
    }
}

fn translate_key(code: u32) -> Option<Key> {
    match code {
        abi::KEY_TAB => Some(Key::Tab),
        abi::KEY_ENTER => Some(Key::Enter),
        abi::KEY_ESCAPE => Some(Key::Escape),
        abi::KEY_ARROW_UP => Some(Key::ArrowUp),
        abi::KEY_ARROW_DOWN => Some(Key::ArrowDown),
        abi::KEY_ARROW_LEFT => Some(Key::ArrowLeft),
        abi::KEY_ARROW_RIGHT => Some(Key::ArrowRight),
        value if value <= 0x7f => char::from_u32(value).map(Key::Character),
        _ => None,
    }
}

fn glyph5x7(character: char) -> [u8; 7] {
    match character {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1b, 0x11],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        '6' => [0x0e, 0x10, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x01, 0x0e],
        '-' => [0, 0, 0, 0x1f, 0, 0, 0],
        ':' => [0, 0x04, 0, 0, 0x04, 0, 0],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        _ => [0; 7],
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    abi::write_stdout("GENERIC GUI: panic\n");
    abi::exit(0xfe)
}

#[alloc_error_handler]
fn allocation_error(_layout: Layout) -> ! {
    abi::write_stdout("GENERIC GUI: allocation failed\n");
    abi::exit(0xfd)
}
