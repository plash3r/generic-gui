# Generic GUI architecture

Generic GUI is intentionally separated from the Generic kernel.

## Boundary

The kernel owns mechanisms:

- processes and address spaces;
- syscall entry and user-pointer validation;
- IPC and shared memory;
- timers;
- input device drivers and normalized input events;
- display/framebuffer handoff;
- VFS/file descriptors.

Generic GUI owns policy:

- compositor and window layout;
- focus and z-order;
- desktop/taskbar/start surface;
- widgets;
- themes;
- graphical applications;
- accessibility and interaction behavior.

The GUI must not call private kernel Rust functions. The integration boundary is
a stable userspace ABI.

## Current core

`generic-gui-core` is `no_std` and only requires `alloc`. It exposes a
renderer trait and consumes normalized input events. This keeps the compositor
logic testable before Generic has ring 3.

The current window model already covers focus, z-order, dragging,
minimize/close, taskbar restore and platform-independent rendering.

## Target runtime

The intended long-term stack is:

    Generic kernel
        |
        | syscalls / IPC / shared memory
        v
    generic-gui display server
        |
        | window protocol
        v
    Generic applications

Applications should render into shared surfaces. The display server composites
those surfaces and owns focus/input routing. Applications never receive direct
kernel framebuffer access.

## Migration rule

Code belongs in the kernel only when it implements a privileged mechanism.
Desktop appearance or window behavior belongs here, even if an early prototype
once ran in kernel mode.
