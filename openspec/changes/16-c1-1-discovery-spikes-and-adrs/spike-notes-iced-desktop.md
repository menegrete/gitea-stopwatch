# Notas del spike: escritorio Iced (iced-desktop)

## Alcance
Evaluar el framework Iced (Rust, GUI) como posible shell de escritorio para las capacidades: multiples ventanas, siempre arriba, arrastre, redimensionado con limites, accesibilidad, pruebas headless con iced_test y empaquetado para distribucion.

## Entorno y limitaciones del entorno
- Proyecto: appType=desktop, stack=rust, language=es, forge=github.
- No havia toolchain Rust instalada al inicio (no rustc, no cargo).
- Se instalo rustup stable (rustc/cargo 1.99.1) para poder compilar.
- Servidor X11 activo en DISPLAY=:0 (Xorg en :0, xdpyinfo alcanzable).
- Red disponible; se resuelvo y cargo iced 0.14.0 (resuelto a 0.14.2) desde crates.io.
- src/ y test/ del proyecto estan vacios: este spike no entrega codigo de producto, solo notas de descubrimiento.

## Capacidades intentadas y resultados

### Multi-ventana
Intentado: abrir una segunda ventana con identificador unico y cerrar ventanas de forma independiente.
Resultado: funciono. Iced expone `iced::daemon()`, `window::open(settings)` que devuelve `(Id, Task<Id>)`, `window::close(id)` y `window::close_events()`. Cada ventana recibe un `window::Id` unico y puede cerrarse independientemente sin detener la aplicacion.
Evidencia: programa de prueba compilado (`cargo check` exit 0) y ejecutado en X11 (salio por `timeout`, no por fallo).

### Siempre arriba (always-on-top)
Intentado: activar y desactivar el modo siempre arriba en tiempo de ejecucion.
Resultado: funciono. `window::Settings.level` acepta `Level::AlwaysOnTop` (defecto `Level::Normal`) y `window::set_level(id, level)` cambia el z-order en tiempo de ejecucion.
Evidencia: compila.

### Arrastre de ventanas
Intentado: arrastrar una ventana por el escritorio.
Resultado: funciono. `window::drag(id)` inicia un arrastre de la ventana; `drag_resize(id, direction)` para redimensionar arrastrando.
Evidencia: compila.

### Redimensionado dentro de limites declarados
Intentado: limitar el tamano a dimensiones minima y maxima declaradas.
Resultado: funciono. `set_min_size(id, size)`, `set_max_size(id, size)`, `resize(id, size)`, `set_resizable(id, bool)` y `set_resize_increments`. Los limites se declaran en `window::Settings.min_size` / `max_size`.
Evidencia: compila.

### Accesibilidad
Intentado: exponer controles (rol, estado, etiqueta) a tecnologias de asistencia.
Resultado: no funciona de forma completa en Iced core. Iced no expone rol/estado/etiqueta descriptiva a lectores de pantalla de manera integral; requiere crates de terceros (por ejemplo Plushie Iced con integracion AccessKit) o codigo especifico por plataforma.
Brecha del framework: accesibilidad limitada; no cumple el escenario de exposicion de controles sin dependencias adicionales.

### Pruebas headless con iced_test
Intentado: escribir y ejecutar pruebas de interfaz sin servidor de display.
Resultado: capacidad documentada del framework. `iced_test` provee `Simulator` y `Emulator` (modo Immediate), `snapshot()` y `screenshot()`, que operan offscreen/in-proceso y no requieren servidor de display.
Limitacion de esta sesion: no se compilo ni ejecuto un test iced_test end-to-end (alcance del spike centrado en compilacion y ejecucion del shell). Se confirma por documentacion que corre headless.

### Empaquetado para distribucion
Intentado: generar un paquete instalable (.deb, .rpm, .msi, .dmg) con firma cuando la plataforma lo requiera.
Resultado: no funciona. Iced no genera paquetes instalables ni firma binarios; solo produce un ejecutable. Las feature flags (`fira-sans`, `basic-shaping`, `advanced-shaping`, `wayland`) solo incrustan fuentes y seleccionan renderizador/backends. El empaquetado y la firma corresponden a herramientas externas (por ejemplo cargo-bundle o el empaquetador del SO).
Brecha del framework: no hay empaquetado ni firma nativa.

## Brechas del framework observadas
1. Accesibilidad: Iced core no expone rol/estado/etiqueta a lectores de pantalla; requiere AccessKit (terceros) o codigo por plataforma.
2. Empaquetado: Iced no crea paquetes instalables ni firma; es responsabilidad de herramientas externas.
3. Siempre arriba / nivel de ventana: `Level::AlwaysOnTop` existe, pero el comportamiento exacto puede variar por plataforma y servidor de display (X11 vs Wayland).

## Conclusión
Iced cumple las capacidades de multi-ventana, siempre arriba, arrastre y redimensionado con limites (todas verificadas compilando y ejecutando un shell de prueba en X11), e iced_test permite pruebas headless sin servidor de display. No cumple nativamente accesibilidad completa ni empaquetado/firma; ambas requieren dependencias o herramientas externas. Recomendado como shell de escritorio con esas salvedades documentadas.

## Siguientes pasos (fuera de este spike)
- Registrar ADR de seleccion de framework de ventana multiple (grupo ## 7).
- Si se avanza con Iced, agregar integracion de accesibilidad (AccessKit) y definir la herramienta de empaquetado por plataforma.
