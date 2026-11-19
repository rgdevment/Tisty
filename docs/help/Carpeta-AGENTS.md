# AGENTS: las reglas, skills y memoria de tus agentes

Uso más de un agente de IA, y cada uno lee sus reglas de un archivo distinto, en una carpeta distinta: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Cambiar una regla en todos era abrir cuatro archivos, y tarde o temprano uno quedaba atrás. AGENTS junta esos archivos en Tisty, como documentos, y Tisty escribe por debajo el archivo que lee cada agente.

> [!TIP]
> **En corto:** la carpeta **AGENTS** guarda como documentos las reglas, las skills y la memoria de Claude Code, Codex, GitHub Copilot CLI, Gemini CLI y Cursor. Lo que escribes en **Global** llega a todos; lo que escribes en la carpeta de un asistente, solo a él. Antes de sobrescribir un archivo, Tisty guarda lo que tenía, un cambio hecho por fuera nunca se pisa: queda marcado hasta que tú decides. Y un agente no escribe tus reglas: te propone el cambio, y solo tú lo apruebas.

![La primera vez de AGENTS en español: el aviso Tus agentes, en un solo lugar, con sus tres líneas sobre privacidad, sincronización y respaldo, y debajo la lista Encontré esto. Elige qué traer, con Claude · 7 archivos abierto: CLAUDE.md y dos archivos de rules marcados como Regla, la skill pr/SKILL.md marcada como Skill, una memoria de proyecto sin marcar como Memoria del agente y un .env dentro de una skill atenuado con credenciales, no entra; más abajo Codex · 2 archivos y Gemini · 1 archivo plegados, y los botones Ahora no y Traer y sincronizar](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-01-primera-vez.png)

## Qué es y cuándo te sirve

AGENTS es una carpeta de tu Biblioteca, arriba de todo, con el icono de un robot. Cada documento que tiene adentro es un archivo de un agente: una regla, una skill o su memoria. Lo editas en Tisty como cualquier documento, con su buscador y sus versiones anteriores, y Tisty escribe el archivo que ese agente lee, byte a byte. Al lado, el panel **Archivo del agente** te dice quién lo lee, dónde se escribe y cuántos respaldos tiene.

Te sirve cuando:

- **Usas varios agentes y quieres las mismas reglas en todos.** Escribes «Responde en español» una vez en **Global**, y llega al `CLAUDE.md` de Claude Code, al `AGENTS.md` de Codex, al `copilot-instructions.md` de Copilot y a las reglas de Gemini.
- **Tus reglas crecieron y ya no las encuentras.** En Tisty se leen como documentos: cada una con su nombre y su icono en la barra lateral, con títulos y búsqueda, en vez de un archivo plano en una carpeta oculta.
- **Trabajas en más de un equipo.** Escribes la regla una vez, y cada equipo elige si la aplica a sus agentes.
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
6. Pulsa **Traer y sincronizar**. Si no marcaste nada, el botón dice **Autorizar y sincronizar**: autoriza este equipo sin traer archivos, y si AGENTS todavía no existe, la crea vacía, con su **Global**.
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

El aviso dice «No encontré archivos de agentes en este equipo. Igual puedes autorizarlo: AGENTS se llena a medida que escribes tus reglas.», y el botón queda en **Autorizar y sincronizar**. Pulsarlo autoriza este equipo y crea igual la carpeta **AGENTS**, con su subcarpeta **Global**. La barra dice **vacía** junto a AGENTS, y entrar en ella no vuelve a abrir el aviso.

Desde ahí tienes dos caminos:

