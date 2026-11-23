# Paso a paso con tu agente de IA

Esta hoja es una página de la guía. Cuenta cómo trabajar con un agente de IA en Tisty: cómo conectarlo, cómo darle o quitarle la voz, cómo hacer que no se le olvide anotar, qué le enseñan las skills de Tisty, qué puede hacer y qué no, y cómo pedirle las cosas.

## Conectarlo

**Configuración › Agentes › Conectados** encuentra los agentes que tienes instalados, como Claude Code, Codex, GitHub Copilot, Cursor o Gemini CLI, y dice cómo está cada uno:

- **Conectado**: ya abre este Tisty.
- **Sin conectar**: **Conectar** escribe Tisty en la configuración de ese agente, y solo toca su propia línea.
- **Apunta a otro Tisty**: una copia vieja o que se movió. **Corregir** lo apunta aquí.
- **Puede que esté**: solo quedó su carpeta, así que quizás ya no está instalado.
- **A mano**: guarda su configuración en un formato que Tisty no escribe, y la fila muestra qué pegar.

**Conectar los 3 detectados**, con los que haya, conecta de una vez todos los que esperan. Si el tuyo no aparece, el buscador bajo la lista lo encuentra entre los que Tisty conoce.

Al conectar Claude Code, Cursor, Codex, Gemini CLI o GitHub Copilot, su fila te ofrece una sola vez lo que lo ayuda a trabajar con Tisty: «Claude Code ya está conectado. Esto lo ayuda a trabajar con Tisty:», con sus hooks y **Ver las skills de Tisty**, que cuenta esta página más abajo. **Ahora no** lo cierra; lo mismo sigue en las filas de siempre. Codex, que no tiene hooks, ve solo las skills.

