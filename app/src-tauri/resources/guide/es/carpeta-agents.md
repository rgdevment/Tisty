# AGENTS: las reglas de tus agentes

Esta hoja es una página de la guía. Cuenta la carpeta AGENTS: cómo guardar en Tisty las reglas, las skills y la memoria de tus agentes de IA, cómo llegan a los archivos que lee cada uno, qué hacer cuando un archivo cambia por fuera o en otro equipo, cómo traer un archivo después o quitar el de una regla que sacas, y cómo aprobar lo que tu agente te propone.

Uso más de un agente de IA, y cada uno lee sus reglas de un archivo distinto, en una carpeta distinta: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Cambiar una regla en todos era abrir cuatro archivos, y tarde o temprano uno quedaba atrás. AGENTS junta esos archivos en Tisty, como documentos, y Tisty escribe por debajo el archivo que lee cada agente.

> [!TIP]
> **En corto:** escribes tus reglas una vez, en Tisty, y Tisty las escribe en el archivo de cada agente. Antes de tocar un archivo guarda lo que tenía, nunca pisa un cambio hecho por fuera, y tu agente no escribe tus reglas: te propone el cambio, y solo tú lo apruebas.

## Qué es y cuándo te sirve

AGENTS es una carpeta de tu Biblioteca, arriba de todo, con el icono de un robot. Cada documento que tiene adentro es un archivo de un agente: una **regla**, una **skill** o su **memoria**. Trabaja con Claude Code, Codex, GitHub Copilot CLI, Gemini CLI y Cursor.

Te sirve cuando:

- usas varios agentes y quieres las mismas reglas en todos;
- tus reglas crecieron y quieres leerlas como documentos, con títulos y búsqueda;
- quieres poder volver atrás: Tisty guarda el archivo original y los últimos 20 antes de cada sobrescritura;
- trabajas en más de un equipo y quieres elegir qué reglas aplica cada uno.

No maneja las reglas de cada repositorio: solo los archivos de tu carpeta de usuario, como `~/.claude/CLAUDE.md`.

## Empezar

1. Abre **Configuración → Asistentes**. En la tarjeta **AGENTS**, pulsa **Crear la carpeta AGENTS**.
2. Se abre **Tus agentes, en un solo lugar**, con lo que encontró en las carpetas de tus asistentes: **solo sus nombres y tamaños**. Todavía no abrió ningún archivo.
3. Abre la fila de un asistente, como **Claude · 7 archivos**, y marca lo que quieres traer. Las reglas y las skills vienen marcadas; la **Memoria del agente**, sin marcar. Los ajustes y las credenciales aparecen atenuados y no entran.
4. Pulsa **Traer y sincronizar**. Tisty abre solo lo que marcaste y te dice cuántos documentos entraron.

Si todavía no tienes archivos de agentes, el botón dice **Autorizar y sincronizar**, y pulsarlo crea igual AGENTS con su subcarpeta **Global**, vacía, lista para tu primera regla. Lo que tu agente cree después lo traes con **Traer otro archivo**, como cuenta más abajo.

**Pruébalo:** abre **Configuración → Asistentes** y pulsa **Crear la carpeta AGENTS** para ver qué encuentra en este equipo. Si solo querías mirar, **Ahora no** cierra el aviso sin leer nada. Si ya la creaste, la tarjeta no ofrece el botón: la carpeta está arriba de todo en tu Biblioteca.

Después, en la barra lateral, cada documento se ve con su nombre y un icono: un libro para una regla, una pieza para una skill y un cerebro para la memoria. Pasa el puntero sobre uno y ves de qué archivo es.

![La barra lateral con la carpeta AGENTS abierta y al día: en Claude, la regla Cómo trabajo con el icono de un libro, la skill pr con el icono de una pieza y 3 archivos, y la memoria MEMORY.md con el icono de un cerebro; el puntero sobre Cómo trabajo muestra claude · CLAUDE.md](agents-barra.png)

## Una regla: en Tisty y en `CLAUDE.md`

Tu `CLAUDE.md` entra a AGENTS como un documento. Arriba lleva sus propiedades; la línea `agent:` dice para quién es, qué es y dónde va: `nivel/tipo/nombre`.

```markdown title="El documento en Tisty, en AGENTS / Claude"
---
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

En la barra se llama **Cómo trabajo**, por su primer título. Y esto es lo que Tisty escribe en el archivo, byte a byte, sin las propiedades:

```markdown title="~/.claude/CLAUDE.md"
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

