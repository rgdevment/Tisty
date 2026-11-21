# Cómo trabaja un agente

Lo que más me cansaba de trabajar con agentes no era explicarles el problema: era recordarles, respuesta tras respuesta, que marcaran lo que hicieron y anotaran lo que descartaron. Las instrucciones solas no alcanzaban. Así que Tisty ahora le pregunta al agente cuando se olvida, le da una forma barata de dejar la tarea al día, enlaza lo que nombra y le sirve todo lo que guardas como un índice que puede recorrer sin leerlo entero.

Esta página sigue una sesión real de un agente, paso a paso, con las llamadas que hace y un extracto de lo que Tisty le responde. Cómo conectar uno está en [[Asistentes y MCP|Asistentes-y-MCP]]; cada herramienta, en [[Herramientas MCP|Herramientas-MCP]]; lo que puede y no puede hacer, en [[Qué puede hacer un asistente|Qué-puede-hacer-un-asistente]].

## Lo que cambió con Brainsty

- **Busca y navega con una sola herramienta, `brain`.** Encuentra tareas y documentos por sus palabras, con el archivo junto a lo demás; trae la ficha de un documento o una tarea con sus vecinos; y muestra un proyecto entero agrupado en bloqueado, en curso, esperándote y sin empezar.
- **Lee con una sola herramienta, `read`.** Una tarea o un documento, entero o por partes, solo lo resaltado, o una versión anterior; y si nada cambió desde su última lectura, solo eso.
- **Deja la tarea al día en una llamada.** `tick` marca los pasos y escribe el estado de la tarea, una sección «Estado» de su página que se reemplaza entera cada vez. Al retomar, `read` le trae ese estado primero.
- **Tisty le pregunta si se olvida.** En sus respuestas, a cualquier asistente; y con los hooks, al empezar y al terminar cada turno.
- **Lo que nombra queda enlazado.** Una tarea o un documento nombrado por su id o por su título exacto entre comillas se vuelve un enlace al guardar, igual que si lo hubieras hecho tú.

## Antes de empezar

En **Configuración › Asistentes**:

1. En la pestaña **Conectados**, **Conectar** pone Tisty en la configuración del asistente. Mira [[Asistentes y MCP|Asistentes-y-MCP]].
2. Ahí mismo, bajo un asistente conectado (Claude Code, Cursor, Gemini CLI o GitHub Copilot), la fila **Hooks: al empezar y al terminar cada turno** dice **Sin agregar**. **Agregar** te dice qué archivo va a escribir; **Agregarlos** los pone. Desde su próxima sesión, el asistente los usa.
3. En la pestaña **Puerta**, dos interruptores, encendidos de entrada:
   - **Pedirle que deje su tarea al día**: la pregunta en las respuestas.
   - **Recordarle al empezar y al terminar cada turno**: los hooks, sin quitarlos del archivo.

## Una sesión, paso a paso

Mario trabaja con Claude Code en la tarea «Pagos con la pasarela nueva», de la lista **Trabajo**, que tiene cuatro partes. Hoy toca «Webhook de confirmación», que todavía no empieza, y la parte «Reembolsos» quedó bloqueada esperando al proveedor.

### 1. Empieza sabiendo dónde quedó

Con los hooks, cada sesión empieza con las últimas tareas abiertas en que escribieron tus agentes en este equipo, cuántos pasos llevan y el siguiente, sin que el agente tenga que pedirlo. Así empieza la sesión de mañana, después del trabajo que cuenta esta página:

```json
"Your last open tasks here, newest first:
- \"Webhook de confirmación\" in Trabajo […], last written 2026-11-18 22:22: under way, 1 of 3 steps ticked, next \"Reintentos\"; state: **Hecho:** la firma HMAC se verifica…
- \"Reembolsos\" in Trabajo […], last written 2026-11-18 22:22: blocked, 0 of 2 steps ticked, next \"Reembolso total\"; state: Bloqueada: el proveedor…
Pressing for the person: 1 today, 26 late."
```

