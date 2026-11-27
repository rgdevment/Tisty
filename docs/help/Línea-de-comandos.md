# Línea de comandos

Desde la 1.31, Tisty no tiene línea de comandos. La ventana es la forma de entrar, para tus tareas y para tus documentos.

Del programa `tisty` queda lo que usa tu agente: **`tisty mcp`**, la puerta por la que entra, y **`tisty hook`**, al que llaman los hooks que Tisty le agrega al empezar una sesión, antes y después de algunas herramientas y al terminar un turno (mira [[Agentes y MCP|Agentes-y-MCP]]). `tisty hook` lee lo que manda el agente, responde en la forma que ese agente entiende y siempre termina sin error, así que nunca detiene un turno. Queda también **`tisty run-hook`**, la única línea con la que un agente corre un hook que creaste y registraste en Agentes. Igual que `tisty hook`, nunca hace fallar el turno: solo tu script, si termina con 2 antes de una herramienta, la detiene. Cualquier otro comando solo muestra un aviso de que la línea de comandos se retiró y te manda a la ventana.

## Dónde vive

Viene dentro de la app: no se instala aparte y no hace falta tenerlo en el PATH. Desde la 1.32.1 también responde a `tisty` a secas mientras la puerta de los agentes está abierta (mira [[Agentes y MCP|Agentes-y-MCP]]). **Configuración › Agentes › Conectados** muestra dónde está en este equipo, y conecta los agentes que encuentra instalados.

## Conectar un agente con él

Para un agente que Tisty no conoce, basta una línea, con la ruta completa que muestra **Configuración › Agentes › Conectados › Cómo conectar uno**, lista para copiar:

```console
$ <tu-agente> mcp add tisty -- <ruta-a-tisty> mcp
```

Donde `<tu-agente>` es como se llame el tuyo. Si cerraste la puerta, vuelve a abrirla en **Configuración › Agentes › Puerta**. Lo demás está en [[Agentes y MCP|Agentes-y-MCP]].

## Si usabas la línea de comandos

- Lo que hacía de mantenimiento vive en la ventana: **Sincronizar ahora** y **Volver a enviar todo** en **Configuración › Sincronización**, confirmar otro equipo desde su aviso o en **Configuración › Seguridad › Tus equipos**, la revisión del almacén en **Configuración › Salud**, y el respaldo en zip en **Configuración › Copias**.
- Los scripts que llamaban a `tisty add` o `tisty ls --json` dejan de funcionar. La [[puerta MCP|Agentes-y-MCP]] es como un programa llega a Tisty ahora.
- La fórmula de Homebrew `tisty-cli` y los zip sueltos de la línea de comandos ya no se publican. Si es lo único que instalaste, se queda en la 1.30: instala la app en su lugar ([[Instalación]]), y encuentra tus tareas y documentos donde están.
