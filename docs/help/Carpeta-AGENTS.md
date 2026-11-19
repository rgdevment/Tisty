# AGENTS: las reglas, skills y memoria de tus agentes

Uso más de un agente de IA, y cada uno lee sus reglas de un archivo distinto, en una carpeta distinta: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Cambiar una regla en todos era abrir cuatro archivos, y tarde o temprano uno quedaba atrás. AGENTS junta esos archivos en Tisty, como documentos, y Tisty escribe por debajo el archivo que lee cada agente.

> [!TIP]
> **En corto:** la carpeta **AGENTS** guarda como documentos las reglas, las skills y la memoria de Claude Code, Codex, GitHub Copilot CLI, Gemini CLI y Cursor. Lo que escribes en **Global** llega a todos; lo que escribes en la carpeta de un asistente, solo a él. Antes de sobrescribir un archivo, Tisty guarda lo que tenía, y un cambio hecho por fuera nunca se pisa: queda marcado hasta que tú decides.

![La primera vez de AGENTS en español: el aviso Tus agentes, en un solo lugar, con sus tres líneas sobre privacidad, sincronización y respaldo, y debajo la lista Encontré esto. Elige qué traer, con Claude · 7 archivos abierto: CLAUDE.md y dos archivos de rules marcados como Regla, la skill pr/SKILL.md marcada como Skill, una memoria de proyecto sin marcar como Memoria del agente y un .env dentro de una skill atenuado con credenciales, no entra; más abajo Codex · 2 archivos y Gemini · 1 archivo plegados, y los botones Ahora no y Traer y sincronizar](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-01-primera-vez.png)

## Qué es y cuándo te sirve

AGENTS es una carpeta de tu Biblioteca, arriba de todo, con el icono de un robot. Cada documento que tiene adentro es un archivo de un agente: una regla, una skill o su memoria. Lo editas en Tisty como cualquier documento, con su índice, su buscador y sus versiones anteriores, y Tisty escribe el archivo que ese agente lee, byte a byte.

Te sirve cuando:

- **Usas varios agentes y quieres las mismas reglas en todos.** Escribes «Responde en español» una vez en **Global**, y llega al `CLAUDE.md` de Claude Code, al `AGENTS.md` de Codex, al `copilot-instructions.md` de Copilot y a las reglas de Gemini.
- **Tus reglas crecieron y ya no las encuentras.** En Tisty se leen como documentos: con títulos, índice y búsqueda, en vez de un archivo plano en una carpeta oculta.
- **Tienes skills con scripts.** Cada archivo de la skill es un documento, y un script se guarda en un bloque de código que se escribe tal cual.
- **Quieres poder volver atrás.** Tisty guarda el archivo original antes de tocarlo por primera vez, y los últimos 20 antes de cada sobrescritura.
- **Quieres ver lo que tu agente anotó de ti.** La memoria que escribe Claude Code en cada proyecto, o el `GEMINI.md` de Gemini, se puede traer para leerla en Tisty, sin que Tisty la cambie nunca.

No te sirve para los archivos de cada repositorio: AGENTS maneja los archivos de tu usuario, los que están en tu carpeta personal, no el `AGENTS.md` o el `CLAUDE.md` que vive dentro de un proyecto.

## La primera vez

1. Abre **Configuración › Asistentes**. En la tarjeta **AGENTS**, pulsa **Crear la carpeta AGENTS**.
2. Se abre **Tus agentes, en un solo lugar**. Tisty miró qué asistentes tienes en este equipo y qué archivos hay en sus carpetas, **solo sus nombres y tamaños**: todavía no abrió ninguno.
3. Arriba, tres líneas te dicen lo que pasa desde ahora:
   - 🔒 **Queda en tu Tisty**: en tus equipos y donde tú sincronices. Nadie más lo ve.
   - ↻ **Se sincroniza solo** con los asistentes que encontró, por ejemplo «Se sincroniza solo con Claude, Codex, Gemini.».
   - ⧉ **Antes de cambiar un archivo de un agente, Tisty guarda un respaldo en este equipo.**
4. Debajo, **Encontré esto. Elige qué traer:**, una fila por asistente, como **Claude · 7 archivos**. Ábrela con un clic para ver cada archivo con su casilla y qué es: **Regla**, **Skill** o **Memoria del agente**. Pasa el puntero sobre un archivo para ver su ruta completa.
5. Marca lo que quieres traer. Las reglas y las skills vienen marcadas; **la memoria del agente viene sin marcar**, porque una vez en Tisty viaja con tu sincronización, y la marcas tú si la quieres.
6. Pulsa **Traer y sincronizar**. Si no marcaste nada, el botón dice **Autorizar y sincronizar**: autoriza este equipo sin traer archivos.
7. Tisty abre solo los archivos que marcaste, los guarda como documentos y te dice cuántos entraron: «Entraron 7 documentos a AGENTS, y desde ahora Tisty los mantiene al día.». Lo que no pudo entrar aparece bajo **Esto no entró:**, con el motivo. **Listo** cierra el aviso.

