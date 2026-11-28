# Herramientas MCP

Las herramientas que `tisty mcp` le ofrece a un agente. El agente las elige por su cuenta; esta lista es para que sepas qué puede alcanzar. Todas funcionan solo en este equipo. La puerta viene abierta sola hasta que elijas —en una instalación nueva, y al actualizar si nunca la cerraste—, y mientras la mantengas cerrada desde **Configuración › Agentes › Puerta**, todo se rechaza.

Cada herramienta le dice además a la app donde corre el agente qué hace con tus cosas: si solo lee, si puede escribir encima o quitar algo que ya está, y si hacerla dos veces cambia algo más. Una app puede usarlo para preguntarte antes de las que escriben.

Las reglas detrás están en [[Qué puede hacer un agente|Qué-puede-hacer-un-agente]].

## Espacio y consulta

| Herramienta | Qué hace |
| :-- | :-- |
| `workspace_catch_up` | Cómo están las cosas al iniciar una sesión: tus listas y cuántas tareas abiertas tiene cada una, las tareas sin lista, carpetas y etiquetas más usadas; las tareas abiertas que apremian, por título y lista, para contártelas y leerlas con `workspace_read` (vencidas, por vencer, de hoy y atrasadas, cinco como máximo y el resto contado, sin las que un agente ya dio por hechas o descartadas ni las que ocultaste); lo que otros cambiaron desde su último `workspace_catch_up`, veinte de cada tipo por vez (o `since` para releer cambios sin mover el cursor); y la tarea abierta en que estaba, con su avance o el motivo por el que ya no está abierta. Nombra **Ideas** y cuántas ideas esperan o están marcadas. El día de tu [Revisión semanal](Tareas#revisión-semanal), indica cuánto espera en cada parada y cómo ayudarte |
| `workspace_search` | Encontrar cosas y consultar taxonomía sin abrir cuerpos enteros (`mode`: `search`, `source`, `project`, `folder`, `taxonomy`, `in_doc`, inferido si se omite). Busca tareas abiertas y archivadas juntas, diez de cada tipo por vez: títulos primero, luego lo más reciente, con la línea coincidente más nueva y nunca una nota reemplazada; cuando nada contiene todas las palabras, devuelve lo que más coincide y lo indica. `mode: "project"` con `part_of` agrupa las partes abiertas de un proyecto en bloqueadas, en curso, esperándote y sin empezar. `mode: "folder"` lista los documentos de una carpeta con su tamaño en caracteres. `mode: "in_doc"` busca líneas dentro de un documento con contexto (`around`). `mode: "taxonomy"` (o `{}`) devuelve listas, carpetas y etiquetas con sus cuentas; `list: "inbox"` trae las tareas sin lista. Lo que ocultaste queda fuera. Las tareas descartadas que coinciden llegan aparte, tres como mucho, solo con título, día y motivo |
| `workspace_read` | Leer una `task` o un `doc` (nunca ambos), con `view`: `content` (por defecto), `card`, `links`, `marks`, `table`, `versions` o `markdown`. En una tarea, `content` trae primero su estado, luego su página (esquema si pasa de 4000 caracteres), pasos (`[~]` si un agente dio por hecho uno tuyo) y notas posteriores al estado (`notes: 0` trae toda la bitácora); `fields` pide un subconjunto de campos. En un documento, `content` lee una sección (`section`), líneas (`from`/`to`) o un tramo (`chars`), devolviendo el esquema si pasa de 12 000 caracteres; **Ideas** llega idea por idea. `view: "card"` da títulos, páginas, enlaces entrantes, señales de orden y parecidos `guessed` (mira [[Parecidos|Red-de-Brainsty#parecidos]]), o el estado, pasos y partes de una tarea; `view: "links"` trae vecinos de un tipo (`cite`, `mention`, `structure`, `tag`, `uses`, `all`); `view: "marks"` trae solo resaltados o negritas; `view: "table"` lee filas, columnas y tipos de una tabla; `view: "markdown"` entrega el documento completo en Markdown estándar para Confluence, Jira, Notion, Linear o GitHub (mira abajo); `view: "versions"` lista sus [versiones anteriores](Documentos#versiones-anteriores) en este equipo y `version` lee una bajo aviso (las versiones `elsewhere` de otro equipo no se leen ni recuperan). Con `print` evita releer si nada cambió |
| `workspace_guide` | Leer las guías de Tisty de forma progresiva (`kind`: `rules`, `markdown` o `help`). Omite `topic` para traer el índice compacto y luego pide solo el tema necesario (`rules`: `tasks`, `documents`, `tidying`, `ideas` o `all`; `markdown` y `help` aceptan `lang`: `en` o `es`) |

Los 36 nombres anteriores de una sola función (como `brain`, `catch_up`, `read`, `propose`, `write_doc`, `edit_doc`, `append_doc`, `flag_doc`, `say_done`, además de `find`, `docs`, `outline_doc`, `lists`, `tags` y `read_doc`) ahora pasan a estas 15 herramientas. Si un agente todavía llama a un nombre anterior, recibe la herramienta y parámetros exactos de reemplazo, y no pasa nada más. Cuando todas las filas de una lista comparten un campo, llega una sola vez junto a las filas. Cómo las usa un agente en una sesión real está en [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

## Tareas

| Herramienta | Qué hace |
| :-- | :-- |
| `task_create` | Crear una tarea con `title` y `for`, de quién es: «agent» si la hace y comprueba el agente, «person» si solo puedes hacerla tú. Lleva pasos, `parts` (partes a un nivel), `part_of`, fechas, recordatorios, prioridad, lista, etiquetas y `source`, o hasta 32 tareas a la vez en `tasks`, cada una con sus campos. Una tarea creada desde una idea en **Ideas** lleva la idea en `source` y la saca de la hoja. Un trabajo que descartaste antes se frena salvo que indique `again` |
| `task_update` | Actualizar metadatos de una tarea abierta: `title`, `description` (reemplaza la anterior), `date` y `deadline` (solo tareas del agente; `null` los limpia), `add_reminders` (horas de aviso, aditivo), `list` y `part_of` (colgarla como parte o `null` para soltarla) |
| `task_edit_steps` | Gestionar pasos y estado de una tarea abierta (`action`: `add`, `tick`, `untick`, `reword`, `remove`, `set_state`). En una tarea tuya, `tick` deja la palabra del agente esperando tu confirmación. `tick` y `set_state` escriben entera la sección «Estado» en su página: lo hecho, lo que falta, decisiones vigentes y descartes. En `remove`, `to` pasa pasos abiertos a otra tarea y `to_person` entrega pasos solo tuyos a tu única tarea compañera (creada la primera vez), que nunca deja abierta la del agente |
| `task_note` | Añadir (`action: "add"`) a la bitácora de una tarea lo que cambia algo —una decisión, un descarte, un bloqueo— hasta 4000 caracteres (lo más largo va en la página de la tarea), con `replaces` para dejar sin efecto una nota previa de un agente (queda marcada como reemplazada); o reescribir (`action: "reword"`) una nota existente por ID |
| `task_resolve` | Dar una tarea abierta por terminada (`outcome: "done"`, con prueba de verificación en `body` y `branch` o `pr` opcionales) o por no hacer (`outcome: "not_doing"`, con el motivo) —también las tuyas, siempre para que tú confirmes. Nunca cierra ni borra |

## Documentos

| Herramienta | Qué hace |
| :-- | :-- |
| `doc_create` | Crear un documento nuevo (`body`, `folder` opcional, `page_of` para adjuntarlo como página a un nivel, o `translation_of` + `lang` para escribir otro idioma de un documento tuyo; uno que escribió otra persona se rechaza) |
| `doc_edit` | Editar o deshacer cambios en un documento o página de tarea (`mode`: `patch`, `append`, `rewrite`, `compact`, `digest_task_notes`, `restore`). `patch` cambia un pasaje (`old`, o `section`/`from`+`to` con `print`). `append` añade al final o bajo un título; con un ID de tarea añade bajo notas del agente; en **Ideas** cada párrafo o ítem se vuelve una nota adhesiva fechada (<=140 caracteres), y añadir una `> [!IDEA]` existente a otro documento la saca de Ideas; en un **Diario** abierto solo añade al final de hoy bajo hora y nombre del agente. `rewrite` reemplaza el cuerpo entero con `print`; `compact` ordena un documento que creció por sesiones (mira abajo). `digest_task_notes` junta todas las notas de agentes en la página de una tarea en un resumen fechado (<=6000 caracteres) conservando tus líneas. `restore` deshace la última edición que reemplazó texto, vuelve a una `version` listada por `workspace_read(view: "versions")`, o devuelve las notas reemplazadas por un resumen |
| `doc_edit_table` | Cambiar una tabla de un documento o página de tarea por filas y columnas (`action`: `set`, `add_row`, `remove_row`, `sort`, `set_type`, `rename_column`, `add_column`, `remove_column`) sin tocar Markdown crudo. Consulta la tabla antes con `workspace_read(view: "table")`. Un valor que su columna no sabe leer se conserva y se nombra en la respuesta |
| `doc_organize` | Organizar la biblioteca (`action`: `create_folder` hasta 6 niveles, `file` uno o varios documentos en una carpeta, `add_page` / `remove_page` / `reorder_pages` para páginas a un nivel, y `link_translation` / `unlink_translation` para juegos multiidioma) |
| `doc_annotate` | Gestionar estados fuera del texto (`action`: `archive`, `unarchive`, `flag`, `summarize`). `archive`/`unarchive` archiva o desarchiva un documento o página solo cuando lo pides. `flag` marca uno o varios documentos (o ideas sueltas en **Ideas** como `hoja#encabezado`, con un ◆ en cada nota y nunca la hoja entera) con el motivo (`duplicate_of`, `moved`, `superseded`, `wrong`, `obsolete`, `scratch`) y qué queda en su lugar: una marca, nunca un borrado. `summarize` deja un resumen solo para agentes (viaja con sincronización y respaldos, nunca sale por Folio ni `.tistyx`, oculto en la ventana) |

## Archivos

| Herramienta | Qué hace |
| :-- | :-- |
| `file_transfer` | Transferir archivos locales entre carpetas permitidas y Tisty (`action`: `attach` copia un archivo de hasta 650 MB en una tarea o documento; `import_doc` trae un archivo Markdown local como documento; `export_doc` saca un documento y sus adjuntos a una carpeta de este equipo) |

Al traer un archivo Markdown se copia lo que su texto señala, pero solo lo que está en la carpeta de ese archivo o más abajo, así que lo que se toma lo limita el archivo que nombraste y no lo que su texto pida. Las respuestas vuelven sin las rutas de tu disco, y un agente nombra lo que tocó por su título y su lista o carpeta, nunca por un identificador.

## El Diario

Tu [Diario](Documentos#diario) queda fuera de todas las herramientas hasta que lo abres a los agentes desde el propio Diario: `workspace_search`, `workspace_catch_up` y los documentos que se ofrecen con `@` lo dejan fuera, y una herramienta que recibe una de sus páginas responde como ante un documento que no existe. Abierto, las herramientas de lectura lo leen y `doc_edit` (`mode: "append"`) añade al final de hoy; toda herramienta que cambiaría, movería, marcaría o archivaría lo que escribiste ahí se rechaza, y ninguna puede empezar un diario. Sus versiones anteriores no se leen nunca, ni con el Diario abierto.

## La carpeta Agentes

Los documentos de la carpeta [[Agentes|Carpeta-AGENTS]] son las reglas, skills y memorias de tus agentes. Por MCP la carpeta se llama `AGENTS`, en cualquier idioma. Un agente los lee como cualquier documento, con `workspace_search`, `workspace_read` y `file_transfer` (`action: "export_doc"`), pero no los cambia por su cuenta:

- **`doc_edit` (`rewrite`, `patch` y `append`)** sobre uno de ellos no lo escribe: guarda el cambio aparte, como una propuesta, y la respuesta lo dice. Tú la apruebas o la descartas en su ficha ([[Una propuesta de tu agente|Carpeta-AGENTS#una-propuesta-de-tu-agente]]). Sobre una memoria, ni eso: se rechaza.
- **`workspace_read`** de un documento con una propuesta trae `proposal`, con `approved: false`, para que el agente sepa que lo que lee no es lo que llega a tus agentes.
- **Toda otra herramienta que los nombre se rechaza**, sin tocar nada: `doc_create` dentro de `AGENTS`, `doc_edit` (`compact`, `restore`), `doc_edit_table`, `doc_organize` (incluido `create_folder` dentro de `AGENTS` o con ese nombre), `doc_annotate` (`archive`, `unarchive`, `flag`, `summarize`) y `file_transfer` (`attach`, `import_doc`). Vale igual cuando el documento llega por `doc`, `docs`, `parent`, `page_of`, `translation_of`, `after`, `before`, `order`, `instead` o `folder`.
- **La línea `agent:`**, la que dice a qué archivo va una regla, no la pone ni la cambia ningún agente, en ningún documento de ninguna carpeta.
- **Un documento de otro de tus equipos** el agente lo lee como datos: la respuesta de `workspace_read` trae `of_computer`, de qué equipo es, y `read_as`, que le dice que no es una regla suya para este equipo, y no puede proponer sobre él. Sobre los documentos de este equipo propone como siempre. No hay ninguna herramienta nueva.

El rechazo le dice qué puede hacer en su lugar: proponer el cambio o contarte qué cambiaría.

## Versiones anteriores de un documento

Un agente ve y recupera las mismas [versiones anteriores](Documentos#versiones-anteriores) que tú, con dos herramientas que ya usa, y solo las que guarda este equipo, porque las versiones no se sincronizan. Vaciarlas no tiene herramienta: eso es tuyo. Le pediste, por ejemplo, «vuelve “Paseos de Rina” a como estaba antes de tu cambio de esta mañana»; esto es lo que hace.

**1. Lista las versiones,** con `workspace_read` y `view: "versions"`:

```json
{ "doc": "k7w2pq9d-0003", "view": "versions" }
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

**2. Lee la que busca,** con `workspace_read` y `version`:

```json
{ "doc": "k7w2pq9d-0003", "version": "7d21e8b0c9aa" }
```

Vuelve su texto bajo un aviso, para que no la confunda con lo que el documento dice ahora: «This is a past version of "Paseos de Rina" …, not what it says now». No trae `print`, así que no sirve para escribir encima con `doc_edit`. Una versión larga se lee por partes, como cualquier documento.

**3. Vuelve a ella,** con `doc_edit` (`mode: "restore"`) y `version`:

```json
{ "doc": "k7w2pq9d-0003", "mode": "restore", "version": "7d21e8b0c9aa" }
```

El documento vuelve a decir lo de esa versión, por encima de todo lo escrito después, sin pedir `even_if_more`, porque la versión se nombró a propósito. Lo que reemplaza queda guardado, la respuesta lo cuenta en `went_back_to`, y `doc_edit` (`mode: "restore"`) sin `version` deshace la vuelta.

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

Si tu agente también llega a Confluence, Jira, Notion, Linear o GitHub, puede leer un documento de Tisty con `workspace_read` (`view: "markdown"`) y entregárselo: el Markdown estándar le sirve a casi todas, y sus propias herramientas lo convierten a su formato.

No todo hace el viaje, y Tisty lo dice en cada respuesta, para que el agente te avise antes de publicar:

- **Los archivos adjuntos** se quedan en Tisty. `file_transfer` (`action: "export_doc"`) escribe el documento con ellos en una carpeta, y desde ahí la otra herramienta puede subirlos. Nunca los deja dentro de la carpeta de un agente, como `~/.claude` o la que nombra `CLAUDE_CONFIG_DIR`, aunque esté en Documentos: esa carpeta solo la escribe [[Agentes|Carpeta-AGENTS]].
- **Los diagramas** quedan como código Mermaid: GitHub, GitLab y Notion los dibujan; Confluence y Jira los muestran como código, salvo con una macro de Mermaid.
- **Los enlaces a otros documentos y tareas** quedan como texto.
- **Las páginas** van por separado: cada una se lee por su cuenta.
- **Los rotuladores, subrayados, iconos y secciones plegables** viajan como etiquetas HTML, que no todas las herramientas dibujan.
- **Un documento muy largo**, de más de 40 000 caracteres en Markdown estándar, no se entrega en la conversación: `file_transfer` (`action: "export_doc"`) lo escribe como archivos.
- **Un documento archivado, o marcado como que ya cumplió su vida,** lo dice primero, para que el agente te pregunte antes de llevarlo a otro lado.

## Ordenar lo que creció

Un agente actualiza lo que existe en vez de acumular páginas nuevas. Cuando un documento creció por sesiones (una página por revisión, planes de trabajo ya terminado, copias), puede compactarlo: lee todas las páginas, lleva lo que vale la pena a donde corresponde, marca con su motivo las páginas que salen y reescribe el documento de una vez, con lo que vale hoy primero, las decisiones con su fecha y una historia de qué se movió adónde. Nunca afirma como hecho lo que solo pudo leer en un texto viejo: eso queda con su fecha y marcado como sin confirmar.

Tú decides sobre cada compactación:

- La ventana muestra **Compactado por … el …** en el documento, con **Deshacer** y **Conservar**. Deshacer deja el documento exactamente como estaba; también puedes pedirle al agente que lo restaure.
- Hasta que lo conserves o lo deshagas, el agente no escribe nada más en ese documento, salvo una compactación más para corregir la suya.
- Un documento que empezaste tú solo se compacta si lo pides en la conversación.
- Las páginas y copias que marcó siguen donde están, con su motivo, hasta que las borres, las archives o les quites la marca. Borrar o quitar la marca se hace una por una, o para todas las de un grupo con un clic.
