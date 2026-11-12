# Spike de la API de Gitea - Notas

## Contexto

- Instancia Gitea: https://git.nsoft.com.ar (Gitea 28.3.0, OpenAPI 3.0.3)
- PAT: leído de `.env` (no se registra aquí por seguridad)
- Repositorio objetivo original: `NSoft/Test` (no existe, 404)
- Repositorio usado: `NSoft/tests` (id:60, issues y time tracking habilitados)
- Issue usado para pruebas: #1 (id:488, título "test issue", estado open)

## 5.1 - Autenticación, GET /user y obtención de issues

### Autenticación con PAT

- **GET /api/v1/user** con `Authorization: token <PAT>` → **200 OK**
  - Respuesta: usuario `mnegrete`, id 1, tipo User, `is_admin: true`, email `martin.negrete@gmail.com`
  - Conclusión: el PAT autentica correctamente y el encabezado `Authorization: token <PAT>` es el formato esperado.

- **GET /api/v1/user** sin PAT → **401 Unauthorized**
  - Respuesta: `{"message":"token is required","url":"http://git.nsoft.com.ar/api/swagger"}`
  - Conclusión: el cliente debe rechazar solicitudes sin PAT configurado antes de enviarlas (requisito del spec).

### Obtención de issues

- **GET /api/v1/repos/NSoft/tests/issues?state=all** → **200 OK**
  - Devuelve 1 issue: id 488, número 1, título "test issue", estado "open", 0 comentarios, `time_estimate: 0`
  - Los campos `id`, `number` (identificador), `title` y `state` están presentes, cumpliendo el requisito de devolver al menos identificador, título y estado.

## 5.2 - POST /times, POST /comments, mapeo de errores y unidad timetracking

### POST /times (registro de tiempo invertido)

- **POST /api/v1/repos/NSoft/tests/issues/1/times** con body `{"time":3600}` → **200 OK**
  - Respuesta: entrada de tiempo id 136, `time: 3600` segundos, `user_id: 1`, `user_name: "mnegrete"`, `issue_id: 488`
  - Conclusión: el endpoint acepta el registro y devuelve la entrada creada con todos los campos relevantes.

### POST /comments (registro de comentarios en issues)

- **POST /api/v1/repos/NSoft/tests/issues/1/comments** con body `{"body":"Comentario de spike..."}` → **201 Created**
  - Respuesta: comentario id 4265, `html_url` con anchor `#issuecomment-4265`, body del comentario devuelto
  - Conclusión: el endpoint crea el comentario y devuelve el comentario creado con su identificador.

### Unidad timetracking (consulta de tiempo acumulado)

- **GET /api/v1/repos/NSoft/tests/issues/1/times** → **200 OK**
  - Devuelve lista con 1 entrada: id 136, `time: 3600` segundos
  - `x-total-count: 1` confirma el acumulado
  - Conclusión: la unidad de control de tiempo mantiene el tiempo por issue y permite consultar el total registrado.

### Mapeo de errores de la API

| Código HTTP | Endpoint | Respuesta | Mapeo de dominio |
|-------------|----------|-----------|------------------|
| 401 | GET /user (sin PAT) | `{"message":"token is required"}` | Error de credenciales inválidas o faltantes |
| 404 | GET /repos/NSoft/Test | `{"message":"not found",...}` | Recurso no encontrado (repo/issue inexistente) |

- 401 distingue errores de autenticación (credenciales inválidas o expiradas) — cumple el spec.
- 404 para recurso no encontrado — aplicable a issues o repos que no existen.
- No se pudo provocar un 500 en esta instancia (error de servidor no observable en el entorno de pruebas).

### Límites de API (rate limits)

- Ninguna respuesta incluyó encabezados de rate limit (por ejemplo `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`).
- Conclusión: el rate limiting no está habilitado o no se expone en esta instancia Gitea 28.3.0. El cliente no debe depender de estos encabezados para esta instancia.

## Brechas observadas frente al spec

1. El repositorio `NSoft/Test` declarado en el source-issue no existe en la instancia; se usó `NSoft/tests` como sustituto funcional equivalente.
2. No se observaron encabezados de rate limit, por lo que el cliente no puede confiar en ellos para esta instancia.
3. No se pudo observar un error 500 (servidor) para validar el mapeo de error de servidor con reintento.

## Conclusión

Todos los endpoints requeridos por el spec gitea-api funcionan correctamente: autenticación PAT, GET /user, obtención de issues, POST /times, POST /comments, y consulta de tiempo acumulado. El mapeo de errores 401 y 404 se verificó. El rate limiting no está expuesto en esta instancia.