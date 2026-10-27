# Asistentes y MCP

**Tisty no es IA y no tiene IA dentro.** El lenguaje natural que convierte «llamar al banco a las 3» en una tarea son reglas que corren en tu equipo: sin modelo, sin petición y sin nube. Eso no va a cambiar.

Pero si ya usas un asistente, seguramente le cuentas cosas que vale la pena guardar: el grupo del colegio pide cartulina para el lunes, la factura vence el 30, el bug que acabas de arreglar y cómo lo hiciste. Por eso Tisty deja una puerta, y tú decides si usarla.

**El trabajo de tus agentes, a la vista.** Con Tisty, un agente planifica en documentos y trabaja en tareas: escribe en Markdown el plan, lo que encontró y lo que decidió, divide el trabajo en tareas con pasos y los va marcando a medida que avanza. Nada queda enterrado en un chat: cada etapa queda en Tisty, en tu equipo, donde puedes seguirla mientras ocurre y auditarla después. Cuando dice que una tarea está lista, eres tú quien la cierra.

![Trae a tus agentes favoritos por MCP: organizan, documentan, agendan y priorizan mientras todo se queda en tu equipo](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/store/es-07-mcp.png)

## Cómo funciona la puerta

Tisty habla [MCP](https://modelcontextprotocol.io) a través de `tisty mcp`, por su propia entrada y salida estándar. Ese `tisty` viene dentro de la app y ya no es una línea de comandos: solo abre la puerta del asistente (mira [[Línea de comandos|Línea-de-comandos]]).

- **Solo en este equipo.** Sin puerto, sin cuenta y sin token. No es un servidor, y nada puede alcanzarlo desde la red.
- **Nada escucha hasta que lo conectas.** El asistente arranca `tisty mcp` cuando lo necesita.
- **Tú manejas la puerta.** En una instalación nueva viene abierta, y **Configuración › Asistentes** la cierra con un clic. Mientras está cerrada, todo lo que intente un asistente se rechaza. Abrirla y cerrarla queda escrito en tu registro.

Desde la 1.32.1, mientras la puerta está abierta, Tisty también responde a `tisty` a secas desde cualquier terminal: la copia del instalador en Windows y la de Homebrew en Mac. Si tienes a la vez la copia de Microsoft Store y la del instalador, responde la más nueva. Cierra la puerta y `tisty` deja de responder. Un asistente que ya estaba abierto necesita reiniciarse para notarlo.

## Conectar un asistente

### Desde Configuración (lo más fácil)

**Configuración › Asistentes** busca los asistentes instalados en este equipo entre los que Tisty conoce (Claude Code, Claude Desktop, Codex, GitHub Copilot CLI, VS Code, Cursor, Devin Desktop y Windsurf, Antigravity, Gemini CLI, Qwen Code, OpenCode, Kilo Code, Zed, Junie, Kiro, Warp, LM Studio, Amp, Augment, Factory Droid, Trae, Cline, Roo Code, BoltAI, Goose y Continue) y dice cómo está cada uno:

- **Conectado**: abre este Tisty.
- **Sin conectar**: **Conectar** escribe Tisty en su configuración, con la forma que ese asistente lee.
- **Apunta a otro Tisty**: una copia vieja o que se movió. **Corregir** lo apunta aquí.
- **Puede que esté**: solo quedó su carpeta, así que quizás se desinstaló.
- **A mano**: Goose y Continue guardan su configuración en YAML, que Tisty no escribe. La fila muestra qué pegar.

**Conectar los detectados** conecta de una vez todos los que esperan. Algunos archivos sirven a más de un asistente: el de GitHub Copilot CLI también lo lee VS Code, y el de Codex lo leen ChatGPT y Warp.

Un asistente lee su configuración al abrirse, así que después de conectarlo sal de él del todo y ábrelo otra vez. Si tiene ventana, cerrarla no basta: en Windows sigue abierto junto al reloj hasta que sales desde ahí, y en un Mac hasta **⌘Q**.

Tisty solo toca su propia entrada. Lo que le agregaste, como `env` o las herramientas que permitiste, se queda, igual que el resto del archivo. La primera versión de cada archivo queda al lado como `.before-tisty`, y la de justo antes de cada escritura como `.before-tisty-last`.

El **⋯** de cada fila muestra cómo conectar ese asistente a mano: su propio comando cuando lo tiene, y las líneas exactas para su archivo. El buscador bajo la lista hace lo mismo con un asistente que Tisty conoce pero no encontró.

### Con un plugin

Algunos asistentes toman Tisty como plugin. El plugin trae un pequeño lanzador que busca el Tisty instalado en este equipo (la copia de Microsoft Store, la del instalador, la app de Mac o Homebrew), usa la más nueva si hay varias y arranca el `tisty mcp` de esa copia, así lo que escribe el asistente siempre coincide con tu app y tus datos. Si Tisty no está instalado, el asistente recibe una sola herramienta, `install_tisty`, que te dice cómo conseguirlo. El lanzador necesita Node.js.

En Claude Code:

```console
/plugin install tisty --marketplace rgdevment/mcp-plugins
```

Cualquier otro asistente que arranque un servidor con un comando, como Cursor, Cline o VS Code, toma el mismo lanzador desde npm:

```json
{
  "mcpServers": {
    "tisty": { "command": "npx", "args": ["-y", "tisty-mcp"] }
  }
}
```

La primera vez descarga el lanzador y puede tardar un momento; después arranca al instante. Si Tisty está en un lugar poco común, pon en `TISTY_PATH` la ruta completa de su `tisty`.

Conecta cada asistente de una sola forma, con el plugin o desde **Configuración › Asistentes**, no con las dos: si no, arranca dos veces el servidor de Tisty y ve cada herramienta repetida. El código del plugin y sus issues están en [mcp-plugins](https://github.com/rgdevment/mcp-plugins).

### Asistentes dentro de WSL

En Windows, un asistente que corre en una distro de Linux bajo WSL lee su configuración en Linux, donde una ruta `C:\...` no existe. **Configuración › Asistentes** muestra tus distros; **Buscar en Ubuntu** lee los archivos de esa distro (y la enciende si estaba apagada) y conecta sus asistentes al mismo Tisty, con su ruta de Linux:

```console
$ claude mcp add --scope user tisty -- /mnt/c/Users/<tú>/AppData/Local/Programs/Tisty/tisty.exe mcp
```

Es el mismo Tisty y las mismas tareas y documentos que en Windows. Si una distro tiene apagados los programas de Windows (`[interop] enabled = false` en `/etc/wsl.conf`), Tisty lo dice y no escribe nada ahí.

### A mano

Para un asistente que Tisty no conoce, basta una línea, con la ruta completa del `tisty` que viene con la app. **Configuración › Asistentes › Cómo conectar uno** muestra las dos formas de abajo con esa ruta ya puesta para este equipo, listas para copiar, así que no hace falta tener nada en el PATH:

```console
$ <tu-asistente> mcp add tisty -- <ruta-a-tisty> mcp
```

Por ejemplo, con Claude Code:

```console
$ claude mcp add tisty -- <ruta-a-tisty> mcp
```

Los clientes que se configuran con un archivo JSON usan la forma de siempre:

```json
{
  "mcpServers": {
    "tisty": { "command": "<ruta-a-tisty>", "args": ["mcp"] }
  }
}
```

Reinicia el asistente. Si cerraste la puerta, vuelve a abrirla en **Configuración › Asistentes**.

No tienes que acordarte de este paso. Cuando un asistente intenta trabajar con Tisty con la puerta cerrada, la ventana te avisa («Claude Code intentó entrar») y **Dejarlo entrar** abre la puerta ahí mismo. También te avisa cuando tus asistentes están conectados pero todavía no pueden hacer nada porque la puerta está cerrada.

## Qué puede hacer, en corto

Anotar una tarea con sus pasos y su fecha, ponerle un recordatorio, añadir a la bitácora de una tarea, escribir y editar documentos, ordenarlos en carpetas y páginas, guardar una copia de un archivo que le indiques, buscar y leer lo que hay, y decir que una tarea que anotó está hecha, **para que tú lo confirmes**.

Cuando te cuenta lo que hizo, nombra una tarea o un documento como lo harías tú: por su título y dónde está, la lista o la carpeta. Nunca te muestra un identificador.

**Nunca** puede cerrar ni borrar una tarea, borrar un documento ni mover un día que pusiste tú. En una tarea que escribiste tú solo dice lo que hizo (que un paso o la tarea entera está hecha) como su palabra, que tú tomas o no.

Las reglas completas están en [[Qué puede hacer un asistente|Qué-puede-hacer-un-asistente]], y cada herramienta en la [[referencia de herramientas|Herramientas-MCP]].

## Firmado con su nombre

Lo que escribe un asistente va firmado con el nombre del programa por el que habló («por Claude Code», «por Codex») en la fila, el detalle y la bitácora. Configuración › Asistentes cuenta lo que escribió cada uno. Lo que se escribió antes de que los programas dijeran su nombre se lee «por un asistente».

## Lee con mesura

Tisty guarda una pequeña ficha de cada documento (sus títulos, cuánto mide, en qué se apoya) que saca del propio texto, en este equipo, y que nunca envía a ninguna parte. Un asistente lee esa ficha para elegir el documento que necesita, y después pide solo la parte que quiere, o cambia esa parte sin leer el resto. Un documento largo no se vuelca en un modelo porque un asistente quería un párrafo.

Después de leer un documento, puede dejar un resumen corto para el siguiente asistente: queda en este equipo, nunca se sincroniza y no forma parte de lo que escribiste.

## Lo que conviene saber antes de abrirla

> [!WARNING]
> Lo que un asistente lee viaja adonde viaja ese asistente. Si funciona contra un servicio en la nube, lo que mira va camino a ese servicio. Tisty sigue sin enviar nada y sin servidor propio, pero esta puerta es real, y es lo único en Tisty que puede sacar tus datos del equipo. Por eso está cerrada hasta que la abres, y cerrarla es un clic.
