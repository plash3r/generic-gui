# Generic GUI architecture

Generic GUI is intentionally separated from the Generic kernel.

## Boundary

The kernel owns mechanisms:

- processes and address spaces;
- syscall entry and user-pointer validation;
- timers;
- input device drivers and normalized input packets;
- display/framebuffer ownership;
- VFS/file descriptors;
- future IPC and shared memory.

Generic GUI owns policy:

- compositor and window layout;
- focus and z-order;
- desktop/taskbar/start surface;
- widgets;
- themes;
- graphical applications;
- accessibility and interaction behavior.

The GUI never calls private kernel Rust functions. The integration boundary is
the versioned Generic userspace ABI.

## Current bridge

`generic-gui-user` is now a freestanding x86_64 ring 3 executable. The first
ABI version intentionally uses a conservative copy/present model:

    generic-gui-user XRGB8888 backbuffer
        |
        | SYS_DISPLAY_PRESENT
        v
    validated userspace range
        |
        v
    kernel display blit -> boot framebuffer

Display discovery uses `SYS_DISPLAY_INFO`. Input uses `SYS_INPUT_POLL`,
which returns normalized keyboard or mouse packets. The adapter translates
those packets into `generic-gui-core::InputEvent`.

This design does not expose framebuffer MMIO as a user mapping, so a GUI bug
cannot directly write arbitrary privileged mappings. It is deliberately a
bootstrap transport; shared surfaces and damage tracking replace the full-frame
copy later.

## Current core

`generic-gui-core` is `no_std` and only requires `alloc`. It exposes a
renderer trait and consumes normalized input events. The window model covers
focus, z-order, dragging, minimize/close, taskbar restore and
platform-independent rendering.

## Target runtime

    Generic kernel
        |
        | syscalls / IPC / shared memory
        v
    generic-gui display server
        |
        | window protocol
        v
    Generic applications

Applications should eventually render into shared surfaces. The display server
composites those surfaces and owns focus/input routing. Applications never
receive direct kernel framebuffer access.

## Migration rule

Code belongs in the kernel only when it implements a privileged mechanism.
Desktop appearance or window behavior belongs here, even if an early prototype
once ran in kernel mode.
