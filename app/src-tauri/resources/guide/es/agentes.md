# Paso a paso con tu agente de IA

Esta hoja es una página de la guía. Cuenta cómo trabajar con un agente de IA en Tisty: cómo conectarlo, cómo darle o quitarle la entrada, qué puede hacer y qué no, y cómo pedirle las cosas.

## Conectarlo

**Configuración → Asistentes** encuentra los asistentes que tienes instalados, como Claude Code, Codex, GitHub Copilot o Cursor, y dice cómo está cada uno:

- **Conectado**: ya abre este Tisty.
- **Sin conectar**: **Conectar** escribe Tisty en la configuración de ese asistente, y solo toca su propia línea.
- **Apunta a otro Tisty**: una copia vieja o que se movió. **Corregir** lo apunta aquí.

Después, cierra el asistente del todo y ábrelo de nuevo: lee su configuración al abrirse. Si prefieres hacerlo a mano o con un plugin, cada forma está en [asistentes y MCP](https://www.tisty.app/es/ayuda/asistentes-y-mcp/).

## Darle o quitarle la entrada

Tisty le deja una puerta a tu agente, y la manejas tú. Viene abierta al instalar, y **Configuración → Asistentes** la cierra con un clic: mientras está cerrada, todo lo que intente un asistente se rechaza y no lee ni escribe nada. Si uno intenta entrar con la puerta cerrada, la ventana te avisa («Claude Code intentó entrar») y **Dejarlo entrar** la abre ahí mismo.

Nada escucha hasta que conectas uno: el asistente arranca Tisty cuando lo necesita, en este equipo, sin puerto ni cuenta. Lo que sí conviene saber: lo que un asistente lee, se lo lleva al servicio con el que trabaja. Más en [privacidad](https://www.tisty.app/es/ayuda/privacidad/).

## Lo que puede hacer

- Proponer tareas con sus pasos y su fecha, ponerles un recordatorio y anotar en su bitácora.
- Trabajar cualquier tarea como un documento, también las tuyas: su descripción, su página, su título, su lista y sus pasos abiertos.
- Escribir documentos, agregar a uno, corregir un pasaje o una tabla, y ordenarlos en carpetas y páginas.
- Anotar ideas en la Lluvia de ideas cuando se lo pides, y ordenarlas contigo: una idea que convierte en tarea o anexa a un documento sale de la hoja, como cuando lo haces tú.
- Marcar un documento o una idea que ya cumplió su vida, con el porqué: tú lo borras, lo archivas o le quitas la marca.
- Decir que una tarea o un paso está listo: la tarea queda en **Por confirmar** hasta que tú la terminas o la devuelves con **Sigue pendiente**.
- Buscar y leer lo que hay, incluido el Archivo.

## Lo que no puede

- Cerrar ni borrar una tarea, ni borrar un documento o una idea.
- Marcar un paso tuyo o terminar una tarea tuya: solo dice que están listos, y tú tomas su palabra o no.
- Mover un día que pusiste tú.
- Escribir encima de lo que cambiaste mientras él pensaba: se le pide que vuelva a leer.

Para lo prohibido no hay herramienta: no es que esté escondida, es que no existe. Todo, regla por regla, está en [qué puede hacer un asistente](https://www.tisty.app/es/ayuda/que-puede-hacer-un-asistente/).

## Cómo pedírselo

Háblale como a una persona: «anota esto como tarea para el viernes», «guarda esta decisión en el documento del proyecto», «¿cómo lo resolvimos la otra vez?». Además, Tisty le entrega siete pedidos listos que tu asistente muestra como comandos; en Claude Code están en el menú de `/`:

| Pedido | Qué hace |
| --- | --- |
| `pick_up` | Retoma donde quedó: lo que cambió y la tarea en la que estaba |
| `leave_off` | Deja la tarea lista para retomarla: pasos marcados y lo que sigue |
| `plan_task` | Planifica un trabajo como una tarea con pasos |
| `solved_before` | Busca en el Archivo cómo se resolvió algo antes |
| `keep_decision` | Guarda una decisión en el documento de su tema |
| `brainstorm` | Hace una lluvia de ideas contigo y anota las que te quedas |
| `sort_ideas` | Ordena las ideas que esperan, sin cambiar nada hasta que lo digas |

Y con `@` le pasas un documento por su título.

## Lo que conviene evitar

- Pedirle que borre: marca lo que ya cumplió su vida, y borrarlo es tuyo, de un clic.
- Esperar que cierre una tarea: cuando dice que terminó, la tarea te espera en **Por confirmar**.
- Conectar el mismo asistente de dos formas, con el plugin y desde Configuración: vería cada herramienta dos veces.

Lo que hace cada herramienta está en [herramientas MCP](https://www.tisty.app/es/ayuda/herramientas-mcp/).
