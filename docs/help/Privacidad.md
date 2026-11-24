# Privacidad

Tisty no recoge nada y no guarda nada tuyo en un servidor. La única excepción la decides tú: un documento que compartes por link va, sellado, a [Folio](https://github.com/rgdevment/Folio-by-Tisty), un servicio aparte. Sin cuenta, sin telemetría, sin analítica, sin reportes de fallos. No es una promesa que tengas que creer: es una propiedad del código, que es [público](https://github.com/rgdevment/Tisty). Si quieres, pásale un monitor de red.

Esta página es un resumen. La política completa, incluidas las partes que no tranquilizan, está en [PRIVACY.md](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md) (en inglés), y el razonamiento de seguridad en [SECURITY.md](https://github.com/rgdevment/Tisty/blob/main/SECURITY.md).

![Software por convicción, no por suscripción: sin cuenta, sin suscripción, sin telemetría, y tus archivos se quedan en tu equipo](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/web/es-10-cierre.png)

## Qué se guarda

Casi todo vive en una carpeta de tu disco:

- **Tareas y eventos**: JSONL en texto plano, un registro que solo crece. Se lee con `cat`, se busca con `grep` y se procesa con `jq`.
- **Documentos**: archivos Markdown normales.
- **Agentes, si la usas**: las reglas, skills y memoria de tus agentes, como documentos. Si la sincronizas, Tisty escribe además, fuera de esta carpeta, los archivos que lee cada agente ([qué lee y cuándo](#tus-agentes-en-la-carpeta-agentes)).
- **Versiones anteriores de tus documentos**: Markdown como los documentos, en la carpeta `history`. Cada una se guarda 30 días desde que fue reemplazada, o lo que elijas en **Configuración › Documentos y firma › Guardar las versiones anteriores**; la más reciente de cada documento, y lo que te devolvería **Volver a lo que decía antes**, no vencen. Se quedan en este equipo: nunca se sincronizan ni van en un paquete, pero sí en tus respaldos. [Cómo funcionan](Documentos#versiones-anteriores).
- **Lo que un guardado rechazó**: si un documento se bloqueó o quedó demasiado largo mientras escribías, o el cambio de una tabla llegó cuando ya habías dejado su página o mientras cambiaba ese mismo lugar, lo que no se pudo guardar queda en el almacenamiento de la propia ventana, en este equipo, hasta que lo vuelvas a poner, lo descartes o elimines el documento en esta ventana. No se sincroniza, ni va en un respaldo, un paquete o un link.
- **Tu diario**: Markdown como cualquier documento, un libro por año. Ningún agente lo ve hasta que lo abres desde el propio Diario, en Tisty 1.48 o más nuevo; nunca se comparte por link, y viaja con tu sincronización y tus respaldos como lo demás, y dentro de un paquete `.tistyx` cuando exportas todo.
- **Adjuntos**: los archivos tal cual, sin modificar.
- **La base de Brainsty**: cómo se unen tus cosas, las palabras de peso de cada documento y tarea y qué cosas se parecen, en la caché de este equipo. Se rehace sola con lo que guardas, así que no se sincroniza ni va en un paquete, a Folio o a una exportación. Tampoco va en tus respaldos. Las tareas ocultas y las descartadas, tu Diario y los documentos de Agentes no dejan ahí palabras, y de lo demás solo cuenta la prosa: ni el código, ni las rutas, ni las notas de avance de una tarea. [Cómo funciona](Red-de-Brainsty#parecidos).
- **La nota de un agente sobre un documento**, la que deja con `sum_up` para el siguiente que lo lea: un evento más del registro, así que viaja a tus otros equipos con la sincronización, a tu nube y a tus respaldos. Nunca sale por Folio ni en un `.tistyx`, y la ventana no la muestra. [Qué hace cada herramienta](Herramientas-MCP#documentos).
- **Configuración local**: incluye el identificador de este equipo y su clave de firma. La mitad privada de la clave nunca sale del equipo.
- **Tu frase, si creas una**: las 24 palabras que cifran tu nube, guardadas en el equipo donde la creaste, en el llavero del sistema en macOS o en un archivo sellado para tu usuario en Windows. Nunca se sincroniza, se respalda ni va en un paquete. Nadie de Tisty ni de Folio te la pedirá; si alguien te la pide, es un engaño.
- **Lo que compartes por link**: la llave de cada link y los tokens que lo administran o editan viven en el registro, para que tus otros equipos puedan manejarlo. Viajan con tu sincronización, así que quien pueda leer una copia sin cifrar puede abrir esos links. Cuando dejas de compartir un documento, los nombres de quienes lo editaron también se quedan en el registro, para que al volver a compartirlo se te recuerde quiénes fueron. Cuánto pidió esperar Folio a este equipo, y qué documentos compartió hoy por primera vez, se quedan en este equipo.
- **Recordatorios agendados en el sistema**: la hora, el título de la tarea y su identificador, por cada recordatorio del próximo mes, se entregan a Windows o macOS para que suenen con Tisty cerrado. Se quedan en este equipo.
- **Una carpeta `private/`**: lo que pongas ahí nunca sale del equipo, por ningún camino.

Nada está ofuscado, y nada está en un formato que solo Tisty sepa leer.

**Nada de tu equipo está cifrado en reposo.** La protección son los permisos de tu sistema operativo. Es una decisión, no un descuido: el cifrado rompería la unión bloque a bloque de documentos de la que depende la sincronización. Lo que Tisty guarda en una nube con la que habla directo es otra cosa: desde la 1.36 lo puedes cifrar con tu frase ([cómo funciona](Sincronizar-y-respaldar#cifrar-tu-nube)). Una carpeta que otro programa mantiene al día nunca se cifra.

> [!WARNING]
> **Un registro que solo crece no olvida.** Si una contraseña termina en una tarea, borrar la tarea la quita de lo que ves, no de la historia que hay debajo; y si ya llegó a la carpeta compartida, está en cada equipo que la bajó, más el historial de versiones de tu proveedor. Deja los secretos fuera, o en `private/`.
>
> Un documento tampoco olvida enseguida: un texto que borras a propósito sigue en sus [versiones anteriores](Documentos#versiones-anteriores), en cada equipo que lo tenía cuando se reemplazó. Sale de ahí cuando la versión que lo tiene pasa su plazo y no es la más reciente ni lo que te devolvería **Volver a lo que decía antes**, o cuando vacías su historial o borras el documento. Los respaldos hechos entretanto lo conservan: los tuyos, y los últimos que Tisty escribe por su cuenta dentro de su carpeta privada antes de un cambio, como cambiar cómo sincronizas.

## Quién puede alcanzar tus datos

| Eliges | Quién más puede alcanzarlos |
| :-- | :-- |
| **Solo local** (lo predeterminado) | Nadie |
| **Una carpeta que sincroniza tu cliente de nube** | Ese proveedor, según sus términos |
| **Google Drive, conectado desde Tisty** | Google, según sus términos. Tisty solo ve su propio espacio oculto ahí ([cómo funciona](Sincronizar-y-respaldar#sincronizar-con-google-drive-directamente)) |
| **OneDrive, conectado desde Tisty** | Microsoft, según sus términos ([cómo funciona](Sincronizar-y-respaldar#sincronizar-con-onedrive-directamente)) |
| **Tu propio servidor WebDAV, conectado desde Tisty** | Quien administre ese servidor ([cómo funciona](Sincronizar-y-respaldar#sincronizar-con-tu-propio-servidor-webdav)) |
| **Cualquiera de esos tres, cifrado con tu frase** | Nadie puede leer tus archivos sin tu frase o sin uno de tus equipos. El proveedor sigue viendo sus nombres, tamaños y cuántos equipos tienes, y puede guardar un tiempo versiones anteriores sin cifrar |
| **Una carpeta en hardware tuyo** (NAS, disco externo) | Quien pueda alcanzar ese hardware |
| **Un documento que compartes por link, con Folio** | Quien tenga el link. Folio lo guarda sellado y no puede leerlo, pero ve lo que [la política completa](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md#sharing-by-link) enumera |
| **Un zip de respaldo** | Donde lo pongas |
| **Un paquete `.tistyx` que le das a alguien** | Esa persona, y a quien se lo pase |
| **Un `.tistyx` cerrado con seis dígitos** | Quien tenga el archivo y el número |
| **Un agente que dejas entrar** | El servicio con que hable ese agente |
| **Lo que guardas en Agentes** | Lo mismo que tus otros documentos y, si la sincronizas, además cada agente que lee esos archivos y el servicio con que hable, aunque no lo dejes entrar |
| **Una tarea que guardas en tu calendario** | Google o Microsoft, según sus términos; con el Calendario de Apple, la app Calendario de tu Mac y aquello con lo que sincronice |

## Las peticiones de red que hace Tisty

**Por su cuenta:** al abrirse y una vez al día después, descarga un archivo pequeño para ver si hay una versión más nueva. No envía nada: ni identificador, ni tareas, ni nombre. Una copia de la Microsoft Store le pregunta a Windows qué tiene la Store, y lee el catálogo público de la Store para ver qué versión es: al abrirse, cada seis horas y cuando tú lo pides. Descarga ese mismo archivo solo si el catálogo no responde. Mientras sincronizas con Google Drive, OneDrive o un servidor WebDAV, además lee, como mucho una vez por hora, un archivo pequeño y firmado en GitHub que puede pausar la sincronización o bajar sus límites; no lleva nada tuyo. Y cuando abres un documento que compartiste y que tiene un idioma que conservaste de otra persona, le pregunta a Folio si ese idioma cambió, como mucho cada cuarto de hora.

**Solo cuando tú lo pides:**

- **Actualizar**, en **Configuración › Ayuda y acerca de**, descarga el archivo de la versión y el instalador, y Tisty rechaza cualquier cosa que no esté firmada con la clave que lleva tu copia.
- **Traer la vista previa** en una [tarjeta de enlace](https://markdown.tisty.app/es/enlaces) le pregunta a esa página una vez por su título y su imagen.
- **La ayuda** dentro de Tisty (el `?` al pie de la barra lateral) viene con tu versión y no pide nada. Su guía **Markdown de Tisty** (también `F1` mientras escribes) la lee desde [markdown.tisty.app](https://markdown.tisty.app/es/), así que siempre es la última; sin conexión, muestra la copia que trae tu versión. Sus imágenes se descargan desde GitHub solo cuando pulsas **Descargar** sobre la ayuda.
- **Mostrarlas**, en la línea sobre un archivo Markdown abierto desde afuera, le pide a cada dirección que nombra el archivo su imagen, una petición por imagen. No se trae nada hasta que lo pulsas, vuelve a preguntar cada vez que abres el archivo y rechaza una dirección de este equipo o de tu propia red. Es solo del visor: un documento de tu almacén no dibuja imágenes de internet.
- **Abrir un enlace**, con `Ctrl`/`⌘` + clic mientras escribes o con un clic mientras lees, abre tu navegador.
- **Guardar en mi calendario**, en una tarea, abre en tu navegador la página de Google Calendar u Outlook con el título, el día y la hora, y el enlace de la tarea en la dirección, y el evento queda en tu calendario solo si pulsas **Guardar** ahí. Tisty no hace esa petición: la hace tu navegador, y la dirección queda en su historial. Con el Calendario de Apple, Tisty deja un archivo `.ics` en su caché y lo abre la app Calendario. La descripción y las notas de la tarea nunca van.
- **Compartir**, en un documento, lee un archivo firmado pequeño en GitHub para saber si [Folio](https://github.com/rgdevment/Folio-by-Tisty) está abierto. Lo que confirmes después va a Folio, cifrado en este equipo, igual que Sincronizar, Pausar, Dejar de compartir y abrir un link compartido. No se pide nada hasta que lo pulsas.
- **Google Drive**, **OneDrive** o **tu servidor WebDAV**, una vez que lo conectas desde la ventana, es adonde Tisty manda tus copias y de donde las trae. Los agentes nunca tocan la red.
- **Revisar un servidor WebDAV**: solo cuando lo conectas, Tisty entra con lo que escribiste y mira su propia carpeta ahí, nada más. Solo se acepta `https://`, salvo hacia este mismo equipo, y la contraseña nunca viaja a otra dirección.
- **Revisar el camino a OneDrive** funciona igual que la revisión para Google Drive, más abajo, con una petición a `graph.microsoft.com` que no lleva nada tuyo.
- **Revisar el camino a Google Drive**: solo mientras conectas Google Drive o te mueves a él, Tisty manda una petición al servidor de la API de Google, `www.googleapis.com`, sin cuenta, sin token y sin nada tuyo. Espera como mucho 10 segundos y pasa por el proxy de tu sistema, nunca por fuera de él. Solo le sirve a Tisty para decirte si un proxy, un filtro de tu trabajo o un antivirus se interpone, o si Google mismo tiene problemas. Nunca corre mientras sincroniza. Cuando una ronda de sincronización no logra llegar a Google, Tisty lee la falla de la misma forma, sin mandar nada más, y guarda en este equipo solo una palabra para lo que se interpuso, cuántas rondas seguidas fallaron y desde cuándo, para avisarte cuando dura. Una conexión que se corta a medio traspaso no deja ninguna palabra.

Un [widget](Widgets-en-documentos) no ejecuta nada ni nombra nada fuera de sí mismo. Un archivo `.html` adjunto es solo un archivo: Tisty nunca lo ejecuta.

## Tu identidad al compartir

**Qué es.** Un par de llaves que Tisty crea en este computador. Una es secreta y nunca sale de tus computadores; la otra es pública y solo sirve para comprobar tu firma.

**Para qué sirve.** Tisty firma con ella cada documento que compartes. Quien lo abre ve una huella, una serie de números que siempre es la misma para ti: así sabe que dos documentos vienen de la misma persona y que nadie los alteró. Puedes firmar con un alias; no tiene que ser tu nombre.

**Qué ve Folio.** Tu documento se cifra en este computador antes de salir. La llave que lo abre viaja dentro del link, después del #, y esa parte nunca llega a Folio, así que Folio no puede leer el título ni lo que escribiste. Ve tu llave pública, para saber que dos documentos vienen de la misma persona, y nunca tu nombre ni tu correo.

**Tu frase de 24 palabras.** Es el respaldo de tu identidad: con ella, tus otros computadores firman como tú. Guárdala fuera de Tisty, en papel o en un PDF aparte. Si pierdes la frase y todos tus computadores, la identidad no se puede recuperar. Nadie de Tisty ni de Folio te la pedirá nunca.

**Si pierdes un computador.** Quítalo en **Configuración › Seguridad**, en **Tus equipos**, con **Quitar**. Su llave deja de valer, y Folio avisa a quien lea lo que firmó después de eso.

## Tus agentes en la carpeta Agentes

En la carpeta **Agentes** guardas como documentos las reglas, las skills y la memoria de tus agentes (Claude Code, Codex, GitHub Copilot, Gemini CLI y Cursor). Con **Traer** solo los copias a Tisty; con **Sincronizar**, Tisty escribe además por debajo los archivos que lee cada uno. Cómo se usa, archivo por archivo y agente por agente, está en [[Agentes|Carpeta-AGENTS]].

**Qué lee y cuándo**

- **Agentes en gris en la barra, y su tarjeta en Configuración › Agentes › Carpeta Agentes** antes de traer nada, solo miran si existe la carpeta de cada agente, como `~/.claude` o `~/.codex`, para decirte cuáles encontró. No miran nada de adentro.
- **Cuando haces clic en Agentes y se abre el aviso de la primera vez**, Tisty solo mira qué archivos hay en las carpetas de tus agentes, con su nombre y su tamaño, para mostrarte la lista. No abre ninguno, y los que empiezan con un punto ni siquiera los lista. Cerrar el aviso con la X o `Esc` no lee nada más.
- **Al entrar en Agentes, también en un equipo que todavía no te dio permiso**, por ejemplo uno al que Agentes llegó traída desde otro equipo, Tisty hace esa misma lista, para saber si hay algo que preguntarte: solo nombres y tamaños, sin abrir ningún archivo. **Buscar de nuevo** en ese equipo hace la misma lista para mostrarte el aviso de la primera vez, y tampoco abre nada antes de tu **Traer** o tu **Traer y sincronizar**.
- **Con Traer o Traer y sincronizar**, abre solo los archivos que marcaste en esa lista y los trae como documentos. Ese permiso vale para este equipo: cada equipo tuyo pide el suyo.
- **Con Traer, Tisty no escribe en los archivos de tus agentes**, ni anota ninguno como escrito por él: quedan como están hasta que elijas Sincronizar. Para mostrarte qué hay para traer, como en **Traer otro archivo** o **Buscar de nuevo**, vuelve a mirar qué archivos hay, con su nombre y su tamaño. Crear en Agentes la subcarpeta de cada agente que encuentra tampoco toca ningún archivo.
- **Después**, abre solo los archivos que corresponden a un documento de Agentes, para ver si cambiaron por fuera y, si sincroniza, para escribirlos; y, mientras sincroniza, los que escribió para uno que ya no está, también los que elegiste dejar, para ver si cambiaron. Un documento de **Global** se escribe en cada agente que Tisty encuentra en este equipo, aunque no hayas marcado sus archivos. Un documento que llega de otro de tus equipos no se escribe hasta que pulsas **Aplicar aquí** en este, y solo llegan a un archivo las palabras que escribiste o aprobaste tú. Un cambio por fuera nunca se pisa: queda marcado hasta que tú decides. En `GEMINI.md` solo agrega una línea, `@tisty-rules.md`; el resto del archivo es de Gemini.
- **La memoria que escribe el propio agente** se trae y se muestra, pero Tisty nunca la sobrescribe.

**Lo que Agentes nunca trae**

- `settings.json` y los demás archivos de ajustes de tus agentes (`config.toml`, `hooks.json`, `mcp.json`…), salvo el que trae una skill dentro de su carpeta, y ningún archivo de credenciales (`.credentials.json`, `auth.json`, `.env`, llaves `.pem`, `.key` o `id_rsa`…). Los salta por su nombre y nunca los trae. Para Agentes y las skills, solo abre el `settings.json` de Claude Code, con tu sí a Agentes o a las skills de Tisty, para ver si el plugin de Tisty está apagado, y no guarda nada de él. Conectar un agente y ponerle sus hooks es otra cosa, con su propia lectura, y la cuenta [Los agentes que conectas](#los-agentes-que-conectas). Y un documento de Agentes solo se escribe donde van las reglas, las skills y la memoria de cada agente, nunca en sus archivos de ajustes o de credenciales.
- Un archivo que parece tener un token, una contraseña o una llave privada se revisa en este equipo y queda fuera, con un aviso. La revisión busca formas conocidas y puede no reconocerlas todas: deja los secretos fuera de las reglas de tus agentes.
- Tampoco sigue enlaces simbólicos ni trae archivos de más de 512 kB.

**Qué viaja y qué se queda**

- **Tus documentos de Agentes viajan como los demás**: con tu sincronización, tu nube (cifrada, si la cifraste), tus respaldos y un paquete `.tistyx` que lo lleva todo. Un agente que dejas entrar por MCP los lee como cualquier documento, pero no los cambia, no los mueve ni crea uno, ni le pone una línea `agent:` a ningún documento tuyo: solo puede proponerte un cambio, que espera aparte hasta que lo apruebas en la ficha ([cómo](Carpeta-AGENTS#una-propuesta-de-tu-agente)). La propuesta viaja con tus documentos, y nada de ella llega a tus agentes antes de tu aprobación.
- **Sincronizando, lo que escribes en Agentes llega a tus agentes**: cada uno lee su archivo y lo envía al servicio con que trabaja, aunque nunca lo conectes a Tisty. Que sincronice o no es de cada equipo, y no viaja.
- **Se queda en este equipo**, en `data/agents/` dentro de la carpeta de Tisty: el permiso de este equipo para Agentes y el de las skills de Tisty, si sincroniza, qué agentes pausaste, qué documentos aplicaste o dejaste fuera en este equipo, una huella de cada archivo que escribió, qué archivos de esos dejaste en tus agentes al eliminar lo que los escribía y los respaldos. Nada de eso viaja con la sincronización, los respaldos ni los paquetes.
- **Antes de sobrescribir** un archivo de tus agentes, Tisty guarda la versión anterior. De cada archivo conserva el original, el respaldo n.º 1, que nunca se borra, y además las últimas 20, y no deja copias en las carpetas de tus agentes.

**Dejar de sincronizar sin perder nada**

Con **Traer y sincronizar** o **Sincronizar**, la sincronización queda activa en este equipo, y puedes pausarla para un agente o dejar de sincronizar toda la carpeta, con **Dejar de sincronizar** en su botón derecho o en Configuración › Agentes › Carpeta Agentes: ningún archivo de tus agentes se toca y ningún documento se borra. Sin sincronizar u oculta de la barra, Agentes no escribe ningún archivo de tus agentes. En pausa, Tisty no abre ni escribe esos archivos por su cuenta: quedan como los dejó la última escritura, y tus documentos siguen en Tisty. La pausa es de este equipo. Eliminar una regla o una skill de Agentes pregunta una vez, en ese momento, si la quitas también de los agentes para los que Tisty la escribió: **Quitar también** borra cada archivo después de un respaldo, y **Solo de Tisty** los deja y los anota en este equipo, listados con **Revisar** en Configuración › Agentes › Carpeta Agentes. Una copia que cambió por fuera, o que lee un agente en pausa, se queda y se pregunta aparte; un archivo que también arman otras reglas se vuelve a escribir sin esta, con respaldo antes. Cuando una regla sale de Agentes de otra forma, o se elimina en otro equipo, Tisty te pregunta en este antes de borrar su archivo. Sin esas respuestas, Tisty nunca vacía ni borra un archivo de un agente, salvo las copias de sus propias skills, abajo.

**Los hooks de tus agentes**

- Agentes nunca toca los archivos de ajustes de tus agentes. Los hooks sí, y solo cuando tú los pides: en **Configuración › Agentes › Conectados**, **Agregar** te dice qué archivo va a escribir y, con tu sí, pone ahí los hooks de Tisty, sin tocar el resto de ese archivo; **Corregir** cambia el Tisty al que llaman, y **Quitar** los saca. Cuándo se leen está en [Los agentes que conectas](#los-agentes-que-conectas).
- **Los hooks de Tisty.** Tisty trae hooks que puedes encender y apagar cuando quieras y en el agente que elijas. Cada uno ayuda a tus agentes de otra forma, como recordarles completar tareas, actualizar documentos o dejar en Tisty lo que otro va a leer. Para eso miran, en tu equipo, qué hace el agente y dónde; no leen la conversación ni guardan lo que hace, y solo dejan marcas que se borran pasados siete días, al empezar una sesión. Lo que le entregan al agente, como los títulos de tus tareas abiertas, va adonde va ese agente.

**Las skills de Tisty**

Tisty trae tres skills, `tisty`, `consolidate` y `writing-for-the-next-agent`, que viven dentro de la app y no en tu almacén: nunca se sincronizan ni van en un respaldo, un paquete o tu nube, y ningún agente puede nombrarlas ni cambiarlas por el MCP. Solo se escriben los bytes de la versión instalada, comprobados contra una huella que trae la app. Cómo se usan está en [[Hooks y skills|Hooks-y-skills]].

- **Tienen su propio sí, en este equipo.** **Encender en este equipo**, en su ficha de **Configuración › Agentes › Conectados**, es ese sí, y también **Encender skills**, o **Agregar hooks y encender skills**, en el recuadro **Hooks y skills para tus agentes**, que dice lo mismo antes de pulsarlo. No necesita la carpeta Agentes, y un registro copiado de otro equipo no cuenta.
- **Antes de ese sí, y sin el de Agentes, su ficha no abre nada** dentro de las carpetas de tus agentes: solo mira qué carpetas de agentes existen y si siguen ahí las copias que Tisty escribió antes. Con el sí de Agentes solo, lee el registro del plugin, el de abajo, y las copias que Tisty escribió antes, para ver si cambiaron.
- **Con él, Tisty lee y escribe solo esto:** una carpeta por skill en la carpeta de skills de cada agente de este equipo (como `~/.claude/skills/tisty`, `~/.cursor/skills/tisty` y la compartida `~/.agents/skills/tisty`); los demás archivos que ya hay en esas carpetas, para dejar como está una skill que no es de Tisty; y, en Claude Code, `plugins/installed_plugins.json` y `settings.json`, solo para saber si el plugin de Tisty ya las trae. Nada más de esos archivos se guarda, se copia ni se muestra, y no abre ninguna regla, memoria ni otro archivo de tus agentes.
- **La pausa de Agentes no las alcanza.** Una pausa, de un agente o de toda la carpeta, detiene solo tus documentos; las skills siguen su propio interruptor.
- **Nunca escriben sobre lo tuyo.** Una carpeta de skill con un archivo que Tisty no escribió, o una copia que cambiaste, queda como está. La única excepción: una carpeta con exactamente los archivos de esta versión, byte por byte y nada más, cuenta como copia de Tisty: se actualiza y sale al apagarlas. Se actualizan solas cuando abres un Tisty más nuevo.
- **Apagarlas** saca de las carpetas de tus agentes solo las copias que siguen como Tisty las escribió, después de guardar un respaldo en `data/agents/`; las skills siguen en Tisty, listas para encenderlas otra vez, y nunca se eliminan. Una copia que cambiaste queda para que la quites tú desde la misma ficha. Ningún archivo se quita solo ni por el MCP.

## Los agentes que conectas

**Configuración › Agentes › Conectados** escribe Tisty en la configuración de tus agentes, y sus hooks si los pides. Para eso Tisty lee archivos de ajustes de tus agentes, en este equipo y sin enviar nada. Cuáles, cuándo y para qué:

- **Al abrir la ventana de Tisty y al abrir Configuración, con la puerta abierta**, lee de cada agente que encuentra en este equipo el archivo donde se conecta, como `~/.claude.json`, `~/.codex/config.toml`, `~/.copilot/mcp-config.json` o `~/.cursor/mcp.json`, y el de los hooks en los que los tienen: `~/.claude/settings.json`, `~/.cursor/hooks.json`, `~/.gemini/settings.json` y `~/.copilot/hooks/tisty.json`. Busca solo su propia entrada y sus propios hooks, y mira si el programa que nombran existe y es este Tisty. **No escribe nada en ellos hasta que haces clic.** Nunca lee así los de una distro de WSL: esos, solo al pulsar **Buscar en…** o, si ya la buscaste, la pestaña **Conectados**.
- **Con la puerta cerrada**, esa lectura no se hace. Las que ya existían siguen: **Configuración › Agentes** lee los mismos archivos cada vez que la abres, para mostrarte cómo está cada uno, y, mientras no respondas la oferta de conectar tus agentes, Tisty los mira al abrir y al volver a su ventana, con la puerta abierta o cerrada, para saber si ofrecértela.
- **Con tu sí a Agentes o a las skills de Tisty**, lee además el registro de plugins de Claude Code, el de abajo, al abrir la app, cuando algo cambia en tus datos mientras está abierta y al mostrar sus skills, aunque estén apagadas, para saber si el plugin ya trae las skills. Para los hooks solo lo usa en **Configuración › Agentes** o cuando pulsas el botón. De él no guarda nada.
- **Al agregar o corregir los hooks de Claude Code**, aunque no hayas dado ninguno de esos dos sí, y sin ellos solo entonces, es decir, al pulsar **Sí, conectar** con sus hooks, **Agregarlos**, **Agregarlo**, **Agregar hooks**, **Corregir**, **Corregir todo** o **Sí, corregir** cuando eso los escribe, o los quita porque el plugin de Tisty ya los trae, lee antes `plugins/installed_plugins.json` de su carpeta (`~/.claude`, o la que diga `CLAUDE_CONFIG_DIR`; en WSL, la de esa distro) y, solo si ahí está el plugin de Tisty, mira `settings.json` para saber si está apagado. Así no escribe hooks que el plugin ya trae, ni quita los que ya no trae. Si los trae no escribe ninguno, si no se pueden leer no escribe ni quita, y en los dos casos te lo dice. De esos dos archivos no guarda nada por esta vía; que el plugin los trae lo recuerda en memoria hasta cerrar la ventana, o hasta ver que ya no.
- **Al pulsar Conectar, Agregar, Corregir o Quitar**, vuelve a leer el archivo y cambia solo su propia entrada o sus propios hooks. Si cambió desde que te lo mostró para corregirlo, no se pudo leer o no sabría cambiarlo sin estropearlo, no escribe.
- **Lo que queda.** Junto a cada archivo que cambia, dos copias: la primera que vio, que termina en `.before-tisty`, y la de justo antes, que termina en `.before-tisty-last`. Y en el almacenamiento de la propia ventana, en este equipo: qué agentes ya viste ofrecidos y, si pulsas **Ahora no** en un aviso, de qué agente era y la ruta a la que apuntaba, para no repetírtelo, hasta que vuelve a encontrar Tisty. Nada de eso se sincroniza ni va en un respaldo, un paquete o un link.

## Sin IA

Tisty no usa IA ni aprendizaje automático sobre tus datos, y no le envía nada a ningún modelo. El lenguaje natural son reglas fijas en tu equipo.

Lo mismo vale para los parecidos de [Brainsty](Red-de-Brainsty#parecidos). Cuando une «Pagar la cuenta del agua» con «Pay the water bill», no entiende ni traduce nada: busca cada palabra en una lista fija de parejas español–inglés, *pagar ≈ pay*, *agua ≈ water*, y cuenta las que coinciden. Esa lista sale del [Open Multilingual Wordnet 2.0](https://github.com/omwn/omw-data), hecho por lingüistas, viene dentro del instalador y nunca sale de tu equipo ni se busca en internet. No aprende de lo que escribes: es la misma para todos y solo cambia con una versión nueva de Tisty. Sus autores y su licencia están en **Configuración › Ayuda y acerca de › Avisos de terceros**.

Si conectas un agente tuyo por [[MCP|Agentes-y-MCP]], lo que lea se va adonde vaya ese agente. Esa puerta viene abierta en una instalación nueva, y **Quitarle la voz**, en Configuración › Agentes › Puerta, la cierra.