**Ahora no** cierra el aviso sin leer nada. Puedes volver cuando quieras desde la misma tarjeta de Configuración.

### Lo que dice la lista junto a un archivo

| En la lista | Qué significa |
| :-- | :-- |
| **Regla**, **Skill**, **Memoria del agente** | Se puede traer. La memoria viene sin marcar |
| **ya está en AGENTS** | Un documento de AGENTS ya lo representa, así que se compara en vez de traerlo de nuevo |
| **ajustes, no entra** | Tiene el nombre de un archivo de ajustes, como `settings.json`, `config.toml`, `hooks.json` o `mcp.json`. Dentro de una skill sí entra |
| **credenciales, no entra** | Su nombre es el de un archivo de credenciales, como `.credentials.json`, `auth.json`, `.env`, o una llave `.pem`, `.key` o `id_rsa` |
| **propio de Tisty** | Un archivo que dejó Tisty o uno temporal, como `tisty.json` o uno que termina en `.tmp` |
| **es un enlace, no entra** | Es un enlace simbólico: Tisty no lo sigue |
| **demasiado grande** | Pasa de 512 kB, lo que cabe en un documento |
| **demasiados archivos** | Tisty ya había visto 2.000 archivos en las carpetas de tus asistentes y dejó de mirar |

Lo que aparece atenuado no se puede marcar. Tisty tampoco entra en carpetas ocultas (las que empiezan con un punto) ni en `node_modules`, ni baja más de ocho niveles.

### Si un archivo no entra

Después de traer, un archivo marcado puede quedar fuera:

- **Parece llevar una credencial, como una clave o un token, así que no entra.** Tisty lo revisó en este equipo y encontró algo con forma de secreto. Saca el secreto del archivo y vuelve a intentarlo.
- **No es texto.** Un archivo binario no puede ser un documento.
- **Su nombre no se puede guardar como documento de agente.** Un nombre con `#`, `:`, `,`, comillas, corchetes o una barra invertida no cabe en la línea que dice dónde va el archivo.

### Si todavía no tienes archivos de agentes

El aviso dice «No encontré archivos de agentes en este equipo.» y el botón queda en **Autorizar y sincronizar**. Pulsarlo autoriza este equipo, pero la carpeta AGENTS nace con el primer archivo que traes: cuando tengas uno, por ejemplo después de usar tu agente por primera vez, vuelve a **Configuración › Asistentes** y pulsa **Crear la carpeta AGENTS** otra vez.

## Cómo queda la carpeta

Después de traer, la Biblioteca tiene una carpeta **AGENTS** con una subcarpeta por asistente y una **Global**. Cada documento se llama como el archivo, con el asistente delante:

```text
AGENTS                         al día
├── Global
├── Claude
│   ├── claude · CLAUDE.md
│   ├── claude · rules/estilo.md
│   ├── claude · pr/SKILL.md
│   └── claude · projects/tisty/memory/MEMORY.md
├── Codex
│   └── codex · AGENTS.md
└── Gemini
    └── gemini · GEMINI.md
```

Las subcarpetas son para que encuentres las cosas: **lo que decide adónde va un documento es su línea `agent:`**, no la carpeta en que está. Puedes mover un documento entre subcarpetas o crear las tuyas dentro de AGENTS, y se sigue escribiendo en el mismo archivo.

Junto al nombre de la carpeta AGENTS, la barra lateral te dice cómo está todo:

| Junto a AGENTS | Qué significa |
| :-- | :-- |
| **al día** | Cada archivo dice lo mismo que su documento |
| **2 con cambios** | Dos archivos cambiaron por fuera, o en los dos lados, y esperan que decidas |
| **apagada** | Este equipo no está autorizado, o pausaste toda la carpeta |
| **vacía** | La carpeta no tiene documentos |

![La barra lateral en español con la carpeta AGENTS abierta, su icono de robot morado y la palabra al día a la derecha; dentro, las subcarpetas Global, Claude, Codex y Gemini, y en Claude los documentos claude · CLAUDE.md, claude · rules/estilo.md y claude · pr/SKILL.md](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-02-barra.png)

### La línea `agent:`

Cada documento de AGENTS empieza con sus propiedades, y una de ellas dice qué archivo es:

```markdown
---
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

El valor tiene tres partes, separadas por `/`:

- **El nivel**: `global`, o un asistente: `claude`, `codex`, `copilot`, `gemini` o `cursor`.
- **El tipo**: `rule`, `skill` o `memory`.
- **El nombre**: la ruta del archivo dentro de la carpeta del asistente, como `CLAUDE.md`, `rules/estilo.md` o `pr/SKILL.md`.

A veces lleva marcas más, que Tisty pone solo al traer un archivo para escribirlo igual que estaba: `crlf` (líneas que terminan como en Windows), `bom` (una marca invisible al principio), `as-is` (finales de línea mezclados) y `open-end` (el archivo no termina en salto de línea). Por ejemplo, `agent: [copilot/rule/copilot-instructions.md, crlf, bom]`. La [guía de propiedades](https://markdown.tisty.app/es/propiedades) explica el resto.

La página no muestra esa línea: en su lugar, arriba del documento, aparece su [ficha](#la-ficha-de-un-documento-de-agente).

## Global o un asistente

**Un documento de Global llega a cada asistente que Tisty encuentra en este equipo.** Uno de la carpeta de un asistente llega solo a ese asistente.

### Una regla en Global

Escribe en **AGENTS › Global** un documento con estas propiedades:

```markdown
---
agent: global/rule/para-todos.md
title: global · para-todos.md
---
# Para todos mis agentes

- Antes de borrar algo, pregunta.
- Nunca subas cambios sin que te lo pida.
```

Esa regla no se escribe en un archivo propio: va **al principio del archivo principal de cada asistente**, antes de las reglas de ese asistente. Si en este equipo tienes Claude Code, Codex y Gemini, Tisty escribe:

- `~/.claude/CLAUDE.md`: la regla de Global, y después lo que dice tu documento `claude · CLAUDE.md`.
- `~/.codex/AGENTS.md`: la regla de Global, y después tu documento `codex · AGENTS.md`, si lo tienes.
- `~/.gemini/tisty-rules.md`: la regla de Global, y después tu documento `gemini/rule/tisty-rules.md`, si lo tienes.

Así queda el `CLAUDE.md` de Claude Code:

```markdown
# Para todos mis agentes

- Antes de borrar algo, pregunta.
- Nunca subas cambios sin que te lo pida.

# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

La ficha de cada uno de esos documentos lo dice: «Claude lo lee en `~/.claude/CLAUDE.md`», y debajo, «junto con otro documento». Cursor no recibe reglas: las suyas viven en sus ajustes, no en un archivo.

> [!IMPORTANT]
> **Deja marcado el archivo principal de cada asistente la primera vez.** Si `~/.codex/AGENTS.md` ya existe y no lo trajiste, Tisty no sabe qué parte es tuya: al escribir una regla de Global, ese archivo queda en **Cambiado en los dos lados**, y desde ahí solo puedes sobrescribirlo, lo que lo reemplaza entero por tus reglas de Global (el original queda como respaldo n.º 1), o decidir después. Traído, Global se escribe delante de lo que ya decía.

### Una regla solo para Claude

Claude Code también lee cada archivo de `~/.claude/rules`. Para una regla que solo Claude debe leer, escribe un documento con `agent: claude/rule/rules/pruebas.md`, en cualquier lugar de AGENTS, y Tisty escribe `~/.claude/rules/pruebas.md`. Copilot hace lo mismo con su carpeta `instructions`: `agent: copilot/rule/instructions/rust.instructions.md`.

Un nombre que el asistente no lee no se escribe en ninguna parte: `agent: claude/rule/notas.md` deja la ficha diciendo «Ningún asistente de este equipo lee este documento todavía.».

### Una skill en Global

Una skill es una carpeta con un `SKILL.md` y, a veces, scripts. Cada archivo es un documento, con el mismo nivel y la misma carpeta de skill:

```text
agent: global/skill/pdf/SKILL.md
agent: global/skill/pdf/scripts/extraer.py
```

Una skill de Global se escribe una sola vez donde la leen varios a la vez:

- en `~/.claude/skills/pdf/`, para Claude Code;
- en `~/.cursor/skills/pdf/`, para Cursor;
- en `~/.agents/skills/pdf/`, una sola copia que leen Codex, Copilot y Gemini.

El `SKILL.md` muestra sus propias propiedades, su nombre y su descripción, como campos bajo la ficha. El script se guarda en un bloque de código y se escribe tal cual, sin el bloque: la ficha dice **Se escribe tal cual**, y un documento Markdown dice **Se escribe como Markdown**.

## Qué archivo lee cada asistente

`~` es tu carpeta de usuario: `C:\Users\tú` en Windows y `/Users/tú` en macOS. Tisty solo trabaja con un asistente cuya carpeta existe en este equipo.