Para cambiar una regla, abre el documento, pulsa **Editar** en su ficha, escribe y pulsa **Listo**. En un instante el archivo dice lo mismo.

### Una regla para todos: Global

Un documento en **AGENTS → Global** llega a cada asistente de este equipo. Va al principio del archivo principal de cada uno, antes de sus propias reglas:

```markdown title="El documento en AGENTS / Global"
---
agent: global/rule/para-todos.md
title: global · para-todos.md
---
# Para todos mis agentes

- Antes de borrar algo, pregunta.
```

```markdown title="~/.claude/CLAUDE.md, con Global delante"
# Para todos mis agentes

- Antes de borrar algo, pregunta.

# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

Lo mismo pasa con el `AGENTS.md` de Codex, el `copilot-instructions.md` de Copilot y las reglas de Gemini. Cursor no recibe reglas: las suyas viven en sus ajustes.

> [!IMPORTANT]
> Trae el archivo principal de cada asistente. Si `~/.codex/AGENTS.md` ya existe y nunca lo trajiste, Tisty no sabe qué parte es tuya y no lo pisa: te pregunta **Existe en los dos lados**, y ahí **Traer a AGENTS** lo trae con sus palabras y escribe Global delante, con su respaldo antes. Lo cuenta **Traer otro archivo**, más abajo.

## Una skill con su `SKILL.md`

Una skill es una carpeta con un `SKILL.md` y, a veces, scripts. Cada archivo es un documento con la misma carpeta de skill en su línea `agent:`. El `SKILL.md` lleva además sus propias propiedades, que la ficha muestra como campos:

```markdown title="El SKILL.md de la skill pr"
---
agent: claude/skill/pr/SKILL.md
title: claude · pr/SKILL.md
---
---
name: pr
description: Abre un PR con el título y el cuerpo en el formato del repositorio.
---

# Abrir un PR

