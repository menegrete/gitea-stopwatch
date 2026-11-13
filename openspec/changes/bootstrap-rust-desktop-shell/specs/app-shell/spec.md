## Purpose
<!-- El shell de escritorio Rust con Iced 0.14.x provee el ciclo de vida de la aplicación: ventana principal vacía con título y tema, cierre limpio, bloqueo de instancia única, persistencia y restricción de la geometría de ventana a los monitores conectados, logging y hook de pánico en el borde, y navegación de marcador de posición para la futura ventana principal. -->

## ADDED Requirements

### Requirement: empty-main-window
La aplicación SHALL abrir una ventana principal vacía con título y tema definidos, y cerrar limpiamente sin dejar procesos residuales.

#### Scenario: window-opens-with-title-and-theme
- **WHEN** el usuario inicia la aplicación
- **THEN** se muestra una ventana principal vacía con el título y el tema configurados

#### Scenario: clean-shutdown
- **WHEN** el usuario cierra la ventana principal
- **THEN** la aplicación termina su proceso y libera los recursos del sistema

### Requirement: single-instance-lock
La aplicación SHALL permitir una única instancia en ejecución; un segundo lanzamiento SHALL enfocar la instancia existente y salir.

#### Scenario: second-launch-focuses-existing-instance
- **WHEN** se lanza una segunda instancia de la aplicación mientras una ya está en ejecución
- **THEN** la instancia existente recibe el foco y la nueva instancia sale sin abrir una segunda ventana

### Requirement: window-geometry-persistence
La aplicación SHALL restaurar la geometría de la ventana (tamaño y posición) desde la última sesión y SHALL ajustarla a los monitores conectados.

#### Scenario: geometry-restored-and-clamped
- **WHEN** la aplicación inicia y existe una geometría de ventana persistida de una sesión previa
- **THEN** la ventana se abre restaurada a esa geometría y ajustada a los monitores conectados

### Requirement: logging-and-panic-hook
La aplicación SHALL conectar el suscriptor de logging y el hook de pánico en el borde del proceso, y escribir los logs en el directorio de datos de usuario sin incluir secretos.

#### Scenario: logging-and-panic-hook-wired
- **WHEN** la aplicación inicia y se produce un pánico no controlado
- **THEN** el hook de pánico captura el evento y escribe el registro en el archivo de log rotativo del directorio de datos de usuario

### Requirement: placeholder-navigation
La aplicación SHALL mostrar navegación de marcador de posición reservada para la futura ventana principal.

#### Scenario: placeholder-navigation-renders
- **WHEN** se muestra la ventana principal vacía
- **THEN** se renderiza un área de navegación de marcador de posición sin funcionalidad de negocio