Sin hooks, o para ver todo, el agente llama a `catch_up`: las listas, las carpetas, las etiquetas más usadas, lo que apremia y `last`, la tarea en que estaba, con cuánto avanzó: su etapa, sus pasos y la primera línea de su estado. El inicio con hooks dice lo mismo con palabras. (La etapa y el estado, aquí y en el inicio con hooks de arriba, llegaron después de esta sesión, y van con la forma que les da el código.)

```json
{ "last": { "title": "Reembolsos", "list": "Trabajo", "at": "2026-11-18 22:22",
            "stage": "blocked", "steps": [0, 2], "state": "Bloqueada: el proveedor habilita los reembolsos parciales…" },
  "pressing": [ { "title": "Comprar pintura para la reja del patio", "list": "Casa", "standing": "today" }, … ],
  "counts": { "open": 54, "late": 26, "today": 1, "docs": 62 } }
```

### 2. Busca con `brain`

`brain` con unas palabras busca en las tareas abiertas, en el archivo y en los documentos a la vez. Primero va lo que tiene las palabras en su título; después, lo que se movió hace menos. Cada documento trae la línea que las dice.

```json
> brain {"query": "webhook firma"}
{ "matches": [ { "title": "Webhook de confirmación", "list": "Trabajo", "status": "open", "part_of": "…" } ],
  "docs": [
    { "title": "Pasarela: notas de integración", "line": "El proveedor firma cada evento con HMAC. Las decisiones vigentes están en «Pagos: decisiones»." },
    { "title": "Pagos: decisiones", "line": "La firma se verifica antes de leer el cuerpo." }, … ],
  "docsTotal": 4 }
```

Si ninguna cosa tiene todas las palabras, trae lo que tiene más y lo dice (`every_word: false`), en vez de no traer nada. `brain` sin nada muestra tus listas, carpetas y etiquetas con sus cuentas.

### 3. Mira el proyecto entero

Con `part_of`, `brain` agrupa las partes abiertas de una tarea según cómo van. Una parte cuyo estado empieza con «Bloqueada:» cuenta como bloqueada.

```json
> brain {"part_of": "…Pagos con la pasarela nueva…"}
{ "stages": { "blocked": 1, "under_way": 0, "waiting": 0, "not_started": 3 },
  "parts": [
    { "title": "Reembolsos", "stage": "blocked", "steps": [0, 2],
      "state": "Bloqueada: el proveedor habilita los reembolsos parciales en el ambiente de prueba recién el lunes." },
    { "title": "Webhook de confirmación", "stage": "not_started", "steps": [0, 3] }, … ] }
```

### 4. Lee la tarea, y del documento solo lo que importa

`read` trae una tarea con su estado primero. Una tarea que todavía no tiene uno llega con uno armado a partir de ella:

```json
> read {"task": "…Webhook de confirmación…"}
{ "state": { "built": true, "done": 0, "of": 3 },
  "steps": [ { "text": "Verificar la firma", "done": false }, { "text": "Reintentos", "done": false }, … ] }
```

La ficha de un documento dice qué tiene y dónde, sin leerlo: sus secciones con sus líneas, quién lo menciona y, en una sección de decisiones, las vigentes primero y la tachada al final, marcada como reemplazada.

```json
> brain {"doc": "…Pagos: decisiones…"}
{ "decisions": [
    { "line": 6, "said": "El webhook se reintenta tres veces, con espera creciente." },
    { "line": 7, "said": "La firma se verifica antes de leer el cuerpo." },
    { "line": 5, "said": "Reintentar el webhook sin límite", "replaced": true } ],
  "mentioned_by": [ { "title": "Pasarela: notas de integración", "kind": "doc" }, … ], "mentioned_by_more": 15,
  "outline": [ { "title": "Decisiones", "line": 3, "to": 8 }, { "title": "Fuera de alcance", "line": 10, "to": 14 } ] }
```

La ficha de un documento o de una tarea trae también lo que Tisty cree que se parece, aparte, en `guessed`: nadie lo enlazó, así que el agente lee los dos antes de unir o fusionar nada.