1. Revisa que la rama esté al día.
2. Corre scripts/abrir-pr.sh.
```

Un script se guarda dentro de un bloque de código, y Tisty lo escribe tal cual, sin el bloque:

````markdown title="El script de la skill"
---
agent: claude/skill/pr/scripts/abrir-pr.sh
title: claude · pr/scripts/abrir-pr.sh
---
```sh
#!/bin/sh
gh pr create --fill
```
````

Tisty escribe la skill en `~/.claude/skills/pr/`. En la barra lateral, la skill es **una sola fila**, **pr**, con **3 archivos** a la derecha: un clic la abre y muestra `SKILL.md`, `scripts/abrir-pr.sh` y el resto. Una skill de **Global** se escribe para todos: en `~/.claude/skills/`, en `~/.cursor/skills/` y una sola copia en `~/.agents/skills/`, que leen Codex, Copilot y Gemini.

## Gemini y la línea `@tisty-rules.md`

Gemini escribe su propio `GEMINI.md` cuando le pides que recuerde algo, así que ese archivo es suyo y Tisty nunca lo sobrescribe. Tus reglas van a un archivo aparte, `~/.gemini/tisty-rules.md`, y Tisty agrega **una sola línea** al final de `GEMINI.md` para que Gemini lo lea:

```markdown title="~/.gemini/GEMINI.md"
## Gemini Added Memories
- La perra de Mario se llama Rina.
@tisty-rules.md
```

No toca nada más de ese archivo, y si la línea ya está, no la repite.

**Si borras esa línea**, Tisty lo toma como tu decisión y no la vuelve a poner. Gemini queda **En pausa**, y Tisty deja de escribir `tisty-rules.md`. La ficha de cada regla que Gemini lee te lo dice: «Gemini no lee esta regla: falta la línea @tisty-rules.md en GEMINI.md, y Tisty no la vuelve a poner sola.». También lo dicen **Configuración → Asistentes**, bajo **Sincronizar Gemini**, y el aviso **Falta la línea de Tisty** al entrar en AGENTS. **Volver a ponerla**, en cualquiera de los tres, agrega otra vez la línea al final de `GEMINI.md` y Gemini vuelve a recibir tus reglas.

## La ficha y el panel Archivo del agente

Un documento de AGENTS se abre para leer, con una **ficha** arriba:

- las etiquetas: **Regla**, **Skill** o **Memoria del agente**; **Global** o el asistente; y el estado, como **Al día**;
- el nombre, **Cómo trabajo**, y debajo, en pequeño, la ruta, `CLAUDE.md`;
- una fila por archivo: «Claude lo lee en `~/.claude/CLAUDE.md`», con cuándo cambió y sus respaldos;
- los botones **Editar**, **Respaldos** y **Pausar la sincronización con Claude**.

Al lado, el panel **Archivo del agente** ocupa el lugar del tamaño de hoja y del índice: **Lo lee**, **Se escribe en**, **Formato**, **Última escritura**, **Respaldos**, **Original guardado** y **Sincroniza en este equipo**, con **Copiar enlace** y, cuando ya tiene versiones anteriores, **Historial**.

![La ficha de Cómo trabajo con las etiquetas Regla, Claude y Al día, el nombre en grande y la ruta CLAUDE.md debajo, la fila Claude lo lee en ~/.claude/CLAUDE.md con 3 respaldos, y los botones Editar, Respaldos y Pausar la sincronización con Claude; a la derecha, el panel Archivo del agente con Lo lee Claude, Formato Markdown · LF y Sincroniza en este equipo Sí](agents-ficha.png)

## Un cambio por fuera: traer o sobrescribir

Puedes seguir editando los archivos de tus agentes con tu editor, o dejar que el agente los cambie. Tisty nunca pisa esos cambios.

**Un ejemplo.** Abres `~/.claude/CLAUDE.md` en tu editor y agregas «- Corre los tests antes de cada commit.». La próxima vez que Tisty mira, la barra dice **1 con cambios** junto a AGENTS, y la ficha de **Cómo trabajo** dice **Cambiado por fuera**:

- **Ver el archivo** muestra lo que tiene ahora.
- **Traer el cambio** lo escribe en el documento. Lo que decía antes queda en su historial.
- **Sobrescribir con Tisty** pregunta antes y vuelve a escribir el archivo desde el documento. Lo que tenía el archivo queda en un respaldo.

Mientras no eliges, Tisty no escribe ese archivo. Si cambiaste el archivo y también el documento, la ficha dice **Cambiado en los dos lados**, y eliges cuál conservar.

![La ficha de Cómo trabajo con la etiqueta Cambiado por fuera en ámbar, el aviso El archivo cambió por fuera de Tisty, y los botones Traer el cambio, Sobrescribir con Tisty y Ver el archivo](agents-por-fuera.png)

## Recuperar un respaldo

Antes de sobrescribir un archivo, Tisty guarda lo que tenía. El **respaldo n.º 1** es el original, el de antes de que Tisty lo tocara, y nunca se borra; además guarda los últimos 20. Viven en este equipo y no viajan con la sincronización.

1. En la ficha, pulsa **Respaldos**, o el enlace **3 respaldos** de la fila.
2. Elige uno, como **Respaldo n.º 2 · 1,1 kB**. **Ver** lo muestra entero.
3. **Recuperar** pregunta «¿Devolver este respaldo a `~/.claude/CLAUDE.md`? Lo que tiene ahora el archivo queda en un respaldo.» Di que sí.
4. El archivo vuelve a decir lo del respaldo, y la ficha pasa a **Cambiado por fuera**: tu documento no cambió. **Traer el cambio** lo deja igual al respaldo; **Sobrescribir con Tisty**, si te arrepentiste.

Así nunca pierdes ninguno de los dos lados.

## Pausar un asistente

Si un día quieres que Tisty no toque los archivos de Claude, por ejemplo mientras pruebas otras reglas a mano:

- en la ficha, **Pausar la sincronización con Claude**; en un documento de Global, el enlace **Pausar Claude** de su fila;
- o en **Configuración → Asistentes**, en la tarjeta **AGENTS**, apaga **Sincronizar Claude**. **Sincronizar la carpeta AGENTS** apaga todo.

En pausa, Tisty no abre ni escribe esos archivos, y tú sigues editando tus documentos. **Reanudar la sincronización con Claude** escribe lo que cambiaste, salvo un archivo que cambió por fuera mientras tanto, que queda marcado para que decidas. La pausa es solo de este equipo.

## Traer otro archivo

La primera vez no tiene que ser la única. Si instalas un asistente nuevo o escribes a mano un archivo de reglas que antes no existía, lo traes después.

**Un ejemplo.** Escribes en tu editor una regla nueva para Claude Code:

```markdown title="~/.claude/rules/pruebas.md"
# Pruebas

