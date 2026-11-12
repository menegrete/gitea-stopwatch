# iced-desktop Specification

## Purpose
Esta especificación define el comportamiento del shell de escritorio basado en Iced, que gestiona múltiples ventanas, se mantiene siempre arriba, permite arrastrar y redimensionar, ofrece accesibilidad, soporta pruebas con iced_test y se empaqueta para distribución.

## Requirements

### Requirement: Gestión de múltiples ventanas
El shell SHALL abrir y mantener varias ventanas de la aplicación simultáneamente. El shell MUST asignar un identificador único a cada ventana y permitir cerrar cada ventana de forma independiente.

#### Scenario: Apertura de una segunda ventana
- **WHEN** el usuario solicita abrir una nueva ventana mientras ya existe una ventana principal
- **THEN** el shell crea una segunda ventana con identificador distinto y ambas permanecen activas

#### Scenario: Cierre independiente de una ventana
- **WHEN** se cierra una ventana secundaria mientras la ventana principal sigue abierta
- **THEN** solo la ventana solicitada se cierra y la aplicación sigue ejecutándose

### Requirement: Siempre arriba (always-on-top)
El shell SHALL mantener las ventanas de la aplicación por encima de otras ventanas del escritorio cuando el modo siempre arriba está activado. El shell MUST permitir activar y desactivar este modo en tiempo de ejecución.

#### Scenario: Activación del modo siempre arriba
- **WHEN** el usuario activa el modo siempre arriba en una ventana
- **THEN** esa ventana se mantiene por encima de las demás ventanas del escritorio

#### Scenario: Desactivación del modo siempre arriba
- **WHEN** el usuario desactiva el modo siempre arriba en una ventana
- **THEN** esa ventana permite que otras ventanas del escritorio se superpongan sobre ella

### Requirement: Arrastre y redimensionado de ventanas
El shell SHALL permitir al usuario arrastrar una ventana por el escritorio y redimensionarla mediante los bordes o esquinas. El shell MUST limitar el redimensionado a las dimensiones mínima y máxima declaradas.

#### Scenario: Arrastre de una ventana por el escritorio
- **WHEN** el usuario arrastra la barra de título de una ventana a otra posición de la pantalla
- **THEN** la ventana se mueve junto al puntero sin perder su contenido ni estado

#### Scenario: Redimensión dentro de límites declarados
- **WHEN** el usuario redimensiona una ventana más allá de su dimensión máxima declarada
- **THEN** el shell limita el tamaño al valor máximo declarado y no permite excederlo

### Requirement: Accesibilidad
El shell SHALL exponer los elementos de la interfaz a las tecnologías de asistencia del sistema operativo. El cliente MUST proporcionar etiquetas descriptivas para los controles interactivos.

#### Scenario: Exposición de controles a tecnologías de asistencia
- **WHEN** un lector de pantalla consulta los controles de una ventana de la aplicación
- **THEN** el shell expone cada control con su rol, estado y etiqueta descriptiva

### Requirement: Pruebas con iced_test
El shell SHALL permitir escribir pruebas de interfaz utilizando el harness de pruebas iced_test. El shell MUST ejecutar esas pruebas en entorno sin interfaz gráfica (headless) y reportar el resultado de cada prueba.

#### Scenario: Ejecución de pruebas de interfaz en modo headless
- **WHEN** se ejecutan las pruebas de interfaz escritas con iced_test en entorno headless
- **THEN** el harness completa la ejecución y reporta éxito o fallo por prueba

### Requirement: Empaquetado para distribución
El shell SHALL generar un paquete instalable de la aplicación de escritorio para cada plataforma objetivo. El shell MUST incluir todos los recursos necesarios en el paquete y firmarlo cuando la plataforma lo requiera.

#### Scenario: Generación del paquete instalable
- **WHEN** se solicita el empaquetado de la aplicación para una plataforma objetivo
- **THEN** el shell produce un paquete instalable que contiene la aplicación y sus recursos
