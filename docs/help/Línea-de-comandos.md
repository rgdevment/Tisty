# Línea de comandos

Desde la 1.31, Tisty no tiene línea de comandos. La ventana es la forma de entrar, para tus tareas y para tus documentos.

Del programa `tisty` queda una sola cosa: **`tisty mcp`**, la puerta por la que entra tu asistente. Cualquier otro comando solo muestra un aviso de que la línea de comandos se retiró y te manda a la ventana.

## Dónde vive

Viene dentro de la app: no se instala aparte y no hace falta tenerlo en el PATH. Desde la 1.32.1 también responde a `tisty` a secas mientras la puerta de los asistentes está abierta (mira [[Asistentes y MCP|Asistentes-y-MCP]]). **Configuración › Asistentes** muestra dónde está en este equipo, y conecta los asistentes que encuentra instalados.

## Conectar un asistente con él

Para un asistente que Tisty no conoce, basta una línea, con la ruta completa que muestra **Configuración › Asistentes › Cómo conectar uno**, lista para copiar:

```console
$ <tu-asistente> mcp add tisty -- <ruta-a-tisty> mcp
```

Donde `<tu-asistente>` es como se llame el tuyo. Después déjalo entrar desde **Configuración › Asistentes**. Lo demás está en [[Asistentes y MCP|Asistentes-y-MCP]].

## Si usabas la línea de comandos

- Lo que hacía de mantenimiento vive en la ventana: **Sincronizar ahora** y **Volver a enviar todo** en Configuración › Sincronizar, confirmar otro equipo desde su aviso o en **Configuración › Sincronizar › Tus equipos**, la revisión del almacén en **Configuración › Mantenimiento**, y el respaldo en zip en **Configuración › Tus datos**.
- Los scripts que llamaban a `tisty add` o `tisty ls --json` dejan de funcionar. La [[puerta MCP|Asistentes-y-MCP]] es como un programa llega a Tisty ahora.
- La fórmula de Homebrew `tisty-cli` y los zip sueltos de la línea de comandos ya no se publican. Si es lo único que instalaste, se queda en la 1.30: instala la app en su lugar ([[Instalación]]), y encuentra tus tareas y documentos donde están.