- Corre los tests antes de cada commit.
```

1. En la barra, botón derecho sobre **AGENTS › Claude**, o sobre cualquier carpeta de AGENTS, y **Traer otro archivo**. También está en **Configuración → Asistentes**, en la tarjeta **AGENTS**.
2. El aviso dice «Estos son los archivos de agentes de este equipo que todavía no están en AGENTS. Elige qué traer:». Esta vez **no viene nada marcado**, y lo que Tisty ya escribe dice **ya está en AGENTS**.
3. Marca `rules/pruebas.md` y pulsa **Traer**. En **AGENTS › Claude** aparece **Pruebas**, y desde ahí la cambias en Tisty.

![El aviso Traer otro archivo, con la línea Estos son los archivos de agentes de este equipo que todavía no están en AGENTS. Elige qué traer:, la fila de Claude abierta con CLAUDE.md atenuado y ya está en AGENTS, y rules/pruebas.md marcado como Regla; abajo, los botones Ahora no y Traer](agents-traer-otro.png)

**Un archivo que nunca trajiste.** Si `~/.codex/AGENTS.md` ya tenía tus reglas de Codex y la primera vez lo dejaste sin marcar, una regla de Global no lo pisa: Tisty te pregunta **Existe en los dos lados**, con «Tisty nunca trajo este archivo. Si lo traes a AGENTS, queda con lo que dice y con lo de Tisty agregado arriba.». **Traer a AGENTS** lo trae como documento y escribe Global delante de lo tuyo, con su respaldo antes:

```markdown title="~/.codex/AGENTS.md, después de Traer a AGENTS"
# Para todos mis agentes

- Antes de borrar algo, pregunta.

# Reglas de Codex

