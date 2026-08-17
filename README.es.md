# Tisty

[English](README.md) · **Español**

Un gestor de tareas local, privado y minimalista para macOS y Windows, con una
línea de comandos y una ventana que hacen lo mismo.

**Lo que terminas es el punto.** Un gestor de tareas que tira lo que resolviste
está tirando el único registro de cómo lo resolviste.

Sin cuenta, sin telemetría, sin servidor. Tus tareas son archivos de texto plano
en tu propio disco, legibles con `cat` y buscables con `grep`. Si Tisty
desaparece mañana, tus datos siguen ahí.

> **Alfa.** Todo lo de abajo funciona y lo usa a diario quien lo escribió. Lo
> que falta es el rodaje que solo dan las máquinas de otras personas. Linux es
> una fase propia y no ha empezado.

---

## Instalación

En macOS, con [Homebrew](https://brew.sh):

```console
$ brew install --cask rgdevment/tap/tisty      # la aplicación
$ brew install rgdevment/tap/tisty-cli         # solo la línea de comandos
```

O descarga la imagen de disco o el instalador desde
[Releases](https://github.com/rgdevment/Tisty/releases).

**No necesitas las dos.** La aplicación lleva la línea de comandos dentro
—Configuración la pone al alcance de tu terminal—, así que la fórmula es para
quien quiere el comando y ninguna ventana. En cualquier caso, el comando que
escribes es `tisty`.

## Por qué existe

La mayoría de los gestores de tareas tratan lo completado como basura: lo tachas
y desaparece. Para una lista de la compra está bien. Para el trabajo, no.

Una tarea como *"arreglar los timeouts intermitentes al guardar"* no es solo un
recordatorio. Cuando está terminada contiene el ticket, el commit, y los dos
párrafos que explican que la causa real era un índice que faltaba en una tabla
que nadie miraba. Ocho meses después, cuando vuelva a pasar en otro sitio, ese es
el único lugar donde vive ese conocimiento — y tacharla equivale a borrarlo.

**Cada tarea completada es una entrada en tu propia base de conocimiento.** No
apuntes que hay que acordarse de escribir: el registro que ya generaste mientras
hacías el trabajo.

Las herramientas que existen resuelven cada una otra cosa. Unas son excelentes en
la terminal pero no tienen dónde anotar qué pasó. Otras son editores potentes que
no son gestores de tareas. Otras están hechas para equipos, con permisos y
asignaciones que estorban cuando trabajas solo. Y casi todas cobran una
suscripción creciente por funciones que nadie pidió.

Lo que faltaba estaba en medio: **un gestor personal de tareas que además sea el
registro de cómo resolviste las cosas.** Local, privado, con una línea de
comandos de primera y una interfaz que no duela mirar.

Esto es una herramienta que construyo porque quiero usarla. Un desarrollador,
una necesidad, y una solución compartida por si también es la tuya. Es software
deliberadamente personal — sin equipos, sin colaboración, sin plan de
crecimiento, sin nada que venderte después. Gratuito y de código abierto, y así
se queda.

## Una tarea no termina cuando la tachas

**Una tarea completada no está terminada: está archivada.**

Deja de ser un recordatorio de lo que hay que hacer y pasa a ser el registro de
cómo se resolvió algo, con su ticket, su merge request, sus enlaces y las notas
de qué pasó realmente. Para la mayoría de las tareas que importan, **el valor
aparece después de tacharlas.**

De ahí salen tres consecuencias que atraviesan todo el diseño:

- **La búsqueda es la interfaz principal del archivo**, no una función
  secundaria.
- **Borrar es la excepción.** El camino normal es completar (archivar) o
  descartar.
- **La captura tiene que seguir siendo instantánea.** Todo campo es opcional y
  solo aparece cuando se usa, así que `tisty "agendar reunión con Pepe mañana"`
  sigue siendo una línea.

Porque también tiene que servir para esa reunión con Pepe, que nace y muere en
veinticuatro horas sin dejar rastro que valga.

## Lee lo que escribes

Así se captura una tarea, tanto en la ventana como en la terminal. Escribes una
frase; Tisty le saca la fecha, deja la frase legible y te enseña qué entendió
**antes** de guardar nada. En la ventana eso aparece como fichas que corriges
con un clic; los ejemplos de abajo usan la terminal porque caben en una página.

```console
$ tisty "entregar el informe mañana a las 10 en la oficina"
  ✓ entregar el informe en la oficina
    mañana 10:00

$ tisty "reservar los vuelos @viajes #urgente !alto"
  ✓ reservar los vuelos
    !1 · @viajes · #urgente
```

**Cuándo ocurre.** Un día, una hora, o las dos. Valen los nombres, las
distancias y las fechas a secas; lo que no sabe leer lo deja en paz en vez de
adivinar.

```console
$ tisty "tomar café a las 3"          →  hoy 15:00
$ tisty "reunión el lunes 15"         →  sáb
$ tisty "revisar la próxima semana"   →  23 ago
```

**Cuándo vence de verdad.** El límite no es lo mismo que el plan: la fecha es
cuándo piensas hacerlo, el límite es la pared de detrás. Tres formas lo abren
—**antes de**, **vence** y **hasta**— y se leen igual.

```console
$ tisty "entregar el informe antes del viernes"     →  límite vie
$ tisty "renovar el dominio vence el 30 de agosto"  →  límite 30 ago
$ tisty "enviar la factura hasta el viernes"        →  límite vie
```

Cuidado con **para**, que hace lo contrario: `para el viernes` es un plan,
`antes del viernes` es un límite.

**Qué vuelve.** Nombrar un día la hace fija; nombrar solo un intervalo la hace
relativa. La basura sale el martes tanto si la sacaste la semana pasada como si
no, pero los tres días empiezan a contar cuando de verdad regaste.

```console
$ tisty "sacar la basura cada martes"      →  mar · ↻ cada semana
$ tisty "regar las plantas cada 3 días"    →  ↻ cada 3 días
```

Una tarea que se repite puede llevar su propio límite, y es el **de esa
ocurrencia**: el alquiler vence el día 5 de cada mes, no una sola vez.

```console
$ tisty "enviar el reporte cada semana antes del viernes"
  ✓ enviar el reporte
    límite vie · ↻ cada semana
```

Y a una serie se le puede decir cuándo parar. **Hasta** termina la repetición:
la última no deja sucesora.

```console
$ tisty "tomar la pastilla cada día a las 9 hasta el 30 de septiembre"
  ✓ tomar la pastilla
    mañana 09:00 · ↻ cada día hasta el 30 sep
```

La misma palabra hace dos trabajos y la frase decide cuál: con una cadencia
dentro, `hasta el 30 de septiembre` termina la serie; sin ella, es el límite. Di
`antes de` o `vence` cuando quieras un límite en una tarea que se repite.

**Y lo que se niega a leer importa más.** Una suposición que suena bien es peor
que ninguna, así que estas conservan cada palabra y no llevan fecha:

```console
$ tisty "revisar el informe del lunes"   # el informe puede llamarse así
$ tisty "contrato de 6 meses"            # una duración no es una fecha
$ tisty "revisar lo de hace 3 días"      # el pasado no se agenda
$ tisty "montar soporte 24/7"            # 24/7 es una expresión, no el 24 de julio
```

Dos reglas más que conviene saber. **Lo entrecomillado no se toca**, así que
`tisty '"reunión el lunes"'` guarda la línea entera. Y cuando la frase va en
medio sin nada que la respalde —`llamar mañana al banco`— la fecha **se aplica
igual, pero marcada como suposición**: la terminal lo dice e imprime el comando
que deshace solo eso, y la ventana la subraya para que un clic la quite.

Cada frase de arriba es un test. El parser lleva detrás un contrato de 261 casos
en español e inglés que fija qué debe leer y, con la misma frecuencia, qué debe
dejar en paz.

En inglés funciona con las mismas reglas y sus propias palabras: `every tuesday`,
`every 3 days`, `before friday`, `due august 30`,
`every day at 9am until september 30`.

## La ventana

Tres columnas como mucho: qué estás mirando, la lista, y la tarea que abriste.
Nada más en pantalla.

**Tareas**, con cuatro rodajas —*hoy*, *próximas*, *se repiten*, *todas*— y
vuelves a la que elegiste la última vez. Debajo tus listas, tus etiquetas, el
archivo, y una búsqueda que llega a todo.

**Una tarea se abre al lado de la lista**, no encima: título, fechas, lista,
etiquetas y prioridad; descripción y bitácora en Markdown; pasos que tachas de
uno en uno; y lo que le hayas soltado encima. Completarla no aleja nada de eso
— pasa al archivo, que es donde la búsqueda hace su mejor trabajo.

**Capturar es un campo arriba.** Lo que Tisty entendió aparece debajo como
fichas antes de guardarse, y una ficha está a un clic de estar equivocada a
propósito. Un atajo global abre un campo pequeño sobre lo que estés haciendo,
para que una tarea que se te ocurre a media cosa no te cueste la cosa.

**Los documentos** viven al lado de las tareas, para el material de consulta que
no tiene fecha y nunca se tacha. Son archivos Markdown en tu almacén, y se editan
como documentos, no como código fuente: tablas, listas de comprobación, código e
imágenes, y lo que pegues de una página o de un ticket conserva su forma. Una
tarea puede apuntar a un documento; un documento nunca crea tareas. **La búsqueda
también los lee** —título y cuerpo—, así una línea que escribiste en un documento
se encuentra igual que una tarea.

Sacarlo de aquí tiene dos formas, y no son lo mismo. **Copiar como Markdown** deja
el texto en el portapapeles, referencias incluidas: pégalo donde quieras, pero una
imagen vive en tu almacén y no te sigue. **Exportar como Markdown** escribe una
carpeta — el documento junto a un `attachments/` propio, con solo los archivos que
ese documento nombra. Esa sí se abre en cualquier sitio, se comprime y viaja.

**Las listas** tienen pantalla propia, cada una con un icono que eliges de un
juego, y **los recordatorios** llegan como notificación del sistema y un sonido
corto que se puede apagar. Las tareas que se repiten vuelven solas, una
ocurrencia cada vez.

**Configuración** guarda tus datos (sincronización, respaldo, dónde vive el
almacén), los avisos, la escritura y el mantenimiento — incluido un informe que
puedes adjuntar a un fallo, y que te enseña exactamente lo que lleva antes de
guardarlo.

La ventana entera funciona con el teclado: flechas por la lista, `Ctrl+Enter`
para completar, `Escape` para cerrar una tarea, y un anillo de foco visible allá
donde vaya.

## Y una línea de comandos, si la usas

No es la vía principal —esa es la ventana—, pero todo lo que hace la ventana lo
hace también la terminal. Lo que cambia es cuántas teclas cuesta, no lo que
se puede.

```console
$ tisty "arreglar los timeouts intermitentes al guardar" --priority 1
  ✓ arreglar los timeouts intermitentes al guardar
    !1

$ tisty log 1 "el presupuesto de reintentos se agotó antes de que el pool se rellenara"
$ tisty done 1

$ tisty search "presupuesto de reintentos"
  «presupuesto de reintentos»                          1 tarea
    1  ✓ arreglar los timeouts intermitentes al guardar
       ✎1
```

A una tarea la nombras por su número en el último listado, por un fragmento de
su título (`tisty done pagos`) o por su identificador. Los filtros se combinan y
se escriben como los marcadores con los que capturas: `tisty ls semana
#seguridad`, `tisty ls @trabajo !1`.

Para automatizar: `--json` en toda orden de lectura, stdout son datos y stderr
es conversación, códigos de salida que significan algo (`0` bien · `1` error ·
`2` mal uso · `4` no encontrado), y `tisty export` te devuelve los datos como
JSON o como un documento Markdown que se lee sin Tisty.

## Tus datos

Un directorio con archivos de texto:

```
<datos de aplicación>/tisty/data/
└── store/
    └── dev_a3f1/
        ├── 000001.tisty      segmento cerrado, ya no cambia
        ├── 000001.count      cuántas líneas trae, para cazar una descarga a medias
        └── active.tisty      una línea por evento
```

```jsonl
{"v":1,"ts":"2026-08-05T08:27:49Z","by":"dev_a3f1","op":"task.add","id":"01KZ8G…","d":{"title":"arreglar los timeouts intermitentes al guardar","priority":1,"tags":["backend","db"]}}
```

Vive en el directorio de datos locales de tu sistema, y eso no se configura:
un gestor de tareas que te deje meter su propio almacén en una carpeta que un
cliente de nube reescribe por detrás te está dando un arma cargada. Sincronizar
es otra cosa: Tisty **deja copias** en una carpeta que tus dos equipos alcancen
y **se trae las que dejaron los demás**. Son dos rutas distintas, y solo una la
eliges tú.

Tu configuración nunca viaja con ellos. El identificador de cada equipo vive en
el archivo de configuración precisamente para quedarse en esta máquina: si
viajara, dos equipos lo compartirían, escribirían en el mismo archivo y la
garantía de abajo se vendría abajo.

Un registro de eventos que solo crece. De ahí salen gratis el historial y el
deshacer, y también una sincronización sin conflictos cuando llegue: **cada
máquina escribe únicamente en su propio directorio**, así que fusionar dos
historias es concatenarlas.

Eso es una sola lista, no una por máquina. Cada equipo lee todos los
directorios y los reproduce en orden; solo escribe en el suyo. Por eso una
carpeta sincronizada nunca produce uno de esos `archivo (copia en conflicto)`:
dos escritores jamás tocan el mismo archivo.

## Dos máquinas

Apunta Tisty a una carpeta que tus dos equipos ya alcancen —la que mantiene al
día tu cliente de Google Drive, OneDrive o iCloud; un NAS montado; un disco
externo que enchufas los viernes— y el resto lo hace solo: baja al abrir la ventana, sube
un rato después de que cambies algo, y hace las dos cosas cada cierto tiempo.
Nunca bloquea lo que estás escribiendo ni te interrumpe; si la carpeta no está
disponible, lo vuelve a intentar en silencio.

Nadie pregunta nunca «¿cuál es más nueva?», porque el directorio de cada equipo
tiene un único escritor: subiendo manda la tuya, bajando manda la suya.

Los documentos son lo único que dos equipos pueden escribir de verdad a la vez, y
ahí Tisty los junta **bloque a bloque**: editas la introducción en el Mac,
alguien edita el cierre en Windows, y las dos cosas llegan sin que tengas que
responder nada. Solo cuando los cambios se pisan hay una pregunta, y ahí decides
tú, con «quedarme las dos» ofrecido primero porque es la única respuesta que no
pierde nada.

Apunta a la misma carpeta dos máquinas que ya venías usando y Tisty se para y
pregunta, porque tu propia segunda máquina y la carpeta de otra persona son el
mismo gesto. Puedes **unir los dos historiales**, conservar esta máquina, o
quedarte con lo que guarda la carpeta. Todas respaldan antes.

Quién mantenga esa carpeta no es asunto de Tisty, y no hay nada nuestro en medio
—ni cuenta, ni servidor, ni proceso residente—. Si no sincronizas nada, Tisty no
abre una conexión en su vida.

**O respalda a mano.** Un zip, guardado donde quieras. Restaurar es como volver a
una fotografía: vuelves a ese momento y lo posterior se pierde a propósito. Las
dos cosas se excluyen, porque la carpeta compartida ya guarda la historia de
todos tus equipos y una copia al lado sería otra verdad compitiendo con ella.

## En qué punto está

| | |
|---|---|
| ✅ | Núcleo: modelo, registro de eventos, almacenamiento, proyección |
| ✅ | CLI: capturar, listar, completar, ver detalle, bitácora, pasos, listas, etiquetas |
| ✅ | Lenguaje natural: `tisty "desplegar la API mañana a las 10"` |
| ✅ | Búsqueda en tareas, deshacer y rehacer, `--json`, `export`, códigos de salida |
| ✅ | Ventana (Tauri): lista y detalle, Markdown, adjuntos, teclado en todo |
| ✅ | Sincronización por una carpeta que ambos equipos alcanzan, y respaldo a mano |
| ✅ | Bandeja y barra de menús, con captura rápida en un atajo global |
| ✅ | Tareas que se repiten, una por ocurrencia, plegadas en el archivo |
| ✅ | Recordatorios, con notificación del sistema y un sonido que puedes apagar |
| ✅ | Un registro de errores, y un informe que puedes adjuntar a un issue |
| ✅ | macOS: imagen de disco universal, firmada y notarizada, y Homebrew |
| ✅ | Documentos: editor, carpetas, adjuntos, y transporte que junta bloque a bloque |
| ✅ | Dos historiales que se encuentran: unirlos, conservar un lado, o adoptar el de la carpeta |
| ◐ | Uso diario, que es lo que saca los fallos que los tests no |
| ◐ | Builds firmadas: el DMG y el `.exe` se publican; el paquete de la Store espera un nombre |
| ⬜ | Linux, fase propia, sin empezar |

Dos cosas son sabidas y aceptadas, no pendientes: no se puede reordenar a mano
en la ventana —el arrastre de HTML no sobrevive al arrastre nativo de archivos
que Tauri necesita para los adjuntos, y quedarse con los adjuntos era el mejor
trato— y nada se ha probado con un lector de pantalla real, aunque el recorrido
de teclado sí.

## Qué nunca va a hacer

Tan importante como la lista anterior. Permanentemente fuera del alcance:
colaboración en tiempo real, tableros kanban, diagramas de Gantt, control de
tiempo, métricas de productividad, bases de datos con propiedades tipadas y
fórmulas, y IA en el camino crítico de cualquier operación.

El intérprete de lenguaje natural será determinista y local. Nada se envía nunca
a ningún modelo.

## Otras herramientas

Las mismas manos, la misma idea: gratuitas, de código abierto, sin anuncios, sin
telemetría, todo en tu máquina.

- **[CopyPaste](https://github.com/rgdevment/CopyPaste)** — un gestor de
  portapapeles para Windows, macOS y Linux.
- **[LinkUnbound](https://github.com/rgdevment/LinkUnbound)** — un selector de
  navegadores para Windows y macOS: pregunta con cuál abrir un enlace en vez de
  suponerlo.

## Sobre hombros ajenos

Tisty es pequeño porque el trabajo pesado lo hace gente que no soy yo. Estas son
las piezas sin las que no existiría, cada una con una licencia que lo permite:

**El núcleo, en Rust** — [Tauri](https://tauri.app) le pone una ventana nativa
sin cargar con un navegador; [serde](https://serde.rs) lee y escribe cada línea
del registro; [jiff](https://github.com/BurntSushi/jiff) resuelve las fechas y
las zonas horarias, que es la parte que nadie debería escribir dos veces;
[SQLite](https://sqlite.org), a través de
[rusqlite](https://github.com/rusqlite/rusqlite), sostiene la caché de lectura;
[clap](https://github.com/clap-rs/clap) es la línea de comandos;
[ULID](https://github.com/dylanhart/ulid-rs) da a cada tarea un identificador
que ordena por tiempo y no necesita coordinar nada con nadie.

**La ventana** — [React](https://react.dev) la dibuja y
[Tailwind CSS](https://tailwindcss.com) la viste;
[TipTap](https://tiptap.dev) y [ProseMirror](https://prosemirror.net) son el
editor de documentos; [markdown-it](https://github.com/markdown-it/markdown-it)
compone la prosa del resto; [Vite](https://vite.dev) la construye y
[Vitest](https://vitest.dev) la prueba.

La lista completa, con versiones y licencias, está en `Cargo.lock` y
`app/package-lock.json`.

## Contribuir

Cómo funcionan de verdad el almacén, la fusión y el transporte está escrito en
[ARCHITECTURE.es.md](ARCHITECTURE.es.md) — una referencia del comportamiento, no
un paseo por el código.

Lee [CONTRIBUTING.md](CONTRIBUTING.md). Abre un issue antes de escribir código
para cualquier cosa que no sea una corrección: Tisty es deliberadamente
minimalista y una funcionalidad bien escrita puede rechazarse igualmente.

## Licencia

[AGPL-3.0](LICENSE), y disponible bajo [términos comerciales](COMMERCIAL.md)
para organizaciones que no puedan cumplirla.

Los binarios firmados de las tiendas llevan términos propios, porque los de las
tiendas no admiten la AGPL — [DISTRIBUTION.md](DISTRIBUTION.md) dice cuál se
aplica a lo que tengas, y por qué. No se retiene nada del código en ningún caso.

Ver también [SECURITY.md](SECURITY.md) y [PRIVACY.md](PRIVACY.md) — el resumen
de esta última es que no se recoge nada y no se envía a ninguna parte.
