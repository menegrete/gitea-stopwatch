# ADR-01: Selección de framework (crate HTTP, ventana múltiple)

**Estado**: Aceptada  
**Fecha**: 2026-11-12  
**Identificador**: ADR-01

## Contexto

El puerto del cronómetro al stack de escritorio requiere un cliente HTTP para la API de Gitea y un framework de escritorio que soporte ventanas múltiples. El spike de gitea-api registró los límites de API y el spike de iced-desktop registró las brechas del framework. La decisión de librería HTTP y runtime se deja al spike (D1, diseño).

## Decisión

Usar `reqwest` como crate HTTP para el cliente de la API de Gitea y `iced` como framework de escritorio para ventanas múltiples, siempre arriba, arrastre y redimensionado. Los spikes se ejecutan como investigaciones acotadas en el tiempo con notas registradas, no como código de producción.

## Consecuencias

Positivas: `reqwest` es un crate HTTP maduro y ampliamente usado en Rust; `iced` soporta el modelo de ventanas múltiples necesario para la aplicación de escritorio. Los spikes como investigaciones desechables evitan implementación prematura de código de producción.

Negativas: Iced puede no soportar ventanas múltiples de forma estable en la versión objetivo (riesgo mitigado con fallback a ventana única según el plan de migración del diseño).
