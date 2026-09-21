# Generic GUI

Графическая оболочка для ядра Generic.

`generic-gui` — отдельный проект графической среды Generic OS. Он не
является частью ядра и не зависит от внутренних модулей `generic-kernel`.

## Текущая архитектура

Репозиторий содержит два слоя:

- `generic-gui-core` — `no_std` платформенно-независимая модель desktop/window manager;
- `generic-gui-user` — freestanding ring 3 display-server prototype для Generic ABI.

GUI core содержит:

- геометрию `Point` / `Rect`;
- события клавиатуры и мыши;
- абстракцию `Renderer`;
- модель desktop/window manager;
- z-order и focus;
- drag окон;
- minimize/close;
- taskbar;
- базовый software rendering policy.

Userspace adapter реализует `Renderer` поверх Generic display ABI, держит
XRGB8888 backbuffer в памяти процесса, передаёт кадр ядру через
`display_present` и преобразует нормализованные kernel input packets в
`InputEvent`.

GUI не обращается напрямую к APIC, PS/2, framebuffer MMIO, page tables или VFS
ядра. Все привилегированные операции проходят через публичный Generic userspace
ABI.

## Проверка

    cargo test --workspace
    cargo fmt --all -- --check
    cargo build -p generic-gui-user --target x86_64-unknown-none --release

Freestanding ELF появляется здесь:

    target/x86_64-unknown-none/release/generic-gui-user

При сборке Generic kernel соседний checkout `../generic-gui` может быть
автоматически собран и добавлен в initramfs как `/bin/generic-gui`.

Подробности: `docs/ARCHITECTURE.md` и `docs/ROADMAP.md`.
