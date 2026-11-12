# ADR-03: Mapeo tiempo/comentario

**Estado**: Aceptada  
**Fecha**: 2026-11-12  
**Identificador**: ADR-03

## Contexto

Cada entrada de tiempo registrada debe asociarse con el comentario de issue correspondiente, y debe ser posible consultar todo el tiempo vinculado a un comentario específico.

## Decisión

Asociar cada entrada de tiempo con el comentario de issue identificado como owner/repo#N. Permitir consulta de todo el tiempo vinculado a un comentario y devolver la suma de todas las entradas de tiempo asociadas a ese comentario.

## Consecuencias

Positivas: trazabilidad entre tiempo y comentarios de issue; permite consultar el tiempo total vinculado a un comentario de issue específico.

Negativas: requiere validar el formato owner/repo#N al asociar una entrada de tiempo con un comentario.