Después, cierra el agente del todo y ábrelo de nuevo: lee su configuración al abrirse. Si prefieres hacerlo a mano, con un plugin o dentro de WSL, cada forma está en [agentes y MCP](https://www.tisty.app/es/ayuda/agentes-y-mcp/).

## Darle o quitarle la voz

Tisty le deja una puerta a tu agente, y la manejas tú. Viene abierta al instalar, y en **Configuración › Agentes › Puerta** la cierras con **Quitarle la voz**: mientras está cerrada, todo lo que intente un agente se rechaza y no lee ni escribe nada. **Dejar que un agente anote** la abre otra vez. Si uno intenta entrar con la puerta cerrada, la ventana te avisa («Claude Code intentó entrar») y **Dejarlo entrar** la abre ahí mismo.

Nada escucha hasta que conectas uno: el agente arranca Tisty cuando lo necesita, en este equipo, sin puerto ni cuenta.

> [!WARNING]
> Lo que un agente lee se lo lleva al servicio con el que trabaja, y eso vale también para lo que le llega al empezar cada sesión con los hooks, más abajo. Más en [privacidad](https://www.tisty.app/es/ayuda/privacidad/).

## Que no se le olvide anotar

Un agente concentrado en el código se olvida de Tisty. Hay dos ayudas, y las dos se apagan en **Configuración › Agentes › Puerta**:

- **Pedirle que deje su tarea al día**: si sigue trabajando y la tarea que abrió lleva diez minutos sin una palabra suya, o mientras escribe otras tres cosas, la siguiente respuesta de Tisty le pide una vez que marque lo que hizo y, si terminó, que lo diga con la prueba. Llega a todos los agentes, aunque solo cuando llaman a Tisty.
- **Los hooks**: en Claude Code, Cursor, Gemini CLI y GitHub Copilot, **Agregar**, bajo la fila del agente conectado, te dice qué archivo va a escribir y pone ahí los hooks de Tisty. Desde su próxima sesión, y mientras la puerta esté abierta, cada una empieza con las últimas tareas abiertas en que escribió ese mismo agente, lo que se movió y lo que apremia, aunque el agente no llame a Tisty; y si un turno editó archivos sin escribir nada en Tisty, se lo devuelve una vez para que marque lo terminado. **Quitar**, junto a los hooks, los saca, y **Recordarle al empezar y al terminar cada turno** los apaga sin quitarlos.

## Que sepa cómo trabajar: las skills de Tisty

Tisty trae tres skills que le enseñan a tu agente a trabajar con él. No hace falta aprender comandos: tu agente las usa cuando le hablas con tus palabras.

| Skill | Qué le enseña | Pídele a tu agente |
| --- | --- | --- |
| `tisty` | Retomar donde quedó, trabajar tareas, documentos e ideas, y dejar la tarea lista para el próximo | «Retoma donde quedamos» · «Deja esto listo para el próximo agente» |
| `consolidate` | Ordenar lo guardado: propone cada arreglo y nunca borra | «Ordena mi Tisty» |
| `writing-for-the-next-agent` | Escribir para que el próximo agente lo entienda sin la conversación | «Deja esta página clara para el próximo agente» |

Se encienden en **Configuración › Agentes › Conectados**, en la fila **Skills de Tisty**, al final de la lista: **Ver** abre su ficha, que dice dónde van en este equipo y qué lee Tisty para escribirlas, y **Encender en este equipo** es su sí. Tienen su propio sí y no necesitan la carpeta Agentes: llegan a `~/.claude/skills`, a `~/.cursor/skills` y a `~/.agents/skills`, la que comparten Codex, Copilot y Gemini.

- **Se actualizan solas** con cada versión de Tisty, y nunca pisan una skill tuya con el mismo nombre ni una copia que cambiaste.
- **Valen solo en este equipo**: en otro lo decides allá. La pausa de Agentes no las alcanza: tienen su propio interruptor.
- **Apagar en este equipo** las saca de las carpetas de tus agentes, con respaldo, sin eliminarlas: siguen en Tisty, listas para encenderlas otra vez.
- **Con el plugin de Tisty** en Claude Code, el plugin ya trae los hooks y las skills: la fila de hooks dice **Los trae el plugin de Tisty**, y Tisty no copia ahí sus skills.

**Un ejemplo.** Yisset conecta Claude Code y, en el recuadro que aparece bajo su fila, agrega sus hooks y enciende las skills. Al otro día le escribe «Retoma donde quedamos»: el agente pide `catch_up`, abre la tarea en que estaba, «Exportar el reporte mensual a CSV», y sigue desde su **Estado**. Antes de irse, «Deja esto listo para el próximo agente» deja los pasos marcados, el estado al día y una nota con lo que descartó. Eso queda en la tarea: otro día, en Codex, nombra la tarea por su título y Codex lee ahí mismo el estado y la nota. «Retoma donde quedamos», en cambio, lleva a cada agente a lo último que dejó él.

Todo, caso por caso y con las llamadas del agente, está en [hooks y skills](https://www.tisty.app/es/ayuda/hooks-y-skills/).

## Lo que lee: Brainsty

Tu agente no lee todo lo que guardas: lo recorre. Con una sola herramienta, `brain`, busca tareas y documentos por sus palabras, con el Archivo junto a lo demás y primero lo que tiene esas palabras en su título. Pide la ficha de un documento (sus secciones con sus líneas, quién lo cita y sus decisiones vigentes primero) y después, con `read`, lee solo la sección que necesita, o solo lo que resaltaste o pusiste en negrita. Si vuelve a leer algo que no cambió, Tisty le dice solo eso, sin mandarle el texto otra vez. Y mira un proyecto entero agrupado en bloqueado, en curso, esperándote y sin empezar: una parte cuyo estado empieza con «Bloqueada:» cuenta como bloqueada. En la ficha le llega aparte, en `guessed`, lo que Tisty cree que se parece y nadie enlazó, con las palabras que lo dicen: lee los dos antes de unir nada. Es la misma red que tú ves dibujada en **Brainsty: todo unido**, una página de esta guía.

Al retomar una tarea, lo primero que le llega es su **Estado**, el que ves en su página, y solo las notas escritas después. Si los pasos cambiaron o llegaron notas después del estado, Tisty le avisa que quedó viejo. Cada llamada, con un extracto de lo que Tisty responde, está en [cómo trabaja un agente](https://www.tisty.app/es/ayuda/como-trabaja-un-agente/).

## Lo que puede hacer

- Proponer tareas con sus pasos, sus partes y su fecha, ponerles un recordatorio y anotar en su bitácora.
- Marcar los pasos que terminó y, en la misma llamada, dejar el **Estado** de la tarea en su página: lo hecho, lo que falta, lo decidido y lo descartado. Tú también lo lees, y la sesión siguiente parte de ahí.
- Trabajar cualquier tarea como un documento, también las tuyas: su descripción, su página, su título, su lista y sus pasos abiertos.
- Escribir documentos, agregar a uno, corregir un pasaje o una tabla, y ordenarlos en carpetas y páginas.
- Nombrar una tarea o un documento por su título exacto entre comillas: Tisty lo convierte en un enlace, como cuando lo haces tú, y lo dudoso se lo devuelve para que decida.
- Anotar ideas en Ideas cuando se lo pides, y ordenarlas contigo: una idea que convierte en tarea o anexa a un documento sale de la hoja, como cuando lo haces tú.
- Marcar un documento o una idea que ya cumplió su vida, con el porqué: tú lo borras, lo archivas o le quitas la marca.
- Decir que una tarea o un paso está listo: la tarea queda en **Por confirmar** hasta que tú la terminas o la devuelves con **Sigue pendiente**; los pasos te esperan con **Marcarlos** o **Todavía no**, y mientras tanto la lista los cuenta aparte, con un ◆: **1/3 ◆1**.
- Buscar y leer lo que hay, incluido el Archivo.
- Leer tu Diario y añadir al final de hoy, con su firma, solo si lo abres a los agentes desde el propio Diario y se lo pides.

Todo lo que escribe va firmado con el nombre del programa por el que habló: «por Claude Code», «por Codex». En **Configuración › Agentes › Cómo se ve su firma** eliges dónde se ve ese nombre, o si solo queda un pequeño ◆.

## Lo que no puede

- Cerrar, reabrir ni borrar una tarea, ni borrar un documento o una idea.
- Marcar un paso tuyo o terminar una tarea tuya: solo dice que están listos, y tú tomas su palabra o no.
- Mover un día que pusiste tú.
- Escribir por su cuenta en la carpeta Agentes: un cambio a tus reglas o skills queda como propuesta, y solo llega a tus agentes si lo apruebas en la ficha. Lo cuenta **Agentes: las reglas de tus agentes**, una página de esta guía.
- Vaciar el historial de un documento ni leer las versiones anteriores de tu diario. Sí puede listar las versiones de un documento, leer una y volver a ella, como tú desde **Ver el historial**; lo cuenta [la portada de esta guía](tisty:guide), en **Volver a lo que decía antes**.
- Leer una tarea que ocultaste o descartaste: donde la enlazaste lee solo «a task out of sight».
- Crear listas ni poner rutinas: sin una de tus listas, lo que anota entra en la bandeja.
- Escribir encima de lo que cambiaste mientras él pensaba: se le pide que vuelva a leer.
- Ver tu Diario mientras dice **Solo tú**, ni cambiar lo que escribiste en él cuando lo abres.

Para cerrar y borrar no hay herramienta: no es que esté escondida, es que no existe. Lo demás se rechaza o queda como su palabra, y se le dice por qué. Todo, regla por regla, está en [qué puede hacer un agente](https://www.tisty.app/es/ayuda/que-puede-hacer-un-agente/).

## Cómo pedírselo

Háblale como a una persona: «anota esto como tarea para el viernes», «guarda esta decisión en el documento del proyecto», «¿cómo lo resolvimos la otra vez?». Además, Tisty le entrega ocho pedidos listos que tu agente muestra como comandos; en Claude Code están en el menú de `/`:

| Pedido | Qué hace |
| --- | --- |
| `pick_up` | Retoma donde quedó: lo que cambió y la tarea en la que estaba |
| `leave_off` | Deja la tarea lista para retomarla: pasos marcados y su estado |
| `plan_task` | Planifica un trabajo como una tarea con pasos |
| `solved_before` | Busca en el Archivo cómo se resolvió algo antes |
| `keep_decision` | Guarda una decisión en el documento de su tema |
| `brainstorm` | Hace una lluvia de ideas contigo y anota las que te quedas |
| `sort_ideas` | Ordena las ideas que esperan, sin cambiar nada hasta que lo digas |
| `consolidate` | Revisa lo que guarda Tisty, una carpeta, una lista o un tema a la vez, y propone arreglos: resúmenes viejos, enlaces que no llevan a ninguna parte, documentos repetidos y estados que quedaron atrás. Hace solo lo que apruebas y nunca borra |

Cada uno acepta una palabra detrás. `/mcp__tisty__consolidate Cocina` revisa solo la carpeta Cocina: tu agente te muestra lo que encontró, numerado y con el arreglo que haría; contestas «1 y 2» y hace solo esos dos. Lo viejo nunca se borra: queda marcado, y lo borras tú. Una pasada entera, llamada por llamada, está en [agentes y MCP](https://www.tisty.app/es/ayuda/agentes-y-mcp/).

Con las skills de Tisty encendidas, el comando no hace falta: «Retoma donde quedamos» hace lo de `pick_up`, «Deja esto listo para el próximo agente», lo de `leave_off`, y «Ordena mi Tisty», lo de `consolidate`.

Y con `@` le pasas un documento por su título.

## Lo que conviene evitar

- Pedirle que borre: marca lo que ya cumplió su vida, y borrarlo es tuyo, de un clic.
- Esperar que cierre una tarea: cuando dice que terminó, la tarea te espera en **Por confirmar**.
- Conectar el mismo agente de dos formas, con el plugin y desde Configuración: vería cada herramienta dos veces.
- Olvidar reiniciar el agente después de conectarlo: lee su configuración al abrirse. Los hooks, en cambio, corren desde su próxima sesión.

Lo que hace cada herramienta está en [herramientas MCP](https://www.tisty.app/es/ayuda/herramientas-mcp/).