- Usa pnpm, nunca npm.
```

Lo mismo pasa con `AGENTS.override.md`, el archivo que Codex lee en vez de `AGENTS.md` cuando existe: si lo creas después, tráelo a AGENTS, y desde ahí Global va ahí.

## Sacar una regla y su archivo

Cuando borras una regla, la mueves fuera de AGENTS o vuelves a una versión sin ficha, Tisty deja de escribirla. Si era la única que armaba su archivo, ese archivo sigue en el disco y tu agente lo sigue leyendo, así que alrededor de un segundo después Tisty te pregunta:

![El aviso ¿Quitar también el archivo?, con la línea Ninguna regla de AGENTS escribe ya este archivo de Claude, pero sigue en el disco y Claude lo sigue leyendo., la ruta ~/.claude/rules/pruebas.md en gris, la línea Si lo quitas, queda un respaldo en este equipo., y los botones Dejarlo y Quitar el archivo](agents-quitar.png)

- **Quitar el archivo** guarda un respaldo en este equipo y lo borra: Claude deja de leer esa regla.
- **Dejarlo** lo deja donde está, y Tisty no lo vuelve a escribir ni a preguntar. Si un día lo quieres de vuelta, aparece en **Traer otro archivo**.

Si lo editaste a mano después de que Tisty lo escribió, el aviso te lo dice antes de que elijas. Nunca pregunta por la memoria de tu agente, ni por un asistente en pausa.

**Vaciar AGENTS.** Si sacas todos sus documentos con Tisty abierto, primero te pregunta por cada archivo, y después la carpeta desaparece. **Crear la carpeta AGENTS**, en **Configuración → Asistentes**, la vuelve a crear, con el aviso de la primera vez y tus reglas y skills ya marcadas.

## Un segundo equipo

Tus documentos de AGENTS viajan con tu sincronización a tus otros equipos, pero **cada equipo elige qué aplica a sus agentes**.

**Un ejemplo.** En el equipo de la casa trajiste tu `CLAUDE.md` y escribiste la regla «Para todos mis agentes». Ahí se aplicaron solas. En el equipo del trabajo:

1. En **Configuración → Asistentes**, pulsa **Autorizar en este equipo** y trae, si quieres, los archivos de este equipo.
2. La barra dice **2 en espera** junto a AGENTS.
3. Abre **Para todos mis agentes**. La ficha dice «Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este?».
4. **Aplicar aquí** lo escribe en los archivos de este equipo, con su respaldo antes. **No en este equipo** lo deja fuera aquí, y **Aplicar aquí** sigue a mano si cambias de idea.

Desde ahí, lo que ya aplicas se pone al día solo cuando lo cambias en el otro equipo. Lo que escribes o traes en un equipo se aplica en ese mismo equipo sin ningún clic. Y la memoria de un agente es del equipo que la trajo: en los demás solo se lee.

![La ficha de Para todos mis agentes en el equipo del trabajo, con las etiquetas Regla y Global, sin filas de archivos, y el aviso ámbar Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este? con los botones Aplicar aquí y No en este equipo; junto a AGENTS, en la barra, 2 en espera](agents-otro-equipo.png)

Dos avisos más que puedes ver en una ficha:

- **Por aprobar**: el texto del documento cambió sin pasar por tus manos, por ejemplo con un paquete importado. No llega a ningún agente hasta que lo lees y pulsas **Aprobar este texto**. Si ese documento ya se escribía en un archivo, el archivo queda **En espera**, como estaba.
- **Escrito por un Tisty anterior**: una versión anterior de Tisty escribió el archivo y hoy no dice lo mismo que tus documentos. **Ver el archivo** y, si te quedas con el documento, **Sobrescribir con Tisty**.

## Lo que tu agente puede y no puede

Tu agente **lee** tus documentos de AGENTS, como cualquier otro: le pides «¿qué reglas mías lees?» y te responde con sus títulos. Pero **no los escribe por su cuenta**: cuando cambia uno, su cambio queda aparte, como una propuesta, y solo llega a tus agentes si lo apruebas. Una regla de AGENTS llega a todos tus agentes, en todos tus equipos, y nadie más que tú debería decidir qué dice.

Tampoco puede mover, archivar ni marcar esos documentos, cambiar su línea `agent:`, proponer sobre una memoria ni crear un documento nuevo con una línea `agent:`, en ninguna carpeta. Si lo intenta, Tisty se lo rechaza sin tocar nada y le dice cómo proponer el cambio o que te cuente qué cambiaría.

## Una propuesta de tu agente

**Un ejemplo.** Le pides a tu agente «agrega a mis reglas que corra los tests antes de cada commit».

1. El agente corrige **Cómo trabajo** con `edit_doc`, como cualquier documento. Tisty guarda su cambio aparte, byte a byte, y le responde que quedó propuesto: el documento y `~/.claude/CLAUDE.md` siguen como estaban. Su guía le pide decírtelo así, «te lo propuse», y no darlo por hecho.
2. La barra dice **1 con propuesta** junto a AGENTS. Si Tisty estaba abierto, aparece sola en uno o dos segundos, aunque la ventana no esté al frente.
3. Abre **Cómo trabajo**. La ficha suma la etiqueta **Propuesta** y un recuadro morado: «Un agente (Claude Code) propone un cambio», y las líneas comparadas, en rojo con `−` lo **Aprobado** y en verde con `+` lo **Propuesto**.
4. **Aprobar el cambio** pregunta si lo apruebas. Con tu sí, el documento pasa a decir lo propuesto y Tisty lo escribe en `CLAUDE.md`, con su respaldo antes. **Descartar** la quita y el documento queda como lo aprobaste.

![La ficha de Cómo trabajo con las etiquetas Regla, Claude, Al día y Propuesta, y el recuadro morado Un agente (Claude Code) propone un cambio, con las líneas comparadas − Aprobado y + Propuesto, la línea nueva + - Corre los tests antes de cada commit. en verde, y los botones Aprobar el cambio y Descartar](agents-propuesta.png)

Cada botón actúa solo sobre lo que la ficha te mostró: si el agente volvió a escribir mientras mirabas, Tisty te pide revisarla de nuevo. Si vuelve a escribir, su cambio va encima de su propia propuesta, así que hay una sola por documento. Y mientras una propuesta espera, lo que ya aprobaste sigue llegando a tus agentes.

## Más en la ayuda

Cada estado, cada asistente con sus archivos y sus variables, lo que nunca se lee y qué hacer cuando algo no cuadra están en [la carpeta AGENTS](https://www.tisty.app/es/ayuda/carpeta-agents/), en la ayuda. Qué guarda Tisty de todo esto, y qué viaja, en [privacidad](https://www.tisty.app/es/ayuda/privacidad/). Y cómo conectar tu agente a Tisty, en [Paso a paso con tu agente de IA](tisty:agent).
