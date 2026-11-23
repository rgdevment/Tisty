# Hooks y skills

Conectar un agente le da las herramientas de Tisty, pero no la costumbre de usarlas: al empezar no sabe dónde quedó, y al terminar se le olvida anotar. Para eso armé dos piezas más, las dos opcionales y cada una con su propio sí. Los **hooks** le avisan a Tisty cuándo empieza y cuándo termina cada turno de tu agente. Las **skills de Tisty** le enseñan a tu agente a trabajar con Tisty: retomar, ordenar y dejar el trabajo listo para el próximo.

Ninguna de las dos es obligatoria. Sin ellas, tu agente usa las mismas herramientas cuando se lo pides, y Tisty le pregunta igual si dejó su tarea al día. Y si prefieres trabajar sin agente, nada de esta página te hace falta: tus tareas y tus documentos funcionan igual, a mano.

Cómo conectar un agente está en [[Agentes y MCP|Agentes-y-MCP]]; una sesión llamada por llamada, en [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

## En corto

| | Hooks | Skills de Tisty |
| :-- | :-- | :-- |
| **Qué son** | Comandos que tu agente corre solo al empezar una sesión, después de usar una herramienta y al terminar un turno | Tres carpetas con un `SKILL.md` que tu agente lee cuando lo que le pides calza con ellas |
| **Qué hacen** | Le traen al agente sus tareas abiertas al empezar y le devuelven el turno una vez si editó archivos sin escribir nada en Tisty | Le enseñan a retomar donde quedó, a ordenar lo que guardas sin borrar nada y a escribir para el próximo agente |
| **Qué agentes** | Claude Code, Cursor, Gemini CLI y GitHub Copilot | Claude Code, Cursor, Codex, GitHub Copilot y Gemini CLI |
| **Dónde se encienden** | **Configuración › Agentes › Conectados**, bajo la fila de cada agente | **Configuración › Agentes › Conectados**, en la fila **Skills de Tisty** |
| **El sí** | Uno por agente, con el archivo que se va a escribir nombrado | Uno para todo el equipo, con las carpetas que se van a escribir nombradas |
| **Cómo se quitan** | **Quitar**, en la misma fila | **Apagar en este equipo**, en su ficha. Nunca se eliminan: siguen en Tisty |
| **Con el plugin de Tisty** | En Claude Code, los trae el plugin | En Claude Code, las trae el plugin |

Las dos valen solo en el equipo donde las enciendes: en otro equipo lo decides allá.

## Los hooks

### Qué hacen

- **Al empezar una sesión**, el agente recibe las tres últimas tareas abiertas en que escribió ese mismo agente en este equipo, la más nueva primero, con cuántos pasos lleva cada una y el siguiente, lo que se movió desde entonces y lo que apremia. Son unos 2.000 caracteres, y llegan aunque el agente no llame a Tisty.
- **Después de cada edición y de cada llamada a Tisty**, el hook deja una marca de esa sesión en la caché de este equipo, sin leer la conversación.
- **Al terminar un turno que editó archivos y no escribió nada en Tisty**, le devuelve el turno una vez: si eso era trabajo de una tarea, que lo deje en un solo `tick`. Si el agente lo deja pasar, el hook calla hasta que vuelva a llamar a Tisty, así que un turno fuera de una tarea cuesta como mucho un turno extra.

Lo que no edita archivos, como planificar o investigar, el hook del final no lo ve: ahí queda la pregunta que Tisty hace en sus propias respuestas, que llega a cualquier agente.

### Dónde se escriben

| Agente | Archivo |
| :-- | :-- |
| **Claude Code** | `~/.claude/settings.json` |
| **Cursor** | `~/.cursor/hooks.json` |
| **Gemini CLI** | `~/.gemini/settings.json` |
| **GitHub Copilot** (su línea de comandos y las sesiones de Copilot en VS Code) | un archivo propio, `~/.copilot/hooks/tisty.json` |

Tisty escribe solo sus propias entradas y deja el resto del archivo como está, con los mismos respaldos `.before-tisty` que la conexión.

### Agregarlos

1. Abre **Configuración › Agentes › Conectados**.
2. Bajo la fila de un agente conectado, como **Claude Code**, la fila **Hooks: al empezar y al terminar cada turno** dice **Sin agregar**. Pulsa **Agregar**.
3. Tisty te dice qué va a hacer antes de tocar nada: «Tisty agregará sus hooks en `~/.claude/settings.json`. Al empezar una sesión, el agente recibe las tareas que dejó abiertas y lo que se movió…», con **Agregarlos** y **Ahora no**.
4. Con **Agregarlos**, la fila dice **Agregados**, y debajo «Agregados. Los usa desde su próxima sesión.».

No hace falta reiniciar el agente para los hooks: los toma en su próxima sesión.

### Quitarlos o apagarlos

- **Quitar**, en la misma fila, saca solo las entradas de Tisty: «Quitados. Su próxima sesión corre sin ellos.».
- **Recordarle al empezar y al terminar cada turno**, en **Configuración › Agentes › Puerta**, los apaga para todos sin quitarlos de ningún archivo. Sirve para una tarde en que no quieres que te devuelvan turnos.
- **Quitarle la voz**, en la misma **Puerta**, cierra la puerta: con ella cerrada, los hooks callan.

### Si el plugin de Tisty ya los trae

El plugin de Tisty para Claude Code trae sus propios hooks, en su propio archivo, desde la versión 1.4.0. Si Tisty sabe que está instalado, sea cual sea su versión, la fila de hooks de Claude Code dice **Los trae el plugin de Tisty**, sin botón **Agregar**, para que no corran dos veces.

Si ya tenías los de Tisty y después instalas el plugin, la fila te avisa: «El plugin de Tisty también los trae, así que corren dos veces. Quita estos para dejar los del plugin.», con **Quitar**.

Tisty sabe del plugin solo después de un sí en este equipo, el de las [skills de Tisty](#las-skills-de-tisty) o el de la carpeta [[Agentes|Carpeta-AGENTS]]: el registro del plugin es un archivo de tu agente, y antes de ese sí no se lee. Sin ninguno de los dos, la fila ofrece **Agregar** aunque el plugin esté; si lo agregas, el aviso de arriba aparece en cuanto des uno de esos sí.

## Las skills de Tisty

Una skill es una carpeta con un `SKILL.md`: tu agente lee su descripción siempre, y el resto solo cuando lo que le pides calza con ella. Tisty trae tres, dentro de la app:

| Skill | Qué le enseña a tu agente | Pídele a tu agente |
| :-- | :-- | :-- |
| `tisty` | Retomar donde quedó, trabajar las tareas, los documentos y las ideas, y dejar la tarea lista para el próximo | «Retoma donde quedamos» · «Deja esto listo para el próximo agente» |
| `consolidate` | Ordenar lo guardado: mira, te propone cada arreglo numerado y hace solo los que eliges. Nunca borra | «Ordena mi Tisty» |
| `writing-for-the-next-agent` | Escribir una página, el estado de una tarea o un archivo de agente para que el próximo lo entienda sin la conversación. Adaptada de mattpocock/skills, con su licencia MIT, que viaja con ella | «Deja esta página clara para el próximo agente» |

Las frases son las mismas que muestra su ficha, en la línea **Pídele a tu agente:**. No tienen que ser exactas: «sigamos con lo de ayer» o «¿en qué íbamos?» también llevan a la skill `tisty`.

**Cada agente retoma lo suyo.** «Retoma donde quedamos» lleva a un agente a lo último que dejó él, no otro: lo que abre la sesión y lo que `catch_up` llama `last` son de cada programa. Lo que pasa de un agente a otro es lo escrito en la tarea, así que, para seguir lo que dejó otro, nómbrala por su título.

**No son los pedidos listos.** Tisty también le entrega a tu agente [ocho pedidos](Agentes-y-MCP#pedidos-listos-y-menciones-con-) que se usan como comandos, como `/mcp__tisty__pick_up` o `/mcp__tisty__consolidate Cocina` en Claude Code. Un pedido lo escribes tú, y llega a cualquier agente que tome pedidos de un servidor; una skill la elige tu agente cuando le hablas con tus palabras. Hacen el mismo trabajo, y puedes usar los dos.

**No son documentos tuyos.** Viven dentro de la app, no en tu almacén: no viajan a tus otros equipos, no salen en una búsqueda, en [[Brainsty|Red-de-Brainsty]], en el historial ni en una exportación, y ningún agente puede nombrarlas ni cambiarlas por el MCP.

### Dónde van

| Agente | Carpeta |
| :-- | :-- |
| **Claude Code** | `~/.claude/skills/tisty/`, `~/.claude/skills/consolidate/`, `~/.claude/skills/writing-for-the-next-agent/` |
| **Cursor** | `~/.cursor/skills/…`, con las mismas tres carpetas |
| **Codex**, **GitHub Copilot** y **Gemini CLI** | `~/.agents/skills/…`, una sola copia que leen los tres |

`~` es tu carpeta de usuario: `C:\Users\tú` en Windows y `/Users/tú` en macOS. La ficha muestra solo las carpetas de los agentes que tienes en este equipo. La compartida dice «Codex · Copilot · Gemini» y lo advierte: «Codex, Copilot y Gemini comparten esta carpeta: encenderlas o apagarlas aquí vale para los tres.».

### Encenderlas

1. Abre **Configuración › Agentes › Conectados**. Al final de la lista está la fila **Skills de Tisty**: «Tres skills que les enseñan a tus agentes a trabajar con Tisty. Tienen su propio sí, en este equipo.», con su estado, **Apagadas**, y **Ver**.
2. **Ver** abre su ficha. Arriba, «Tisty trae tres skills que les enseñan a tus agentes a trabajar con él. Vienen dentro de la app: no las escribe ningún agente y nadie puede cambiarlas por el MCP.». Debajo, cada skill con su línea y su **Pídele a tu agente:**, y **Dónde van en este equipo**, con una fila por carpeta.
3. Lee el sí, al pie: «Para encenderlas, Tisty escribe solo una carpeta por skill en las carpetas de arriba, y lee los archivos que ya hay en esas carpetas de skill para no escribir nunca sobre uno tuyo. En Claude Code lee además el registro de su plugin (installed_plugins.json y settings.json) para no copiarlas dos veces. No lee reglas, memoria ni otros archivos de tus agentes, y no necesita la carpeta Agentes.».
4. Pulsa **Encender en este equipo**. Cada carpeta pasa a **Al día**, y la fila de Configuración dice **En este equipo**.

Al pie de la ficha, «De Tisty 1.50.0» dice de qué versión son las que tienes. Tu agente las ve desde su próxima sesión.

**No necesitan la carpeta Agentes.** Si nunca abriste [[Agentes|Carpeta-AGENTS]], o la tienes sin sincronizar, las skills llegan igual a todos los agentes de este equipo: su sí es otro y más chico. Tampoco hay que conectar los agentes por MCP para que las lean, aunque sin conexión no tienen con qué trabajar.

### Lo que dice cada carpeta

| La ficha dice | Qué pasa |
| :-- | :-- |
| **Al día** | La copia de esa carpeta es la de esta versión de Tisty |
| **Por escribir** | Encendidas, se escribe en la próxima vuelta, como al abrir una versión nueva |
| **Una skill que no es de Tisty: se deja como está** | En esa carpeta ya hay una skill con el mismo nombre, o un archivo que Tisty no escribió. Tisty no deja nada suyo ahí: esa skill entera queda fuera |
| **Cambiada fuera de Tisty: se deja como está** | Editaste o borraste un archivo de la copia de Tisty. Se queda como la dejaste, y ya no se actualiza |
| **La carpeta es un enlace: se deja como está** | La carpeta, alguna de las que la contienen o un archivo suyo es un enlace simbólico o un junction. Tisty no escribe a través de él |
| **No se puede leer: se deja como está** | Tisty no pudo leer o escribir un archivo de esa carpeta |

Antes de encenderlas, una carpeta no muestra estado.

Si ya tienes en la carpeta **Agentes** una skill tuya con el mismo nombre, como tu propia `consolidate`, la ficha lo dice en lugar de su línea: «Ya tienes en Agentes una skill llamada consolidate, así que la de Tisty no se agrega.». La tuya gana, en todos los agentes, si este equipo ya dio su sí en Agentes; sin él, la cuida la carpeta: una skill que no es de Tisty se deja como está.

### Se actualizan solas

Cuando actualizas Tisty, la próxima vez que lo abres escribe la versión nueva de cada skill en las carpetas que están **Al día**. Una copia que cambiaste, o una carpeta que no es de Tisty, no se toca. La ficha lo dice: «Se actualizan cuando actualizas Tisty, y nunca pisan una skill tuya con el mismo nombre.».

### Apagarlas

**Apagar en este equipo**, en la misma ficha. Antes te dice qué hace: «Apagarlas saca de las carpetas de tus agentes las copias intactas de Tisty, con respaldo, y las skills siguen en Tisty, listas para encenderlas otra vez. Una copia que cambiaste se queda en esta ficha.».

- **Apagar no es borrar.** Las skills siguen dentro de Tisty, y **Encender en este equipo** las vuelve a poner.
- **Una copia que cambiaste** no se saca, y con ella se queda su skill entera en esa carpeta, aunque solo cambie un archivo: queda listada en la ficha, para que la quites tú si quieres.
- **Nada las elimina.** No hay **Eliminar** ni en la ficha ni en la carpeta Agentes, y ningún agente puede quitarlas por el MCP.

### En la carpeta Agentes

Si usas la carpeta [[Agentes|Carpeta-AGENTS]], **Agentes › Global** muestra una fila fija, **Skills de Tisty**, con el mismo estado que en Configuración. Un clic abre la misma ficha.

- **Ocultar.** Al pasar el puntero sobre la fila aparece **Ocultar**, que la saca de Agentes en este equipo. Las skills siguen como estaban, encendidas o apagadas. En Configuración, la fila dice entonces «Ocultas de Agentes en este equipo; siguen como están.», con **Mostrar en Agentes**.
- **No tiene menú.** Ni **Eliminar**, ni **Pausar**, ni **Documento nuevo**: no es un documento.
- **Traer no las trae.** Si encendiste las skills antes de abrir Agentes, la lista de la primera vez, **Traer otro archivo** y **Buscar de nuevo** muestran sus copias como **propio de Tisty**, y no se traen: no entran como documentos tuyos.
- **La pausa de Agentes no las alcanza.** Pausar un agente, o dejar de sincronizar toda la carpeta, vale para tus documentos. Las skills de Tisty siguen su propio interruptor: encendidas, llegan y se actualizan en todos los agentes del equipo. La ficha lo recuerda: «Solo en este equipo: en otro equipo lo decides allí. La pausa de Agentes no las alcanza: tienen su propio interruptor.».
- **Los dos sí son independientes.** Dejar de sincronizar Agentes no apaga las skills, y encender las skills no trae nada a Agentes.

### Con el plugin de Tisty

- **Claude Code.** El plugin de Tisty trae las mismas tres skills. Si está instalado para tu usuario y no lo desactivaste, Tisty no las copia a `~/.claude/skills`, y la ficha lo dice: «Claude Code ya las tiene por el plugin de Tisty, así que Tisty no las copia ahí.». Codex, Copilot, Gemini y Cursor las siguen recibiendo.
- **Llega el plugin después.** Las copias que Tisty había dejado en `~/.claude/skills` aparecen en la ficha bajo «Copias que sobran: Tisty ya no las escribe ahí. Si quitas una, queda un respaldo.», cada ruta con **Quitar el archivo**. Nunca se quitan solas.
- **Cursor.** Su plugin no deja un registro que Tisty pueda leer, así que la ficha siempre avisa: «Si instalaste el plugin de Tisty en Cursor, Cursor mostrará cada skill dos veces: Tisty no puede saberlo.». Si te pasa, deja una sola de las dos: el plugin en Cursor o las skills de Tisty.

## Al conectar un agente

Cuando conectas un agente desde **Configuración › Agentes › Conectados**, con **Conectar** o con **Conectar los 3 detectados**, su fila muestra un recuadro, una sola vez. También sale al conectar con **Corregir** o **Conectar igual**, si ese agente no lo había visto, y no sale en los agentes que corren dentro de WSL:

> **Claude Code ya está conectado. Esto lo ayuda a trabajar con Tisty:**
>
> Con sus hooks, Claude Code le avisa a Tisty al empezar y al terminar cada turno.
> **Hooks: al empezar y al terminar cada turno** · Sin agregar · **Agregar**
>
> Las skills de Tisty le enseñan a Claude Code a retomar, ordenar y dejar el trabajo listo para el próximo agente. **Ver las skills de Tisty**
>
> **Ahora no**

- **Agregar** pide el mismo sí que en la fila de siempre, con el archivo nombrado.
- **Ver las skills de Tisty** abre la ficha, con **Encender en este equipo**.
- **Ahora no** cierra el recuadro.

Se ofrece una vez por agente y por equipo: cuenta desde que aparece, aunque salgas de Configuración sin responder.

Cambia según el agente y lo que ya tienes:

| Si… | El recuadro |
| :-- | :-- |
| ya diste uno de los dos sí y el plugin de Tisty ya trae los hooks | dice «Sus hooks ya los trae el plugin de Tisty.» |
| ya diste uno de los dos sí y el plugin ya trae las skills | dice «Claude Code ya tiene las skills de Tisty por el plugin de Tisty.», sin botón |
| las skills ya están encendidas | dice «Las skills de Tisty ya están en este equipo.», sin botón |
| el agente es Codex | trae solo la línea de las skills: Codex no tiene hooks |
| el agente no tiene ni hooks ni skills, como Claude Desktop | no aparece |

**Si ya tenías el agente conectado** antes de la 1.50, el recuadro no aparece: se ofrece solo al conectar. Usa las filas de siempre, la de hooks bajo el agente y **Skills de Tisty** › **Ver**, al final de la lista. Es el mismo sí.

## Un caso de punta a punta

Yisset usa Codex, que tiene conectado a Tisty desde hace meses. Hoy actualiza Tisty a la 1.50 e instala Claude Code en el mismo equipo.

### 1. Conecta Claude Code

En **Configuración › Agentes › Conectados**, Claude Code aparece **Sin conectar**. Yisset pulsa **Conectar**, y bajo la fila aparece el recuadro «Claude Code ya está conectado. Esto lo ayuda a trabajar con Tisty:».

### 2. Agrega sus hooks

Pulsa **Agregar** en la fila de hooks. Tisty le muestra el archivo, `~/.claude/settings.json`, y lo que harán los hooks. Con **Agregarlos**, la fila dice **Agregados**.

### 3. Enciende las skills de Tisty

Pulsa **Ver las skills de Tisty**. La ficha muestra las tres skills y dos carpetas en **Dónde van en este equipo**: la de Claude Code, `~/.claude/skills`, y la compartida, `~/.agents/skills`, para Codex. Lee el sí y pulsa **Encender en este equipo**. Las dos carpetas pasan a **Al día**. Cierra la ficha, y en el recuadro, **Ahora no**.

Como Claude Code lee su configuración al abrirse, lo cierra del todo y lo abre otra vez.

### 4. Le encarga un trabajo

Yisset le pide a Claude Code que planifique la exportación del reporte. El agente la anota como una tarea con sus pasos:

```json
> propose {"title": "Exportar el reporte mensual a CSV", "list": "Trabajo",
           "steps": ["Leer el formato que pide contabilidad", "Escribir la exportación",
                     "Probar con los datos de octubre", "Agregar el botón en la pantalla de reportes"]}
```

Esa tarde lee el formato, marca el primer paso y deja el estado: separador punto y coma y fechas `2026-10-31`, como pide contabilidad.

### 5. «Retoma donde quedamos»

Al otro día, al abrir la sesión y sin que nadie lo pida, el hook le entrega a Claude Code las tareas abiertas en que escribió él en este equipo. Entre lo demás, le dice:

```text
Your last open tasks here, newest first:
- "Exportar el reporte mensual a CSV" in Trabajo […], last written 2026-11-20 18:05: under way, 1 of 4 steps ticked, next "Escribir la exportación"; state: **Hecho:** el formato de contabilidad está leído…
Pressing for the person: 1 today, 3 late.
```

Yisset escribe «Retoma donde quedamos». La frase calza con la skill `tisty`, que le dice al agente que empiece por `catch_up` y abra la tarea en que estaba:

```json
> catch_up {}
{ "last": { "title": "Exportar el reporte mensual a CSV", "list": "Trabajo", "at": "2026-11-20 18:05",
            "stage": "under_way", "steps": [1, 4], "state": "**Hecho:** el formato de contabilidad está leído…" },
  "counts": { "open": 31, "late": 3, "today": 1, "docs": 48, … } }
```

```json
> read {"task": "…Exportar el reporte mensual a CSV…"}
{ "steps": [ { "text": "Leer el formato que pide contabilidad", "done": true },
             { "text": "Escribir la exportación", "done": false }, … ] }
```

El estado llega primero, con lo decidido el día antes. Claude Code le responde a Yisset dónde quedaron y sigue con «Escribir la exportación».

### 6. Trabaja y marca

Al terminar la exportación, la prueba con octubre y marca los dos pasos y el estado en una sola llamada:

```json
> tick {"task": "…", "steps": ["Escribir la exportación", "Probar con los datos de octubre"],
        "state": "**Hecho:** la exportación escribe el CSV con punto y coma; octubre cuadra con el total del sistema.\n**Falta:** el botón en la pantalla de reportes.\n**Decisiones vigentes:** fechas 2026-10-31, montos sin separador de miles."}
{ "ticked": 2, "left": 1, "next": "Agregar el botón en la pantalla de reportes", "stated": true,
  "said": "Ticked 2 step(s) on \"Exportar el reporte mensual a CSV\". 1 still unticked; next: \"Agregar el botón en la pantalla de reportes\". Left its state on its page, under «Estado»." }
```

Si se le hubiera olvidado, el hook se lo recuerda al final del turno, porque editó archivos sin escribir nada en Tisty:

```json
{ "decision": "block",
  "reason": "You edited files this turn and wrote nothing in Tisty. If this was work on a Tisty task, leave it in one `tick`…" }
```

### 7. «Deja esto listo para el próximo agente»

Yisset tiene que irse. Escribe «Deja esto listo para el próximo agente». La skill `tisty` le pide al agente dejarlo en la tarea, no en la conversación, que se archiva: el estado al día, con `tick`, y una nota con dónde quedó el trabajo, lo que descartó, el siguiente paso y la rama o los archivos donde vive.

```json
> note {"task": "…", "body": "Descartado: exportar con la librería de planillas; mueve los montos a texto y contabilidad no los suma. Sigue el botón, junto a «Imprimir», como dice «Reportes: decisiones». El código está en la rama feat/monthly-csv-export."}
{ "references": { "linked": [ { "said": "Reportes: decisiones", "doc": "…", "title": "Reportes: decisiones" } ] } }
```

Termina diciéndole a Yisset qué dejó y dónde: los pasos marcados, el estado, la nota y la rama.

### 8. Lo que ve Yisset en Tisty

- **En la lista Trabajo**, la fila de la tarea dice **3/4 · ✎1** y «por un agente»; al pasar el puntero, «Claude Code escribió en este equipo».
- **En su página**, la sección **Estado**, con la fecha y «Claude Code», dice lo hecho, lo que falta y lo decidido. Encima, a la derecha, **↓ Bitácora · 1 nota** la lleva a la nota del agente, con el enlace a «Reportes: decisiones».
- **Nada se cerró.** Falta un paso, y cuando el agente diga que la tarea está hecha, esperará en **Por confirmar** hasta que Yisset la cierre.

Si la tarea la hubiera escrito Yisset, y ella hubiera marcado el primer paso, las marcas del agente serían su palabra: la fila diría **1/4 ◆2**, con «2 por confirmar» al pasar el puntero, hasta que ella las tome con **Marcarlos**. Lo cuenta [[Qué puede hacer un agente|Qué-puede-hacer-un-agente]].

### 9. Después, otro agente

Yisset vuelve a Codex. Codex no tiene hooks, así que nada le llega al empezar, y como ya estaba conectado antes de la 1.50 nunca vio el recuadro; pero lee la skill `tisty`, que Yisset encendió en el paso 3 y quedó en `~/.agents/skills`. «Retoma donde quedamos», a secas, lo llevaría a lo último que dejó **él**, así que Yisset nombra la tarea: «Retoma la exportación del reporte mensual».

```json
> brain {"query": "exportar reporte mensual"}
{ "matches": [ { "title": "Exportar el reporte mensual a CSV", "list": "Trabajo", "status": "open" } ] }
> read {"task": "…Exportar el reporte mensual a CSV…"}
```

Lee el estado y la nota de Claude Code, sabe que no debe volver a probar la librería de planillas y sigue con el botón. También habría llegado por lo que se movió, que `catch_up` le cuenta desde la última vez que él miró.

### 10. Semanas después, el plugin

Yisset instala el plugin de Tisty en Claude Code. La próxima vez que abre **Configuración › Agentes**:

- la fila de hooks de Claude Code dice «El plugin de Tisty también los trae, así que corren dos veces. Quita estos para dejar los del plugin.». Pulsa **Quitar**, y la fila queda en **Los trae el plugin de Tisty**;
- la ficha de las skills dice «Claude Code ya las tiene por el plugin de Tisty, así que Tisty no las copia ahí.», y lista bajo **Copias que sobran** los archivos que Tisty había dejado en `~/.claude/skills`. Pulsa **Quitar el archivo** en cada uno, y quedan sus respaldos. La carpeta compartida de Codex sigue **Al día**.

## Casos que conviene conocer

- **En otro equipo.** Los hooks y las skills se deciden en cada equipo: en el equipo del trabajo, la fila **Skills de Tisty** empieza **Apagadas** y los hooks **Sin agregar**, aunque en la casa estén encendidos. Un registro copiado de otro equipo no cuenta como sí de las skills.
- **Antes de cualquier sí**, el de las skills o el de Agentes, la ficha no abre nada dentro de las carpetas de tus agentes: solo mira cuáles existen y si siguen ahí las copias que Tisty escribió antes. Tampoco lee el registro del plugin. Con el sí de Agentes solo, lee ese registro y esas copias, para ver si cambiaron.
- **Con la puerta cerrada.** Las skills siguen en las carpetas de tus agentes, y tu agente puede leerlas, pero cada llamada a Tisty se rechaza y los hooks callan. **Quitarle la voz** no toca ninguna de las dos piezas.
- **Un agente en pausa en Agentes.** La pausa no alcanza a las skills de Tisty: siguen llegando y actualizándose en ese agente. Para que no las tenga ninguno, **Apagar en este equipo**.
- **Codex y Claude Desktop** no tienen hooks. Codex recibe las skills; Claude Desktop, ninguna de las dos, y le llega solo la pregunta en las respuestas de Tisty.
- **Gemini CLI** corre los hooks solo en las carpetas en que confiaste. Su extensión de Tisty también los trae, pero Tisty solo reconoce el plugin de Claude Code: si usas la extensión, no agregues además los hooks desde Configuración.
- **Cursor** también corre los hooks de Claude Code. Aunque se disparen los dos, Tisty devuelve el turno una sola vez.
- **GitHub Copilot** puede gastar una solicitud premium en el turno que se le devuelve.
- **Una build de desarrollo** de Tisty no trae skills. La ficha lo dice, «Esta build de Tisty no trae skills: solo las versiones instaladas.», y **Encender en este equipo** queda apagado.
- **Una skill tuya con el mismo nombre** en la carpeta de un agente siempre gana, y una en Agentes también, en un equipo que ya dio su sí en Agentes. La de Tisty queda fuera, sin pisar nada.
- **Si Tisty perdió su registro**, por ejemplo al reinstalar el sistema, al encender las skills reconoce como suyas las copias que son idénticas, byte a byte, a las de esta versión: vuelven a actualizarse y a salir al apagarlas. Si a una le falta un archivo, tiene uno distinto o uno tuyo al lado, esa skill entera sigue siendo tuya y no se toca.

## Lo que leen y lo que viaja

- **Los hooks** escriben solo sus entradas en el archivo nombrado. Sus marcas de cada sesión se quedan en la caché de este equipo y no se sincronizan. Lo que le entregan al agente al empezar, los títulos de algunas tareas, viaja adonde viaja ese agente.
- **Las skills** escriben solo una carpeta por skill. Leen los archivos de esas carpetas, para no pisar uno tuyo, y en Claude Code el registro de su plugin, solo para saber si ya las trae. Nada más de tus agentes se abre.
- **El sí de las skills** se guarda en `data/agents/` dentro de la carpeta de Tisty, en este equipo, con los respaldos de lo que Tisty quitó. No viaja con la sincronización, los respaldos ni los paquetes. Los hooks no guardan un sí aparte: se agregan al pulsar **Agregarlos**, después de leer en qué archivo se escriben, se quitan con **Quitar**, y Tisty sabe si están, leyendo ese archivo.

Todo, archivo por archivo, está en [[Privacidad|Privacidad]].

## Para seguir

- [[Agentes y MCP|Agentes-y-MCP]]: conectar un agente, la puerta, los pedidos listos y lo que cuesta.
- [[Cómo trabaja un agente|Cómo-trabaja-un-agente]]: una sesión entera, llamada por llamada.
- [[Qué puede hacer un agente|Qué-puede-hacer-un-agente]]: su palabra sobre tus tareas y **Por confirmar**.
- [[Agentes: reglas, skills y memoria|Carpeta-AGENTS]]: tus propias reglas y skills, escritas una vez para todos tus agentes.
- [[Herramientas MCP|Herramientas-MCP]]: cada herramienta que usan las skills.
