# Configuración

Configuración es donde Tisty se acomoda a ti: cómo se ve, cómo te avisa, dónde deja tus copias, quién escribe en él y cómo está todo. La ordené para que lo que pide algo de ti salga primero, en **Resumen**, y para que cualquier ajuste se encuentre escribiendo su nombre, sin recorrer página por página. Sirve igual si trabajas con tu agente o solo: nada de lo que hay aquí exige conectar un agente.

## Abrir la Configuración

- **El engranaje** al pie de la barra lateral. **Volver**, arriba a la izquierda, o `Esc` te devuelven a donde estabas.
- **La paleta de comandos**: escribe «configuración» y elige **Configuración** (los atajos están en [Atajos de teclado](Atajos-de-teclado)).
- **Desde un aviso**: el pie de la barra lateral, cuando la sincronización pide algo, te lleva a la página que lo resuelve.

El engranaje y la paleta abren siempre en **Resumen**. A la izquierda tienes, en este orden, **Volver**, **Buscar un ajuste**, **Resumen** y las diez páginas en cuatro grupos:

| Grupo | Páginas | Qué guarda |
|---|---|---|
| **Este equipo** | [Aspecto y ventana](#aspecto-y-ventana), [Tareas y avisos](#tareas-y-avisos), [Documentos y firma](#documentos-y-firma) | Cómo se ve y se porta Tisty en este equipo, cómo anotas y te avisa, y con qué nombre firmas |
| **Tus datos** | [Sincronización](#sincronización), [Seguridad](#seguridad), [Copias](#copias), [Almacenamiento](#almacenamiento) | Dónde viajan tus copias, quién eres en tus equipos, tus respaldos y cuánto ocupa todo |
| **Conexiones** | [Agentes](#agentes) | La puerta para tu agente y los agentes conectados en este equipo |
| **Tisty** | [Salud](#salud), [Ayuda y acerca de](#ayuda-y-acerca-de) | La revisión de tu historial, la versión que tienes y dónde pedir ayuda |

Una página que tiene algo por atender lleva un número a la derecha de su nombre, el mismo que cuenta **Resumen**: con **Seguridad 1** y **Copias 1** sabes, sin abrir nada, que hay una cosa en cada una.

Con el teclado, `↑` y `↓` recorren la columna, `Inicio` y `Fin` saltan a la primera y a la última página, y `Enter` abre la que tienes marcada: las flechas solo mueven el foco, así que pasar por **Salud** no lanza su revisión hasta que la abres.

Cada página empieza con su nombre y una línea que dice para qué está, por ejemplo **Copias**: «Un respaldo es un archivo tuyo, fuera de Tisty. Sincronizar no lo reemplaza: la nube también copia un error o un borrado a todos tus equipos.».

## Resumen

**Resumen** dice cómo está tu Tisty de un vistazo: «Cómo está tu Tisty, de un vistazo. Lo que pide algo de ti va primero.». Tiene hasta tres partes, de arriba abajo.

### Deja Tisty a tu medida

Después de instalar, **Resumen** te propone cinco pasos, cada uno con **Ir ›**, que te lleva a la fila justa:

1. **Elige dónde van tus copias** — o quédate solo en este equipo. Te lleva a [Sincronización](#sincronización).
2. **Crea tu frase de 24 palabras** — tu identidad en todos tus equipos. Te lleva a **Tu identidad**, en [Seguridad](#seguridad).
3. **Haz tu primer respaldo** — un archivo tuyo, fuera de Tisty. Te lleva a **Guardar una copia**, en [Copias](#copias).
4. **Ponle tu alias a lo que escribes** — con lo que firmas tus documentos. Te lleva a **Alias**, en [Documentos y firma](#documentos-y-firma).
5. **Conecta un agente** — opcional. Te lleva a la pestaña **Conectados** de [Agentes](#agentes).

El título cuenta lo hecho, **Deja Tisty a tu medida · 1 de 5**, y cada paso hecho queda tachado con un ✓. Cuando terminas los cuatro primeros, la lista se va sola: el quinto es opcional y no la retiene.

**Ejemplo.** Marisol instala Tisty en su portátil nuevo y, en la bienvenida, elige **Solo en este equipo**. Al abrir Configuración ve **Deja Tisty a tu medida · 1 de 5**: el paso 1 ya está tachado, porque quedarse en este equipo también es elegir. Pulsa **Ir ›** en el paso 2, crea su frase en **Seguridad** y vuelve: ahora dice **2 de 5**. Hace su respaldo, escribe «Marisol» como alias y la lista desaparece, aunque todavía no conecta ningún agente.

### Por atender

Debajo, **Por atender · 2** junta lo que pide algo de ti, cada cosa con un punto ámbar y su botón, que te lleva a la fila donde se resuelve:

| Lo que dice | Botón | Te lleva a |
|---|---|---|
| **Tisty 1.50.0 ya está disponible** (o **La Store tiene una versión nueva**) | **Actualizar** si Tisty puede instalarla desde aquí; si no, **Ver** | Ayuda y acerca de |
| **Tu nube pide que vuelvas a iniciar sesión** | **Ver** | Sincronización |
| **La sincronización tiene algo que revisar** | **Ver** | Sincronización |
| **La sincronización está en pausa en este equipo** | **Ver** | Sincronización |
| **Llevas días sin recibir nada de tus otros equipos** (tres días o más) | **Ver** | Sincronización |
| **Tu frase todavía no está confirmada** | **Confirmarla** | Seguridad › Tu identidad |
| **Portátil de Jaime escribe sin que hayas confirmado su clave** | **Confirmar** | Seguridad › Tus equipos |
| **Tu último respaldo es de lun, 5 oct 11:47** (30 días o más) | **Crear respaldo** | Copias › Guardar una copia |
| **3 cosas por revisar en tu historial** | **Revisar** | Salud |
| **La captura rápida se quedó sin atajo** | **Elegir otro** | Tareas y avisos › Captura rápida |
| **Claude Code apunta a un Tisty que ya no existe** | **Corregir** | Agentes › Conectados |

Cuando no queda nada, dice **Nada pide tu atención.** con un punto verde. Esa frase solo aparece si Tisty pudo leer todo lo que revisa al abrir: si una lectura falló o sigue en camino, no la dice, porque no puede prometerlo. Mientras están los pasos de **Deja Tisty a tu medida**, tampoco.

**Ejemplo.** Jaime vuelve de vacaciones y abre Configuración. **Por atender · 2** dice **Tu frase todavía no está confirmada** y **Tu último respaldo es de lun, 5 oct 11:47**, y en la columna **Seguridad** y **Copias** llevan un 1. **Crear respaldo ›** lo deja en **Guardar una copia**, con la fila resaltada un momento; pulsa **Crear respaldo…**, guarda el archivo en su disco externo y, de vuelta en **Resumen**, queda una sola cosa.

Cada una de estas cosas sale también en su fila, como nota ámbar **Atención**: es la misma, no otra, y al resolverla se va de las dos partes y del número de la columna a la vez. Más en [Atención, en su fila](#atención-en-su-fila).

Algunas cosas solo se saben después de mirar: los equipos que esperan tu confirmación salen cuando ya abriste **Seguridad** o **Salud**, lo que hay que revisar en tu historial, cuando ya abriste **Salud**, y un agente que apunta a otro Tisty, cuando ya abriste **Agentes**. **Resumen** no lanza esas revisiones por su cuenta.

### Estado

**Estado** tiene seis mosaicos, uno por página: **Sincronización**, **Copias**, **Seguridad**, **Agentes**, **Salud** y **Ayuda y acerca de**. Cada uno dice lo esencial con un punto de color, y al pulsarlo abre su página.

- **Punto verde**: está bien y Tisty lo sabe de primera mano.
- **Punto ámbar**: algo sigue funcionando, pero conviene mirarlo.
- **Punto gris**: está apagado, todavía no hay nada, o Tisty no puede asegurar que esté bien.
- **«…» con punto gris**: el dato todavía no llega. Nunca se dibuja un valor que Tisty no leyó.

| Mosaico | Verde | Ámbar | Gris |
|---|---|---|---|
| **Sincronización** | **En Google Drive** (o tu carpeta, tu OneDrive, tu WebDAV) con una ronda que terminó bien o en curso | Algo te pide a ti, **sin noticias desde …** (tres días sin nada de tus otros equipos) o **La eligió un Tisty más nuevo** | **Desactivada** · solo en este equipo; sin conexión, con el límite de hoy o esperando; o **todavía no se pudo leer el estado de la nube** |
| **Copias** | **Último respaldo** con su fecha | El último respaldo tiene 30 días o más | **Sin respaldo** · ninguna todavía |
| **Seguridad** | **Frase confirmada** | **Frase sin confirmar**, **Frase en otro equipo**, o un equipo que espera tu confirmación | **Sin frase todavía** · tu identidad nace con ella |
| **Agentes** | **Puerta abierta** · firma como «palmera 95» | Un agente apunta a un Tisty que ya no existe | **Puerta cerrada** · se le niega lo que pida un agente |
| **Salud** | **Todo en orden** · revisado 10:42 | **3 cosas por revisar**, o **Algunas revisiones no se pudieron hacer** | **Sin revisar todavía** · se revisa al abrir Salud |
| **Ayuda y acerca de** | **Tisty 1.49.0** · estás al día, si Tisty buscó versiones hoy y le respondieron | **Hay una 1.50.0** · tienes la 1.49.0 | **aún no busca versiones nuevas**, o **buscó por última vez …** si la última respuesta no es de hoy |

**Salud** es el único que no se revisa al abrir Configuración: muestra la última revisión que hiciste en esta ventana, con su hora, así que ves cuánto tiene.

**En otro equipo**, **Resumen** habla de ese equipo: cada uno guarda sus respaldos, su revisión de **Salud** y sus agentes, así que en tu escritorio puede decir **Sin respaldo** aunque el portátil tenga uno de ayer. La frase, en cambio, es tuya en todos: en un equipo donde todavía no la escribiste, **Seguridad** dice **Frase en otro equipo** · escríbela aquí para reconocer este.

## Buscar un ajuste

**Buscar un ajuste**, arriba en la columna, encuentra cualquier ajuste por su nombre, su descripción o palabras que la gente usa para él. No hace falta escribirlo completo ni con tildes, y no distingue mayúsculas: cada palabra que escribes tiene que aparecer en el ajuste.

1. Haz clic en **Buscar un ajuste** y escribe, por ejemplo, «atajo».
2. La página muestra «atajo» como título, cuántos encontró, **2 ajustes**, y una fila por cada uno con su página a la derecha: **Captura rápida** · Tareas y avisos y **Idea rápida** · Tareas y avisos.
3. Pulsa una fila: se abre su página y la fila se resalta un momento en el centro.

Los resultados van en el orden de la columna, de **Aspecto y ventana** a **Ayuda y acerca de**. Algunos ejemplos:

| Escribes | Encuentra |
|---|---|
| respaldo | **Guardar una copia** y **Documentos de otro Tisty**, en Copias |
| idioma | **Idioma**, en Aspecto y ventana, y **La bienvenida**, en Ayuda y acerca de |
| cifrar | **Cifrar mis datos en la nube**, en Seguridad, entre otros |
| teclado | **Captura rápida** e **Idea rápida**, en Tareas y avisos |
| carpeta | **Carpetas ocultas**, **Dónde se sincroniza**, **Desactivar la sincronización** y **Tu carpeta**, entre otros |

Si nada coincide, dice «Nada con esas palabras. Prueba con «respaldo», «atajo» o «idioma».». Un ajuste que existe pero que hoy no se ve, como **Tus equipos** en Sincronización mientras no sincronizas, abre su página desde arriba, sin resaltar nada.

`Esc` dentro del buscador borra lo que escribiste; con el buscador vacío, sale de Configuración.

Buscar un ajuste busca en la Configuración. Para buscar en esta ayuda, usa el buscador de la ayuda; para tus tareas y documentos, el de la ventana.

## Este equipo

Lo que se ve y se oye en el equipo que tienes delante.

### Aspecto y ventana

«Cómo se ve Tisty y qué hace su ventana en este equipo.»

- **Aspecto**: **Idioma** (español o inglés; si no eliges, Tisty habla el idioma de tu equipo), **Aspecto** (**Claro**, **Oscuro** o **Seguir al equipo**) y **Que Tisto se asome**, el castor que se asoma sobre la barra mientras escribes una tarea.
- **Esta ventana**: **Al cerrar la ventana**, por ejemplo **Dejarlo en la bandeja**, e **Iniciar con el equipo**, que abre Tisty apartado, sin ventana, al iniciar sesión. Si Windows le quitó el inicio a Tisty, desde el Administrador de tareas o desde Configuración → Aplicaciones → Inicio, la fila dice «Tisty se abre solo cuando tú lo abres.», el interruptor queda gris y la nota **Lo tiene el sistema** te dice dónde volver a activarlo, con **Abrir Configuración de Windows**. Mientras Windows lo tenga apagado, Tisty tampoco te pregunta **¿Abrir Tisty al iniciar sesión?**.
- **Carpetas ocultas** («Solo este equipo»): las carpetas que sacaste de la barra lateral en este equipo, cada una con **Volver a mostrar**. Siguen guardando todo, y la búsqueda, la paleta y «Mover a» las encuentran. Si no ocultaste ninguna, dice «No hay carpetas ocultas.».

### Tareas y avisos

«Cómo anotas sin abrir Tisty y cómo te avisa cuando vence un recordatorio.»

- **Captura rápida**: **Captura rápida** elige el atajo que abre un campo pequeño encima de lo que estés haciendo, `Ctrl` + `Shift` + `Space` por defecto. Lo que escribes va a Tareas, sin lista. **Idea rápida** elige otro atajo para anotar directo en [Ideas](Documentos#ideas); sin atajo, `Tab` en la captura rápida igual la cambia a idea.
- **Avisos**: **Una notificación del sistema**, **Un tono corto**, **Recordatorios importantes** (que un recordatorio importante atraviese «No molestar», si lo permites en el sistema) y **Tu calendario**, dónde guarda «Guardar en mi calendario» una tarea con fecha.

**Recordatorios importantes** depende de dos cosas. Con **Una notificación del sistema** apagada, su interruptor queda gris y la nota **Falta algo** dice «Primero enciende «Una notificación del sistema»: sin ella ningún recordatorio llega al sistema.», con **Encenderla**. Ya encendido, la nota **Lo tiene el sistema** te recuerda el paso que falta en Windows o macOS, con **Abrir la configuración del sistema**. Si tu sistema o tu copia de Tisty no lo permiten, la fila lo dice en vez de ofrecerlo.

Si otro programa ya usa tu atajo, Tisty toma otro y lo dice en esta misma fila; si se queda sin ninguno, **Resumen** te lo pide con **La captura rápida se quedó sin atajo**. Los avisos se preparan y se muestran en este mismo equipo: nada de esto sale a internet. Más en [Reparto y recordatorios](Reparto-y-recordatorios).

### Documentos y firma

«Con qué nombre firmas lo que escribes y cuánto pasado guarda cada documento.»

- **Quién escribe aquí**: **Alias** («Viaja a tus equipos»), el nombre con que firmas lo que escribes, con **Firmar los anteriores** para ponérselo a lo que escribiste antes; y **Nombre para firmar**, cómo apareces en tus PDF. Cada uno tiene **Saber más** con la explicación larga.
- **Historial de documentos**: **Guardar las versiones anteriores** («Solo este equipo»), 1, 5, 15 o 30 días. Más en [Versiones anteriores](Documentos#versiones-anteriores).

**Ejemplo.** Lucas escribe «Lucas» como alias en su escritorio. Al sincronizar, su portátil también firma «Lucas», porque el alias viaja. Si en el portátil deja **Guardar las versiones anteriores** en 5 días, el escritorio sigue en 30: cada equipo guarda su propio historial.

## Tus datos

Tus copias, tu identidad y tu espacio. Las tres páginas con algo que no se deshace, **Sincronización**, **Seguridad** y **Copias**, lo guardan en su [Zona delicada](#zona-delicada).

### Sincronización

«Dónde deja Tisty copias para que tus otros equipos las recojan.»

- **Dónde se sincroniza**: el estado, por ejemplo «Desactivada. Tus tareas viven solo en este equipo.», y **Activar…**, que te deja elegir **Tu servicio de nube** (Google Drive, OneDrive o WebDAV, directo) o **Nube por carpeta o NAS**. Sincronizando con una carpeta: **Sincronizar ahora**, **Abrir la carpeta**, cuándo fue tu última ronda y lo último que llegó de otro equipo; y bajo **Configuración:**, **Cambiar carpeta** y, si esta copia puede conectar una nube, **Mudarme a otro lugar…**, con **Volver a enviar todo** dentro de **Más opciones**. Con una nube directa, la tarjeta de tu nube con lo que dice y lo que puedes hacer.
- **Tus equipos**: sincronizando, una fila con **Ver en Seguridad**, porque la lista de tus equipos vive en Seguridad.
- **Zona delicada**: con una carpeta, **Desactivar la sincronización** (**Desactivar**): este equipo deja de dejar copias y lo que ya está en la carpeta se queda ahí. Con una nube directa, **Desconectar este equipo**, que suelta tu nube solo aquí; si tu nube ya no deja entrar a Tisty, **Salir sin Google Drive** (o sin la nube que uses); y con Google Drive u OneDrive, **Quitar el acceso de Tisty en todos tus equipos**. Cada uno te pregunta antes: lo ves en [Qué te pregunta antes](#qué-te-pregunta-antes).

Todo lo que pasa en esta página está en [Sincronizar y respaldar](Sincronizar-y-respaldar).

### Seguridad

«Tu identidad, los equipos que reconoces como tuyos y el cifrado de tu nube.»

- **Tu identidad**: nace con tu frase de 24 palabras. **Crear mi frase** la crea; **Ya tengo mi frase** la escribe en este equipo, si la creaste en otro. Después, **Ver la frase** te la vuelve a mostrar en el equipo donde la creaste.
- **Tus equipos**: una sola lista con cada equipo que escribe donde sincronizas, este marcado como **Este equipo**. Cada uno con su código y **Ver el código**; los demás, con **Quitar**. Sincronizando, debajo, **Cambiar nombre** y **Mostrar en grande** para este equipo.
- **Cifrar mis datos en la nube**: **Cifrar mi nube…**; con la nube cifrada, **Autorizar un equipo nuevo**. Sin una nube directa, la fila lleva la nota **Falta algo**: «Conecta una nube en Sincronización para cifrar lo que Tisty guarda ahí.», con **Ir a Sincronización**. Con la nube cifrada, la **Zona delicada** guarda **Borrar la copia sin cifrar** y **Descifrar mi nube**.

Paso a paso en [Confirmar tus equipos](Sincronizar-y-respaldar#confirmar-tus-equipos) y [Cifrar tu nube](Sincronizar-y-respaldar#cifrar-tu-nube).

### Copias

«Un respaldo es un archivo tuyo, fuera de Tisty. Sincronizar no lo reemplaza: la nube también copia un error o un borrado a todos tus equipos.»

- **Guardar una copia**: **Crear respaldo…** escribe un archivo con todo tu historial, de todos tus equipos, con lo que lleva y cuándo fue el último.
- **Documentos de otro Tisty**: **Exportar…** lleva tus documentos a otro Tisty e **Importar…** trae los que alguien te pasó. No es un respaldo: importar un archivo dos veces deja una segunda copia.
- **Zona delicada**: **Restaurar** (**Restaurar desde un archivo…**) sustituye todo lo de este equipo por lo que trae el archivo, y no aparece si sincronizas con Google Drive u OneDrive; **Salir de Tisty** (**Salir…**) te guía para respaldar, desconectar la nube y dejar este equipo limpio, con tus tareas y documentos en su carpeta.

Más en [Respaldos](Sincronizar-y-respaldar#respaldos).

### Almacenamiento

«Dónde viven tus archivos en este equipo y cuánto ocupan.»

- **Dónde vive**: **Tu carpeta**, con su ruta y **Mostrar carpeta**. No se puede cambiar: saber siempre dónde están tus archivos es parte de que sean tuyos.
- **Cuánto ocupa**: tareas, listas y eventos de tu historial, y cuánto pesan el historial, los documentos y los adjuntos.
- **Adjuntos**: **Dónde se quedan los adjuntos grandes**, para los de más de 25 MB. Sin carpeta compartida, el selector queda gris y la nota **Falta algo** dice «Todavía no hay carpeta compartida, así que todo se queda en este equipo.», con **Ir a Sincronización**.
- **Liberar espacio**: **Adjuntos que no abres hace tiempo**, **Adjuntos que nada usa** y **El mismo archivo guardado dos veces**, con **Ver cuáles**. Con una nube directa, también **Uso de hoy en la nube**. Si no hay nada que soltar, dice «Nada que liberar por ahora».

Más en [Adjuntos grandes](Sincronizar-y-respaldar#adjuntos-grandes) y [Privacidad](Privacidad).

## Conexiones

### Agentes

«Tisty no lleva IA dentro. La pone tu agente; Tisty es la memoria que comparten.» La página tiene cinco pestañas:

- **Puerta**: **Un agente puede anotar aquí**, con **Quitarle la voz** para cerrarla (y **Dejar que un agente anote** para abrirla otra vez), **Pedirle que deje su tarea al día** y **Recordarle al empezar y al terminar cada turno**.
- **Conectados**: **Agentes en este equipo**, con **Buscar de nuevo**, un buscador para los que Tisty conoce y no encontró, **Escribieron en Tisty desde otros lugares**, la fila **Skills de Tisty**, con **Ver**, y **Cómo conectar uno**, plegado, para conectarlo a mano. Bajo cada agente que los acepta, la fila **Hooks: al empezar y al terminar cada turno**, con **Agregar** o **Quitar**; y al conectar uno, una vez, un recuadro que ofrece sus hooks y las skills. Lo cuenta [Hooks y skills](Hooks-y-skills).
- **Carpeta Agentes**: la tarjeta **Agentes**, para traer y sincronizar las reglas, skills y memoria de tus agentes. Más en [Agentes: reglas, skills y memoria](Carpeta-AGENTS).
- **Cómo se ve su firma**: **Quién firma lo que escribe un agente**, con una **Vista previa de una fila** que cambia al momento, **Solo un rombo**, **Nombrar al agente en la lista de tareas** y **Nombrar al agente dentro de una tarea**.
- **Qué puede y qué no**: **Lo que puede hacer** y **Lo que no podrá hacer nunca**, lado a lado.

Con el teclado, `←` y `→` cambian de pestaña, e `Inicio` y `Fin` saltan a la primera y a la última.

Con la puerta cerrada, la página te lo recuerda donde importa. En **Puerta**, **Pedirle que deje su tarea al día** y **Recordarle al empezar y al terminar cada turno** quedan en gris con la nota **Falta algo**: «Estos dos ajustes actúan cuando la puerta está abierta.». En **Conectados**, si tienes agentes conectados, **Agentes en este equipo** dice «Aun así no anotarán nada hasta que abras la puerta.», con **Ir a Puerta**. Y en **Cómo se ve su firma**, con **Solo un rombo** encendido, los dos interruptores de nombre quedan en gris: «Los dos de arriba quedan apagados mientras «Solo un rombo» esté encendido.».

**Quitarle la voz** te pregunta antes de cerrar: «Ningún agente podrá anotar en Tisty.», todo lo que escribió se queda en su sitio, y los agentes conectados siguen conectados, pero se les niega lo que pidan. No pide casilla, porque **Dejar que un agente anote** la abre otra vez.

Si trabajas sin agente, esta página puede esperar: la puerta abierta no hace nada mientras ningún agente llame. Todo lo demás en [Agentes y MCP](Agentes-y-MCP).

## Tisty

### Salud

«Tisty revisa tu historial, tus documentos y tus adjuntos al abrir esta página.»

- Arriba, el resultado, por ejemplo **Todo en orden · revisado 10:42**, con **Revisar de nuevo**. Si encuentra algo, **Por revisar** lo lista con lo que puedes hacer en cada caso, como documentos que faltan aquí o un equipo que firma con otra clave.
- **Herramientas**: **Índice rápido**, con **Rehacer el índice rápido**, que vuelve a leer tu historial; no cambia nada de lo que escribiste.
- **Si algo anda mal**: **Reportar un problema**, con **Preparar…**, arma un archivo con lo necesario para entender un fallo, que lees antes de compartir. No se envía nada.

### Ayuda y acerca de

«La versión que tienes, cómo se actualiza y dónde pedir ayuda.»

- La versión, la licencia y qué es Tisty. **Buscar actualizaciones**, y **Actualizar** cuando hay una versión nueva; **Recibir versiones candidatas**, para probar las versiones antes de que estén terminadas.
- **Ayuda**: **La bienvenida**, con **Verla otra vez** (vuelve a preguntar idioma, dónde van tus copias, si inicia con el equipo y qué hace al cerrar la ventana, sin tocar lo que escribiste) y **Abrir la guía**; y **Ayuda en tisty.app**, con **Abrir ↗**.
- **Apoyar a Tisty**, con **Dar una estrella en GitHub**, y **Otras herramientas**.

Cuando hay una versión nueva, su fila lleva la nota **Dato**: «Este aviso también sale en el Resumen y como punto en la barra, no solo aquí.». Si **Buscar actualizaciones**, **Actualizar** o **Recibir versiones candidatas** fallan, el error sale en esa misma fila, con su causa. Sin conexión, por ejemplo, **Buscar actualizaciones** te dice que Tisty no pudo llegar a su lista de versiones. La fila dice «Estás en la versión más reciente» solo después de una respuesta de hoy; si no, dice «Aún no hay respuesta sobre versiones nuevas» o el día de la última.

En la copia de la Microsoft Store, la Store mantiene Tisty al día: cuando te ofrece una versión, **Actualizar** se la pide; si la Store ya la está trayendo por su cuenta, **Resumen** te deja **Ver** cómo va. Ahí **Recibir versiones candidatas** lleva la nota **No aplica aquí**: «La Microsoft Store mantiene esta copia al día y no ofrece candidatas.».

## Zona delicada

Lo que no se deshace, o que cambia mucho de una vez, vive en un recuadro rojo al final de su página, **Zona delicada**, lejos de las filas de todos los días:

| Página | En la Zona delicada |
|---|---|
| **Sincronización** | Con una carpeta, **Desactivar la sincronización**. Con una nube directa, **Desconectar este equipo**; **Salir sin Google Drive** si tu nube ya no deja entrar a Tisty; y, con Google Drive u OneDrive, **Quitar el acceso de Tisty en todos tus equipos** |
| **Seguridad** | **Borrar la copia sin cifrar**, mientras quede esa copia, y **Descifrar mi nube**, con la nube cifrada |
| **Copias** | **Restaurar**, salvo que sincronices con Google Drive u OneDrive, y **Salir de Tisty** |

Sus botones van en rojo, y cada fila dice en una línea lo que hace el suyo. Si la página no tiene nada delicado en ese momento, por ejemplo **Sincronización** con la sincronización desactivada, no muestra la zona.

**Ejemplo.** Yisset deja de usar su portátil viejo. Antes de borrarlo, abre **Copias**, hace un respaldo con **Crear respaldo…** y recién entonces baja a la **Zona delicada** y pulsa **Salir…**: Tisty la guía para respaldar, desconectar la nube y dejar ese equipo limpio, y sus tareas y documentos se quedan en su carpeta.

### Qué te pregunta antes

Ningún botón rojo hace nada al primer clic. Abre una ventana con su nombre, lo que va a pasar en una lista corta, **Cancelar** y el botón que lo hace. Cuando no tiene vuelta atrás, ese botón se queda gris hasta que marcas **Entiendo que no se puede deshacer**, y la casilla empieza sin marcar cada vez que se abre la ventana.

| Pulsas | La ventana dice | Botón | Casilla |
|---|---|---|---|
| **Desactivar** | **Desactivar la sincronización**: «Tus tareas vivirán solo en este equipo.» Lo que ya está en la carpeta compartida se queda ahí, y tus otros equipos dejan de recibir lo que escribas aquí. | **Desactivar** | No |
| **Desconectar este equipo** | «Este equipo deja de llegar a Google Drive; tus otros equipos siguen sincronizando.» Tus copias se quedan en tu Google Drive, y puedes volver a conectarlo cuando quieras. | **Desconectar** | No |
| **Quitar el acceso de Tisty en todos tus equipos** | «Google le quita a Tisty el acceso a esta cuenta en todos tus equipos a la vez.» Tus copias se quedan en tu Google Drive, y cada equipo tendrá que conectarse de nuevo. | **Quitar el acceso** | No |
| **Salir sin Google Drive** | «Ya no se puede llegar a Google Drive desde este equipo.» Tisty conserva aquí tus tareas y documentos y no sincroniza con nada. Antes de preguntarte, cuenta los adjuntos que se quedan solo en la nube, por ejemplo «3 adjuntos están solo en Google Drive y se quedan ahí, fuera del alcance de Tisty.», o te dice «Todos los adjuntos ya están en este equipo.». | **Salir sin Google Drive** | Sí |
| **Borrar la copia sin cifrar** | **¿Borrar la copia antigua sin cifrar?**: «Solo se borran los archivos cuya copia nueva ya está en Google Drive.» Puede tardar unos minutos y puedes seguir trabajando. Esto no se puede deshacer. | **Borrar la copia** | Sí |
| **Descifrar mi nube** | «Google Drive volverá a ver tus datos sin cifrar. Tu frase sigue siendo tuya.» Tarda unos minutos y puedes seguir trabajando. | **Descifrar** | No |
| **Restaurar desde un archivo…**, después de elegir el zip | **Restaurar desde un archivo**: «Restaurar vuelve a ese momento, y todo lo posterior se pierde.» Sustituye todo lo que hay en este equipo, y este equipo deja de compartir su carpeta: al volver a elegirla te preguntará qué lado manda. | **Restaurar** | Sí |
| **Salir…** | El diálogo guiado de siempre, paso a paso: el respaldo, la nube, lo que Tisty dejó en este equipo y lo que queda en tus manos. | **Cerrar Tisty** | No |

Cuatro botones fuera de la zona preguntan igual, en su propia fila:

- **Quitar**, en un equipo de **Seguridad › Tus equipos**: **¿Quitar este equipo?** Por ejemplo, «Portátil de Jaime deja de poder escribir en la carpeta compartida.»; lo que ya escribió se queda ahí y todos los equipos lo siguen viendo; si vuelve, entra como equipo nuevo, respaldado y vaciado antes; y cuándo escribió por última vez. Botón **Quitar el equipo**, con casilla.
- **Quitarle la voz**, en **Agentes › Puerta**: lo ves en [Agentes](#agentes). Sin casilla.
- **Mover 5 a la papelera · 41 MB**, en **Almacenamiento › Adjuntos que nada usa**: **¿Mover a la papelera 5 adjuntos que nada usa?**, con «Los de este equipo quedan ahí por 30 días.» y el aviso de que los que están solo en la carpeta compartida se borran de ella y tu nube los guarda en su propia papelera. Sin casilla.
- **Borrar para siempre**, en **Salud › Por revisar**: **¿Borrar «Recetas» para siempre?**, con «Aquí no queda copia.» para un documento suelto, o «El documento y lo que tiene se van para siempre, en todos tus equipos.» para uno que falta. Con casilla.

Lo que se puede deshacer, o es un paso dentro de un diálogo guiado, sigue con una pregunta simple: volver a enviar todo, cancelar un cifrado a medias, mover un solo adjunto a la papelera y borrar las copias de la nube vieja al mudarte.

![La ventana Restaurar desde un archivo, con sus tres frases, la casilla Entiendo que no se puede deshacer sin marcar y el botón Restaurar todavía en gris](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/settings/es-02-restaurar.png)

**Ejemplo.** Lucas borró sin querer una lista entera y quiere volver al respaldo del viernes. En **Copias** pulsa **Restaurar desde un archivo…** y elige el zip. Tisty le muestra **Restaurar desde un archivo** y el botón **Restaurar** sigue gris: lo que escribió desde el viernes se pierde y, como sincroniza con una carpeta, su equipo deja de compartirla hasta que la elija otra vez. Lo piensa, marca **Entiendo que no se puede deshacer** y restaura. Si hubiera pulsado **Cancelar**, no habría cambiado nada.

## Lo que dice cada fila

Lo que pasa cuando tocas algo sale justo debajo de la fila que lo causó, nunca arriba de la página, así que siempre sabes a qué se refiere. Si usas un lector de pantalla, lo anuncia en cuanto aparece.

| Lo que ves | Qué quiere decir |
|---|---|
| **Guardado**, en verde | Tisty ya guardó el cambio. Se va solo en un par de segundos; una frase más larga, como «Quedará listo la próxima vez que inicies sesión», se queda unos segundos más. |
| **Guardando… 3 s**, o **Sincronizando…**, **Guardando el respaldo…**, **Restaurando…**, **Revisando…** | Esa fila está trabajando, y los segundos cuentan cuánto lleva. Solo sale si tarda, y solo en esa fila. |
| **Esperando a que termine «Respaldo».** | Otra fila está trabajando con tus datos, y esta espera su turno con el botón en gris. |
| **Error**, en rojo | Algo falló, y la nota dice por qué. Cuando se puede repetir, como un respaldo, restaurar, una revisión, cifrar o preparar un informe, trae **Reintentar**. |
| **Falta algo**, con borde punteado | Este ajuste depende de otro que está apagado o que todavía no existe. El botón de la nota te lleva a arreglarlo. |
| **Lo tiene el sistema**, con borde punteado | Lo decide Windows o macOS, no Tisty. La nota dice dónde cambiarlo y, cuando puede, te abre esa pantalla. |
| **No aplica aquí**, con borde punteado | En esta copia de Tisty ese ajuste no tiene sentido, y la nota dice por qué. |
| **Atención**, en ámbar | Lo mismo que cuenta **Por atender**, aquí en la fila donde se resuelve. |
| **Dato**, en azul | Contexto que no te pide nada. |

Un interruptor en gris nunca está roto: siempre lleva debajo la nota que dice por qué y, si se puede, el botón que lo destraba.

![La página Tareas y avisos, con Recordatorios importantes en gris y debajo la nota Falta algo que pide encender Una notificación del sistema, con el botón Encenderla](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/settings/es-01-notas.png)

| Página › fila | Nota | Cuándo | Lo que dice | Botón |
|---|---|---|---|---|
| Aspecto y ventana › **Iniciar con el equipo** | **Lo tiene el sistema** | Windows le quitó el inicio a Tisty | «Ahora ese interruptor lo tiene Windows. Vuelve a activarlo en Configuración → Aplicaciones → Inicio.» | **Abrir Configuración de Windows** |
| Tareas y avisos › **Recordatorios importantes** | **Falta algo** | **Una notificación del sistema** está apagada | «Primero enciende «Una notificación del sistema»: sin ella ningún recordatorio llega al sistema.» | **Encenderla** |
| Tareas y avisos › **Recordatorios importantes** | **Lo tiene el sistema** | Está encendido y falta permitirlo en el sistema | En Windows: «En Configuración → Sistema → Notificaciones → Tisty, permite las notificaciones importantes. Windows también puede preguntártelo en un aviso la primera vez: permítelo ahí también.» En un Mac, el paso de Ajustes del Sistema y Concentración | **Abrir la configuración del sistema** |
| Seguridad › **Cifrar mis datos en la nube** | **Falta algo** | No hay una nube directa | «Conecta una nube en Sincronización para cifrar lo que Tisty guarda ahí.» | **Ir a Sincronización** |
| Almacenamiento › **Dónde se quedan los adjuntos grandes** | **Falta algo** | No hay carpeta compartida | «Todavía no hay carpeta compartida, así que todo se queda en este equipo.» | **Ir a Sincronización** |
| Agentes › Puerta › **Recordarle al empezar y al terminar cada turno** | **Falta algo** | La puerta está cerrada | «Estos dos ajustes actúan cuando la puerta está abierta.» | — |
| Agentes › Conectados › **Agentes en este equipo** | **Falta algo** | Hay agentes conectados y la puerta está cerrada | «Aun así no anotarán nada hasta que abras la puerta.» | **Ir a Puerta** |
| Agentes › Cómo se ve su firma › **Nombrar al agente dentro de una tarea** | **Falta algo** | **Solo un rombo** está encendido | «Los dos de arriba quedan apagados mientras «Solo un rombo» esté encendido.» | — |
| Ayuda y acerca de › **Recibir versiones candidatas** | **No aplica aquí** | La copia de la Microsoft Store | «La Microsoft Store mantiene esta copia al día y no ofrece candidatas.» | — |
| Ayuda y acerca de › la actualización | **Dato** | Hay una versión nueva | «Este aviso también sale en el Resumen y como punto en la barra, no solo aquí.» | — |

### Una cosa a la vez con tus datos

Todo lo que lee o reescribe tu historial, la carpeta compartida o tu nube va de una cosa por vez: sincronizar, respaldar y restaurar, cifrar y descifrar, tu frase y tus equipos, llevar y traer documentos, la revisión de **Salud**, rehacer el índice rápido, liberar espacio, los adjuntos grandes, los botones de la **Zona delicada** y **Salir de Tisty**. Mientras una trabaja, las demás dicen **Esperando a que termine «…»** con el nombre de la que trabaja, y sus botones esperan en gris.

Lo de todos los días sigue libre: idioma, aspecto, Tisto, qué hace al cerrar la ventana, iniciar con el equipo, los atajos, los avisos, tu calendario, tu alias, tu nombre para firmar, el historial de documentos, la puerta y la firma de tu agente, la guía y la ayuda.

**Ejemplo.** Jaime pulsa **Crear respaldo…** y elige su disco externo. Al rato, bajo **Guardar una copia** aparece **Guardando el respaldo… 4 s**, y en la **Zona delicada** **Restaurar** dice «Esperando a que termine «Respaldo».», con el botón en gris. Mientras tanto va a **Aspecto y ventana** y elige **Oscuro**: sale **Guardado** al momento, porque el aspecto no toca sus datos. Si el disco se desconecta a mitad de camino, la fila del respaldo muestra **Error** con la causa y **Reintentar**; lo conecta otra vez y reintenta desde ahí.

### Atención, en su fila

Lo que **Por atender** junta en **Resumen** sale también en la fila donde se resuelve, en ámbar, con el título en negrita y, cuando hace falta, el porqué debajo:

| Página › fila | Título | El porqué |
|---|---|---|
| Sincronización › **Dónde se sincroniza** | **Llevas días sin recibir nada de tus otros equipos** | «Puede que estén apagados, o que el programa que sube esta carpeta no esté en marcha.» |
| Seguridad › **Tu identidad** | **Tu frase todavía no está confirmada** | «Escríbela una vez para comprobar que la guardaste bien. Sin eso no puedes cifrar tu nube.» |
| Seguridad › **Tus equipos** | **Portátil de Jaime escribe sin que hayas confirmado su clave** | «No ha dicho con qué firma. Nada de lo que escribe se comprueba hasta que lo confirmes.» |
| Tareas y avisos › **Captura rápida** | **La captura rápida se quedó sin atajo** | «Otra aplicación ya los usa. Elige otra combinación.» |
| Copias › **Guardar una copia** | **Tu último respaldo es de lun, 5 oct 11:47** | — |
| **Salud**, arriba | **3 cosas por revisar en tu historial** | — |
| Agentes › Conectados › **Agentes en este equipo** | **Claude Code apunta a un Tisty que ya no existe** | — |

Con una carpeta, **Dónde se sincroniza** también muestra ahí si tu nube pide iniciar sesión, si hay algo que revisar o si la sincronización está en pausa; con una nube directa, eso lo dice la tarjeta de tu nube, con su botón. La actualización no repite su aviso: su fila ya es la oferta, con la nota **Dato**.

**Ejemplo.** Marisol pulsa **Confirmarla ›** en **Resumen** y llega a **Tu identidad**, con la nota ámbar **Tu frase todavía no está confirmada** y, debajo, «Escríbela una vez para comprobar que la guardaste bien. Sin eso no puedes cifrar tu nube.». Escribe sus palabras, y la nota se va de la fila, de **Por atender** y del 1 junto a **Seguridad** a la vez.

## Solo este equipo y Viaja a tus equipos

Algunos ajustes llevan una etiqueta gris junto al nombre, para que sepas qué pasa en tus otros equipos sin tener que probarlo:

- **Solo este equipo**: el ajuste vale aquí y nada más. **Carpetas ocultas** y **Guardar las versiones anteriores** la llevan: ocultar una carpeta en el portátil no la oculta en el escritorio.
- **Viaja a tus equipos**: el ajuste llega a tus otros equipos cuando sincronizas. **Alias** la lleva: lo escribes una vez y todos firman igual.

Las etiquetas van donde la respuesta no es obvia. Lo que viaja con tus copias y lo que se queda, entero, está en [Sincronizar y respaldar](Sincronizar-y-respaldar).

## Cómo funciona

Cada página, menos **Resumen**, tiene **Cómo funciona ↗** arriba a la derecha. Abre esta ayuda en el artículo de esa página, y a veces en la sección justa: en **Seguridad**, por ejemplo, abre [Cifrar tu nube](Sincronizar-y-respaldar#cifrar-tu-nube); en **Copias**, [Respaldos](Sincronizar-y-respaldar#respaldos); en **Aspecto y ventana** y en **Salud**, su sección de este artículo. En **Ayuda y acerca de** abre el inicio de la ayuda. La ayuda viene dentro de Tisty, así que se abre también sin conexión, y te sugiere otros artículos para esa misma página.

**Saber más**, bajo algunas filas, es otra cosa: despliega ahí mismo la explicación larga de ese ajuste, sin salir de la página.

## Dónde quedó cada cosa

Si venías de una versión anterior, cuando Configuración tenía las páginas General, Sincronizar, Tus datos, Mantenimiento y Acerca de:

| Antes | Ahora |
|---|---|
| General › Idioma, Aspecto, Que Tisto se asome | Aspecto y ventana › Aspecto |
| General › Al cerrar la ventana, Iniciar con el equipo | Aspecto y ventana › Esta ventana |
| General › Carpetas ocultas | Aspecto y ventana › Carpetas ocultas |
| General › Captura rápida, Idea rápida | Tareas y avisos › Captura rápida |
| General › Avisos y Tu calendario | Tareas y avisos › Avisos |
| General › Alias, Nombre para firmar | Documentos y firma › Quién escribe aquí |
| General › La bienvenida, Abrir la guía | Ayuda y acerca de › Ayuda |
| Sincronizar | Sincronización |
| Sincronizar › Tus equipos | Seguridad › Tus equipos, una sola lista |
| Sincronizar › Dónde se quedan los adjuntos grandes | Almacenamiento › Adjuntos |
| Tus datos › Respaldo, Restaurar, Documentos de otro Tisty, Salir de Tisty | Copias |
| Tus datos › Historial de documentos | Documentos y firma › Historial de documentos |
| Tus datos › Tu carpeta | Almacenamiento › Dónde vive |
| Mantenimiento › la revisión, Por revisar, el índice rápido, Reportar un problema | Salud |
| Mantenimiento › Liberar espacio y las cifras | Almacenamiento › Liberar espacio y Cuánto ocupa |
| Acerca de | Ayuda y acerca de |
| Agentes, en una sola página | Agentes, en cinco pestañas |

## Cuando algo no se ve como aquí

- **Recién instalado**: **Resumen** empieza con **Deja Tisty a tu medida** y casi todos los mosaicos en gris. Es lo esperado: gris quiere decir «todavía nada», no «algo anda mal».
- **Sin sincronizar**: **Sincronización** muestra solo **Dónde se sincroniza** con **Activar…**; **Tus equipos** y su **Zona delicada** aparecen al sincronizar.
- **Una lectura que falla**: el mosaico se queda en «…» o en gris, y **Resumen** no dice **Nada pide tu atención.** hasta poder leerlo todo. Volver a **Resumen** lo lee otra vez.
- **Otro equipo**: cada equipo tiene su **Resumen**, sus respaldos y su revisión; lo que lleva **Viaja a tus equipos** llega al sincronizar.
- **La copia de la Microsoft Store**: las actualizaciones las trae la Store. Si todavía no dice qué número trae, **Resumen** dice **La Store tiene una versión nueva**.
- **Un agente**: ningún agente puede tocar la Configuración. Lo dice **Qué puede y qué no**: «Tocar la sincronización, los ajustes ni las máquinas» está entre lo que no podrá hacer nunca.