- **Escribir tu primera regla.** Crea un documento dentro de AGENTS con su línea `agent:`, como en [Global o un asistente](#global-o-un-asistente), y Tisty crea el archivo apenas lo guardas.
- **Traer lo que aparezca después.** Cuando tu agente cree sus archivos, por ejemplo después de usarlo por primera vez, tráelos con [Traer otro archivo](#traer-otro-archivo).

## Traer otro archivo

La primera vez no tiene que ser la única. Si instalas un asistente nuevo, si creas un archivo de reglas que antes no existía o si dejaste algo sin marcar, lo traes después:

- **Desde la barra lateral:** botón derecho sobre AGENTS, o sobre cualquier carpeta dentro de ella, y **Traer otro archivo**.
- **Desde Configuración › Asistentes:** en la tarjeta **AGENTS**, el botón **Traer otro archivo**.

Se abre **Traer otro archivo**, con «Estos son los archivos de agentes de este equipo que todavía no están en AGENTS. Elige qué traer:» y la misma lista de la primera vez, con una diferencia: **no viene nada marcado**, ni siquiera las reglas. Marca lo que quieres y pulsa **Traer**: el botón no se activa hasta que marcas algo. Lo que Tisty ya escribe desde un documento dice **ya está en AGENTS**, y si no queda nada por traer, el aviso dice «Todos los archivos de agentes de este equipo ya están en AGENTS.».

**Un ejemplo.** Un día escribes a mano en tu editor una regla nueva para Claude Code, `~/.claude/rules/pruebas.md`:

```markdown title="~/.claude/rules/pruebas.md"
# Pruebas

- Corre los tests antes de cada commit.
```

1. Botón derecho sobre **AGENTS › Claude** y **Traer otro archivo**.
2. Abre la fila de Claude: `CLAUDE.md` aparece atenuado, con **ya está en AGENTS**, y `rules/pruebas.md`, sin marcar, como **Regla**.
3. Márcalo y pulsa **Traer**. El aviso dice «Entró 1 documento a AGENTS, y desde ahora Tisty lo mantiene al día.», y **Listo** lo cierra.
4. En **AGENTS › Claude** aparece **Pruebas**, con el icono de un libro. Desde ahora lo cambias en Tisty, como cualquier regla.

![El aviso Traer otro archivo en español, sin las tres líneas de la primera vez: arriba Estos son los archivos de agentes de este equipo que todavía no están en AGENTS. Elige qué traer:, y debajo la fila de Claude abierta, con CLAUDE.md atenuado y ya está en AGENTS, y rules/pruebas.md marcado como Regla; abajo, los botones Ahora no y Traer](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-10-traer-otro.png)

Si este equipo todavía no autorizó AGENTS, **Traer otro archivo** abre el aviso de la primera vez, con sus tres líneas y lo que encontró ya marcado.

### Un archivo que nunca trajiste

**Un ejemplo.** Tienes `~/.codex/AGENTS.md` con tus reglas de Codex, y la primera vez lo dejaste sin marcar. Después escribes una regla en **Global**, que también va a ese archivo. Tisty no sabe qué parte es tuya, así que no lo pisa: te pregunta **Existe en los dos lados**, al entrar en AGENTS o con **Revisar** en Configuración › Asistentes. Como nunca trajo ese archivo, el aviso suma «Tisty nunca trajo este archivo. Si lo traes a AGENTS, queda con lo que dice y con lo de Tisty agregado arriba.» y el botón **Traer a AGENTS**.

Con **Traer a AGENTS**, el archivo entra como un documento en **AGENTS › Codex**, con sus palabras, y Tisty lo vuelve a escribir con tu regla de Global arriba. Lo que tenía antes queda en sus [respaldos](#respaldos):

```markdown title="~/.codex/AGENTS.md, después de Traer a AGENTS"
# Para todos mis agentes

- Antes de borrar algo, pregunta.

# Reglas de Codex

- Usa pnpm, nunca npm.
```

**Traer otro archivo** hace lo mismo: ese `AGENTS.md` aparece en su lista, sin marcar, y al traerlo queda igual, con Global arriba y su respaldo.

### El `AGENTS.override.md` de Codex

Codex lee `~/.codex/AGENTS.override.md` en vez de `AGENTS.md` cuando existe, así que tus reglas de Global van a ese archivo mientras exista. Si lo creas después de la primera vez y tienes reglas en Global, Tisty te pregunta **Existe en los dos lados** con **Traer a AGENTS**, como con cualquier archivo que nunca trajo. Sin reglas en Global no pregunta nada: tráelo con **Traer otro archivo**. Al traerlo:

- `AGENTS.override.md` queda con tu regla de Global arriba y lo que escribiste debajo, y desde ahí Codex recibe Global ahí;
- `AGENTS.md` queda solo con tus reglas de Codex, sin Global. Codex no lo lee mientras exista el override.

## Cómo queda la carpeta

Después de traer, la Biblioteca tiene una carpeta **AGENTS** con una subcarpeta por asistente y una **Global**. En la barra lateral, cada documento se ve con su nombre y un icono que dice qué es:

```text
AGENTS                           al día
├── Global
├── Claude
│   ├── [libro]    Cómo trabajo
│   ├── [libro]    Estilo de código
│   ├── [pieza]    pr                3 archivos
│   └── [cerebro]  MEMORY.md
├── Codex
│   └── [libro]    Reglas de Codex
└── Gemini
    └── [cerebro]  GEMINI.md
```

- **El icono**: un libro para una **regla**, una pieza de rompecabezas para una **skill** y un cerebro para la **memoria del agente**.
- **El nombre**: el primer título del documento, como «Cómo trabajo» en un `CLAUDE.md` que empieza con `# Cómo trabajo`. Una skill toma el `name` de su `SKILL.md`. Un script, o un archivo sin títulos, se llama como el archivo: `MEMORY.md`, `extraer.py`.
- **La ruta**: pasa el puntero sobre una fila y aparece, con el asistente delante: `claude · CLAUDE.md`, `claude · rules/estilo.md`.
- **Una skill es una sola fila**, con cuántos archivos tiene a la derecha, como **pr** y **3 archivos**. Un clic la abre y muestra sus archivos por su ruta dentro de la skill: `SKILL.md`, `scripts/abrir-pr.sh`. Así, una skill de 200 archivos ocupa una línea hasta que la abres. Una skill de un solo archivo se ve como un documento más.

Las subcarpetas son para que encuentres las cosas: **lo que decide adónde va un documento es su línea `agent:`**, no la carpeta en que está. Puedes mover un documento entre subcarpetas o crear las tuyas dentro de AGENTS, y se sigue escribiendo en el mismo archivo.

Junto al nombre de la carpeta AGENTS, la barra lateral te dice cómo está todo:

| Junto a AGENTS | Qué significa |
| :-- | :-- |
| **al día** | Cada archivo dice lo mismo que su documento |
| **2 con cambios** | Dos archivos esperan que decidas: cambiaron por fuera o en los dos lados, los escribió un Tisty anterior, ya [no son de ninguna regla](#sacar-una-regla-y-su-archivo), o falta [la línea de Gemini](#gemini-en-pausa) |
| **3 en espera** | Tres documentos llegaron de [otro de tus equipos](#en-varios-equipos) y esperan que elijas si los aplicas en este |
| **1 por aprobar** | Un documento cambió sin pasar por tus manos, y no llega a ningún agente hasta que [lo apruebes](#por-aprobar) |
| **1 con propuesta** | Un agente [propone un cambio](#una-propuesta-de-tu-agente) a un documento, y espera que lo apruebes o lo descartes |
| **apagada** | Este equipo no está autorizado, o pausaste toda la carpeta |
| **vacía** | La carpeta no tiene documentos, y nada espera que decidas |

Si hay más de una cosa, se ve la más urgente: **vacía** cuando no hay documentos ni nada que decidir, y si no, primero **apagada**, después **con cambios**, después **en espera**, después **por aprobar** y al final **con propuesta**.

![La barra lateral en español con la carpeta AGENTS abierta, su icono de robot morado y la palabra al día a la derecha; dentro, las subcarpetas Global, Claude, Codex y Gemini. En Claude, la regla Cómo trabajo con el icono de un libro, la regla Estilo de código, la skill pr con el icono de una pieza y 3 archivos a la derecha, abierta, mostrando SKILL.md, scripts/abrir-pr.sh y plantilla.md, y la memoria MEMORY.md con el icono de un cerebro; el puntero sobre Cómo trabajo muestra claude · CLAUDE.md](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-02-barra.png)

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

- `~/.claude/CLAUDE.md`: la regla de Global, y después lo que dice tu documento de ese archivo, **Cómo trabajo** en el ejemplo.
- `~/.codex/AGENTS.md`: la regla de Global, y después tu documento de `AGENTS.md`, si lo tienes.
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
> **Trae el archivo principal de cada asistente.** Si `~/.codex/AGENTS.md` ya existe y nunca lo trajiste, Tisty no sabe qué parte es tuya: al escribir una regla de Global, la ficha marca ese archivo **Cambiado en los dos lados**, y **Sobrescribir con Tisty** lo reemplazaría entero por tus reglas de Global. En vez de eso, usa **Traer a AGENTS**: lo trae con sus palabras y escribe Global delante, con el original como respaldo n.º 1. Lo cuenta [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste).

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
- **Codex.** Codex lee `AGENTS.override.md` en vez de `AGENTS.md` cuando el primero existe, así que tus reglas de Global van a ese archivo mientras exista. Si lo creas después de la primera vez, [tráelo](#el-agentsoverridemd-de-codex) para que Global vaya ahí sin pisar lo tuyo. Sus skills van a `~/.agents/skills`, que `CODEX_HOME` no mueve.
- **GitHub Copilot CLI.** Trae y escribe `copilot-instructions.md` y cada archivo de `instructions/`, como `rust.instructions.md`.
- **Gemini CLI.** Gemini escribe su propio `GEMINI.md` cuando le pides que recuerde algo, así que ese archivo es suyo: Tisty lo puede traer como memoria y nunca lo sobrescribe. Tus reglas van a un archivo aparte, `~/.gemini/tisty-rules.md`, y Tisty agrega **una sola línea** al final de `GEMINI.md`, `@tisty-rules.md`, para que Gemini lo lea. No toca nada más de ese archivo, y si la línea ya está, no la repite. Si después la borras, Tisty no la vuelve a poner: Gemini queda [en pausa](#gemini-en-pausa) hasta que tú lo decides.
- **Cursor.** Solo skills, propias o de Global.

**Las variables que mueven una carpeta** se leen del entorno con que arrancó Tisty. Si usas `CLAUDE_CONFIG_DIR` para guardar Claude Code en otra parte, defínela como variable de tu usuario y vuelve a abrir Tisty, para que la vea.

## La ficha de un documento de agente

Un documento de AGENTS se abre **para leer**, con una ficha arriba en lugar de sus propiedades, y el panel **Archivo del agente** a su lado:

![Un documento de agente abierto en español. Arriba, la ficha de Cómo trabajo: las etiquetas Regla, Claude y Al día en verde; debajo el nombre Cómo trabajo en grande y la ruta CLAUDE.md pequeña y gris; la fila Claude lo lee en ~/.claude/CLAUDE.md con Se escribe como Markdown · junto con otro documento · el archivo cambió hace 5 minutos y el enlace 3 respaldos a la derecha, y los botones Editar, Respaldos y Pausar la sincronización con Claude. A la derecha, el panel Archivo del agente con Lo lee Claude, Se escribe en ~/.claude/CLAUDE.md, Formato Markdown · LF, Última escritura, Respaldos 3 respaldos, Original guardado Sí, Sincroniza en este equipo Sí, y los botones Historial y Copiar enlace](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-03-ficha.png)

- **Las etiquetas**: el tipo (**Regla**, **Skill** o **Memoria del agente**), el nivel (**Global** o el asistente) y el estado.
- **El nombre**, el mismo que en la barra lateral, y debajo, en pequeño, **la ruta** del archivo dentro de la carpeta del asistente, como `CLAUDE.md` o `pr/SKILL.md`.
- **Una fila por archivo en que se escribe**: «Claude lo lee en» y la ruta completa. Debajo, **Se escribe como Markdown** o **Se escribe tal cual**, si se arma «junto con otro documento» o «junto con otros 2 documentos», cuándo cambió el archivo, y el enlace a sus respaldos. Un documento de Global tiene una fila por asistente, cada una con su enlace **Pausar Claude**, **Pausar Codex**…
- **Los botones**: **Editar** abre el documento para escribir y **Listo** lo vuelve a cerrar; **Respaldos**, cuando el archivo tiene alguno; y **Pausar la sincronización con Claude**, cuando el documento se escribe en un solo archivo.

Un documento de memoria no tiene **Editar**: lo escribe el agente, y la ficha dice «Este archivo lo escribe el agente: Tisty lo muestra y lo trae, y nunca lo sobrescribe.».

### El panel Archivo del agente

En un documento de AGENTS, la columna de al lado no muestra el tamaño de hoja, el índice ni **Compartir**: un archivo de agente no se imprime ni se comparte. En su lugar está **Archivo del agente**:

| Fila | Qué dice | Ejemplo |
| :-- | :-- | :-- |
| **Lo lee** | Qué asistentes de este equipo leen el archivo | Claude, Codex |
| **Se escribe en** | La ruta del archivo. Un documento de Global tiene una fila por asistente: **Archivo de Claude**, **Archivo de Codex** | `~/.claude/CLAUDE.md` |
| **Formato** | Markdown, o el tipo del script; cómo terminan sus líneas, y las marcas que conserva | Markdown · LF, o PY · CRLF · BOM |
| **Última escritura** | Cuándo cambió el archivo por última vez | hace 5 minutos |
| **Respaldos** | Cuántos guarda este equipo | 3 respaldos, o Ninguno todavía |
| **Original guardado** | Si ya está el respaldo n.º 1, el archivo de antes de que Tisty lo tocara | Sí, el de antes de la primera escritura de Tisty |
| **Sincroniza en este equipo** | Si Tisty lo mantiene al día aquí | Sí; Sí, salvo Codex (en pausa); Tisty solo lo lee; No, AGENTS está apagada; No: espera «Aplicar aquí»; No: su texto espera tu aprobación. Con una propuesta esperando, se suma «un cambio propuesto por un agente espera tu aprobación» |

Debajo, **Historial**, cuando el documento ya tiene versiones anteriores, abre su [historial](Documentos#versiones-anteriores), y **Copiar enlace**, su [enlace](Documentos#el-enlace-de-un-documento). Si ya compartías ese documento antes, el panel de compartir sigue ahí.

### Cada estado

| Estado | Qué significa | Qué hacer |
| :-- | :-- | :-- |
| **Al día** | El archivo dice lo mismo que el documento | Nada |
| **Por escribir** | Cambiaste el documento y Tisty lo escribe en un instante | Nada |
| **Cambiado por fuera** | Alguien cambió el archivo fuera de Tisty, o lo borró | Elegir: [traer o sobrescribir](#un-cambio-por-fuera-traer-o-sobrescribir) |
| **Cambiado en los dos lados** | Cambiaron el archivo y el documento desde la última escritura | Elegir cuál conservar |
| **Escrito por un Tisty anterior** | Una versión anterior de Tisty escribió el archivo sin anotar de qué documentos venía, y hoy no dice lo mismo que ellos | **Ver el archivo**, y después **Sobrescribir con Tisty** si te quedas con el documento |
| **En espera** | Uno de los documentos que arman este archivo [cambió sin tu aprobación](#por-aprobar), o salió de AGENTS por mano de un agente, así que el archivo queda como estaba. La ficha nombra ese documento | Aprobar ese documento, o volver a ponerlo y sacarlo tú |
| **En pausa** | Pausaste la sincronización de ese asistente, o de toda la carpeta. En Gemini, también cuando falta la línea `@tisty-rules.md` en `GEMINI.md` | **Reanudar** cuando quieras; en Gemini, [**Volver a ponerla**](#gemini-en-pausa) |
| **Sin permiso en este equipo** | Este equipo no autorizó AGENTS | **Autorizar en este equipo**, en Configuración › Asistentes |
| **Nada que escribir** | El documento quedó vacío, y Tisty nunca vacía un archivo de un agente | Escribir algo, o sacar el documento de AGENTS |
| **El archivo es un enlace** | El archivo, o una carpeta en su camino, es un enlace simbólico; escribirlo reemplazaría el enlace | Cambiar el enlace por el archivo real |
| **El bloque de código está roto** | El bloque que guarda un script ya no está entero | Arreglar el bloque en el documento |
| **No se puede leer** | El archivo existe, pero Tisty no pudo leerlo | Revisar sus permisos |

Dos avisos más pueden ocupar el lugar de las filas:

- **«Ningún asistente de este equipo lee este documento todavía.»** El asistente no está en este equipo, o el nombre no es un archivo que ese asistente lea.
- **«Otro documento de AGENTS ya escribe este archivo, así que este queda fuera.»** Dos documentos tienen la misma línea `agent:`: solo uno se escribe.

Y si el documento todavía no se aplica en este equipo, un aviso ámbar dice por qué, con el botón que lo resuelve: lo cuenta [En varios equipos](#en-varios-equipos).

## Un cambio por fuera: traer o sobrescribir

Puedes seguir editando los archivos de tus agentes con tu editor, o dejar que el agente los cambie. Tisty nunca pisa esos cambios.

**Un ejemplo.** Abres `~/.claude/CLAUDE.md` en tu editor y agregas «- Corre los tests antes de cada commit.». La próxima vez que Tisty mira (al iniciar, al volver a la ventana o al abrir el documento), la barra dice **1 con cambios** junto a AGENTS, y la ficha de **Cómo trabajo**, el documento de ese `CLAUDE.md`, dice **Cambiado por fuera**, con un aviso:

![La ficha en español de Cómo trabajo, con la ruta CLAUDE.md debajo del nombre, la etiqueta Cambiado por fuera en ámbar y el aviso El archivo cambió por fuera de Tisty, ~/.claude/CLAUDE.md ya no dice lo mismo que este documento, y los botones Traer el cambio, Sobrescribir con Tisty y Ver el archivo](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-04-por-fuera.png)

- **Ver el archivo** muestra lo que el archivo tiene ahora.
- **Traer el cambio** escribe en el documento lo que dice el archivo. Lo que decía antes el documento queda en sus [versiones anteriores](Documentos#versiones-anteriores). Si alguien cambió el documento después de que se dibujó la ficha, Tisty no lo pisa: te pide leerlo de nuevo y volver a elegir.
- **Sobrescribir con Tisty** pregunta «¿Escribir este documento sobre `~/.claude/CLAUDE.md`? Lo que tiene ahora el archivo queda en un respaldo.» y, si dices que sí, vuelve a escribir el archivo desde el documento.

Mientras no eliges, Tisty no escribe ese archivo, aunque cambies el documento.

**Traer el cambio** solo aparece cuando el cambio se puede atribuir a este documento: el documento es de ese mismo asistente, no de Global, y no lo estás editando. Si `CLAUDE.md` se arma con una regla de Global y tu documento de Claude, y el cambio cae en la parte de Global, solo queda **Sobrescribir con Tisty**: copia a mano en Global lo que quieras conservar y después sobrescribe.

**Si borraron el archivo**, la ficha dice **Cambiado por fuera** sin **Ver el archivo**: **Sobrescribir con Tisty** lo vuelve a crear.

### Cambiado en los dos lados

Si cambiaron el archivo y también el documento, la ficha dice **Cambiado en los dos lados**. Cuando el documento es el único que arma ese archivo, aparece **Quedarse con el archivo**, que pregunta «¿Dejar este documento como está ahora …? Lo que dice ahora queda en su historial.»; si no, solo **Sobrescribir con Tisty**.

Lo mismo puede pasar la primera vez, cuando ya tenías en AGENTS un documento de un archivo que entretanto cambió. Entonces aparece **Existe en los dos lados**, con las líneas que difieren: en rojo, con `−`, lo que hay **En Tisty**; en verde, con `+`, lo que hay **En el archivo**. Eliges **Conservar el de Tisty**, **Conservar el archivo** o **Decidir después**, y lo que no conservas queda guardado: el archivo en sus [respaldos](#respaldos), el documento en sus versiones anteriores. Si el archivo se arma con varios documentos, solo puedes conservar el de Tisty o decidir después. Si Tisty nunca trajo ese archivo, el aviso suma **Traer a AGENTS**, como cuenta [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste). Desde **Configuración › Asistentes**, «1 archivo dice algo distinto en Tisty y en el agente.» y **Revisar** te traen de vuelta a ese aviso.

![El aviso en español Existe en los dos lados para ~/.codex/AGENTS.md: arriba la ruta, debajo las líneas comparadas, en rojo con un signo menos la que solo está En Tisty y en verde con un signo más la que solo está En el archivo, y los botones Decidir después, Conservar el archivo y Conservar el de Tisty](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-07-dos-lados.png)

## Respaldos

**Antes de sobrescribir un archivo de un agente, Tisty guarda lo que tenía.** Los respaldos viven en este equipo, en `data/agents/` dentro de la carpeta de Tisty, nunca en las carpetas de tus agentes, y no viajan con la sincronización ni con los paquetes.

- **El respaldo n.º 1 es el original**: el archivo como estaba antes de que Tisty lo sobrescribiera por primera vez. Ese no se borra nunca.
- **Además, los últimos 20.** Cuando llega el 21, se va el más antiguo después del original.
- **Quitar un archivo también deja uno.** Si [quitas el archivo](#sacar-una-regla-y-su-archivo) de una regla que sacaste, sus respaldos siguen en este equipo, y si una regla vuelve a escribir esa misma ruta, aparecen en su **Respaldos**.

Para verlos, pulsa **Respaldos** en la ficha, o el enlace **3 respaldos** en la fila del archivo:

![El cuadro Respaldos en español para ~/.claude/CLAUDE.md, con la línea Lo que tenía ~/.claude/CLAUDE.md antes de que Tisty lo sobrescribiera. Se guardan los últimos 20., y tres filas, Respaldo n.º 3 · 1,2 kB, Respaldo n.º 2 · 1,1 kB y Respaldo n.º 1 · 0,9 kB, cada una con Ver y Recuperar](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-05-respaldos.png)

- **Ver** muestra el respaldo completo.
- **Recuperar** pregunta «¿Devolver este respaldo a …? Lo que tiene ahora el archivo queda en un respaldo.» y lo escribe en el archivo.

**Un respaldo recuperado no cambia tu documento.** El archivo vuelve a decir lo que decía el respaldo, y la ficha pasa a **Cambiado por fuera**: eliges **Traer el cambio**, para que el documento diga lo mismo, o **Sobrescribir con Tisty**, si te arrepentiste. Así nunca pierdes ninguno de los dos lados.

Las versiones anteriores del **documento**, en cambio, son las de cualquier documento: están en su [historial](Documentos#versiones-anteriores).

## Pausar, apagar y vaciar

**Pausar un asistente.** En la ficha, **Pausar la sincronización con Claude**, o el enlace **Pausar Claude** en su fila. También en **Configuración › Asistentes**, en la tarjeta **AGENTS**, con el interruptor **Sincronizar Claude**. En pausa, Tisty no abre ni escribe los archivos de ese asistente: quedan como los dejó la última escritura, y tú sigues editando tus documentos. **Reanudar la sincronización con Claude** escribe lo que cambiaste, salvo un archivo que cambió por fuera mientras tanto, que queda marcado para que decidas.

**Apagar toda la carpeta.** En la misma tarjeta, **Sincronizar la carpeta AGENTS**. Apagada, la barra dice **apagada** junto a AGENTS, y la ficha de cada documento dice «La sincronización está apagada para toda la carpeta AGENTS.».

![La tarjeta AGENTS de Configuración › Asistentes en español: la línea Las reglas, skills y memoria de tus agentes como documentos de Tisty, el botón Traer otro archivo, y debajo el interruptor Sincronizar la carpeta AGENTS encendido y Sincronizar Claude, Sincronizar Codex y Sincronizar Gemini, con Codex apagado y la nota Apagado: sus archivos quedan como están y no se borra nada en Tisty](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-06-configuracion.png)

La pausa y el permiso son de este equipo.

### Gemini en pausa

Tisty pone la línea `@tisty-rules.md` en tu `GEMINI.md` una vez. Si después la borras, o borras `GEMINI.md` entero, Tisty lo toma como tu decisión y no la vuelve a poner: Gemini queda **En pausa**, deja de recibir tus reglas, y Tisty deja de escribir `tisty-rules.md`. Lo ves en tres lugares:

- **En la ficha** de cada regla que Gemini lee: la etiqueta **En pausa** y «Gemini no lee esta regla: falta la línea @tisty-rules.md en GEMINI.md, y Tisty no la vuelve a poner sola.», con el botón **Volver a ponerla**.
- **En Configuración › Asistentes**, bajo **Sincronizar Gemini**: «En pausa: falta la línea @tisty-rules.md en GEMINI.md.», con **Volver a ponerla**.
- **Al entrar en AGENTS**, el aviso **Falta la línea de Tisty**: «La línea @tisty-rules.md ya no está en GEMINI.md, así que Gemini no lee tus reglas. Tisty no la vuelve a poner sola: Gemini queda en pausa hasta que tú lo hagas.», con **Dejarlo en pausa** y **Volver a ponerla**.

**Volver a ponerla** agrega otra vez la línea al final de `GEMINI.md`, sin tocar lo demás, y Tisty vuelve a escribir tus reglas en `tisty-rules.md`. Mientras no lo haces, la barra cuenta a Gemini en **con cambios** junto a AGENTS.

### Sacar una regla y su archivo

Tisty deja de escribir un documento cuando lo sacas de AGENTS: cuando lo borras, lo archivas o lo mueves fuera, cuando cambias su línea `agent:` por la de otro archivo, y cuando vuelves a una [versión anterior](Documentos#versiones-anteriores) que no tenía ficha, o tenía otra.

- **Si el archivo se armaba con varios documentos**, como `CLAUDE.md` con una regla de Global, Tisty lo vuelve a escribir sin esa parte, y guarda antes un respaldo.
- **Si era el único**, ninguna regla escribe ya ese archivo, pero sigue en el disco y tu agente lo sigue leyendo. Por eso, alrededor de un segundo después, Tisty te pregunta qué hacer con él.

**Un ejemplo.** Tenías la regla **Pruebas**, de `~/.claude/rules/pruebas.md`, y ya no la quieres. La borras, y se abre **¿Quitar también el archivo?**: «Ninguna regla de AGENTS escribe ya este archivo de Claude, pero sigue en el disco y Claude lo sigue leyendo.», con la ruta debajo y «Si lo quitas, queda un respaldo en este equipo.».

![El aviso en español ¿Quitar también el archivo?, con la línea Ninguna regla de AGENTS escribe ya este archivo de Claude, pero sigue en el disco y Claude lo sigue leyendo., la ruta ~/.claude/rules/pruebas.md en gris, la línea Si lo quitas, queda un respaldo en este equipo., y los botones Dejarlo y Quitar el archivo](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-11-quitar-archivo.png)

- **Quitar el archivo** guarda un respaldo y lo borra: Claude deja de leer esa regla. Si era el último archivo de la carpeta de una skill, esa carpeta se va con él; las carpetas propias del asistente, como `rules`, quedan.
- **Dejarlo** lo deja donde está, y Tisty deja de responder por él: no lo escribe ni te vuelve a preguntar. Si un día lo quieres de vuelta en AGENTS, aparece en [Traer otro archivo](#traer-otro-archivo).

Si el archivo cambió después de que Tisty lo escribió, por ejemplo porque lo editaste a mano, el aviso suma «Cambió después de la última vez que Tisty lo escribió.». El respaldo guarda el archivo como está ahora, con ese cambio.

Si quitas `tisty-rules.md`, el archivo de reglas de Gemini, Tisty también saca la línea `@tisty-rules.md` de `GEMINI.md`, y nada más.

Mientras no respondes, la barra suma ese archivo en **con cambios**, y la tarjeta **AGENTS** de **Configuración › Asistentes** dice «1 archivo ya no es de ninguna regla de AGENTS.», con **Revisar** para volver a la pregunta.

Tisty no pregunta, y deja el archivo como está:

- por la memoria del agente, que escribe el propio agente: la de cada proyecto de Claude Code ni el `GEMINI.md` de Gemini;
- por un archivo de un asistente que pausaste o apagaste;
- por un archivo cuyo documento [espera tu aprobación](#por-aprobar);
- por un archivo que ya no existe.

En tus otros equipos pasa lo mismo: cuando la regla borrada les llega con la sincronización, cada uno te pregunta por su propio archivo la próxima vez que Tisty lo mira, y el respaldo queda en ese equipo.

### Vaciar AGENTS

Si sacas todos los documentos de AGENTS mientras Tisty está abierto, primero te pregunta por los archivos que escribían sus últimas reglas, uno por uno, como en [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo). Cuando respondiste el último, la carpeta AGENTS y sus subcarpetas desaparecen de Tisty, y los archivos que elegiste dejar quedan como estaban.

Solo se va una AGENTS que tenía documentos y vaciaste con la ventana abierta. Una AGENTS vacía de otra forma se queda: la que se creó sin encontrar archivos, o la que ya estaba vacía al abrir Tisty.

Si después quieres AGENTS otra vez, **Crear la carpeta AGENTS**, en Configuración › Asistentes, abre de nuevo el aviso de la primera vez, con las reglas y las skills que encuentre ya marcadas, también las que dejaste, y la memoria sin marcar, como siempre.

### Ocultarla de la barra

Si no quieres verla todo el tiempo, botón derecho sobre AGENTS y **Ocultar de la barra**. Solo desaparece de la barra lateral en este equipo: se sigue sincronizando, y la encuentras en **Ocultas**, al pie de la barra, o con **Mostrar ocultas**.

Oculta y todo, si algo espera tu decisión, su fila en **Ocultas** lo dice, como **1 con cambios** o **2 en espera**. Si todo está al día, no dice nada.

### Lo que la deja de convertir en AGENTS

Tisty reconoce la carpeta porque está arriba de todo, se llama **AGENTS** y lleva su icono de robot. Si la renombras, le cambias el icono, la mueves dentro de otra carpeta o la archivas, Tisty deja de escribir los archivos, y los deja como estaban, sin preguntar por ellos.

## En varios equipos

Si usas Tisty en más de un equipo, por ejemplo el computador de la casa y el del trabajo, tus documentos de AGENTS viajan con tu sincronización como cualquier otro. Pero **cada equipo elige qué aplica a sus agentes**: un documento que llega de otro equipo no se escribe en ningún archivo hasta que tú lo dices, en ese equipo y desde la ventana de Tisty.

### Un segundo equipo, paso a paso

**Un ejemplo.** En el equipo de la casa trajiste tu `CLAUDE.md` y escribiste en **AGENTS › Global** la regla «Para todos mis agentes». Ahí se aplicaron solas: las trajiste y las escribiste tú, en ese equipo. Ahora instalas Tisty en el equipo del trabajo y sincronizas:

1. En **Configuración › Asistentes**, la tarjeta **AGENTS** dice «Este equipo todavía no lo autorizó.». Pulsa **Autorizar en este equipo**.
2. Se abre el mismo aviso de la primera vez, con los archivos de este equipo. Los que ya tienen documento dicen **ya está en AGENTS**, y si todo lo que encontró ya está, el aviso dice «Todo lo que encontré ya está en AGENTS. Autoriza este equipo para mantenerlo al día:». Marca lo que quieras traer de este equipo, como la memoria de sus proyectos, y pulsa **Traer y sincronizar** o **Autorizar y sincronizar**.
3. La barra dice **2 en espera** junto a AGENTS: los dos documentos que vienen de la casa.
4. Abre **Para todos mis agentes**. La ficha todavía no tiene filas de archivos, y un aviso ámbar dice «Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este?», con dos botones:
   - **Aplicar aquí** lo escribe en los archivos de los asistentes de este equipo, con su respaldo antes, como siempre. Si un archivo de este equipo ya decía otra cosa, Tisty no lo pisa: la ficha queda en **Cambiado en los dos lados** y eliges.
   - **No en este equipo** lo deja fuera aquí. La ficha dice «Elegiste no aplicarlo en este equipo.», y **Aplicar aquí** sigue a mano por si cambias de idea.
5. Haz lo mismo con **Cómo trabajo**, tu `CLAUDE.md`. La barra vuelve a **al día**.

![La ficha en español de Para todos mis agentes en el equipo del trabajo: las etiquetas Regla y Global, el nombre Para todos mis agentes con la ruta para-todos.md debajo, ninguna fila de archivos, y el aviso ámbar Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este? con los botones Aplicar aquí y No en este equipo; en la barra lateral, junto a AGENTS, 2 en espera](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-08-otro-equipo.png)

Lo que eliges vale para ese documento en ese equipo, y queda:

- **Un documento que ya aplicas se pone al día solo.** Si mañana cambias «Para todos mis agentes» en la casa, el trabajo lo escribe en cuanto llega, sin volver a preguntar.
- **Lo que escribes o traes en un equipo se aplica en ese equipo sin ningún clic.** Solo lo que llega de otro espera.
- **En espera cuenta solo lo que este equipo leería.** Un documento de Codex no espera en un equipo sin Codex.
- **Una vez aplicado, la ficha no ofrece deshacerlo.** Para que un asistente de este equipo deje de recibirlo, pausa ese asistente aquí; para que ningún equipo lo reciba, sácalo de AGENTS.

### La memoria es del equipo que la trajo

La memoria de un agente habla de lo que pasó en ese equipo, así que nunca se escribe en otro. En los demás equipos, su ficha dice «Memoria del agente de otro equipo: en este solo se lee.», sin botones. Cada equipo puede traer su propia memoria del mismo proyecto: aparece sin marcar en el aviso de la primera vez, aunque otro equipo ya haya traído la suya.

### Por aprobar

**Solo las palabras que tú escribiste o aprobaste llegan al archivo de un agente.** Lo que escribes en la ventana de Tisty, lo que traes la primera vez y lo que traes con **Traer el cambio** ya cuenta como aprobado. Un texto que cambió sin pasar por tus manos no: por ejemplo, si alguien editó el archivo del documento dentro de la carpeta de datos de Tisty, o lo trajo un paquete importado.

Entonces la barra dice **1 por aprobar** junto a AGENTS, y la ficha avisa «Su texto aún no tiene tu aprobación, así que no llega a ningún agente hasta que lo apruebes.», con **Aprobar este texto**. Léelo antes: el botón pregunta «¿Apruebas lo que dice ahora este documento? Léelo antes: se escribe en los archivos de tus agentes.», y si el documento cambió entre que lo leíste y tu sí, Tisty no aprueba nada y te pide leerlo de nuevo.

Si ese documento ya se escribía en un archivo, el archivo no cambia mientras tanto: su ficha dice **En espera** y nombra el documento que lo detiene, por ejemplo «Este archivo queda como estaba: claude · rules/estilo.md cambió o salió de AGENTS sin tu aprobación. Apruébalo, o vuelve a ponerlo y sácalo tú.».

Un documento que puso en AGENTS un agente, y no tú, tampoco se escribe: su ficha dice «Lo puso aquí un agente, así que ningún agente lo lee hasta que lo muevas tú.». Muévelo tú a otra subcarpeta de AGENTS que hayas creado tú (cambiarlo de orden no basta), y se aplica. Si un agente creó o renombró la carpeta en que está, pasa lo mismo con todo lo que hay dentro: muévelo a una carpeta tuya.

### Escrito por un Tisty anterior

Una versión anterior de Tisty escribía los archivos sin anotar de qué documentos venía cada uno. La primera vez que la versión nueva los mira, compara: si el archivo dice lo mismo que sus documentos, lo adopta sin decir nada. Si dice otra cosa, la ficha queda en **Escrito por un Tisty anterior**, con el aviso «Un Tisty anterior escribió este archivo sin anotar de qué documentos venía, así que queda como está hasta que elijas». **Ver el archivo** te muestra qué tiene, y **Sobrescribir con Tisty** lo escribe desde tus documentos, con su respaldo antes.

## Lo que nunca se lee

- **Los ajustes de tus asistentes**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` y `keybindings.json`. Tisty solo mira donde van las reglas, las skills y la memoria, nunca la carpeta principal de un asistente, y dentro de una skill un archivo con esos nombres sí entra, porque es de la skill.
- **Las credenciales**, por su nombre y sin abrirlas: `.credentials.json`, `credentials.json`, `auth.json`, `.env` y sus variantes, `.netrc`, y las llaves `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` o `id_ed25519`.
- **Un archivo que parece llevar un secreto**, como un token o una llave privada, aunque su nombre sea inocente. La revisión se hace en este equipo y busca formas conocidas, así que puede no reconocerlas todas: deja los secretos fuera de las reglas de tus agentes.
- **Las conversaciones de Claude Code**: de su carpeta `projects` solo se mira la carpeta `memory` de cada proyecto.
- **Los enlaces simbólicos** y los archivos de más de 512 kB.

Y un documento de AGENTS solo se escribe donde van las reglas, las skills y la memoria de cada asistente, nunca en sus ajustes ni en sus credenciales.

**La memoria del agente** viene sin marcar la primera vez, y una vez en Tisty es solo para leer: el agente la sigue escribiendo, y cuando cambia, la ficha dice **Cambiado por fuera** y **Traer el cambio** la pone al día en Tisty. No tiene **Editar**, **Sobrescribir con Tisty** ni **Recuperar**, y en tus otros equipos [solo se lee](#la-memoria-es-del-equipo-que-la-trajo).

Qué guarda Tisty de todo esto, y qué viaja, está en [Privacidad](Privacidad#tus-asistentes-en-agents).

## Con tu agente, por MCP

Un agente que dejas entrar por [[MCP|Asistentes-y-MCP]] **lee** tus documentos de AGENTS como cualquier otro, y su guía le explica qué son. **No los escribe: te propone el cambio, y solo tú lo apruebas.** Por ejemplo, le pides a Claude Code «¿Qué reglas mías lees?».

**1. Lee su guía.** `rules` con el tema «documents» trae, entre otras, la sección sobre AGENTS:

```json
{ "name": "rules", "arguments": { "topic": "documents" } }
```

```text
## Assistants' files in AGENTS

A document whose properties hold an `agent:` line is an assistant's own file kept in the folder
AGENTS — a rule file such as `CLAUDE.md`, a skill, a script — and Tisty writes its text, byte for
byte, to the file that assistant reads, on each computer where the person applied the document.
`read` gives the text the person approved. You change one only by proposing: `write_doc` with
`doc` and `print`, `edit_doc` or `append_doc` keep your text byte for byte, apart, and nothing
reaches any file until the person approves it on the document's card: say so, never call it done. …
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

**3. Mira su ficha y lo lee.** `brain` con `doc` da su índice y su ubicación; `read` con `doc`, el texto que aprobaste, entero, con la línea `agent:`:

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

### Una propuesta de tu agente

Le pides «agrega a mis reglas que corra los tests antes de cada commit». El agente corrige el documento como cualquier otro, pero en AGENTS su cambio no se escribe: **queda aparte, como una propuesta**, byte a byte. Nada cambia en el documento ni en los archivos de tus agentes hasta que la apruebas.

**1. El agente propone.** Usa `edit_doc`, `append_doc` o `write_doc` con `doc`, `print` y el texto entero:

```json
{ "name": "edit_doc", "arguments": { "doc": "k4r5j4qr-0001", "old": "- Ramas con la forma tipo/nombre.", "new": "- Ramas con la forma tipo/nombre.\n- Corre los tests antes de cada commit." } }
```

```text
Proposed that change to "claude · CLAUDE.md" [k4r5j4qr-0001]. It waits beside the approved text for
the person, who approves or discards it on the document's card in Tisty; no assistant's file
changes until they approve it. Your next write to this document goes on top of this proposal, at
this `print`.
```

Su guía le pide decírtelo así, «te lo propuse», y nunca darlo por hecho.

**2. La ves en Tisty.** La barra dice **1 con propuesta** junto a AGENTS. Abre el documento: la ficha suma la etiqueta **Propuesta** y, debajo, un recuadro morado:

- «Un agente (Claude Code) propone un cambio», con cuándo lo propuso;
- «Nada de esto llega a los archivos de tus agentes hasta que lo apruebes. Lo que cambia en el texto aprobado:»;
- las líneas comparadas: en rojo, con `−`, lo **Aprobado** que se va; en verde, con `+`, lo **Propuesto** que llega;
- los botones **Aprobar el cambio** y **Descartar**.

![La ficha en español de Cómo trabajo con las etiquetas Regla, Claude, Al día y Propuesta, y debajo el recuadro morado Un agente (Claude Code) propone un cambio · hace 2 minutos, la línea Nada de esto llega a los archivos de tus agentes hasta que lo apruebes, las líneas comparadas con la cabecera − Aprobado y + Propuesto, la línea nueva + - Corre los tests antes de cada commit. en verde, y los botones Aprobar el cambio y Descartar; en la barra, junto a AGENTS, 1 con propuesta](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-09-propuesta.png)

Si Tisty está abierto mientras el agente propone, la propuesta aparece sola en uno o dos segundos, aunque la ventana no esté al frente.

**3. Apruebas o descartas.**

- **Aprobar el cambio** pregunta «¿Apruebas este cambio? Se escribe en el documento y en los archivos de tus agentes.». Con tu sí, el documento pasa a decir lo propuesto, con tu aprobación, y en cada equipo donde ese documento se aplica, Tisty lo escribe en `~/.claude/CLAUDE.md` como cualquier cambio tuyo, con su respaldo antes. Lo que decía antes queda en su historial.
- **Descartar** pregunta «¿Descartas esta propuesta? El documento queda como lo aprobaste.». La propuesta se va y nada más cambia.

Cada botón actúa solo sobre lo que la ficha te mostró. Si el agente volvió a escribir mientras mirabas, Tisty no elige por ti: «La propuesta cambió desde que la ficha la mostró. Revísala de nuevo antes de elegir.». Mientras editas el documento, **Aprobar el cambio** no aparece: pulsa **Listo** primero.

**Lo que conviene saber:**

- **Una propuesta por documento.** Si el agente vuelve a escribir, su cambio va encima de su propia propuesta, y `read` se la entrega aparte del texto aprobado, en `proposal`. Si escribe de vuelta el texto aprobado, la retira.
- **Lo aprobado sigue llegando a tus agentes** mientras una propuesta espera: una propuesta en una regla no detiene nada más. El panel **Archivo del agente** lo dice: «Sí · un cambio propuesto por un agente espera tu aprobación».
- **Si el texto aprobado cambió después**, la ficha avisa «Se propuso sobre una versión anterior, así que aprobarla puede deshacer un cambio hecho después.». Mira bien las líneas antes de aprobar.
- **Si el documento está por aprobar**, primero apruebas su texto con **Aprobar este texto** y después la propuesta: «El texto de este documento aún no tiene tu aprobación. Apruébalo primero y después la propuesta.».
- **Los `#palabras` de un documento de agente no son etiquetas de Tisty.** Una regla que dice «usa #rust» no aparece bajo esa etiqueta.

### Lo que no puede proponer

**La línea `agent:` no cambia**, porque decide en qué archivo se escribe el documento:

```json
{ "name": "edit_doc", "arguments": { "doc": "k4r5j4qr-0001", "old": "claude/rule/CLAUDE.md", "new": "codex/rule/AGENTS.md" } }
```

```text
"claude · CLAUDE.md" [k4r5j4qr-0001] would get another `agent:` line, which only the person
changes, so nothing was proposed. Tell the person what you would change and why.
```

**Mover, archivar, marcar o resumir** un documento de AGENTS, adjuntarle un archivo o volver a una versión anterior se rechaza, y Tisty le dice cómo proponer:

```text
the document "claude · CLAUDE.md" holds an assistant's own files in AGENTS, which only the person
changes, in Tisty: nothing an assistant moves, files, marks or sums up there reaches them. To
change its text, propose it: `write_doc` with `doc`, `print` and the whole `body`, `edit_doc`, or
`append_doc` at its end, and nothing else with them. The person approves or discards it on the
document's card.
```

**Meter otro documento o crear una carpeta** dentro de AGENTS también se rechaza, y ahí Tisty le pide contarte qué cambiaría, para que lo hagas tú.

**Un documento nuevo con una línea `agent:`** se rechaza en cualquier carpeta, para que nadie pueda traer una regla desde fuera:

```text
a body with an `agent:` line is an assistant's own file, which only the person makes, in AGENTS.
Write it without that line, or tell the person what the file should say.
```

**La memoria del agente** no acepta propuestas: la escribe el propio agente, en su archivo. Tampoco entra una propuesta que deje roto el bloque de código de un script, que pase de 64.000 caracteres o de 2.000 líneas, las que la ficha alcanza a comparar.

Y editar el archivo en disco tampoco sirve: ese cambio queda como **Cambiado por fuera** y espera tu decisión. **Por qué así:** una regla de AGENTS llega a todos tus agentes, en todos tus equipos; si un agente pudiera escribirla sola, una instrucción escondida en una página que leyó podría terminar en las reglas de los demás. Con la propuesta, ves cada línea antes de que llegue.

## Preguntas frecuentes

**¿Tisty reemplaza mis archivos?** No los mueve ni los convierte: los sigue escribiendo donde tu agente los busca. Antes de la primera escritura guarda el original, el respaldo n.º 1, que no se borra nunca.

**¿Puedo seguir editando `CLAUDE.md` a mano?** Sí. Tisty marca el archivo como **Cambiado por fuera** y espera que elijas: traer tu cambio al documento o sobrescribirlo.

**¿Qué pasa si cierro o desinstalo Tisty?** Tus agentes siguen leyendo sus archivos tal como Tisty los dejó. Nada depende de que Tisty esté abierto.

**¿AGENTS toca `settings.json`, los hooks o mi configuración MCP?** No: AGENTS no los lee ni los escribe. Conectar un asistente es otra cosa: ahí Tisty agrega su servidor MCP y sus hooks a la configuración de ese asistente, como cuenta [[Asistentes y MCP|Asistentes-y-MCP]].

**¿Por qué la memoria del agente viene sin marcar?** Porque la escribe el agente, puede hablar de tus proyectos, y una vez en Tisty viaja con tu sincronización y tus respaldos como cualquier documento. La traes solo si quieres leerla en Tisty.

**¿Qué es la línea `@tisty-rules.md` en mi `GEMINI.md`?** Es la única línea que Tisty agrega a ese archivo, para que Gemini lea tus reglas, que van en `tisty-rules.md`. El resto de `GEMINI.md` lo escribe Gemini. Si la borras, Tisty no la vuelve a poner: Gemini queda [en pausa](#gemini-en-pausa) hasta que pulsas **Volver a ponerla**.

**¿AGENTS ve los asistentes que uso dentro de WSL?** No. En Windows, Tisty mira tu carpeta de usuario de Windows, y un asistente que corre dentro de WSL guarda sus archivos en tu carpeta de Linux.

**¿Y las reglas de cada repositorio?** Quedan donde están. AGENTS maneja los archivos de tu usuario, no el `AGENTS.md` o el `CLAUDE.md` de cada proyecto.

**¿Puedo escribir una regla nueva desde cero?** Sí: crea un documento dentro de AGENTS y escribe sus propiedades con la línea `agent:`, como en [Global o un asistente](#global-o-un-asistente). Tisty lo escribe apenas lo guardas.

**¿Cuántos respaldos guarda?** El original y los últimos 20 de cada archivo, en este equipo.

**¿Puedo pedirle a mi agente que cambie mis reglas?** Sí, y te lo propone: su cambio queda aparte, en la ficha del documento, hasta que pulsas **Aprobar el cambio** o **Descartar**. Nada llega a tus agentes antes. Mira [Una propuesta de tu agente](#una-propuesta-de-tu-agente).

**Uso Tisty en dos equipos. ¿Mis reglas llegan solas al otro?** Llegan como documentos, con tu sincronización, pero en el otro equipo esperan tu **Aplicar aquí** la primera vez. Después se ponen al día solas. Mira [En varios equipos](#en-varios-equipos).

**¿Puedo tener una regla solo en un equipo?** Sí: en el equipo donde no la quieres, pulsa **No en este equipo** en su ficha. Lo que elige cada equipo es solo suyo.

**Borré una regla. ¿Qué pasa con su archivo?** Tisty te pregunta «¿Quitar también el archivo?»: **Quitar el archivo** lo borra y guarda un respaldo, y **Dejarlo** lo deja donde está, sin que Tisty vuelva a escribirlo. Mira [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo).

**La primera vez dejé un archivo sin marcar. ¿Lo puedo traer ahora?** Sí, con **Traer otro archivo**, desde el botón derecho sobre AGENTS o desde Configuración › Asistentes. Mira [Traer otro archivo](#traer-otro-archivo).

**¿Por qué AGENTS dice vacía?** No tiene documentos: la creaste sin que Tisty encontrara archivos, o sus documentos salieron mientras Tisty estaba cerrado, por ejemplo al sincronizar con otro equipo. Escribe una regla con su línea `agent:`, o trae un archivo con **Traer otro archivo**.

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
- **En espera**: otro documento del mismo archivo espera tu aprobación. La ficha lo nombra; [apruébalo](#por-aprobar).
- **Escrito por un Tisty anterior**: mira el archivo y, si te quedas con el documento, **Sobrescribir con Tisty**.
- **Un aviso ámbar sobre otro equipo o tu aprobación**: el documento todavía no se aplica en este equipo. Mira [En varios equipos](#en-varios-equipos).

**Mi regla no llega a mi otro equipo.** En ese equipo, revisa que AGENTS esté autorizado (**Autorizar en este equipo**, en Configuración › Asistentes) y abre el documento: si dice «Este documento viene de otro equipo», pulsa **Aplicar aquí**; si dice «Elegiste no aplicarlo en este equipo.», también. Si la ficha dice «Ningún asistente de este equipo lee este documento todavía.», ese equipo no tiene el asistente de la regla.

**La barra dice por aprobar y yo no cambié nada.** El texto de un documento cambió sin pasar por la ventana de Tisty, por ejemplo con un paquete importado o una edición directa en la carpeta de datos. Ábrelo, léelo, y si está bien, **Aprobar este texto**.

**Escribí una regla de Global y un archivo quedó en Cambiado en los dos lados.** Ese archivo ya existía y no lo trajiste, así que Tisty no sabe qué parte es tuya. No lo sobrescribas: en **Configuración › Asistentes**, pulsa **Revisar** y, en **Existe en los dos lados**, **Traer a AGENTS**. El archivo queda con Global delante de lo tuyo, y el original, como respaldo n.º 1. Mira [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste).

**Me pregunta «¿Quitar también el archivo?» y no borré nada.** También cuenta como sacar una regla cambiar su línea `agent:` por la de otro archivo, o volver a una versión anterior que no tenía ficha. Si no quieres tocar nada, **Dejarlo** lo deja donde está.

**AGENTS desapareció.** Sacaste todos sus documentos con Tisty abierto, y cuando eso pasa la carpeta se va. Si la quieres de vuelta, **Crear la carpeta AGENTS** en Configuración › Asistentes trae otra vez lo que encuentre. Si no la vaciaste, búscala en **Ocultas**, al pie de la barra.

**Toda la carpeta dejó de escribir.** Revisa que AGENTS siga arriba de todo en la Biblioteca, con ese nombre y su icono de robot, y sin archivar.

**Codex no lee mis reglas.** Si tienes `~/.codex/AGENTS.override.md`, Codex lee solo ese archivo. Tus reglas de Global van ahí, pero un documento con `agent: codex/rule/AGENTS.md` se escribe en `AGENTS.md`, que Codex no lee mientras exista el otro. Si creaste el override después de la primera vez, [tráelo a AGENTS](#el-agentsoverridemd-de-codex).

**Gemini no lee mis reglas.** Si su ficha dice **En pausa**, falta la línea `@tisty-rules.md` en `~/.gemini/GEMINI.md`: pulsa **Volver a ponerla**, en la ficha o en Configuración › Asistentes. Tisty no la vuelve a poner por su cuenta, porque borrarla es tu decisión. Mira [Gemini en pausa](#gemini-en-pausa).

**No aparece Traer el cambio.** El documento es de Global, el cambio cae en la parte de otro documento del mismo archivo, o lo estás editando: pulsa **Listo** y vuelve a mirar.

**«El archivo cambió mientras Tisty lo escribía, así que no lo tocó.»** Otro programa escribió el archivo al mismo tiempo. Tisty no pisa nada: vuelve a mirar la ficha y elige.

**Recuperé un respaldo y dice Cambiado por fuera.** Es lo esperado: [un respaldo recuperado no cambia tu documento](#respaldos). Elige **Traer el cambio** para quedarte con él.

## Ver también

- [Privacidad](Privacidad#tus-asistentes-en-agents): qué lee AGENTS, cuándo, y qué se queda en este equipo.
- [[Asistentes y MCP|Asistentes-y-MCP]]: conectar un agente a Tisty.
- [Versiones anteriores](Documentos#versiones-anteriores): el historial de un documento.
- [Carpetas](Documentos#carpetas): filtrar, archivar y desarchivar.
- [Propiedades](https://markdown.tisty.app/es/propiedades), en la guía de Markdown de Tisty: la línea `agent:` y sus marcas.
