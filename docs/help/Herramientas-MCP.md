# Herramientas MCP

Las herramientas que `tisty mcp` le ofrece a un asistente. El asistente las elige por su cuenta; esta lista es para que sepas qué puede alcanzar. Todas funcionan solo en este equipo, y todo se rechaza hasta que dejas entrar al asistente desde **Configuración › Asistentes**.

Las reglas detrás están en [[Qué puede hacer un asistente|Qué-puede-hacer-un-asistente]].

## Mirar

| Herramienta | Qué hace |
| :-- | :-- |
| `rules` | Las reglas completas para trabajar aquí; con `topic` «tidying», cómo poner las cosas en orden. Los asistentes las piden primero |
| `language` | El lenguaje de Tisty, para que lo que escribe sea algo que Tisty conoce |
| `help` | Cómo funciona Tisty, desde estas mismas páginas de ayuda, para que responda en vez de suponer |
| `catch_up` | Cómo están las cosas y qué se movió desde la última vez que miró |
| `find` | Buscar en la lista y en el archivo |
| `read` | Leer una tarea completa |
| `lists` | Las listas que existen |
| `tags` | Las etiquetas que ya están en uso |

## Tareas

| Herramienta | Qué hace |
| :-- | :-- |
| `propose` | Proponer una tarea con un `title`, y sus pasos, fecha, lista y origen, o varias a la vez en `tasks`, cada una con sus propios campos |
| `remind` | Hacer que una tarea suene a una hora |
| `reschedule` | Mover el día de una tarea que anotó |
| `describe` | Describir una tarea que todavía no tiene descripción |
| `rename` | Renombrar una tarea que anotó un agente |
| `note` | Añadir a la bitácora de una tarea |
| `reword_note` | Reescribir una nota de la bitácora |
| `plan` | Añadir pasos a una tarea |
| `reword_step` | Reescribir un paso abierto |
| `unplan` | Quitar pasos abiertos de una tarea o, con `to`, pasarlos a la tarea a la que pertenecen |
| `move` | Colgar una tarea de otra como parte, soltar una parte o mover a otra lista una tarea que anotó |
| `tick` | Marcar los pasos que hizo; en una tarea tuya esperan a que los confirmes |
| `untick` | Quitar una marca que puso por error |
| `say_done` | Decir que una tarea está terminada, también las tuyas, para que lo confirmes |
| `say_not_doing` | Decir que una tarea no debería hacerse, también las tuyas, para que lo confirmes |

## Documentos

| Herramienta | Qué hace |
| :-- | :-- |
| `docs` | Los documentos y las carpetas, con de qué va cada uno |
| `outline_doc` | Qué hay en un documento: sus títulos y partes, y las señales de que pide orden: documentos con título parecido, títulos repetidos entre páginas y enlaces que no llevan a nada |
| `read_doc` | Leer un documento, o una parte; o, con `as`, el documento completo en Markdown estándar, para llevarlo a Confluence, Jira, Notion, Linear o GitHub con sus propias herramientas (mira más abajo) |
| `sum_up` | Dejar un resumen corto de qué trata un documento, para el siguiente que lo lea |
| `write_doc` | Escribir un documento nuevo, o reescribir uno entero; con `compact`, poner en orden uno que creció por sesiones (ver abajo) |
| `append_doc` | Añadir a un documento, al final o bajo un título; en la página de una tarea, reescribir entera su sección de notas con `replace` |
| `edit_doc` | Cambiar un pasaje de un documento |
| `restore_doc` | Volver un documento a como estaba, desde la copia guardada junto a los documentos |
| `folder` | Crear una carpeta |
| `file_doc` | Poner uno o varios documentos en una carpeta |
| `page_doc` | Hacer de documentos páginas de otro o sacarlas (`doc`), u ordenar las páginas de uno (`order` con `page_of`) |
| `archive_doc` | Archivar un documento o una de sus páginas, o desarchivarlo |
| `flag_doc` | Marcar uno o varios documentos para que los borres, con el motivo (una copia de otro, movido, reemplazado, erróneo, obsoleto, un borrador) y lo que queda en su lugar. Una marca, nunca un borrado |
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
- **Un documento archivado, o marcado como obsoleto,** lo dice primero, para que el asistente te pregunte antes de llevarlo a otro lado.

## Ordenar lo que creció

Un asistente actualiza lo que existe en vez de acumular páginas nuevas. Cuando un documento creció por sesiones (una página por revisión, planes de trabajo ya terminado, copias), puede compactarlo: lee todas las páginas, lleva lo que vale la pena a donde corresponde, marca con su motivo las páginas que salen y reescribe el documento de una vez, con lo que vale hoy primero, las decisiones con su fecha y una historia de qué se movió adónde. Nunca afirma como hecho lo que solo pudo leer en un texto viejo: eso queda con su fecha y marcado como sin confirmar.

Tú decides sobre cada compactación:

- La ventana muestra **Compactado por … el …** en el documento, con **Deshacer** y **Conservar**. Deshacer deja el documento exactamente como estaba; también puedes pedirle al asistente que lo restaure.
- Hasta que lo conserves o lo deshagas, el asistente no escribe nada más en ese documento, salvo una compactación más para corregir la suya.
- Un documento que empezaste tú solo se compacta si lo pides en la conversación.
- Las páginas y copias que marcó siguen donde están, con su motivo, hasta que las borres: una por una, o todas las de un grupo con un clic.
