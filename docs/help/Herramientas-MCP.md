# Herramientas MCP

Las herramientas que `tisty mcp` le ofrece a un asistente. El asistente las elige por su cuenta; esta lista es para que sepas qué puede alcanzar. Todas funcionan solo en este equipo. La puerta viene abierta sola hasta que elijas —en una instalación nueva, y al actualizar si nunca la cerraste—, y mientras la mantengas cerrada desde **Configuración › Asistentes**, todo se rechaza.

Cada herramienta le dice además a la app donde corre el asistente qué hace con tus cosas: si solo lee, si puede escribir encima o quitar algo que ya está, y si hacerla dos veces cambia algo más. Una app puede usarlo para preguntarte antes de las que escriben.

Las reglas detrás están en [[Qué puede hacer un asistente|Qué-puede-hacer-un-asistente]].

## Mirar

| Herramienta | Qué hace |
| :-- | :-- |
| `rules` | Las reglas para trabajar aquí, una guía a la vez («tasks», «documents», «tidying», «ideas» o «all»); sola, lo básico y un índice. Los asistentes piden la que necesitan primero |
| `language` | El lenguaje de Tisty, para que lo que escribe sea algo que Tisty conoce |
| `help` | Cómo funciona Tisty, desde estas mismas páginas de ayuda, para que responda en vez de suponer |
| `catch_up` | Cómo están las cosas, con las listas, las carpetas y las etiquetas más usadas; las tareas abiertas que apremian, por título y lista, para que te las cuente y las lea con `read`: las vencidas, después las por vencer, las de hoy y las atrasadas, cinco como máximo y el resto contado, sin las que un asistente ya dijo que están hechas o que no deben hacerse, que te esperan a ti, ni las que ocultaste; lo que otros cambiaron desde su último `catch_up`, veinte de cada tipo a la vez, y el siguiente trae el resto; y la tarea abierta en la que estaba o, si ya no lo está, por qué: dijo que estaba hecha o que no debía hacerse, o la cerraste, la descartaste o la ocultaste. Nombra la **Lluvia de ideas** y dice cuántas ideas esperan y cuántas están marcadas |
| `find` | Buscar en tareas y documentos, el archivo incluido; los documentos marcados como que ya cumplieron su vida van después de los vigentes, y lo dicen |
| `read` | Leer una tarea: su página, sus pasos y sus últimas notas, con quién escribió cada una. Una página de más de unos 4000 caracteres llega como sus secciones, con el resumen y la última de las notas del asistente; la bitácora trae las últimas notas hasta unos 3000 caracteres, y más si las pide |

`lists` y `tags` todavía le responden a un asistente que las pide por su nombre, pero ya no se ofrecen: `catch_up` ya trae las dos.

## Tareas

| Herramienta | Qué hace |
| :-- | :-- |
| `propose` | Proponer una tarea con un `title`, y sus pasos, fecha, lista y origen, o varias a la vez en `tasks`, cada una con sus propios campos. Una tarea hecha a partir de una idea de **Lluvia de ideas** lleva la idea como origen, y la idea sale de la hoja, como cuando tú la conviertes en tarea |
| `remind` | Hacer que una tarea suene a una hora |
| `reschedule` | Mover el día de una tarea que anotó un agente |
| `describe` | Escribir la descripción de una tarea abierta, también las tuyas, reemplazando la anterior |
| `rename` | Renombrar cualquier tarea abierta, también las tuyas |
| `note` | Añadir a la bitácora de una tarea: hasta 4000 caracteres por nota; lo más largo va a la página de la tarea |
| `reword_note` | Reescribir una nota de la bitácora de una tarea |
| `plan` | Añadir pasos a una tarea |
| `reword_step` | Reescribir un paso abierto |
| `unplan` | Quitar pasos abiertos de una tarea o, con `to`, pasarlos a la tarea a la que pertenecen |
| `move` | Colgar una tarea de otra como parte, soltar una parte o mover una tarea a otra lista |
| `tick` | Marcar los pasos que hizo; en una tarea tuya esperan a que los confirmes |
| `untick` | Quitar una marca que puso por error |
| `say_done` | Decir que una tarea está terminada, también las tuyas, para que lo confirmes |
| `say_not_doing` | Decir que una tarea no debería hacerse, también las tuyas, para que lo confirmes |

## Documentos

