use core::{arch::asm, mem::size_of};

pub const ABI_VERSION: u32 = 1;
pub const DISPLAY_FORMAT_XRGB8888: u32 = 1;

pub const INPUT_NONE: u32 = 0;
pub const INPUT_KEY: u32 = 1;
pub const INPUT_MOUSE: u32 = 2;

pub const KEY_TAB: u32 = 0x100;
pub const KEY_ENTER: u32 = 0x101;
pub const KEY_ESCAPE: u32 = 0x102;
pub const KEY_BACKSPACE: u32 = 0x103;
pub const KEY_F12: u32 = 0x104;
pub const KEY_ARROW_UP: u32 = 0x110;
pub const KEY_ARROW_DOWN: u32 = 0x111;
pub const KEY_ARROW_LEFT: u32 = 0x112;
pub const KEY_ARROW_RIGHT: u32 = 0x113;

const SYS_EXIT: u64 = 0;
const SYS_WRITE: u64 = 2;
const SYS_DISPLAY_INFO: u64 = 3;
const SYS_DISPLAY_PRESENT: u64 = 4;
const SYS_INPUT_POLL: u64 = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DisplayInfo {
    pub abi_version: u32,
    pub width: u32,
    pub height: u32,
    pub source_stride: u32,
    pub source_bytes_per_pixel: u32,
    pub source_pixel_format: u32,
    pub native_pixel_format: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputPacket {
    pub kind: u32,
    pub code: u32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub flags: u32,
}

pub fn display_info() -> Option<DisplayInfo> {
    let mut info = DisplayInfo::default();
    let result = syscall3(
        SYS_DISPLAY_INFO,
        (&mut info as *mut DisplayInfo) as u64,
        size_of::<DisplayInfo>() as u64,
        0,
    );
    if result == 0
        && info.abi_version == ABI_VERSION
        && info.source_pixel_format == DISPLAY_FORMAT_XRGB8888
    {
        Some(info)
    } else {
        None
    }
}

pub fn present(pixels: *const u32, count: usize) -> bool {
    let Some(bytes) = count.checked_mul(size_of::<u32>()) else {
        return false;
    };
    syscall3(SYS_DISPLAY_PRESENT, pixels as u64, bytes as u64, 0) == 0
}

pub fn poll_input() -> Option<InputPacket> {
    let mut packet = InputPacket::default();
    let result = syscall3(
        SYS_INPUT_POLL,
        (&mut packet as *mut InputPacket) as u64,
        size_of::<InputPacket>() as u64,
        0,
    );
    if result == 1 && packet.kind != INPUT_NONE {
        Some(packet)
    } else {
        None
    }
}

pub fn write_stdout(message: &str) {
    let _ = syscall3(SYS_WRITE, 1, message.as_ptr() as u64, message.len() as u64);
}

pub fn exit(code: u64) -> ! {
    let _ = syscall3(SYS_EXIT, code, 0, 0);
    loop {
        core::hint::spin_loop();
    }
}

fn syscall3(number: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let mut result = number;
    // SAFETY: Generic's userspace ABI uses int 0x80 with the syscall number in
    // RAX and the first three arguments in RDI/RSI/RDX. The kernel preserves
    // all registers except the RAX return value.
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") result,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
        );
    }
    result
}