```json
> brain {"task": "…Pedir presupuesto del techo…"}
{ "guessed": [ { "id": "…", "kind": "doc", "title": "Arreglos de la casa", "state": "open", "score": 0.42, "shared": ["presupuesto", "techo"], "method": "words" } ] }
"Guessed alike by Tisty, linked by no one: \"Arreglos de la casa\" […] (open) (shares presupuesto, techo)."
```

En `shared`, una palabra sola la escribieron los dos. Una pareja `a≈b` es una palabra que cada lado escribió distinto, y `method` dice cuál de las tres pistas los unió: `words`, palabras en común; `typo`, un error de tipeo o un plural, como `impresora≈imrpesora` o `vacuna≈vacunas`, y `dictionary`, una traducción del diccionario español–inglés de Tisty, como `pagar≈pay`. La frase de abajo dice lo mismo con palabras: «shares», «spelled almost the same:» o «alike by translation:». Así el agente sabe que «Pagar la cuenta del agua» y «Pay the water bill» se parecen por el idioma, no porque alguien las haya unido, y las lee antes de decidir si son la misma tarea:

```json
> brain {"task": "…Pagar la cuenta del agua…"}
{ "guessed": [ { "id": "…", "kind": "task", "title": "Pay the water bill", "state": "open", "score": 0.61, "shared": ["pagar≈pay", "agua≈water"], "method": "dictionary" } ] }
"Guessed alike by Tisty, linked by no one: \"Pay the water bill\" […] (open) (alike by translation: pagar≈pay, agua≈water)."
```