| Herramienta | Qué hace |
| :-- | :-- |
| `docs` | Los documentos y las carpetas, con de qué va cada uno |
| `outline_doc` | Qué hay en un documento: sus títulos y partes, y las señales de que pide orden: documentos con título parecido, títulos repetidos entre páginas y enlaces que no llevan a nada |
| `read_doc` | Leer un documento, o una parte; o, con `as`, el documento completo en Markdown estándar, para llevarlo a Confluence, Jira, Notion, Linear o GitHub con sus propias herramientas (mira más abajo). **Lluvia de ideas** llega por partes, idea por idea, nunca como una lista de horas |
| `sum_up` | Dejar un resumen corto de qué trata un documento, para el siguiente que lo lea |
| `write_doc` | Escribir un documento nuevo, o reescribir uno entero; con `compact`, poner en orden uno que creció por sesiones (ver abajo); con `translation_of`, escribirlo como otro idioma de un documento tuyo (uno que escribió otra persona se rechaza) |
| `append_doc` | Añadir a un documento, al final o bajo un título; en la página de una tarea, añadir bajo las notas del asistente o, con `replace`, juntar las notas de todos los asistentes en un resumen fechado de hasta 6000 caracteres, que conserva las líneas que escribiste y que `restore_doc` puede deshacer. En **Lluvia de ideas**, cada párrafo o elemento de una lista se vuelve una idea propia, con su fecha y su color, de hasta 140 caracteres; la idea misma, con su línea `> [!IDEA]` y sus palabras tal como están en la hoja, añadida a otro documento sale de la hoja, como cuando la anexas desde la ventana; una nota `[!IDEA]` que no nombra ninguna idea de ahí queda como nota adhesiva propia |
| `edit_doc` | Cambiar un pasaje de un documento o de la página de una tarea |
| `edit_table` | Leer o cambiar una tabla de un documento por sus filas y columnas: poner una celda, añadir, quitar u ordenar filas, renombrar una columna o darle un tipo. Un valor que su columna no sabe leer se conserva y se nombra en la respuesta |
| `restore_doc` | Volver un documento a como estaba, desde la copia guardada junto a los documentos; con una tarea, devuelve las notas que reemplazó un resumen |
| `folder` | Crear una carpeta |
| `file_doc` | Poner uno o varios documentos en una carpeta |
| `page_doc` | Hacer de documentos páginas de otro o sacarlas (`doc`), u ordenar las páginas de uno (`order` con `page_of`) |
| `tongue_doc` | Unir un documento a los idiomas de otro, o separarlo de los suyos |
| `archive_doc` | Archivar un documento o una de sus páginas, o desarchivarlo, solo cuando lo pides |
| `flag_doc` | Marcar uno o varios documentos que ya cumplieron su vida, con el motivo (una copia de otro, movido, reemplazado, erróneo, obsoleto, un borrador) y lo que queda en su lugar. Tú los borras, los archivas o les quitas la marca: una marca, nunca un borrado. Marca igual las ideas de **Lluvia de ideas**, una por una, con un ◆ en cada nota, y nunca la hoja entera: la marca no saca ninguna idea, y un asistente nunca archiva la Lluvia de ideas, ni la pone en una carpeta, ni la une a los idiomas de otro documento |
| `import_doc` | Traer un archivo Markdown de este equipo como documento |
| `export_doc` | Sacar un documento a una carpeta de este equipo |

## Archivos

| Herramienta | Qué hace |
| :-- | :-- |
| `attach` | Guardar una copia de un archivo en una tarea o dentro de un documento |

Al traer un archivo Markdown se copia lo que su texto señala, pero solo lo que está en la carpeta de ese archivo o más abajo, así que lo que se toma lo limita el archivo que nombraste y no lo que su texto pida. Las respuestas vuelven sin las rutas de tu disco, y un asistente nombra lo que tocó por su título y su lista o carpeta, nunca por un identificador.

## Llevar un documento a otra herramienta

Si tu asistente también llega a Confluence, Jira, Notion, Linear o GitHub, puede leer un documento de Tisty con `as: "markdown"` y entregárselo: el Markdown estándar le sirve a casi todas, y sus propias herramientas lo convierten a su formato.

No todo hace el viaje, y Tisty lo dice en cada respuesta, para que el asistente te avise antes de publicar:

- **Los archivos adjuntos** se quedan en Tisty. `export_doc` escribe el documento con ellos en una carpeta, y desde ahí la otra herramienta puede subirlos.
- **Los diagramas** quedan como código Mermaid: GitHub, GitLab y Notion los dibujan; Confluence y Jira los muestran como código, salvo con una macro de Mermaid.
- **Los enlaces a otros documentos y tareas** quedan como texto.
- **Las páginas** van por separado: cada una se lee por su cuenta.
- **Un documento archivado, o marcado como que ya cumplió su vida,** lo dice primero, para que el asistente te pregunte antes de llevarlo a otro lado.

## Ordenar lo que creció

Un asistente actualiza lo que existe en vez de acumular páginas nuevas. Cuando un documento creció por sesiones (una página por revisión, planes de trabajo ya terminado, copias), puede compactarlo: lee todas las páginas, lleva lo que vale la pena a donde corresponde, marca con su motivo las páginas que salen y reescribe el documento de una vez, con lo que vale hoy primero, las decisiones con su fecha y una historia de qué se movió adónde. Nunca afirma como hecho lo que solo pudo leer en un texto viejo: eso queda con su fecha y marcado como sin confirmar.

Tú decides sobre cada compactación:

- La ventana muestra **Compactado por … el …** en el documento, con **Deshacer** y **Conservar**. Deshacer deja el documento exactamente como estaba; también puedes pedirle al asistente que lo restaure.
- Hasta que lo conserves o lo deshagas, el asistente no escribe nada más en ese documento, salvo una compactación más para corregir la suya.
- Un documento que empezaste tú solo se compacta si lo pides en la conversación.
- Las páginas y copias que marcó siguen donde están, con su motivo, hasta que las borres, las archives o les quites la marca. Borrar o quitar la marca se hace una por una, o para todas las de un grupo con un clic.
