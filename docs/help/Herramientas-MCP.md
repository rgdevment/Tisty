# Herramientas MCP

Las herramientas que `tisty mcp` le ofrece a un agente. El agente las elige por su cuenta; esta lista es para que sepas qué puede alcanzar. Todas funcionan solo en este equipo. La puerta viene abierta sola hasta que elijas —en una instalación nueva, y al actualizar si nunca la cerraste—, y mientras la mantengas cerrada desde **Configuración › Agentes › Puerta**, todo se rechaza.

Cada herramienta le dice además a la app donde corre el agente qué hace con tus cosas: si solo lee, si puede escribir encima o quitar algo que ya está, y si hacerla dos veces cambia algo más. Una app puede usarlo para preguntarte antes de las que escriben.

Las reglas detrás están en [[Qué puede hacer un agente|Qué-puede-hacer-un-agente]].

## Mirar

| Herramienta | Qué hace |
| :-- | :-- |
| `rules` | Las reglas para trabajar aquí, una guía a la vez («tasks», «documents», «tidying», «ideas» o «all»); sola, lo básico y un índice. Los agentes piden la que necesitan primero |
| `language` | El lenguaje de Tisty, para que lo que escribe sea algo que Tisty conoce |
| `help` | Cómo funciona Tisty, desde estas mismas páginas de ayuda, para que responda en vez de suponer |
| `catch_up` | Cómo están las cosas, con las listas y cuántas tareas abiertas tiene cada una, las que no están en ninguna, las carpetas y las etiquetas más usadas; las tareas abiertas que apremian, por título y lista, para que te las cuente y las lea con `read`: las vencidas, después las por vencer, las de hoy y las atrasadas, cinco como máximo y el resto contado, sin las que un agente ya dijo que están hechas o que no deben hacerse, que te esperan a ti, ni las que ocultaste; lo que otros cambiaron desde su último `catch_up`, veinte de cada tipo a la vez, y el siguiente trae el resto; y la tarea abierta en la que estaba, con cuánto avanzó (sin empezar, en curso con cuántos pasos marcados y cuántos dijo hechos un agente, bloqueada o con un estado más viejo que su trabajo) o, si ya no lo está, por qué: dijo que estaba hecha o que no debía hacerse, o la cerraste, la descartaste o la ocultaste. Nombra **Ideas** y dice cuántas ideas esperan y cuántas están marcadas. El día que toca tu [Revisión semanal](Tareas#revisión-semanal), dice cuánto espera en cada parada y cómo ayudarte |
| `brain` | Encontrar cosas y ver cómo se conectan, sin abrirlas. Tareas y documentos por sus palabras, el archivo junto a lo demás, diez de cada uno por vez: primero lo que tiene las palabras en su título, y delante el título que dice justo esas palabras; después lo que se movió hace menos, cada uno con la línea más nueva que las dice y nunca una nota que otra posterior reemplazó; cuando nada tiene todas las palabras, lo que tiene más, y lo dice. La ficha de un documento —sus títulos, páginas, lo que lo cita y las señales de que pide orden— o de una tarea —su estado, pasos y partes—, y sus vecinos de un tipo cuando los pide. Las partes abiertas de un proyecto, agrupadas en bloqueadas, en curso, esperándote y sin empezar. Los documentos de una carpeta, cada uno con su tamaño en caracteres, para que un agente vea cuál es largo antes de abrirlo. Sin pedir nada, tus listas, carpetas y etiquetas con sus cuentas; `list` «inbox» trae las tareas sin lista. Una lista que no existe se rechaza nombrando las tuyas, y una archivada igual responde. Al filtrar tareas, una respuesta que no las trae todas dice cuántas están abiertas y archivadas y cuántas apremian. En la ficha, aparte y en `guessed`, lo que Tisty cree que se parece y nadie enlazó, con lo que comparten y `method` (`words`, `typo` o `dictionary`): mira [[Parecidos|Red-de-Brainsty#parecidos]]. Lo que ocultaste queda fuera, ni siquiera como vecino o parecido. Una tarea descartada que tiene las palabras llega aparte, tres como mucho, como decidida que no se hace, solo con su título, su día y el motivo que quedó anotado |
| `read` | Leer una tarea: primero su estado, después su página, sus pasos y las notas escritas después de ese estado, con quién escribió cada una. Una tarea sin estado todavía llega con uno armado a partir de ella: cuántos pasos están hechos y la nota que la dio por hecha. Un paso tuyo que un agente dijo hecho llega como `[~]`, «said done, waiting on the person», y se cuenta aparte de los marcados hasta que tomes su palabra. Su página no copia la bitácora, y la respuesta dice cuántas notas tiene, haya página o no. Una página de más de unos 4000 caracteres llega como sus secciones, con el resumen y la última de las notas del agente; la bitácora trae hasta unos 3000 caracteres de notas, y entera si la pide. O leer un documento, o una parte; o, con `as`, el documento completo en Markdown estándar, para llevarlo a Confluence, Jira, Notion, Linear o GitHub con sus propias herramientas (mira más abajo); con `marks`, solo lo que resaltaste o pusiste en negrita, bajo cada sección. **Ideas** llega por partes, idea por idea, nunca como una lista de horas. Si relee con el `print` que le dio la lectura anterior y nada cambió, solo dice eso. Con `versions`, en vez del texto, lista sus [versiones anteriores](Documentos#versiones-anteriores), la más nueva primero: cuándo se escribió cada una, quién y cuánto pesa; el id de una tarea lista las de su página. Con `version` lee una, bajo un aviso de que no es lo que dice ahora. Una versión que reemplazó otro de tus equipos sale como `elsewhere`, con el nombre de ese equipo, y no se lee ni se recupera |

`find`, `docs`, `outline_doc`, `lists` y `tags` pasaron a `brain`, y `read_doc` a `read`. Un agente que todavía llama a una de ellas por su nombre recibe dónde quedó, y no pasa nada más. Cuando todas las filas de una lista de una respuesta comparten un campo, llega una sola vez, junto a las filas. Cómo las usa un agente en una sesión real está en [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

## Tareas

| Herramienta | Qué hace |
| :-- | :-- |
| `propose` | Proponer una tarea con un `title` y `for`, de quién es: «agent» si la hace el agente, «person» si solo puedes hacerla tú. Lleva sus pasos, fecha, lista y origen, o varias a la vez en `tasks`, cada una con sus propios campos. Una tarea hecha a partir de una idea de **Ideas** lleva la idea como origen, y la idea sale de la hoja, como cuando tú la conviertes en tarea. Un trabajo que descartaste antes se frena salvo que ponga `again` |
| `remind` | Hacer que una tarea suene a una hora |
| `reschedule` | Mover el día de una tarea que anotó un agente |
| `describe` | Escribir la descripción de una tarea abierta, también las tuyas, reemplazando la anterior |
| `rename` | Renombrar cualquier tarea abierta, también las tuyas |
| `note` | Añadir a la bitácora de una tarea lo que cambia algo (una decisión, algo descartado, un bloqueo): hasta 4000 caracteres por nota; lo más largo va a la página de la tarea. Una nota que deja sin efecto otra de un agente lo dice, y la vieja queda en la historia marcada como reemplazada |
| `reword_note` | Reescribir una nota de la bitácora de una tarea |
| `plan` | Añadir pasos a una tarea |
| `reword_step` | Reescribir un paso abierto |
| `unplan` | Quitar pasos abiertos de una tarea; con `to`, pasarlos a la tarea a la que pertenecen, y con `to_person`, a tu tarea compañera, que se crea la primera vez |
| `move` | Colgar una tarea de otra como parte, soltar una parte o mover una tarea a otra lista |
| `tick` | Marcar los pasos que hizo; en una tarea tuya esperan a que los confirmes. En la misma llamada deja el estado de la tarea, una sección «Estado» en su página que se escribe entera cada vez: lo hecho, lo que falta, las decisiones vigentes y lo descartado |
| `untick` | Quitar una marca que puso por error |
| `say_done` | Decir que una tarea está terminada, también las tuyas, para que lo confirmes, con la rama (`branch`) o el PR (`pr`) si los hay |
| `say_not_doing` | Decir que una tarea no debería hacerse, también las tuyas, para que lo confirmes |

## Documentos

| Herramienta | Qué hace |
| :-- | :-- |
| `sum_up` | Dejar un resumen corto de qué trata un documento, para el siguiente que lo lea. Reemplaza el anterior, viaja a tus otros equipos con la sincronización y entra en tus respaldos; nunca sale por Folio ni en un `.tistyx`, y la ventana no lo muestra |
| `write_doc` | Escribir un documento nuevo, o reescribir uno entero; con `compact`, poner en orden uno que creció por sesiones (ver abajo); con `translation_of`, escribirlo como otro idioma de un documento tuyo (uno que escribió otra persona se rechaza) |
| `append_doc` | Añadir a un documento, al final o bajo un título; en la página de una tarea, añadir bajo las notas del agente o, con `replace`, juntar las notas de todos los agentes en un resumen fechado de hasta 6000 caracteres, que conserva las líneas que escribiste y que `restore_doc` puede deshacer. En **Ideas**, cada párrafo o elemento de una lista se vuelve una idea propia, con su fecha y su color, de hasta 140 caracteres; la idea misma, con su línea `> [!IDEA]` y sus palabras tal como están en la hoja, añadida a otro documento sale de la hoja, como cuando la anexas desde la ventana; una nota `[!IDEA]` que no nombra ninguna idea de ahí queda como nota adhesiva propia. En el **Diario**, cuando lo abres a los agentes, solo añade al final de hoy, bajo una línea con el minuto y el nombre del agente |
| `edit_doc` | Cambiar un pasaje de un documento o de la página de una tarea |
| `edit_table` | Leer o cambiar una tabla de un documento por sus filas y columnas: poner una celda, añadir, quitar u ordenar filas, renombrar una columna o darle un tipo. Un valor que su columna no sabe leer se conserva y se nombra en la respuesta |
| `restore_doc` | Deshacer el último cambio que reemplazó texto de un documento, un pasaje cambiado o una escritura entera, desde sus [versiones anteriores](Documentos#versiones-anteriores) en este equipo; lo añadido al final y tus guardados no dejan nada que deshacer, y se niega si alguien escribió después. Deshacerlo dos veces lo devuelve. Con `version`, vuelve a una de las que lista `read`, por encima de todo lo escrito después, y lo que reemplaza queda guardado; sin `version`, deshace esa vuelta. Con una tarea, devuelve las notas que reemplazó un resumen |
| `folder` | Crear una carpeta |
| `file_doc` | Poner uno o varios documentos en una carpeta |
| `page_doc` | Hacer de documentos páginas de otro o sacarlas (`doc`), u ordenar las páginas de uno (`order` con `page_of`) |
| `tongue_doc` | Unir un documento a los idiomas de otro, o separarlo de los suyos |
| `archive_doc` | Archivar un documento o una de sus páginas, o desarchivarlo, solo cuando lo pides |
| `flag_doc` | Marcar uno o varios documentos que ya cumplieron su vida, con el motivo (una copia de otro, movido, reemplazado, erróneo, obsoleto, un borrador) y lo que queda en su lugar. Tú los borras, los archivas o les quitas la marca: una marca, nunca un borrado. Marca igual las ideas de **Ideas**, una por una, con un ◆ en cada nota, y nunca la hoja entera: la marca no saca ninguna idea, y un agente nunca archiva Ideas, ni la pone en una carpeta, ni la une a los idiomas de otro documento |
| `import_doc` | Traer un archivo Markdown de este equipo como documento |
| `export_doc` | Sacar un documento a una carpeta de este equipo |

## Archivos

| Herramienta | Qué hace |
| :-- | :-- |
| `attach` | Guardar una copia de un archivo en una tarea o dentro de un documento |

Al traer un archivo Markdown se copia lo que su texto señala, pero solo lo que está en la carpeta de ese archivo o más abajo, así que lo que se toma lo limita el archivo que nombraste y no lo que su texto pida. Las respuestas vuelven sin las rutas de tu disco, y un agente nombra lo que tocó por su título y su lista o carpeta, nunca por un identificador.

## El Diario

Tu [Diario](Documentos#diario) queda fuera de todas las herramientas hasta que lo abres a los agentes desde el propio Diario: `brain`, `catch_up` y los documentos que se ofrecen con `@` lo dejan fuera, y una herramienta que recibe una de sus páginas responde como ante un documento que no existe. Abierto, las herramientas de lectura lo leen y `append_doc` añade al final de hoy; toda herramienta que cambiaría, movería, marcaría o archivaría lo que escribiste ahí se rechaza, y ninguna puede empezar un diario. Sus versiones anteriores no se leen nunca, ni con el Diario abierto.

## La carpeta Agentes

Los documentos de la carpeta [[Agentes|Carpeta-AGENTS]] son las reglas, skills y memorias de tus agentes. Por MCP la carpeta se llama `AGENTS`, en cualquier idioma. Un agente los lee como cualquier documento, con `brain`, `read` y `export_doc`, pero no los cambia por su cuenta:

- **`write_doc`, `edit_doc` y `append_doc`** sobre uno de ellos no lo escriben: guardan el cambio aparte, como una propuesta, y la respuesta lo dice. Tú la apruebas o la descartas en su ficha ([[Una propuesta de tu agente|Carpeta-AGENTS#una-propuesta-de-tu-agente]]). Sobre una memoria, ni eso: se rechaza.
- **`read`** de un documento con una propuesta trae `proposal`, con `approved: false`, para que el agente sepa que lo que lee no es lo que llega a tus agentes.
- **Toda otra herramienta que los nombre se rechaza**, sin tocar nada: `page_doc`, `tongue_doc`, `attach`, `flag_doc`, `archive_doc`, `file_doc`, `sum_up`, `restore_doc`, un `edit_table` que cambia, y `folder` dentro de `AGENTS` o con ese nombre. Vale igual cuando el documento llega por `doc`, `page_of`, `translation_of`, `of`, `after`, `before`, `order`, `instead`, `folder` o `inside`.
- **La línea `agent:`**, la que dice a qué archivo va una regla, no la pone ni la cambia ningún agente, en ningún documento de ninguna carpeta.
- **Un documento de otro de tus equipos** el agente lo lee como datos: la respuesta de `read` trae `of_computer`, de qué equipo es, y `read_as`, que le dice que no es una regla suya para este equipo, y no puede proponer sobre él. Sobre los documentos de este equipo propone como siempre. No hay ninguna herramienta nueva.

El rechazo le dice qué puede hacer en su lugar: proponer el cambio o contarte qué cambiaría.

## Versiones anteriores de un documento

Un agente ve y recupera las mismas [versiones anteriores](Documentos#versiones-anteriores) que tú, con dos herramientas que ya usa, y solo las que guarda este equipo, porque las versiones no se sincronizan. Vaciarlas no tiene herramienta: eso es tuyo. Le pediste, por ejemplo, «vuelve “Paseos de Rina” a como estaba antes de tu cambio de esta mañana»; esto es lo que hace.

**1. Lista las versiones,** con `read` y `versions`:

```json
{ "doc": "k7w2pq9d-0003", "versions": true }
```

La respuesta trae la más nueva primero, cuántas guarda este equipo y su plazo en días (recortada aquí):

```json
{
  "title": "Paseos de Rina",
  "kept_here": 2,
  "kept_days": 30,
  "versions": [
    {
      "version": "a3f9c2e17b04",
      "written": "2026-11-18 10:42",
      "by": "Claude Code",
      "until": "2026-11-18 11:05",
      "bytes": 3120
    },
    {
      "version": "7d21e8b0c9aa",
      "written": "2026-11-17 19:10",
      "by": "the person",
      "until": "2026-11-18 10:42",
      "bytes": 2988
    }
  ]
}
```

`written` es cuándo se escribió ese texto y `until`, hasta cuándo lo dijo el documento. `version` es el comienzo de su huella, que basta para nombrarla. Con el id de una tarea lista las versiones de su página. Llegan de a 40, y `next` dice desde dónde pedir las más viejas con `cursor`.

**2. Lee la que busca,** con `read` y `version`:

```json
{ "doc": "k7w2pq9d-0003", "version": "7d21e8b0c9aa" }
```

Vuelve su texto bajo un aviso, para que no la confunda con lo que el documento dice ahora: «This is a past version of "Paseos de Rina" …, not what it says now». No trae `print`, así que no sirve para escribir encima con `edit_doc` ni `write_doc`. Una versión larga se lee por partes, como cualquier documento.

**3. Vuelve a ella,** con `restore_doc` y `version`:

```json
{ "doc": "k7w2pq9d-0003", "version": "7d21e8b0c9aa" }
```

El documento vuelve a decir lo de esa versión, por encima de todo lo escrito después, sin pedir `even_if_more`, porque la versión se nombró a propósito. Lo que reemplaza queda guardado, la respuesta lo cuenta en `went_back_to`, y `restore_doc` sin `version` deshace la vuelta.

**Una versión que guarda otro equipo** sale en la lista con `elsewhere` y el nombre de ese equipo:

```json
{
  "version": "c0ffee12ab34",
  "written": "2026-11-16 08:30",
  "by": "the person",
  "until": "2026-11-16 21:15",
  "bytes": 2950,
  "elsewhere": "Portátil de Mario"
}
```

Leerla o volver a ella se rechaza sin cambiar nada: «that version … was written over on Portátil de Mario, not on this machine». El agente te dice en qué equipo está, para que la abras allá.

## Llevar un documento a otra herramienta

Si tu agente también llega a Confluence, Jira, Notion, Linear o GitHub, puede leer un documento de Tisty con `as: "markdown"` y entregárselo: el Markdown estándar le sirve a casi todas, y sus propias herramientas lo convierten a su formato.

No todo hace el viaje, y Tisty lo dice en cada respuesta, para que el agente te avise antes de publicar:

- **Los archivos adjuntos** se quedan en Tisty. `export_doc` escribe el documento con ellos en una carpeta, y desde ahí la otra herramienta puede subirlos. Nunca los deja dentro de la carpeta de un agente, como `~/.claude` o la que nombra `CLAUDE_CONFIG_DIR`, aunque esté en Documentos: esa carpeta solo la escribe [[Agentes|Carpeta-AGENTS]].
- **Los diagramas** quedan como código Mermaid: GitHub, GitLab y Notion los dibujan; Confluence y Jira los muestran como código, salvo con una macro de Mermaid.
- **Los enlaces a otros documentos y tareas** quedan como texto.
- **Las páginas** van por separado: cada una se lee por su cuenta.
- **Los rotuladores, subrayados, iconos y secciones plegables** viajan como etiquetas HTML, que no todas las herramientas dibujan.
- **Un documento muy largo**, de más de 40 000 caracteres en Markdown estándar, no se entrega en la conversación: `export_doc` lo escribe como archivos.
- **Un documento archivado, o marcado como que ya cumplió su vida,** lo dice primero, para que el agente te pregunte antes de llevarlo a otro lado.

## Ordenar lo que creció

Un agente actualiza lo que existe en vez de acumular páginas nuevas. Cuando un documento creció por sesiones (una página por revisión, planes de trabajo ya terminado, copias), puede compactarlo: lee todas las páginas, lleva lo que vale la pena a donde corresponde, marca con su motivo las páginas que salen y reescribe el documento de una vez, con lo que vale hoy primero, las decisiones con su fecha y una historia de qué se movió adónde. Nunca afirma como hecho lo que solo pudo leer en un texto viejo: eso queda con su fecha y marcado como sin confirmar.

Tú decides sobre cada compactación:

- La ventana muestra **Compactado por … el …** en el documento, con **Deshacer** y **Conservar**. Deshacer deja el documento exactamente como estaba; también puedes pedirle al agente que lo restaure.
- Hasta que lo conserves o lo deshagas, el agente no escribe nada más en ese documento, salvo una compactación más para corregir la suya.
- Un documento que empezaste tú solo se compacta si lo pides en la conversación.
- Las páginas y copias que marcó siguen donde están, con su motivo, hasta que las borres, las archives o les quites la marca. Borrar o quitar la marca se hace una por una, o para todas las de un grupo con un clic.
