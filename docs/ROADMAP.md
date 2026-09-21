# Generic GUI roadmap

1. GUI core — working:
   - no_std platform-independent geometry and input events;
   - Renderer abstraction;
   - window model, z-order, focus and dragging;
   - minimize/close and taskbar restore;
   - host tests.

2. Generic userspace adapter — first bridge working:
   - freestanding ring 3 ELF;
   - Generic ABI syscall wrappers;
   - display discovery and full-frame XRGB8888 present;
   - normalized keyboard/mouse packet translation;
   - interactive desktop/window model in userspace.

3. Display transport — next:
   - shared-memory surfaces;
   - damage/dirty rectangles;
   - IPC window protocol;
   - buffer lifecycle and synchronization;
   - blocking input/event wait instead of polling.

4. Desktop:
   - compositor;
   - panel/taskbar/start UI;
   - terminal window;
   - file manager;
   - settings application.

5. Toolkit:
   - labels/buttons/text fields;
   - lists/menus/scrolling;
   - keyboard focus;
   - clipboard;
   - theme API.

6. Hardware/productization:
   - resolution changes;
   - USB HID through kernel input API;
   - accelerated display backend when the kernel exposes one;
   - crash isolation and display-server restart;
   - accessibility and localization.

The GUI repository remains buildable independently from the kernel source tree.
Integration is performed only through versioned Generic ABI contracts.
