# ADR-04: Almacenamiento de credenciales cifrado

**Estado**: Aceptada  
**Fecha**: 2026-11-12  
**Identificador**: ADR-04

## Contexto

Las credenciales de la API de Gitea (PAT) deben protegerse en reposo según la postura de seguridad de escritorio. El almacén de credenciales difiere por plataforma: macOS Keychain, Windows Credential Manager, Linux Secret Service (D4, diseño).

## Decisión

Almacenar las credenciales de acceso a la API de Gitea de forma cifrada en el almacén de credenciales del sistema operativo. Abstraer el almacén detrás de una interfaz con implementaciones por plataforma. Recuperar las credenciales solo para el usuario autenticado y nunca exponerlas en texto plano.

## Consecuencias

Positivas: credenciales protegidas en reposo; nunca expuestas en texto plano; recuperación restringida al usuario autenticado.

Negativas: complejidad de la abstracción multiplataforma para el almacén de credenciales.