Los parecidos salen solo de la prosa: el código, el HTML, las rutas, las notas de avance de una tarea y los documentos de Agentes no cuentan. Lo cuenta [[Parecidos|Red-de-Brainsty#parecidos]].

Con la línea de cada sección, el agente lee solo la que necesita, con la misma herramienta que lee una tarea: `read` con `doc` trae el documento entero, una sección (`section`), unas líneas (`from`, `to`) o un trozo a la vez (`chars`). Con `marks`, trae solo lo que el documento resalta o pone en negrita, agrupado por sección: en un documento de 24 KB, unos 2.300 bytes. Con `versions`, en vez del texto, lista sus versiones anteriores en este equipo, y `version` lee una, bajo un aviso de que no es lo que dice ahora; `restore_doc` con esa `version` vuelve a ella. Nunca vacía el historial ni lee las versiones de tu Diario. Lo cuentan [[Versiones anteriores|Documentos#versiones-anteriores]] y [[Herramientas MCP|Herramientas-MCP]].

**Releer cuesta casi nada.** Cada lectura trae un `print`. Si el agente vuelve a leer con ese `print` y nada cambió, Tisty le responde solo eso, con la hora de su lectura anterior. La respuesta tiene esta forma (este extracto no viene de la sesión de arriba, sino del código de Tisty):

```json
> read {"task": "…", "print": "…"}
{ "id": "…", "status": "open", "unchanged": true, "print": "…" }
"Nothing on \"Webhook de confirmación\" changed since you read it, 22:22: what you hold still stands. Without `print`, `read` brings it whole again."
```

Medido sobre una tienda de 5.000 tareas, releer una tarea así cuesta 336 bytes en vez de 4.360, y un documento entero 332 en vez de 9.321. Si algo cambió, un paso marcado o una línea nueva, trae todo de nuevo. Una sección siempre llega con su texto.

### 5. Marca y deja el estado en una llamada

Al terminar un paso, el agente lo marca y escribe el estado de la tarea en la misma llamada: lo hecho, lo que falta, las decisiones vigentes y lo descartado.

```json
> tick {"task": "…", "steps": ["Verificar la firma"],
        "state": "**Hecho:** la firma HMAC se verifica antes de leer el cuerpo…\n**Falta:** los reintentos y el registro de cada evento.\n**Decisiones vigentes:** tres reintentos con espera creciente.\n**Descartado:** reintentar sin límite."}
{ "ticked": 1, "left": 2, "next": "Reintentos", "stated": true,
  "said": "Ticked 1 step(s) on \"Webhook de confirmación\". 2 still unticked; next: \"Reintentos\". Left its state on its page, under «Estado»." }
```

En la página de la tarea aparece la sección **Estado**, entre los pasos y las notas, con la fecha y el nombre del agente. La lees tú igual que él. Cada escritura la reemplaza entera, así que nunca crece.

- Si escribiste una línea tuya bajo **Estado**, el agente tiene que conservarla: un estado que la deja fuera se rechaza y no marca nada.
- Si después cambian los pasos o llegan notas, `read` avisa que el estado quedó viejo: «older than 3 later note(s)».
- El estado tiene lugar para unos 3.000 caracteres.

### 6. Anota solo lo que cambia algo

La bitácora queda para lo que cambia algo: una decisión, algo descartado o un bloqueo. Tisty nunca le pide «anota tu avance»: eso ya está en el estado. Cuando una nota deja sin efecto otra anterior del agente, la nueva lo dice con `replaces`, y la vieja queda en la historia, atenuada y marcada como reemplazada, y deja de llegar primero.

### 7. Lo que nombra queda enlazado

```json
> note {"task": "…", "body": "Los reintentos van en la cola de trabajos que ya existe, como dice «Pagos: decisiones». La conciliación espera a esta parte: 01M7RJAQKHGE785QFPK3NHK1CV"}
{ "references": { "linked": [
    { "said": "Pagos: decisiones", "doc": "…", "title": "Pagos: decisiones" },
    { "said": "01M7RJAQKHGE785QFPK3NHK1CV", "task": "…", "title": "Conciliación diaria" } ] } }
```

La nota queda con dos enlaces, el mismo que harías tú con `[[` y el de una tarea. Lo dudoso no se enlaza: se le devuelve al agente para que decida. Un título de una sola palabra, como «Auto», puede estar en una frase sin querer nombrar nada:

```json
{ "references": { "maybe": [ { "said": "Auto", "could_be": [ { "doc": "…", "title": "Auto" } ] } ] } }
```

Un id de tarea solo en su párrafo, en un documento, se vuelve la tarjeta de esa tarea. Los encabezados, las tablas, el código y las líneas que ya estaban no se tocan nunca.

### 8. Una imagen sin palabras recibe una pregunta

```json
> attach {"task": "…", "path": "…\\Pictures\\IMG_2041.jpg"}
{ "ask": { "said": "What does \"rina-ce960510.jpg\" show? The words in a picture's brackets are all an agent or a screen reader knows of it, and a file's name says nothing: write it between the brackets with `reword_note`…" } }
```

El nombre de un archivo no cuenta como descripción. Lo que el agente escribe entre los corchetes es lo único que otro agente, o un lector de pantalla, sabrá de esa imagen.

### 9. Si se le olvida, Tisty pregunta

El agente siguió con otras cosas: anotó en un documento y creó dos carpetas, sin volver a la tarea. La tercera respuesta trae la pregunta:

```json
> folder {"name": "Pruebas", "inside": "Trabajo"}
{ "ask": { "title": "Webhook de confirmación", "left": 2,
  "said": "Is \"Webhook de confirmación\" up to date? You wrote 3 other things without writing on it, and 2 of 3 steps are still unticked, the next \"Reintentos\". Leave it in one `tick`…" } }
```

Pregunta una vez por tramo: a los diez minutos sin una palabra en la tarea, o a la tercera escritura en otra parte. Vuelve a preguntar solo después de que el agente escribe en esa tarea.

### 10. Al terminar el turno

Con los hooks, cuando un turno editó archivos y no escribió nada en Tisty, el turno vuelve al agente una vez:

```json
{ "decision": "block",
  "reason": "You edited files this turn and wrote nothing in Tisty. If this was work on a Tisty task, tick the steps you finished; if the work is done, `say_done` with the proof. A decision, something ruled out or a block goes in `note`. If it was not work on a task, end your turn without writing anything." }
```

Si el agente lo deja pasar, el hook calla hasta que vuelva a llamar a Tisty: un turno fuera de una tarea cuesta como mucho un turno extra.

Al final de la sesión, `brain` con `part_of` ya muestra «Webhook de confirmación» **en curso**, con 1 de 3 pasos y la primera línea de su estado.

## Lo que ves tú

- **En la página de la tarea**, la sección **Estado** con la fecha y el agente, y en las notas los enlaces a «Pagos: decisiones» y a «Conciliación diaria».
- **En el documento**, «Pasarela: notas de integración» muestra la tarjeta de la parte que el agente citó.
- **En Enlazado desde**, «Pagos: decisiones» ahora nombra la tarea.
- **En la red de Brainsty**, las nuevas líneas entran a la base a los pocos segundos, y las ves la próxima vez que abres la red.
- **Nada se cierra sin ti.** Cuando el agente dice que una tarea está hecha, espera en **Por confirmar**.

## Las herramientas que pasaron a `brain` y a `read`

| Antes | Ahora |
| :-- | :-- |
| `find` | `brain` con `query`, o con los campos para filtrar (`list`, `tag`, `from`, `to`, `by_agent`, `said_done`, `part_of`, `scope`); `source` sigue sirviendo para ver si algo ya se anotó |
| `docs` | `brain` con `folder`, o sin nada para ver las carpetas con sus cuentas |
| `outline_doc` | `brain` con `doc`: la ficha, con el índice, el `print` para editar por sección y quién lo menciona |
| `lists`, `tags` | `brain` sin nada |
| `read_doc` | `read` con `doc`, con las mismas partes (`section`, líneas, `chars`, `as`) y además `marks`, `versions` y `version` |

Un asistente que todavía llama a una de ellas recibe dónde quedó («`find` moved into `brain`», «`read_doc` moved into `read`») y no pasa nada más. Cuando todas las filas de una lista comparten un campo, como la lista o la tarea de la que son partes, llega una sola vez, en `each`.

## Lo que conviene saber

- **Asistentes sin hooks.** Codex y Claude Desktop no corren hooks: les llega solo la pregunta en las respuestas, y solo cuando llaman a Tisty.
- **Trabajo que no edita archivos**, como planificar o investigar, no lo ve el hook del final del turno: ahí queda la pregunta en las respuestas.
- **GitHub Copilot** puede gastar una solicitud premium en el turno que se le devuelve.
- **Cursor** también corre los hooks de Claude Code; aunque se disparen los dos, el turno vuelve una sola vez.
- **Gemini CLI** corre los hooks solo en las carpetas en que confiaste.
- **Lo oculto queda fuera.** Una tarea que ocultaste o descartaste nunca se nombra: ni en una búsqueda, ni como vecina en una ficha, ni como parecido, ni en el inicio de una sesión. Tu Diario, mientras no lo abras a los agentes, tampoco.
- **Donde la enlazaste, lee «a task out of sight».** Si un documento cita o menciona una de esas tareas, el agente lo lee con esas palabras en lugar del enlace, sin título ni id. Si después reescribe el documento, Tisty pone de vuelta tu enlace tal como estaba, byte a byte: lo oculto no se pierde porque el agente no lo vio.
- **En la carpeta Agentes solo propone.** Lee tus reglas y skills como cualquier documento, pero un cambio suyo queda como propuesta hasta que lo apruebas en la ficha. Lo cuenta [[Una propuesta de tu agente|Carpeta-AGENTS#una-propuesta-de-tu-agente]].
- **Con la puerta cerrada**, las herramientas rechazan todo y los hooks callan.
- **Lo que cuesta.** Una nota tiene lugar para 4.000 caracteres y el estado para unos 3.000; si los enlaces la pasan de ese largo, queda sin enlazar en vez de rechazarse. El inicio con hooks pesa unos 2.000 caracteres.

## Para seguir

- [[Asistentes y MCP|Asistentes-y-MCP]]: conectar un asistente, los pedidos listos y lo que cuesta.
- [[Herramientas MCP|Herramientas-MCP]]: la lista completa.
- [[La red de Brainsty|Red-de-Brainsty]]: la misma red, dibujada para ti.
- [[Tareas en documentos|Tareas-en-documentos]]: las tarjetas que también deja tu agente.