| Asistente | Su carpeta | La mueve | Reglas que escribe Tisty | Skills | Memoria, solo para leer |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Claude Code** | `~/.claude` | `CLAUDE_CONFIG_DIR` | `CLAUDE.md` y lo que va en `rules/` | `skills/` en su carpeta | `projects/*/memory/`, la memoria de cada proyecto |
| **Codex** | `~/.codex` | `CODEX_HOME` | `AGENTS.md`, y Global en `AGENTS.override.md` cuando ese archivo existe | `~/.agents/skills/` | — |
| **GitHub Copilot CLI** | `~/.copilot` | `COPILOT_HOME` | `copilot-instructions.md` y lo que va en `instructions/` | `skills/` en su carpeta; Global en `~/.agents/skills/` | — |
| **Gemini CLI** | `~/.gemini` | `GEMINI_CLI_HOME`, la carpeta que contiene `.gemini` | `tisty-rules.md`, y la línea `@tisty-rules.md` en `GEMINI.md` | `skills/` en su carpeta; Global en `~/.agents/skills/` | `GEMINI.md` |
| **Cursor** | `~/.cursor` | — | Ninguna: sus reglas viven en sus ajustes | `skills/` en su carpeta | — |

Lo que hace Tisty con cada uno:

- **Claude Code.** Trae `CLAUDE.md`, cada archivo de `rules/`, cada skill y, si la marcas, la memoria de cada proyecto, solo la carpeta `memory` y nunca las conversaciones guardadas. Escribe `CLAUDE.md` y `rules/`, y las skills.
- **Codex.** Codex lee `AGENTS.override.md` en vez de `AGENTS.md` cuando el primero existe, así que tus reglas de Global van a ese archivo mientras exista. Sus skills van a `~/.agents/skills`, que `CODEX_HOME` no mueve.
- **GitHub Copilot CLI.** Trae y escribe `copilot-instructions.md` y cada archivo de `instructions/`, como `rust.instructions.md`.
- **Gemini CLI.** Gemini escribe su propio `GEMINI.md` cuando le pides que recuerde algo, así que ese archivo es suyo: Tisty lo puede traer como memoria y nunca lo sobrescribe. Tus reglas van a un archivo aparte, `~/.gemini/tisty-rules.md`, y Tisty agrega **una sola línea** al final de `GEMINI.md`, `@tisty-rules.md`, para que Gemini lo lea. No toca nada más de ese archivo, y si la línea ya está, no la repite.
- **Cursor.** Solo skills, propias o de Global.

**Las variables que mueven una carpeta** se leen del entorno con que arrancó Tisty. Si usas `CLAUDE_CONFIG_DIR` para guardar Claude Code en otra parte, defínela como variable de tu usuario y vuelve a abrir Tisty, para que la vea.

## La ficha de un documento de agente

Un documento de AGENTS se abre **para leer**, con una ficha arriba en lugar de sus propiedades:

![La ficha en español de claude · CLAUDE.md: arriba las etiquetas Regla, Claude y Al día en verde; debajo el nombre CLAUDE.md, la fila Claude lo lee en ~/.claude/CLAUDE.md con Se escribe como Markdown · junto con otro documento · el archivo cambió hace 5 minutos, el enlace 3 respaldos a la derecha, y los botones Editar, Respaldos y Pausar la sincronización con Claude](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-03-ficha.png)

- **Las etiquetas**: el tipo (**Regla**, **Skill** o **Memoria del agente**), el nivel (**Global** o el asistente) y el estado.
- **El nombre** del archivo.
- **Una fila por archivo en que se escribe**: «Claude lo lee en» y la ruta. Debajo, **Se escribe como Markdown** o **Se escribe tal cual**, si se arma «junto con otro documento» o «junto con otros 2 documentos», cuándo cambió el archivo, y el enlace a sus respaldos. Un documento de Global tiene una fila por asistente, cada una con su enlace **Pausar Claude**, **Pausar Codex**…
- **Los botones**: **Editar** abre el documento para escribir y **Listo** lo vuelve a cerrar; **Respaldos**, cuando el archivo tiene alguno; y **Pausar la sincronización con Claude**, cuando el documento se escribe en un solo archivo.

Un documento de memoria no tiene **Editar**: lo escribe el agente, y la ficha dice «Este archivo lo escribe el agente: Tisty lo muestra y lo trae, y nunca lo sobrescribe.».

### Cada estado

