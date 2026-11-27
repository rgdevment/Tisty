# Hooks y skills

Conectar un agente le da las herramientas de Tisty, pero no la costumbre de usarlas: al empezar no sabe dónde quedó, y al terminar se le olvida anotar. Para eso armé dos piezas más, las dos opcionales y cada una con su propio sí. Los **hooks** ayudan a tu agente en algunos momentos: al empezar una sesión, al terminar un turno, después de un push o un merge, y antes de dejar algo fuera de Tisty. Las **skills de Tisty** le enseñan a tu agente a trabajar con Tisty: retomar, ordenar y dejar el trabajo listo para el próximo.

Ninguna de las dos es obligatoria. Sin ellas, tu agente usa las mismas herramientas cuando se lo pides, y Tisty le pregunta igual si dejó su tarea al día. Y si prefieres trabajar sin agente, nada de esta página te hace falta: tus tareas y tus documentos funcionan igual, a mano.

Cómo conectar un agente está en [[Agentes y MCP|Agentes-y-MCP]]; una sesión llamada por llamada, en [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

## En corto

| | Hooks | Skills de Tisty |
| :-- | :-- | :-- |
| **Qué son** | Comandos que tu agente corre solo al empezar una sesión, antes y después de algunas herramientas y al terminar un turno | Tres carpetas con un `SKILL.md` que tu agente lee cuando lo que le pides calza con ellas |
| **Qué hacen** | Le traen al agente sus tareas abiertas al empezar, le devuelven el turno una vez si editó archivos sin escribir nada en Tisty, le devuelven el turno una vez si dejó todos los pasos marcados sin decirla hecha, le preguntan después de un push o un merge si terminó un paso o la tarea, y le preguntan una vez antes de dejar un archivo de texto fuera de un repo | Le enseñan a retomar donde quedó, a ordenar lo que guardas sin borrar nada y a escribir para el próximo agente |
| **Qué agentes** | Claude Code, Codex, Cursor, Antigravity CLI, Gemini CLI y GitHub Copilot | Claude Code, Cursor, Codex, GitHub Copilot, Gemini CLI y DeepSeek Harness |
| **Dónde se encienden** | **Configuración › Agentes › Conectados**, bajo la fila de cada agente | **Configuración › Agentes › Conectados**, bajo la fila de cada agente o desde **Tisty en tus agentes**, al final |
| **Los dos a la vez** | **Tisty en tus agentes**, al final de **Conectados** o en **Agentes › Global**, y **Hooks y skills para tus agentes**, sobre las pestañas | Los mismos |
| **El sí** | Uno por agente, al conectarlo o en su interruptor, con el archivo que se va a escribir nombrado | Uno para todo el equipo, al conectar o en su interruptor, con las carpetas que se van a escribir nombradas |
| **Cómo se quitan** | El mismo interruptor: los saca todos de ese agente. Otro, en **Puerta**, los calla en todos tus agentes | El mismo interruptor, o **Apagar en este equipo**, en su ficha. Nunca se eliminan: siguen en Tisty |
| **Con el plugin de Tisty** | En Claude Code, los trae el plugin | En Claude Code, las trae el plugin |

Las dos valen solo en el equipo donde las enciendes: en otro equipo lo decides allá.

## Los hooks

### Qué hacen

- **Al empezar una sesión**, el agente recibe las tres últimas tareas abiertas en que escribió ese mismo agente en este equipo, la más nueva primero, con cuántos pasos lleva cada una y el siguiente, lo que se movió desde entonces y lo que apremia. Son unos 2.000 caracteres, y llegan aunque el agente no llame a Tisty.
- **Al terminar un turno que editó archivos y no escribió nada en Tisty**, le devuelve el turno una vez: si eso era trabajo de una tarea, que lo deje en un solo `tick`. Si el agente lo deja pasar, el hook calla hasta que vuelva a llamar a Tisty, así que un turno fuera de una tarea cuesta como mucho un turno extra.
- **Cuando un turno dejó todos los pasos de su tarea marcados** sin decirla hecha, le devuelve el turno una vez para que haga `say_done` con la prueba. Nunca con una tarea tuya.
- **Después de un `git push`, un `gh pr create` o un `gh pr merge`**, le pregunta si terminó un paso o la tarea. En Claude Code la pregunta llega en el acto; en los demás agentes, al terminar el turno.
- **Antes de dejar algo fuera**, si va a crear un archivo de texto fuera de un repo, o a proponer por su cuenta una tarjeta de tarea en la aplicación de Claude, le pregunta una vez si va en Tisty. Lo cuenta [Antes de dejar algo fuera](#antes-de-dejar-algo-fuera).

Qué miran y qué queda está en [[Privacidad]].

Lo que no edita archivos, como planificar o investigar, el hook del final no lo ve: ahí queda la pregunta que Tisty hace en sus propias respuestas, que llega a cualquier agente.

### Antes de dejar algo fuera

Un agente con Tisty delante a veces deja el encargo para otro en un `.md`, donde nadie lo va a buscar. Este hook se lo pregunta **antes**, una sola vez, y lo deja pasar si insiste.

| Qué va a dejar | Qué le nombra Tisty |
| :-- | :-- |
| Algo que hacer: un encargo, un plan, lo que sigue otro día | Una tarea: `propose`, o `note` y `tick` con `state` en la que ya existe |
| Algo que se vuelve a leer: un análisis, un contrato, una decisión | Un documento: `brain` y luego `append_doc`, `edit_doc` o `write_doc` |
| Una tarjeta de tarea por su cuenta | Una tarea: `propose` en su lista, o `note` |

**Mira archivos de texto nuevos**: `.md`, `.markdown`, `.txt`, `.rst`, `.adoc`, `.org`, y los que se llaman `HANDOFF`, `NOTES`, `TODO`, `PLAN`, `PROMPT` o `BRIEF`, sin extensión. Editar uno que ya existe no pregunta.

**No pregunta** por el código, ni por lo que va en un repo o en un proyecto (una carpeta con `.git`, `package.json`, `Cargo.toml` y similares), una bóveda de notas (`.obsidian`, `.logseq`), una carpeta temporal o de configuración, `node_modules`, ni un `README`, una `LICENSE`, un `AGENTS.md` o un `requirements.txt`. Tampoco por los recordatorios, la lista de pasos del turno ni los subagentes.

| Agente | Archivo nuevo | Terminal | Tarjeta |
| :-- | :-- | :-- | :-- |
| **Claude Code** | Sí | Sí | Sí, en la aplicación de Claude |
| **Codex** | Sí | Sí | — |
| **Cursor** | Sí | Sí | — |
| **Antigravity CLI** | Sí | Sí | — |
| **Gemini CLI** | Sí | No | — |
| **GitHub Copilot** | Sí, menos con `apply_patch` | No | — |

En Cursor, la pregunta sale de sus propios hooks: con solo los de Claude Code, que Cursor también corre, calla. De un comando mira solo dónde escribe lo que él mismo dice (`echo … > nota.md`, `cat > nota.md`, `tee`, `Set-Content`) y si es un `git push`, un `gh pr create` o un `gh pr merge`: lo que escriba un programa que corra, no. Claude Desktop no tiene hooks, y un agente en la nube no corre los de tu equipo.

**Una vez por sesión y carpeta**, con las de adentro, y tres carpetas por sesión como mucho; las tarjetas, una vez. Si el turno ya llevó esa pregunta, al terminar no se le devuelve además.

**Ejemplo.** Marisol le pide a Claude Code un resumen de lo que falta «para que lo siga Codex». El agente va a crear `C:\Users\Marisol\Documents\encargo.md`, y el hook le responde:

```json
{ "hookSpecificOutput": { "hookEventName": "PreToolUse", "permissionDecision": "deny",
  "permissionDecisionReason": "This would leave a text file outside any repository. If someone will read it later (the person, another agent, you on another day), it belongs in Tisty: …" } }
```

Entonces lo deja en una tarea, con sus pasos, y le da a Marisol el título:

```json
propose { "title": "Cerrar la migración de pagos", "for": "agent", "list": "Trabajo",
  "steps": ["Probar los reintentos", "Avisar al proveedor"] }
```

Si Marisol hubiera pedido justo ese archivo, el agente repite la misma llamada y se crea, sin preguntarle nada a Marisol.

### Dónde se escriben

| Agente | Archivo |
| :-- | :-- |
| **Claude Code** | `~/.claude/settings.json` |
| **Codex** | `~/.codex/hooks.json` |
| **Cursor** | `~/.cursor/hooks.json` |
| **Antigravity CLI** | `~/.gemini/config/hooks.json`, bajo el nombre `tisty` |
| **Gemini CLI** | `~/.gemini/settings.json` |
| **GitHub Copilot** (su línea de comandos y las sesiones de Copilot en VS Code) | un archivo propio, `~/.copilot/hooks/tisty.json` |

Tisty escribe solo sus propias entradas y deja el resto del archivo como está, con los mismos dos respaldos que la conexión, `.before-tisty`, la primera versión que vio, y `.before-tisty-last`, la de justo antes. Si el archivo no se pudo leer, la fila dice **No se pudo leer** y Tisty no lo toca.

### Agregarlos

1. Abre **Configuración › Agentes › Conectados**.
2. Bajo la fila de un agente conectado, como **Claude Code**, la fila **Hooks de Tisty** tiene un interruptor. Enciéndelo.
3. Tisty te dice qué va a hacer antes de tocar nada: «Tisty agregará sus hooks en `~/.claude/settings.json`. Al empezar una sesión, el agente recibe las tareas que dejó abiertas y lo que se movió…», con **Agregarlos** y **Ahora no**. En Claude Code suma una línea: «En Claude Code, Tisty lee antes el registro de su plugin (installed_plugins.json y settings.json) para no escribir hooks que el plugin ya trae.».
4. Con **Agregarlos**, el interruptor queda encendido, y debajo «Agregados. Los usa desde su próxima sesión.».

**Si conectas el agente ahora, los hooks van incluidos y encendidos**: [Al conectar un agente](#al-conectar-un-agente) lo cuenta.

**Si los agregaste antes de la 1.51**, no se agrega nada solo. Bajo la fila aparece «Les faltan los más nuevos, el de antes de dejar algo fuera de un repo y el de después de un push o un merge: apágalos y enciéndelos para tenerlos.». Al encenderlos otra vez se escriben todos como vienen: un tiempo o un filtro que les habías cambiado vuelve al de Tisty.

No hace falta reiniciar el agente para los hooks: los toma en su próxima sesión.

### Cuando apuntan a otro Tisty

Los hooks guardan la ruta del Tisty que los puso. Si mueves Tisty o cambias de instalador, siguen llamando al de antes, y el agente no siempre lo dice. La fila sí:

| La fila dice | Qué pasa |
| :-- | :-- |
| Interruptor encendido | Llaman a este Tisty, o a uno cuya ruta Tisty no puede comprobar, como `tisty` a secas |
| **Escritos a tu manera** | «Corre Tisty con hooks escritos a tu manera. Tisty los deja como están y no agrega otros al lado.». Son los que llaman a Tisty desde un script tuyo o de otra forma que Tisty no sabe leer: no se corrigen, no se quitan y la fila no ofrece encenderlos |
| **Usan un Tisty que ya no existe** | «Sus hooks apuntan a `C:\…\tisty.exe`, que ya no existe». **Corregir** los apunta aquí |
| **Usan otro Tisty** | Ese otro sigue en este equipo. **Corregir** los apunta aquí, si quieres |
| **No se pudo leer** | Tisty no pudo leer su archivo, así que no lo toca |
| Interruptor apagado | No hay hooks de Tisty en ese archivo |
| **Los trae el plugin de Tisty** | Los trae el plugin, en su propio archivo |

**Corregir** te muestra el archivo y espera tu **Sí, corregir**. Cambia solo el programa al que llaman: los matchers, los tiempos y un envoltorio tuyo quedan como estaban. Los que ya no existen salen también en el aviso de la ventana y en **Corregir todo**, como cuenta [Cuando un agente deja de encontrar Tisty](Agentes-y-MCP#cuando-un-agente-deja-de-encontrar-tisty).

**Ejemplo.** Jaime probaba Tisty desde una carpeta de descargas y la borró al instalarlo de verdad. Volvió a conectar Cursor, pero nunca sus hooks: la fila de Cursor dice **Conectado** y, debajo, **Usan un Tisty que ya no existe**. Pulsa **Corregir**, lee `~/.cursor/hooks.json` bajo **Escribe en:**, pulsa **Sí, corregir** y abre Cursor otra vez. El tiempo que le había subido a un hook sigue como lo dejó.

### Quitarlos o apagarlos

- Apagar el interruptor de la fila saca solo las entradas de Tisty: «Quitados. Su próxima sesión corre sin ellos.». Si además hay otros a tu manera: «Quitados los de Tisty. Los escritos a tu manera quedan.».
- **Recordarle al empezar y al terminar cada turno**, en **Configuración › Agentes › Puerta**, los apaga todos para todos los agentes sin quitarlos de ningún archivo. Sirve para una tarde en que no quieres que te devuelvan turnos.
- **Quitarle la voz**, en la misma **Puerta**, cierra la puerta: con ella cerrada, los hooks callan.

### Si el plugin de Tisty ya los trae

El plugin de Tisty para Claude Code trae sus propios hooks, en su propio archivo, desde la versión 1.4.0. Trae los dos primeros: el de antes de dejar algo fuera todavía no viene con él, y en ese caso Claude Code corre sin ese. Si Tisty sabe que está instalado, sea cual sea su versión, la fila de hooks de Claude Code dice **Los trae el plugin de Tisty**, sin interruptor, para que no corran dos veces.

Si ya tenías los de Tisty y después instalas el plugin, la fila te avisa: «El plugin de Tisty también los trae, así que corren dos veces. Apaga estos para dejar los del plugin.», con su interruptor.

El registro del plugin es un archivo de tu agente, así que Tisty lo lee solo con un sí tuyo. Con el de las [skills de Tisty](#las-skills-de-tisty) o el de la carpeta [[Agentes|Carpeta-AGENTS]], lo sabe al abrir **Configuración › Agentes**; para las skills lo mira también al abrir la app y cuando algo cambia en tus datos, aunque estén apagadas. Sin ninguno de los dos, la fila ofrece encenderlos aunque el plugin esté, y Tisty lo lee justo al pulsar **Sí, conectar** con sus hooks, **Agregarlos**, **Agregar hooks**, **Corregir**, **Corregir todo** o **Sí, corregir**, antes de escribir hooks de Claude Code. Si el plugin ya los trae, no escribe nada: «El plugin de Tisty ya trae los hooks de Claude Code, así que los de sus ajustes quedaron como estaban.». Si el registro no se puede leer, tampoco, y te dice qué archivo fue.

Si Tisty sabe que el plugin los trae y los que quedaron de antes apuntan a un Tisty que ya no existe, **Corregir** los quita, porque con los del plugin correrían dos veces, y te lo dice antes: «El plugin de Tisty ya trae sus hooks, así que estos se quitan. Tisty lee antes su registro (installed_plugins.json y settings.json) para comprobarlo.». Ese registro se lee con tu clic, justo antes de quitarlos. Si el plugin ya no los trae, no se quita ninguno: «El plugin de Tisty ya no trae los hooks de Claude Code, así que estos no se quitaron. Busca de nuevo para ver cómo están.».

## Tus propios hooks

Además de los de Tisty, puedes escribir los tuyos: un script que tu agente corre en un momento que eliges, por ejemplo uno que detiene un `rm -rf` antes de que corra.

### Crear uno

1. En la barra lateral, **Agentes › Global › Hooks**, pulsa **+**.
2. **Nuevo hook** pide el nombre, el tipo de script (`.ps1`, `.sh`, `.py` o `.js`), **Cuándo** (al empezar una sesión, al enviar un mensaje, antes o después de una herramienta, al terminar un turno), las herramientas (terminal, escribir archivos, MCP o cualquiera), los segundos como máximo y los agentes. **Solo este equipo** viene marcado: así nace en la carpeta de tu equipo; sin marcar, en Global, para todos.
3. **Crear** abre su página. El script va en el bloque de código, y el disparador en sus líneas `hook-`: `hook-event`, `hook-tools`, `hook-timeout` y `hook-agents`. Si una de esas líneas falta, se repite o no se entiende, el hook no se registra.

### Registrar

**Registrar**, bajo el hook, te muestra el script entero y los archivos que va a tocar: «Registrar guard.ps1 en este equipo». Con **Sí, registrar**, Tisty guarda el script en este equipo, en la carpeta de datos de Tisty, y pone en cada agente elegido una sola línea que lo llama: `tisty run-hook --name guard.ps1 --agent codex`.

- **Es un sí por hook y por equipo.** **Aplicar aquí** nunca registra un hook, y en otro equipo se pregunta de nuevo.
- **Si su texto cambia**, deja de correr hasta que lo registres otra vez, en cada equipo: la fila dice «Cambió desde que lo registraste: no corre hasta que lo registres de nuevo».
- **Si lo eliminas o lo archivas**, Tisty lo saca de cada agente.
- **Codex** lo corre solo después de que lo apruebes en él: «Apruébalo en Codex con /hooks».

| Agente | Dónde | Lo que no tiene |
| :-- | :-- | :-- |
| **Claude Code** | `~/.claude/settings.json` | |
| **Codex** | `~/.codex/hooks.json` | |
| **Cursor** | `~/.cursor/hooks.json` | |
| **GitHub Copilot** | `~/.copilot/hooks/tisty.json` | Herramientas MCP |
| **Antigravity CLI** | `~/.gemini/config/hooks.json`, bajo `tisty-` y el nombre | Al enviar un mensaje |
| **Gemini CLI** | `~/.gemini/settings.json` | |

### Lo que recibe y lo que devuelve

El script recibe por su entrada lo mismo que manda el agente, más `TISTY_HOOK_EVENT` y `TISTY_HOOK_AGENT`.

- **Termina con 0:** deja pasar.
- **Termina con 2 antes de una herramienta:** la detiene, y el agente lee lo que escribió en stderr. Tisty lo traduce a lo que entiende cada agente.
- **Cualquier otra cosa**, un error, un intérprete que falta o el tiempo vencido: deja pasar.

Se registran en los agentes de este equipo, no en los de una distro de WSL. En Windows, `.ps1` corre con Windows PowerShell, `.sh` con el bash de Git (nunca con el de WSL), `.py` con `py -3` y `.js` con Node. Con la puerta de los agentes cerrada, no corre ninguno.

### Verificar

**Verificar** dice, agente por agente, lo que hay hoy en su archivo: «Registrado, corre este Tisty», «No está en su archivo», «Corre otro Tisty», «Escrito a tu manera: queda como está» o «No tiene ese momento». **Probarlo una vez** corre el script de verdad, en este equipo, con una muestra de lo que mandaría el agente, y te dice cómo terminó: «Terminó con 2: detiene la llamada», con lo que escribió. Antes de registrarlo te muestra el script y pide **Correrlo**. Un agente puede ver otro PATH que Tisty: lo que corre en la prueba puede no correr allá.

Ningún agente crea, cambia ni propone un hook: por el MCP se rechaza siempre.

## Las skills de Tisty

Una skill es una carpeta con un `SKILL.md`: tu agente lee su descripción siempre, y el resto solo cuando lo que le pides calza con ella. Tisty trae tres, dentro de la app:

| Skill | Qué le enseña a tu agente | Pídele a tu agente |
| :-- | :-- | :-- |
| `tisty` | Retomar donde quedó, trabajar las tareas, los documentos y las ideas, y dejar la tarea lista para el próximo | «Retoma donde quedamos» · «Deja esto listo para el próximo agente» |
| `consolidate` | Ordenar lo guardado: mira, te propone cada arreglo numerado y hace solo los que eliges. Nunca borra | «Ordena mi Tisty» |
| `next-agent` | Escribir una página, el estado de una tarea o un archivo de agente para que el próximo lo entienda sin la conversación. Adaptada de mattpocock/skills, con su licencia MIT, que viaja con ella | «Deja esta página clara para el próximo agente» |

Las frases son las mismas que muestra su ficha, en **Cómo se usan**: «Escríbeselo a tu agente con tus palabras; no hay que llamarlas por su nombre.». No tienen que ser exactas: «sigamos con lo de ayer» o «¿en qué íbamos?» también llevan a la skill `tisty`.

**Cada agente retoma lo suyo.** «Retoma donde quedamos» lleva a un agente a lo último que dejó él, no otro: lo que abre la sesión y lo que `catch_up` llama `last` son de cada programa. Lo que pasa de un agente a otro es lo escrito en la tarea, así que, para seguir lo que dejó otro, nómbrala por su título.

**No son los pedidos listos.** Tisty también le entrega a tu agente [ocho pedidos](Agentes-y-MCP#pedidos-listos-y-menciones-con-) que se usan como comandos, como `/mcp__tisty__pick_up` o `/mcp__tisty__consolidate Cocina` en Claude Code. Un pedido lo escribes tú, y llega a cualquier agente que tome pedidos de un servidor; una skill la elige tu agente cuando le hablas con tus palabras. Hacen el mismo trabajo, y puedes usar los dos.

**No son documentos tuyos.** Viven dentro de la app, no en tu almacén: no viajan a tus otros equipos, no salen en una búsqueda, en [[Brainsty|Red-de-Brainsty]], en el historial ni en una exportación, y ningún agente puede nombrarlas ni cambiarlas por el MCP.

### Dónde van

| Agente | Carpeta |
| :-- | :-- |
| **Claude Code** | `~/.claude/skills/tisty/`, `~/.claude/skills/consolidate/`, `~/.claude/skills/next-agent/` |
| **Codex**, **GitHub Copilot**, **Gemini CLI**, **DeepSeek Harness** y **Cursor** | `~/.agents/skills/…`, una sola copia que leen los cinco |

`~` es tu carpeta de usuario: `C:\Users\tú` en Windows y `/Users/tú` en macOS. La ficha muestra solo las carpetas de los agentes que tienes en este equipo. La compartida dice «Codex · Copilot · Gemini · DeepSeek Harness · Cursor» y lo advierte: «Codex, Copilot, Gemini, DeepSeek Harness y Cursor comparten esta carpeta: encenderlos o apagarlos aquí vale para los cinco.». Una copia que un Tisty anterior escribió en `~/.cursor/skills/` se va sola, con respaldo, mientras siga como Tisty la escribió.

**Dentro de WSL.** Cada distro que aparece en **Conectados** tiene su propia fila **Skills de Tisty**, con su propio sí. Tisty escribe solo en la carpeta del usuario por defecto de esa distro, en LF, y nunca pone en marcha por su cuenta una distro detenida: la fila espera, u ofrece **Poner Ubuntu en marcha y escribir**. Los hooks nunca se escriben en una distro.

### Encenderlas

1. Abre **Configuración › Agentes › Conectados**. Al final de la lista está la fila **Tisty en tus agentes**: «Skills y hooks que ayudan a tus agentes a trabajar con Tisty.», con su punto y **Ver**. **Ver** abre una ficha con las dos partes.
2. **Ajustar**, en **Skills**, abre su ficha. Arriba, «Tisty trae tres skills que les enseñan a tus agentes a trabajar con él. Vienen dentro de la app: no las escribe ningún agente y nadie puede cambiarlas por el MCP.». Debajo, **Dónde van en este equipo**, con una fila por carpeta, y plegados, **Cómo se usan**, con lo que le escribes a tu agente y para qué sirve cada una, y **Más sobre ellas**.
3. Lee el sí, al pie: «Para encenderlas, Tisty escribe solo una carpeta por skill en las carpetas de arriba, y lee los archivos que ya hay en esas carpetas de skill para no escribir nunca sobre uno tuyo. En Claude Code lee además el registro de su plugin (installed_plugins.json y settings.json) para no copiarlas dos veces. No lee reglas, memoria ni otros archivos de tus agentes, y no necesita la carpeta Agentes.».
4. Pulsa **Encender en este equipo**. Cada carpeta pasa a **Al día**, y en la ficha **Tisty en tus agentes** el punto de **Skills** queda verde.

Bajo cada agente conectado que las lee hay otra fila **Skills de Tisty**, con un interruptor que vale para todo este equipo: al encenderlo se abre la misma ficha, y al apagarlo se apagan en todos tus agentes. Y si tienes agentes conectados sin ellas, el recuadro [Hooks y skills para tus agentes](#los-dos-a-la-vez) las enciende junto con los hooks, con el mismo sí escrito antes.

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

La fila bajo cada agente lo resume:

| La fila dice | Qué pasa |
| :-- | :-- |
| Interruptor encendido, sin más | Las tres están al día en la carpeta de ese agente |
| Interruptor apagado | No las encendiste en este equipo. Al encenderlo se abre la ficha |
| **Por poner al día** | Falta escribir la versión de este Tisty. Sale también en el recuadro de arriba, con **Corregir** |
| **No todas al día** | Una copia no es de Tisty, la cambiaste o no se puede leer, o ya tienes en Agentes una skill con ese nombre; debajo dice qué pasa. Tisty no la toca, tampoco con **Corregir** |
| **Aquí no** | «Tisty no encontró una carpeta de este agente donde ponerlas.», o «Ya tienes en Agentes skills con esos nombres, así que las de Tisty quedan fuera.» |
| **Las trae el plugin de Tisty** | Ese agente las tiene por el plugin |

Si ya tienes en la carpeta **Agentes** una skill tuya con el mismo nombre, como tu propia `consolidate`, la ficha lo dice en lugar de su línea: «Ya tienes en Agentes una skill llamada consolidate, así que la de Tisty no se agrega.». La tuya gana, en todos los agentes, si este equipo ya dio su sí en Agentes; sin él, la cuida la carpeta: una skill que no es de Tisty se deja como está.

### Se actualizan solas

Cuando actualizas Tisty, la próxima vez que lo abres escribe la versión nueva de cada skill en las carpetas que están **Al día**. Una copia que cambiaste, o una carpeta que no es de Tisty, no se toca. La ficha lo dice: «Se actualizan cuando actualizas Tisty, y nunca pisan una skill tuya con el mismo nombre.».

### Apagarlas

El interruptor de la fila **Skills de Tisty**, bajo cualquier agente, o **Apagar en este equipo**, en la ficha, que antes te dice qué hace: «Apagarlas saca de las carpetas de tus agentes las copias intactas de Tisty, con respaldo, y las skills siguen en Tisty, listas para encenderlas otra vez. Una copia que cambiaste se queda en esta ficha.».

- **Apagar no es borrar.** Las skills siguen dentro de Tisty, y **Encender en este equipo** las vuelve a poner.
- **Una copia que cambiaste** no se saca, y con ella se queda su skill entera en esa carpeta, aunque solo cambie un archivo: queda listada en la ficha, para que la quites tú si quieres.
- **Nada las elimina.** No hay **Eliminar** ni en la ficha ni en la carpeta Agentes, y ningún agente puede quitarlas por el MCP.

### En la carpeta Agentes

Si usas la carpeta [[Agentes|Carpeta-AGENTS]], **Agentes › Global** muestra una fila fija bajo **De Tisty**, **Skills y hooks**, con el mismo punto que en Configuración, si este equipo tiene algún agente, si las skills están encendidas o si quedan copias suyas. Un clic abre la ficha **Tisty en tus agentes**.

- **Encendidas o Apagadas.** La fila lo dice, en verde o en gris, y abre la misma ficha, donde se encienden o se apagan, con el sí y las carpetas. Si la habías ocultado antes, vuelve a verse.
- **No tiene menú.** Ni **Eliminar**, ni **Pausar**, ni **Documento nuevo**: no es un documento.
- **Traer no las trae.** Si encendiste las skills antes de abrir Agentes, la lista de la primera vez, **Traer otro archivo** y **Buscar de nuevo** muestran sus copias como **propio de Tisty**, y no se traen: no entran como documentos tuyos.
- **La pausa de Agentes no las alcanza.** Pausar un agente, o dejar de sincronizar toda la carpeta, vale para tus documentos. Las skills de Tisty siguen su propio interruptor: encendidas, llegan y se actualizan en todos los agentes del equipo. La ficha lo recuerda: «Solo en este equipo: en otro equipo lo decides allí. La pausa de Agentes no las alcanza: tienen su propio interruptor.».
- **Los dos sí son independientes.** Dejar de sincronizar Agentes no apaga las skills, y encender las skills no trae nada a Agentes.

### Con el plugin de Tisty

- **Claude Code.** El plugin de Tisty trae las mismas tres skills. Si está instalado para tu usuario y no lo desactivaste, Tisty no las copia a `~/.claude/skills`, y la ficha lo dice: «Claude Code ya las tiene por el plugin de Tisty, así que Tisty no las copia ahí.». Codex, Copilot, Gemini, Cursor y DeepSeek Harness las siguen recibiendo.
- **Llega el plugin después.** Las copias que Tisty había dejado en `~/.claude/skills` aparecen en la ficha bajo «Copias que sobran: Tisty ya no las escribe ahí. Si quitas una, queda un respaldo.», cada ruta con **Quitar el archivo**. Nunca se quitan solas.
- **Cursor.** Su plugin no deja un registro que Tisty pueda leer, así que la ficha siempre avisa: «Si instalaste el plugin de Tisty en Cursor, Cursor mostrará cada skill dos veces: Tisty no puede saberlo.». Si te pasa, deja una sola de las dos: el plugin en Cursor o las skills de Tisty.

## Los dos a la vez

La fila **Tisty en tus agentes**, al final de **Configuración › Agentes › Conectados**, y la fila **Skills y hooks**, bajo **De Tisty** en **Agentes › Global**, abren la misma ficha. Su punto dice cómo están: verde si los dos están encendidos, amarillo si falta uno y gris hueco si están apagados. Al pasar el mouse lo dice en palabras, como «Faltan los hooks» o «Hooks solo en Claude Code». La ficha cuenta qué hacen:

> Las skills le enseñan a tu agente a usar Tisty, y los hooks le recuerdan solos dónde quedó y qué anotar.
>
> Te recomendamos los dos: tu agente retoma sin que le expliques de nuevo, y su avance queda en Tisty, donde lo ves.

Debajo, **Skills** y **Hooks**, cada una con su punto y **Ajustar**: el de las skills abre su ficha, y el de los hooks lleva a **Conectados**, donde está el interruptor de cada agente. **Encender los dos**, o **Encender lo que falta**, dice «Esto escribe solo aquí y deja el resto como está. Puedes apagarlos cuando quieras.» y abre dos casillas, **Hooks en** tus agentes y **Skills en este equipo**, cada una con los archivos o las carpetas que va a escribir. Plegado, **Qué hacen y qué lee Tisty** trae el mismo sí escrito del recuadro de abajo. **Encender** escribe lo que dejaste marcado y te dice qué quedó, agente por agente. Si lo que falta no se puede encender desde ahí, como hooks callados en **Puerta**, la ficha lo dice y te deja **Ajustar**.

Además, si ya tienes agentes conectados y a alguno le faltan los hooks o las skills, **Configuración › Agentes** los ofrece juntos, sobre las pestañas. Entran los que no pasaron por [Al conectar un agente](#al-conectar-un-agente), porque ese ya se los ofreció:

> **Hooks y skills para tus agentes**
>
> Opcional. Con los hooks, un agente empieza cada sesión sabiendo lo que dejó abierto. Con las skills de Tisty aprende a trabajar con él.
>
> Hooks para Claude Code, Cursor
> Skills de Tisty en todo este equipo: Claude Code, Cursor, Codex · Copilot · Gemini · DeepSeek Harness
>
> **Agregar hooks y encender skills** · **Ahora no**

1. **Agregar hooks y encender skills** todavía no escribe nada: abre dos casillas marcadas, una con los archivos donde van los hooks y otra con las carpetas donde van las skills y lo que Tisty lee para escribirlas.
2. Desmarca lo que no quieras. El botón cambia con lo que dejas: **Agregar hooks** o **Encender skills**.
3. Púlsalo otra vez. Tisty enciende las skills, agrega los hooks y te dice qué quedó, agente por agente: **Claude Code** · hooks, o «sus hooks ya los trae el plugin de Tisty» si el plugin los trae.
4. Sal de cada uno del todo y ábrelo otra vez.

Si todos ya tienen los hooks, el recuadro ofrece solo las skills, y al revés. **Ahora no**, o desmarcar una parte, hace que no se vuelva a ofrecer; lo que falló sí se ofrece de nuevo, y las filas de siempre quedan. **Resumen** lo recuerda en una línea, **Tus agentes pueden usar los hooks y las skills de Tisty · Opcional**, que no cuenta como pendiente. Espera si un agente dejó de encontrar Tisty: eso va primero. Y no ofrece los hooks mientras **Recordarle al empezar y al terminar cada turno** esté apagado.

## Al conectar un agente

Al conectar un agente desde **Configuración › Agentes › Conectados**, sus hooks y las skills de Tisty van incluidos, con un solo sí, y quedan encendidos. **Conectar**, o **Conectar igual**, te muestra primero qué va a escribir:

> **Conectar Claude Code con Tisty**
>
> Escribe en:
> `~/.claude.json`
> `~/.claude/settings.json`
>
> ☑ **Con sus hooks: Claude Code**
> Tisty agregará sus hooks en `~/.claude/settings.json`. Al empezar una sesión…
>
> ☑ **Skills de Tisty en todo este equipo: Claude Code**
> `~/.claude/skills`
> Para encenderlas, Tisty escribe solo una carpeta por skill en las carpetas de arriba…
>
> **Sí, conectar** · **Ahora no**

- **Sí, conectar** escribe lo que dejaste marcado. Lo que desmarcas queda apagado, con su interruptor bajo la fila del agente.
- **Conectar los 3 detectados** muestra lo mismo para todos, con los archivos de cada uno. También los que están dentro de WSL.
- **Si los hooks no se pueden agregar**, porque su archivo no se pudo leer o el plugin de Tisty ya los trae, el agente se conecta igual y Tisty te lo dice, sin hooks duplicados.
- **Si las skills no se pueden encender**, el agente se conecta igual y Tisty te dice por qué.
- Las skills valen para todo el equipo: si ya están encendidas, no se preguntan otra vez. DeepSeek Harness, que no tiene hooks, muestra solo las skills; un agente sin hooks ni skills, como Claude Desktop, se conecta con un clic. **Corregir** no agrega nada nuevo.
- Con **Recordarle al empezar y al terminar cada turno** apagado, quedan agregados y callados.

Después, bajo su fila quedan los dos interruptores, **Skills de Tisty** y **Hooks de Tisty**, encendidos.

**Si ya tenías el agente conectado** antes de la 1.50, nada se enciende solo. Para esos está [Hooks y skills para tus agentes](#los-dos-a-la-vez), sobre las pestañas, o los interruptores bajo el agente. Es el mismo sí.

## Un caso de punta a punta

Yisset usa Codex, que tiene conectado a Tisty desde hace meses. Hoy actualiza Tisty a la 1.50 e instala Claude Code en el mismo equipo.

### 1. Conecta Claude Code

Abre **Configuración › Agentes**. Sobre las pestañas, **Hooks y skills para tus agentes** le ofrece encender las skills, porque Codex no las tiene; lo deja para después de conectar. En **Conectados**, Claude Code aparece **Sin conectar**. Yisset pulsa **Conectar**, y Tisty le muestra qué va a escribir: `~/.claude.json`, `~/.claude/settings.json` y las carpetas de las skills.

### 2. Con sus hooks y las skills

Lee lo que harán los hooks y dónde van las skills, en la carpeta de Claude Code, `~/.claude/skills`, y en la compartida, `~/.agents/skills`, para Codex. Deja las dos casillas marcadas y pulsa **Sí, conectar**.

### 3. Quedan encendidos

La fila dice **Conectado** y, debajo, los dos interruptores están encendidos: **Skills de Tisty**, que valen también para Codex, y **Hooks de Tisty**.

Como Claude Code lee su configuración al abrirse, lo cierra del todo y lo abre otra vez.

### 4. Le encarga un trabajo

Yisset le pide a Claude Code que planifique la exportación del reporte. El agente la anota como una tarea con sus pasos:

```json
> propose {"title": "Exportar el reporte mensual a CSV", "for": "agent", "list": "Trabajo",
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
- **Nada se cerró.** Falta un paso, y cuando el agente diga que la tarea está hecha, esperará en **Lo hace el agente**, bajo **Por confirmar**, con las primeras líneas de su prueba, hasta que Yisset la confirme.

Si la tarea la hubiera escrito Yisset, y ella hubiera marcado el primer paso, las marcas del agente serían su palabra: la fila diría **1/4 ◆2**, con «2 por confirmar» al pasar el puntero, hasta que ella las tome con **Marcarlos**. Lo cuenta [[Qué puede hacer un agente|Qué-puede-hacer-un-agente]].

### 9. Después, otro agente

Yisset vuelve a Codex. A Codex no le agregó los hooks de Tisty, así que nada le llega al empezar, y como ya estaba conectado antes de la 1.50 nunca se le preguntó al conectar; pero lee la skill `tisty`, que Yisset encendió en el paso 3 y quedó en `~/.agents/skills`. «Retoma donde quedamos», a secas, lo llevaría a lo último que dejó **él**, así que Yisset nombra la tarea: «Retoma la exportación del reporte mensual».

```json
> brain {"query": "exportar reporte mensual"}
{ "matches": [ { "title": "Exportar el reporte mensual a CSV", "list": "Trabajo", "status": "open" } ] }
> read {"task": "…Exportar el reporte mensual a CSV…"}
```

Lee el estado y la nota de Claude Code, sabe que no debe volver a probar la librería de planillas y sigue con el botón. También habría llegado por lo que se movió, que `catch_up` le cuenta desde la última vez que él miró.

### 10. Semanas después, el plugin

Yisset instala el plugin de Tisty en Claude Code. La próxima vez que abre **Configuración › Agentes**:

- la fila de hooks de Claude Code dice «El plugin de Tisty también los trae, así que corren dos veces. Apaga estos para dejar los del plugin.». Los apaga, y la fila queda en **Los trae el plugin de Tisty**;
- la ficha de las skills dice «Claude Code ya las tiene por el plugin de Tisty, así que Tisty no las copia ahí.», y lista bajo **Copias que sobran** los archivos que Tisty había dejado en `~/.claude/skills`. Pulsa **Quitar el archivo** en cada uno, y quedan sus respaldos. La carpeta compartida de Codex sigue **Al día**.

## Casos que conviene conocer

- **En otro equipo.** Los hooks y las skills se deciden en cada equipo: en el equipo del trabajo, la fila **Skills de Tisty** empieza **Apagadas** y los hooks apagados, aunque en la casa estén encendidos. Un registro copiado de otro equipo no cuenta como sí de las skills.
- **Antes de cualquier sí**, el de las skills o el de Agentes, la ficha no abre nada dentro de las carpetas de tus agentes: solo mira cuáles existen y si siguen ahí las copias que Tisty escribió antes. Tampoco lee el registro del plugin, salvo que pulses agregar o corregir los hooks de Claude Code, y solo en ese momento. Con el sí de Agentes solo, lee ese registro y esas copias, para ver si cambiaron.
- **Con la puerta cerrada.** Las skills siguen en las carpetas de tus agentes, y tu agente puede leerlas, pero cada llamada a Tisty se rechaza y los hooks callan. **Quitarle la voz** no toca ninguna de las dos piezas.
- **Un agente en pausa en Agentes.** La pausa no alcanza a las skills de Tisty: siguen llegando y actualizándose en ese agente. Para que no las tenga ninguno, **Apagar en este equipo**.
- **Codex** corre los hooks solo después de que los apruebes en él: bajo su fila, «Apruébalo en Codex con /hooks.». Cada vez que Tisty los cambia, como al corregir a qué Tisty apuntan, Codex vuelve a pedirlo.
- **Claude Desktop** no tiene hooks ni skills, y le llega solo la pregunta en las respuestas de Tisty. **DeepSeek Harness** no tiene hooks, pero recibe las skills.
- **Gemini CLI** corre los hooks solo en las carpetas en que confiaste. Su extensión de Tisty también los trae, pero Tisty solo reconoce el plugin de Claude Code: si usas la extensión, no agregues además los hooks desde Configuración.
- **Cursor** también corre los hooks de Claude Code. Aunque se disparen los dos, Tisty devuelve el turno una sola vez.
- **GitHub Copilot** puede gastar una solicitud premium en el turno que se le devuelve.
- **Una build de desarrollo** de Tisty no trae skills. La ficha lo dice, «Esta build de Tisty no trae skills: solo las versiones instaladas.», y **Encender en este equipo** queda apagado.
- **Una skill tuya con el mismo nombre** en la carpeta de un agente siempre gana, y una en Agentes también, en un equipo que ya dio su sí en Agentes. La de Tisty queda fuera, sin pisar nada.
- **Si Tisty perdió su registro**, por ejemplo al reinstalar el sistema, al encender las skills reconoce como suyas las copias que son idénticas, byte a byte, a las de esta versión: vuelven a actualizarse y a salir al apagarlas. Si a una le falta un archivo, tiene uno distinto o uno tuyo al lado, esa skill entera sigue siendo tuya y no se toca.

## Lo que leen y lo que viaja

- **Los hooks** escriben solo sus entradas en el archivo nombrado. Con la puerta abierta, Tisty lee ese archivo, y el de la conexión, al abrir su ventana y al abrir Configuración, para ver si apuntan a este Tisty, sin escribir nada hasta que haces clic. En Claude Code, al agregarlos o corregirlos, también cuando corregir es quitarlos porque el plugin los trae, lee además el registro de su plugin; sin el sí de las skills ni el de Agentes, solo entonces. Sus marcas de cada sesión se quedan en la caché de este equipo y no se sincronizan. Lo que le entregan al agente al empezar, los títulos de algunas tareas, viaja adonde viaja ese agente.
- **Las skills** escriben solo una carpeta por skill. Leen los archivos de esas carpetas, para no pisar uno tuyo, y en Claude Code el registro de su plugin, solo para saber si ya las trae. Nada más de tus agentes se abre.
- **El sí de las skills** se guarda en `data/agents/` dentro de la carpeta de Tisty, en este equipo, con los respaldos de lo que Tisty quitó. No viaja con la sincronización, los respaldos ni los paquetes. Los hooks no guardan un sí aparte: se agregan al pulsar **Agregarlos** o **Agregar hooks**, después de leer en qué archivo se escriben, se quitan al apagar su interruptor, y Tisty sabe si están, leyendo ese archivo.

Todo, archivo por archivo, está en [[Privacidad|Privacidad]].

## Para seguir

- [[Agentes y MCP|Agentes-y-MCP]]: conectar un agente, la puerta, los pedidos listos y lo que cuesta.
- [[Cómo trabaja un agente|Cómo-trabaja-un-agente]]: una sesión entera, llamada por llamada.
- [[Qué puede hacer un agente|Qué-puede-hacer-un-agente]]: su palabra sobre tus tareas y **Por confirmar**.
- [[Agentes: reglas, skills y memoria|Carpeta-AGENTS]]: tus propias reglas y skills, escritas una vez para todos tus agentes.
- [[Herramientas MCP|Herramientas-MCP]]: cada herramienta que usan las skills.
