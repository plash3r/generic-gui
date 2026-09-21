# Generic GUI roadmap

1. GUI core — working now:
   - no_std platform-independent geometry and input events;
   - Renderer abstraction;
   - window model, z-order, focus and dragging;
   - minimize/close and taskbar restore;
   - host tests.

2. Generic userspace adapter — blocked on kernel process infrastructure:
   - ring 3 executable;
   - stable syscall ABI;
   - normalized keyboard/mouse event stream;
   - monotonic timers.

3. Display transport:
   - shared-memory surfaces;
   - damage/dirty rectangles;
   - IPC window protocol;
   - buffer lifecycle and synchronization.

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

The GUI repository must remain buildable independently from the kernel source
tree. Integration is performed through versioned Generic ABI contracts.