| Estado | Qué significa | Qué hacer |
| :-- | :-- | :-- |
| **Al día** | El archivo dice lo mismo que el documento | Nada |
| **Por escribir** | Cambiaste el documento y Tisty lo escribe en un instante | Nada |
| **Cambiado por fuera** | Alguien cambió el archivo fuera de Tisty, o lo borró | Elegir: [traer o sobrescribir](#un-cambio-por-fuera-traer-o-sobrescribir) |
| **Cambiado en los dos lados** | Cambiaron el archivo y el documento desde la última escritura | Elegir cuál conservar |
| **En pausa** | Pausaste la sincronización de ese asistente, o de toda la carpeta | **Reanudar** cuando quieras |
| **Sin permiso en este equipo** | Este equipo no autorizó AGENTS | **Autorizar en este equipo**, en Configuración › Asistentes |
| **Nada que escribir** | El documento quedó vacío, y Tisty nunca vacía un archivo de un agente | Escribir algo, o sacar el documento de AGENTS |
| **El archivo es un enlace** | El archivo, o una carpeta en su camino, es un enlace simbólico; escribirlo reemplazaría el enlace | Cambiar el enlace por el archivo real |
| **El bloque de código está roto** | El bloque que guarda un script ya no está entero | Arreglar el bloque en el documento |
| **No se puede leer** | El archivo existe, pero Tisty no pudo leerlo | Revisar sus permisos |

Dos avisos más pueden ocupar el lugar de las filas:

- **«Ningún asistente de este equipo lee este documento todavía.»** El asistente no está en este equipo, o el nombre no es un archivo que ese asistente lea.
- **«Otro documento de AGENTS ya escribe este archivo, así que este queda fuera.»** Dos documentos tienen la misma línea `agent:`: solo uno se escribe.

## Un cambio por fuera: traer o sobrescribir

Puedes seguir editando los archivos de tus agentes con tu editor, o dejar que el agente los cambie. Tisty nunca pisa esos cambios.

**Un ejemplo.** Abres `~/.claude/CLAUDE.md` en tu editor y agregas «- Corre los tests antes de cada commit.». La próxima vez que Tisty mira (al iniciar, al volver a la ventana o al abrir el documento), la barra dice **1 con cambios** junto a AGENTS, y la ficha de `claude · CLAUDE.md` dice **Cambiado por fuera**, con un aviso:

![La ficha en español de claude · CLAUDE.md con la etiqueta Cambiado por fuera en ámbar y el aviso El archivo cambió por fuera de Tisty, ~/.claude/CLAUDE.md ya no dice lo mismo que este documento, y los botones Traer el cambio, Sobrescribir con Tisty y Ver el archivo](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-04-por-fuera.png)

- **Ver el archivo** muestra lo que el archivo tiene ahora.
- **Traer el cambio** escribe en el documento lo que dice el archivo. Lo que decía antes el documento queda en sus [versiones anteriores](Documentos#versiones-anteriores).
- **Sobrescribir con Tisty** pregunta «¿Escribir este documento sobre `~/.claude/CLAUDE.md`? Lo que tiene ahora el archivo queda en un respaldo.» y, si dices que sí, vuelve a escribir el archivo desde el documento.

Mientras no eliges, Tisty no escribe ese archivo, aunque cambies el documento.

**Traer el cambio** solo aparece cuando el cambio se puede atribuir a este documento: el documento es de ese mismo asistente, no de Global, y no lo estás editando. Si `CLAUDE.md` se arma con una regla de Global y tu documento de Claude, y el cambio cae en la parte de Global, solo queda **Sobrescribir con Tisty**: copia a mano en Global lo que quieras conservar y después sobrescribe.

**Si borraron el archivo**, la ficha dice **Cambiado por fuera** sin **Ver el archivo**: **Sobrescribir con Tisty** lo vuelve a crear.

### Cambiado en los dos lados

Si cambiaron el archivo y también el documento, la ficha dice **Cambiado en los dos lados**. Cuando el documento es el único que arma ese archivo, aparece **Quedarse con el archivo**, que pregunta «¿Dejar este documento como está ahora …? Lo que dice ahora queda en su historial.»; si no, solo **Sobrescribir con Tisty**.

Lo mismo puede pasar la primera vez, cuando ya tenías en AGENTS un documento de un archivo que entretanto cambió. Entonces aparece **Existe en los dos lados**, con las líneas que difieren: en rojo, con `−`, lo que hay **En Tisty**; en verde, con `+`, lo que hay **En el archivo**. Eliges **Conservar el de Tisty**, **Conservar el archivo** o **Decidir después**, y lo que no conservas queda guardado: el archivo en sus [respaldos](#respaldos), el documento en sus versiones anteriores. Si el archivo se arma con varios documentos, solo puedes conservar el de Tisty o decidir después. Desde **Configuración › Asistentes**, «1 archivo dice algo distinto en Tisty y en el agente.» y **Revisar** te traen de vuelta a ese aviso.

![El aviso en español Existe en los dos lados para ~/.codex/AGENTS.md: arriba la ruta, debajo las líneas comparadas, en rojo con un signo menos la que solo está En Tisty y en verde con un signo más la que solo está En el archivo, y los botones Decidir después, Conservar el archivo y Conservar el de Tisty](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-07-dos-lados.png)

## Respaldos

**Antes de sobrescribir un archivo de un agente, Tisty guarda lo que tenía.** Los respaldos viven en este equipo, en `data/agents/` dentro de la carpeta de Tisty, nunca en las carpetas de tus agentes, y no viajan con la sincronización ni con los paquetes.

- **El respaldo n.º 1 es el original**: el archivo como estaba antes de que Tisty lo sobrescribiera por primera vez. Ese no se borra nunca.
- **Además, los últimos 20.** Cuando llega el 21, se va el más antiguo después del original.

Para verlos, pulsa **Respaldos** en la ficha, o el enlace **3 respaldos** en la fila del archivo:

![El cuadro Respaldos en español para ~/.claude/CLAUDE.md, con la línea Lo que tenía ~/.claude/CLAUDE.md antes de que Tisty lo sobrescribiera. Se guardan los últimos 20., y tres filas, Respaldo n.º 3 · 1,2 kB, Respaldo n.º 2 · 1,1 kB y Respaldo n.º 1 · 0,9 kB, cada una con Ver y Recuperar](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-05-respaldos.png)

- **Ver** muestra el respaldo completo.
- **Recuperar** pregunta «¿Devolver este respaldo a …? Lo que tiene ahora el archivo queda en un respaldo.» y lo escribe en el archivo.

**Un respaldo recuperado no cambia tu documento.** El archivo vuelve a decir lo que decía el respaldo, y la ficha pasa a **Cambiado por fuera**: eliges **Traer el cambio**, para que el documento diga lo mismo, o **Sobrescribir con Tisty**, si te arrepentiste. Así nunca pierdes ninguno de los dos lados.

Las versiones anteriores del **documento**, en cambio, son las de cualquier documento: están en su [historial](Documentos#versiones-anteriores).

## Pausar, apagar y vaciar

**Pausar un asistente.** En la ficha, **Pausar la sincronización con Claude**, o el enlace **Pausar Claude** en su fila. También en **Configuración › Asistentes**, en la tarjeta **AGENTS**, con el interruptor **Sincronizar Claude**. En pausa, Tisty no abre ni escribe los archivos de ese asistente: quedan como los dejó la última escritura, y tú sigues editando tus documentos. **Reanudar la sincronización con Claude** escribe lo que cambiaste, salvo un archivo que cambió por fuera mientras tanto, que queda marcado para que decidas.

**Apagar toda la carpeta.** En la misma tarjeta, **Sincronizar la carpeta AGENTS**. Apagada, la barra dice **apagada** junto a AGENTS, y la ficha de cada documento dice «La sincronización está apagada para toda la carpeta AGENTS.».

![La tarjeta AGENTS de Configuración › Asistentes en español: la línea Las reglas, skills y memoria de tus agentes como documentos de Tisty, el interruptor Sincronizar la carpeta AGENTS encendido y debajo Sincronizar Claude, Sincronizar Codex y Sincronizar Gemini, con Codex apagado y la nota Apagado: sus archivos quedan como están y no se borra nada en Tisty](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-06-configuracion.png)

La pausa y el permiso son de este equipo.

**Sacar un documento.** Si mueves un documento fuera de AGENTS, lo archivas o lo borras, Tisty deja de escribirlo:

- si era el único que armaba su archivo, el archivo queda como estaba;
- si el archivo se armaba con varios, como `CLAUDE.md` con una regla de Global, Tisty lo vuelve a escribir sin esa parte, y guarda antes un respaldo.

Tisty nunca vacía ni borra un archivo de un agente.

**Vaciar AGENTS.** Si sacas todos los documentos de AGENTS, la carpeta y sus subcarpetas desaparecen de Tisty, y los archivos de tus agentes quedan como estaban.

**Ocultarla de la barra.** Si no quieres verla todo el tiempo, botón derecho sobre AGENTS y **Ocultar de la barra**. Solo desaparece de la barra lateral en este equipo: se sigue sincronizando, y la encuentras en **Ocultas**, al pie de la barra, o con **Mostrar ocultas**.

**Lo que la deja de convertir en AGENTS.** Tisty reconoce la carpeta porque está arriba de todo, se llama **AGENTS** y lleva su icono de robot. Si la renombras, le cambias el icono, la mueves dentro de otra carpeta o la archivas, Tisty deja de escribir los archivos, como si la hubieras vaciado.

## Lo que nunca se lee

- **Los ajustes de tus asistentes**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` y `keybindings.json`. Tisty solo mira donde van las reglas, las skills y la memoria, nunca la carpeta principal de un asistente, y dentro de una skill un archivo con esos nombres sí entra, porque es de la skill.
- **Las credenciales**, por su nombre y sin abrirlas: `.credentials.json`, `credentials.json`, `auth.json`, `.env` y sus variantes, `.netrc`, y las llaves `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` o `id_ed25519`.
- **Un archivo que parece llevar un secreto**, como un token o una llave privada, aunque su nombre sea inocente. La revisión se hace en este equipo y busca formas conocidas, así que puede no reconocerlas todas: deja los secretos fuera de las reglas de tus agentes.
- **Las conversaciones de Claude Code**: de su carpeta `projects` solo se mira la carpeta `memory` de cada proyecto.
- **Los enlaces simbólicos** y los archivos de más de 512 kB.

Y un documento de AGENTS solo se escribe donde van las reglas, las skills y la memoria de cada asistente, nunca en sus ajustes ni en sus credenciales.

**La memoria del agente** viene sin marcar la primera vez, y una vez en Tisty es solo para leer: el agente la sigue escribiendo, y cuando cambia, la ficha dice **Cambiado por fuera** y **Traer el cambio** la pone al día en Tisty. No tiene **Editar**, **Sobrescribir con Tisty** ni **Recuperar**.

Qué guarda Tisty de todo esto, y qué viaja, está en [Privacidad](Privacidad#tus-asistentes-en-agents).

## Con tu agente, por MCP

Un agente que dejas entrar por [[MCP|Asistentes-y-MCP]] ve tus documentos de AGENTS como cualquier otro, y su guía le explica qué son. Por ejemplo, le pides a Claude Code «¿Qué reglas mías lees?».

**1. Lee su guía.** `rules` con el tema «documents» trae, entre otras, la sección sobre AGENTS:

```json
{ "name": "rules", "arguments": { "topic": "documents" } }
```

```text
## Assistants' files in AGENTS

A document whose properties hold an `agent:` line is an assistant's own file kept in the folder
AGENTS — a rule file such as `CLAUDE.md`, a skill, a script — and Tisty writes its text, byte for
byte, to the file that assistant reads, on each computer where the person turned that on. …
```

**2. Busca el documento.** `brain` con las palabras del archivo:

```json
{ "name": "brain", "arguments": { "query": "CLAUDE.md" } }
```

```text
0 task(s) and 1 document(s) match "CLAUDE.md"; showing 0 and 1.
"claude · CLAUDE.md" (document) [k4r5j4qr-0001]
```

`brain` con `folder` y el nombre o la ruta de una subcarpeta lista los documentos de ese asistente. La carpeta AGENTS sola responde con cuántos hay en cada subcarpeta.

**3. Mira su ficha y lo lee.** `brain` con `doc` da su índice y su ubicación; `read` con `doc`, el texto entero, con la línea `agent:`:

```json
{ "name": "read", "arguments": { "doc": "k4r5j4qr-0001" } }
```

```text
---
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

El agente te contesta con el título y la carpeta, «tu documento claude · CLAUDE.md, en AGENTS / Claude», nunca con el identificador. Cada herramienta está en [[Herramientas MCP|Herramientas-MCP]].

## Preguntas frecuentes

**¿Tisty reemplaza mis archivos?** No los mueve ni los convierte: los sigue escribiendo donde tu agente los busca. Antes de la primera escritura guarda el original, el respaldo n.º 1, que no se borra nunca.

**¿Puedo seguir editando `CLAUDE.md` a mano?** Sí. Tisty marca el archivo como **Cambiado por fuera** y espera que elijas: traer tu cambio al documento o sobrescribirlo.

**¿Qué pasa si cierro o desinstalo Tisty?** Tus agentes siguen leyendo sus archivos tal como Tisty los dejó. Nada depende de que Tisty esté abierto.

**¿AGENTS toca `settings.json`, los hooks o mi configuración MCP?** No: AGENTS no los lee ni los escribe. Conectar un asistente es otra cosa: ahí Tisty agrega su servidor MCP y sus hooks a la configuración de ese asistente, como cuenta [[Asistentes y MCP|Asistentes-y-MCP]].

**¿Por qué la memoria del agente viene sin marcar?** Porque la escribe el agente, puede hablar de tus proyectos, y una vez en Tisty viaja con tu sincronización y tus respaldos como cualquier documento. La traes solo si quieres leerla en Tisty.

**¿Qué es la línea `@tisty-rules.md` en mi `GEMINI.md`?** Es la única línea que Tisty agrega a ese archivo, para que Gemini lea tus reglas, que van en `tisty-rules.md`. El resto de `GEMINI.md` lo escribe Gemini.

**¿AGENTS ve los asistentes que uso dentro de WSL?** No. En Windows, Tisty mira tu carpeta de usuario de Windows, y un asistente que corre dentro de WSL guarda sus archivos en tu carpeta de Linux.

**¿Y las reglas de cada repositorio?** Quedan donde están. AGENTS maneja los archivos de tu usuario, no el `AGENTS.md` o el `CLAUDE.md` de cada proyecto.

**¿Puedo escribir una regla nueva desde cero?** Sí: crea un documento dentro de AGENTS y escribe sus propiedades con la línea `agent:`, como en [Global o un asistente](#global-o-un-asistente). Tisty lo escribe apenas lo guardas.

**¿Cuántos respaldos guarda?** El original y los últimos 20 de cada archivo, en este equipo.

## Si algo no cuadra

**Un asistente no aparece en la lista.** Tisty solo ve un asistente cuya carpeta existe: `~/.claude`, `~/.codex`, `~/.copilot`, `~/.gemini` o `~/.cursor`. Si la moviste con una variable como `CODEX_HOME`, defínela como variable de tu usuario y vuelve a abrir Tisty. Si nunca usaste ese asistente en este equipo, ábrelo una vez para que cree su carpeta.

**Un archivo aparece atenuado.** El motivo está a su lado: mira [lo que dice la lista](#lo-que-dice-la-lista-junto-a-un-archivo).

**Mi cambio en Tisty no llega al archivo.** Mira el estado en la ficha del documento:

- **En pausa**: reanuda ese asistente, o la carpeta, en la ficha o en Configuración › Asistentes.
- **Sin permiso en este equipo**: pulsa **Autorizar en este equipo** en Configuración › Asistentes.
- **Cambiado por fuera** o **Cambiado en los dos lados**: Tisty espera tu decisión antes de escribir.
- **Nada que escribir**, **El archivo es un enlace** o **El bloque de código está roto**: la tabla de [cada estado](#cada-estado) dice qué hacer.
- **«Ningún asistente de este equipo lee este documento todavía.»**: revisa la línea `agent:`. El nivel debe ser un asistente que esté en este equipo, y el nombre, un archivo que ese asistente lea.
- **«Otro documento de AGENTS ya escribe este archivo»**: dos documentos tienen la misma línea `agent:`. Deja uno.

**Escribí una regla de Global y un archivo quedó en Cambiado en los dos lados.** Ese archivo ya existía y no lo trajiste, así que Tisty no sabe qué parte es tuya. Para conservar lo que decía, pulsa **Ver el archivo** en la ficha, copia lo tuyo a un documento nuevo dentro de AGENTS con la línea `agent: codex/rule/AGENTS.md` (o la del asistente que sea), y después **Sobrescribir con Tisty**: el archivo queda con Global delante de lo tuyo, y el original, como respaldo n.º 1. Mira [Una regla en Global](#una-regla-en-global).

**Toda la carpeta dejó de escribir.** Revisa que AGENTS siga arriba de todo en la Biblioteca, con ese nombre y su icono de robot, y sin archivar.

**Codex no lee mis reglas.** Si tienes `~/.codex/AGENTS.override.md`, Codex lee solo ese archivo. Tus reglas de Global van ahí, pero un documento `codex · AGENTS.md` se escribe en `AGENTS.md`, que Codex no lee mientras exista el otro.

**Gemini no lee mis reglas.** Revisa que `~/.gemini/GEMINI.md` tenga la línea `@tisty-rules.md`. Si la borraste, Tisty la vuelve a agregar la próxima vez que revisa tus archivos: al iniciar, o cuando cambias un documento de AGENTS.

**No aparece Traer el cambio.** El documento es de Global, el cambio cae en la parte de otro documento del mismo archivo, o lo estás editando: pulsa **Listo** y vuelve a mirar.

**«El archivo cambió mientras Tisty lo escribía, así que no lo tocó.»** Otro programa escribió el archivo al mismo tiempo. Tisty no pisa nada: vuelve a mirar la ficha y elige.

**Recuperé un respaldo y dice Cambiado por fuera.** Es lo esperado: [un respaldo recuperado no cambia tu documento](#respaldos). Elige **Traer el cambio** para quedarte con él.

## Ver también

- [Privacidad](Privacidad#tus-asistentes-en-agents): qué lee AGENTS, cuándo, y qué se queda en este equipo.
- [[Asistentes y MCP|Asistentes-y-MCP]]: conectar un agente a Tisty.
- [Versiones anteriores](Documentos#versiones-anteriores): el historial de un documento.
- [Carpetas](Documentos#carpetas): filtrar, archivar y desarchivar.
- [Propiedades](https://markdown.tisty.app/es/propiedades), en la guía de Markdown de Tisty: la línea `agent:` y sus marcas.
