# Generic GUI

Графическая оболочка для ядра Generic.

`generic-gui` — отдельный проект графической среды Generic OS. Он не
является частью ядра и не должен зависеть от внутренних модулей
`generic-kernel`.

## Текущая архитектура

Репозиторий начинается с `generic-gui-core` — `no_std` библиотеки, которая
содержит платформенно-независимые основы оконной системы:

- геометрию `Point` / `Rect`;
- события клавиатуры и мыши;
- абстракцию `Renderer`;
- модель desktop/window manager;
- z-order и focus;
- drag окон;
- minimize/close;
- taskbar;
- базовый software rendering policy.

GUI core не обращается напрямую к APIC, PS/2, framebuffer, page tables или VFS
ядра. Эти механизмы будут приходить через публичный Generic userspace ABI.

## Почему отдельный репозиторий

`generic-kernel` остаётся ядром: память, interrupts, timer, input, VFS,
storage, процессы, syscalls и IPC. Этот репозиторий отвечает за desktop,
compositor, widgets и приложения.

До появления ring 3/syscalls/IPC `generic-gui-core` тестируется отдельно на host.
После появления userspace ABI поверх него будет добавлен Generic display-server
adapter.

## Проверка

    cargo test --workspace
    cargo fmt --all -- --check

Подробности: `docs/ARCHITECTURE.md` и `docs/ROADMAP.md`.
