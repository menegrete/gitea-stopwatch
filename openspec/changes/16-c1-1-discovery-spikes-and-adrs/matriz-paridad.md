# Matriz de paridad del puerto

## Alcance congelado

El alcance del puerto está congelado a partir de la aprobación de esta matriz. Ninguna capacidad puede cambiar de clasificación sin una revisión explícita y una nueva aprobación documentada según el procedimiento definido en esta sección.

## Clasificación de capacidades

| Capacidad | Clasificación | Justificación | Origen (spike/ADR) |
|---|---|---|---|
| Autenticación con PAT (Authorization: token <PAT>) | portada | El spike gitea-api validó la autenticación con PAT en cada solicitud y el rechazo de solicitudes sin PAT configurado. | spike gitea-api |
| Consulta del usuario autenticado (GET /user) | portada | El spike gitea-api ejecutó la consulta GET /user y confirmó la devolución del perfil con identificador, nombre de usuario y correo. | spike gitea-api |
| Obtención de issues de un repositorio | portada | El spike gitea-api obtuvo la lista de issues filtrada por repositorio con identificador, título y estado. | spike gitea-api |
| Registro de tiempo invertido (POST /times) | portada | El spike gitea-api validó el registro de tiempo mediante POST /times con identificador de issue y cantidad de tiempo. | spike gitea-api |
| Registro de comentarios en issues (POST /comments) | portada | El spike gitea-api publicó comentarios mediante POST /comments con identificador de issue y cuerpo del comentario. | spike gitea-api |
| Mapeo de errores de la API (401, 500) | portada | El spike gitea-api mapeó códigos HTTP a errores de dominio: autenticación, cliente y servidor con reintento. | spike gitea-api |
| Unidad de control de tiempo (timetracking) | portada | El spike gitea-api expuso una unidad de control de tiempo que acumula tiempo por issue y permite consultar el total. | spike gitea-api |
| Gestión de múltiples ventanas | portada | El spike iced-desktop abrió y mantuvo varias ventanas simultáneamente con identificador único y cierre independiente. | spike iced-desktop |
| Siempre arriba (always-on-top) | portada | El spike iced-desktop activó y desactivó el modo siempre arriba en tiempo de ejecución. | spike iced-desktop |
| Arrastre y redimensionado de ventanas | portada | El spike iced-desktop permitió arrastrar ventanas y redimensionar dentro de límites mínimos y máximos declarados. | spike iced-desktop |
| Accesibilidad | portada | El spike iced-desktop expuso controles a tecnologías de asistencia con rol, estado y etiqueta descriptiva. | spike iced-desktop |
| Pruebas con iced_test headless | portada | El spike iced-desktop ejecutó pruebas de interfaz con iced_test en entorno sin interfaz gráfica y reportó resultados. | spike iced-desktop |
| Empaquetado para distribución | adaptada | El spike iced-desktop generó paquetes instalables por plataforma, pero cada plataforma requiere su propio proceso de firma y recursos, por lo que el empaquetado se adapta por plataforma objetivo. | spike iced-desktop |
| Identidad de issue como owner/repo#N | portada | El ADR de identidad de issue estableció el formato owner/repo#N como identificador inequívoco y el rechazo de formatos inválidos. | ADR de identidad de issue |
| Almacenamiento de credenciales cifrado | portada | El ADR de almacenamiento de credenciales definió el cifrado en el almacén del sistema operativo y la recuperación solo para el usuario autenticado. | ADR de almacenamiento de credenciales |
| Mapeo entre tiempo y comentarios | portada | El ADR de mapeo tiempo/comentario asoció cada entrada de tiempo con el comentario de issue correspondiente y permitió consultar el tiempo vinculado. | ADR de mapeo tiempo/comentario |
| Selección de framework HTTP y runtime | adaptada | El ADR de selección de framework dejó abierta la elección del crate HTTP (reqwest u otra) y el runtime, pendiente de confirmación en el spike gitea-api. | ADR de selección de framework |

## Resumen de clasificación

- Portada: 15 capacidades
- Adaptada: 2 capacidades
- Descartada: 0 capacidades

## Congelamiento del alcance del puerto

El alcance del puerto queda congelado tras la aprobación de esta matriz. La clasificación de cada capacidad no puede modificarse sin una nueva aprobación documentada. Cualquier intento de cambiar la clasificación sin aprobación será rechazado y se conservará la clasificación anterior.

## Procedimiento de cambio con aprobación documentada

Cuando se requiera modificar la clasificación de una capacidad en alcance congelado, se debe seguir el siguiente procedimiento:

1. Solicitud de cambio: quien proponga el cambio documenta la capacidad afectada, la clasificación actual, la clasificación solicitada y la justificación del cambio.
2. Revisión técnica: un revisor técnico evalúa el impacto del cambio en el alcance del puerto y los spikes o ADR relacionados.
3. Aprobación documentada: el cambio solo se aplica si existe un registro de aprobación documentada que incluya fecha, aprobador, capacidad afectada, clasificación anterior y nueva clasificación.
4. Actualización de la matriz: una vez aprobado, la matriz se actualiza con la nueva clasificación y se registra la aprobación en el historial de cambios.

Sin una nueva aprobación documentada, el alcance permanece inalterado y se conserva la clasificación vigente.

## Trazabilidad

Cada clasificación de la matriz mantiene trazabilidad al spike o ADR que fundamenta la decisión, indicado en la columna Origen (spike/ADR). La matriz permite consultar, para cada capacidad, el origen de su clasificación.
