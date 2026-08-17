# Arquitectura

[English](ARCHITECTURE.md) · **Español**

Cómo Tisty guarda, fusiona y lee sus datos. Es una referencia de cómo se
comporta el sistema, no una crónica de por qué se diseñó así.

## Las dos capas

```
<data>/store/                    la verdad. Se sincroniza. Sobrevive a todo.
├── dev_a3f1/
│   ├── 000001.tisty             segmento cerrado, nunca se vuelve a escribir
│   └── active.tisty             el único archivo al que este dispositivo agrega
└── dev_9f2c/
    └── active.tisty             de otro dispositivo, este nunca lo toca

<cache>/read.db                  una fotografía. Local, desechable, se rehace a demanda.
```

El almacén es un registro de eventos. La caché es el estado que esos eventos
producen. Borrar la caché cuesta una lectura lenta; borrar el almacén pierde
datos.

Ninguno de los dos vive en la carpeta Documentos del sistema —que no es lo mismo
que el `docs/` de Tisty— y la ruta no se configura.

## El registro de eventos

Un objeto JSON por línea. Los archivos solo crecen por el final, y un segmento
cerrado no se vuelve a modificar.

```jsonl
{"v":1,"ts":"2026-08-06T22:22:18.006Z","by":"dev_a3f1","op":"task.add","id":"01KZ…","d":{"title":"comprar pan","order":"V"}}
```

| Campo | Significado |
|---|---|
| `v` | versión del esquema; una más nueva se rechaza, no se adivina |
| `ts` | cuándo ocurrió, en UTC |
| `by` | qué dispositivo lo escribió |
| `n` | secuencia dentro de ese dispositivo, ausente cuando es cero |
| `tx` | agrupa los eventos de una misma acción del usuario |
| `un` / `re` | marca una compensación o la reejecución de una |
| `op`, `id`, `d` | la operación, la entidad que afecta, su carga |

`active.tisty` se cierra como `NNNNNN.tisty` cada 5.000 eventos. Los segmentos
cerrados se numeran desde uno y sin huecos.

### Escritura

Siempre y solo en el directorio propio de este dispositivo. Dos reglas sostienen
todo lo demás:

**El tiempo solo avanza.** El almacén recuerda la última marca que escribió y
nunca emite una menor o igual. Un reloj que retrocede reutiliza el instante y
sube `n` en su lugar.

**El registro se escribe antes que la caché**, con `fsync`. Una caída entre los
dos deja la caché *atrás*, y eso se repara solo en la siguiente lectura. Nunca
puede dejar la caché *adelante*, que significaría datos que no existen en
ninguna otra parte.

Las dos reglas valen por directorio de dispositivo, y más de un proceso puede
ser dueño del mismo: una ventana abierta y un comando `tisty` son el mismo
dispositivo. Por eso una escritura toma un bloqueo exclusivo y **lo mantiene
solo durante esa escritura** — un proceso que lo retuviera rechazaría todos los
comandos mientras siguiera abierto. Si el bloqueo se encuentra ocupado se espera
un momento, porque una escritura dura microsegundos; solo el que se queda
ocupado se reporta como conflicto.

Soltarlo implica que los contadores pueden quedar desactualizados, así que la
siguiente escritura los vuelve a leer primero. Compara el tamaño del registro
activo con el que vio la última vez, y solo vuelve a parsear cuando difieren.
Sin eso, dos procesos ponen marcas desde relojes que nunca vieron los eventos
del otro, y `(ts, by, n)` deja de ser único — que es justo lo único que el orden
de fusión no puede resistir.

### Lectura

Cada dispositivo lee todos los directorios, incluido el suyo. Los eventos de
todos se concatenan y se ordenan por `(ts, by, n)` — el mismo orden en cada
máquina, así que todas las máquinas llegan al mismo estado.

La lectura se niega a continuar, en vez de devolver una historia más corta,
cuando:

- falta un segmento cerrado en la secuencia,
- hay un segmento cerrado presente pero vacío,
- un segmento cerrado contiene una cantidad de eventos distinta de la que
  declara su `.count`,
- alguna línea no se puede parsear,
- un evento declara una versión de esquema que esta compilación no conoce.

La sincronización sostiene la misma línea desde el otro extremo: la historia de
un dispositivo que llega **más corta que la que ya se tiene** se deja donde
está. La contigüidad por sí sola no lo detecta — el hueco no está *entre*
segmentos cerrados sino *antes* de `active`, y nada en la carpeta dice cuántos
cerrados debería haber. Por eso se comparan las cantidades. Un cliente de nube
bien puede entregar un `active` ya rotado antes que el segmento
cerrado que lleva lo que soltó; ese orden no puede costarle a nadie su copia.

## Cómo funciona la fusión

Nada se edita nunca. Se registran hechos sobre una entidad, y la entidad se
identifica con un ULID que significa lo mismo en todas partes.

```
dev_mac      task.add    MNKMPX  "comprar pan"
dev_windows  task.log    MNKMPX  "fui a la tienda de la esquina"
             task.done   MNKMPX
```

Tres eventos, dos archivos, cero coordinación. Cualquier dispositivo que lea los
dos archivos produce la misma tarea: creada, completada, con una entrada de
bitácora.

Los archivos nunca se fusionan. **La fusión ocurre en memoria, en cada
lectura.**

Los campos que nadie más tocó simplemente se conservan. Cuando dos dispositivos
escriben el mismo campo sin haberse visto, gana la marca más tardía. Las
colecciones —entradas de bitácora, pasos, etiquetas— se acumulan en vez de
competir, porque cada elemento lleva su propio identificador.

Borrar es la excepción: deja una lápida, y nada sobre esa entidad se vuelve a
aplicar. Eso es lo que impide que un evento atrasado la resucite.

## Repetición

Una repetición es **una tarea por ocurrencia**, no una entidad que va
coleccionando finalizaciones. Terminar «sacar la basura cada martes» escribe dos
eventos en un mismo lote: la finalización y la tarea del martes que viene.

```
task.done   MNKMPX
task.add    MNKMQ2   "sacar la basura"   2026-08-18
```

Un solo lote, porque deshacer tiene que revertir los dos — si no, cada deshacer
dejaría una copia y la serie crecería sola.

Cuesta una fila por ocurrencia, y compra aquello para lo que existe el archivo:
muestra que lo hiciste doce veces. Cada ocurrencia tiene su propia bitácora, así
que «esta semana no pasó el camión» tiene dónde vivir. El archivo pliega las
repeticiones de un mismo mes en una sola línea para que las filas no se vuelvan
ruido.

Una cadencia la abre una palabra (`every`, `each`), dos (`todos los`), o un
adverbio que es la cadencia entera (`daily`, `weekly`, `annually`). Un día por
repetición: «martes y jueves» son dos tareas, no una.

Cómo se calcula la fecha siguiente depende de cómo se escribió. Nombrar un día
la fija al calendario —la basura sale el martes, haya salido o no la semana
pasada— y nombrar solo un intervalo cuenta desde el hacer, que es lo que
significa un hábito. En cualquiera de los dos casos la siguiente cae después de
hoy **y** después del día en que se terminó: volver de quince días fuera no te
deja quince días de basura pendiente de golpe, y terminar la de hoy no te
entrega otra para hoy. La hora del día se conserva tal como se pidió —terminar a
las 08:04 la pastilla de las 09:00 no la corre a las 08:04 para siempre— y los
meses y los años se cuentan por el calendario aunque se digan como intervalo, o
el arriendo se iría corriendo unos días cada mes.

Nada se crea nunca por adelantado. No hay temporizador ni planificador: una
tarea solo puede nacer de que tú la escribas o de que termines una repetición.
Sáltate un día y sigue habiendo exactamente una, esperando, atrasada — nunca
dos.

## La caché de lectura

SQLite en el directorio de caché, con el estado proyectado: tareas, listas y
lápidas.

La frescura se decide con una huella: el nombre y el tamaño de cada archivo del
registro. Si coincide, el estado se carga desde la caché. Si no, se reproduce el
registro y se reescribe la caché.

Después de una escritura, la caché se **actualiza**, no se descarta: solo se
reescribe la entidad que el evento tocó. Un evento que llega más lejos que su
propia entidad —borrar una lista devuelve a la bandeja de entrada todas las
tareas que tenía— renuncia al camino rápido e invalida.

Cualquier cosa que salga mal al abrir o leer la caché cae de vuelta al registro.
La caché se puede borrar en cualquier momento.

### Revisarla

```sh
tisty doctor            # reproduce el registro y lo compara con la caché
tisty doctor --repair   # descarta la caché; la siguiente lectura la reconstruye
```

La caché está desactualizada, ausente, de acuerdo, o **equivocada**. Solo la
última sale con código distinto de cero. `doctor` informa y nunca repara por su
cuenta, porque el registro gana cualquier desacuerdo y reconstruir es la única
reparación que existe.

## Sincronización

A través de una carpeta que ambas máquinas alcanzan. Nada más.

```sh
tisty config set remote <folder>   # dónde van las copias
tisty sync                         # deja la nuestra, trae la de todos los demás
tisty sync --push                  # solo dejar
tisty sync --pull                  # solo traer
tisty sync --join <backup.zip>     # respalda esta máquina, la vacía, toma la de la carpeta
tisty sync --take-over <backup.zip> # respalda la carpeta, la vacía, deja la nuestra
tisty sync --merge <backup.zip>    # respalda, y después se queda con las dos historias
```

Tisty siempre trabaja en su propio directorio local. Sincronizar **deja una
copia** en esa carpeta y **se trae las copias que dejaron los demás**. Lo que
mantenga viva esa carpeta —el cliente de Google Drive, OneDrive o iCloud que ya
tienes andando, un NAS montado, un disco externo que enchufas una vez por
semana— no es asunto de Tisty.

Esa carpeta **no** es el directorio de datos, y apuntar un cliente de nube a
`AppData` sigue siendo un error. El almacén se queda en tu disco; lo único que
viaja son copias.

**Solo escriben ahí las máquinas que están en la lista.** Estar en ella es lo
que le da voz a una máquina; la que fue removida conserva su copia y no vuelve a
subir nunca más. Te unes adoptando, no pidiendo permiso —alcanzar esos archivos
es la autorización—, así que **remover es el único acto privilegiado**. Una
máquina que vuelve no fusiona: primero se respalda y se vacía, que es lo que
hace `--join`.

La carpeta es además **escritura ajena**, y se la trata como tal: no se escribe
nada a través de un enlace simbólico —ni en la carpeta compartida misma, ni en
ningún directorio de dispositivo o de estante dentro de ella—, un adjunto tiene
que contener los bytes que su nombre promete, uno que fue retirado no se vuelve
a traer, y un cuerpo de documento que pasa el techo del lector se rechaza en vez
de traerse para reemplazar a uno que sí se podía abrir.

### Por qué no hay nada que fusionar

Cada directorio de dispositivo tiene **exactamente un escritor**. Sube solo el
tuyo: nadie más lo toca, así que tu copia es la que manda. Baja solo los ajenos:
tú nunca los escribes, así que la copia de ellos es la que manda. La pregunta
«¿cuál es más nueva?» no aparece nunca, y dos máquinas no pueden producir un
archivo en conflicto.

Lo que llega se lee antes de escribirse. Un `000002` sin su `000001`, un
segmento descargado a medias, una copia en conflicto que dejó un cliente de nube
— todos se rechazan en la puerta, porque leer uno roto se lleva abajo el almacén
entero, con todos los dispositivos dentro. **Ese rechazo es solo de esa
máquina**: un directorio de dispositivo ilegible en la carpeta se queda afuera y
se nombra en el resultado, y todo lo demás —tu propia escritura sobre todo— pasa
igual. Los archivos que ya son idénticos se saltan, así que sincronizar dos
veces seguidas no mueve nada la segunda vez.

Los adjuntos también viajan. Se llaman por su propio sha-256, así que un nombre
que coincide es un archivo que coincide y dos máquinas no pueden discrepar sobre
uno — y al entrar se verifican los bytes contra ese nombre.

Los cuerpos de los documentos viajan por **tres huellas y ningún reloj**: la
local, la de la carpeta, y la última que llevó esta máquina. Si un lado se
movió, se copia sin preguntar. Un reloj sería peor que inútil — un notebook que
despierta llega con una hora de desfase, y eso ya nos costó un fallo real.

Si se movieron los dos, las dos versiones se **fusionan bloque a bloque** antes
de preguntarle a nadie. La unidad es el bloque —el texto entre líneas en
blanco—, lo que regala atomicidad gratis: una tabla y una lista ordenada no
llevan líneas en blanco dentro, así que cada una es un bloque entero y no puede
quedar armada a medias entre un lado y el otro. El código delimitado por cercas
conserva sus líneas en blanco, porque ahí una línea en blanco es contenido. Solo
las ediciones que se pisan son una pregunta; dos contiguas se toman las dos y
listo.

El motor rechaza en vez de adivinar, y cada rechazo cae en el mismo camino
probado: la fusión no devuelve nada, el documento queda sin decidir, y **decide
la persona**, con «quedarme las dos» ofrecido primero porque es la única
respuesta que no pierde nada. Rechaza cuando los dos lados reescribieron el
mismo bloque de forma distinta, cuando la comparación costaría más de cuatro
millones de celdas, cuando el resultado tendría un bloque más veces de las que
lo tiene cualquiera de los dos lados o menos de las que conservaron ambos,
cuando el texto tejido no se volvería a partir exactamente en los bloques con
los que se hizo, y cuando el tejido dejaría dos listas pegadas una a la otra —
Markdown lee eso como una sola lista, y la costura solo se rechaza cuando fue la
fusión la que la creó.

Un documento con front matter YAML no se fusiona en absoluto: el editor no puede
devolverlo intacto, así que fusionarlo solo produciría ruido. Va directo a la
pregunta.

Los finales de línea se normalizan a la salida. Es deliberado: si cada máquina
conservara los suyos, sus huellas nunca coincidirían y el documento quedaría en
conflicto para siempre.

### Dos historias se unen solo cuando tú lo dices

Un almacén que lleva otro nombre se **rechaza antes de que se mueva nada**. Un
marcador `.store-id` lo custodia — una máquina que nunca vio la carpeta
**adopta** su nombre, y a una carpeta vacía se le **da** uno.

Cuando los dos lados tienen historia no hay suposición segura: tu propia segunda
máquina y la carpeta de un desconocido son el mismo gesto. Así que el rechazo no
es el final del camino, es una pregunta con cuatro respuestas. Todas escriben un
respaldo primero, y ninguna se puede deshacer desde la aplicación.

**Fusionarlas.** El almacén termina conteniendo las dos. Funciona por la misma
razón por la que funciona sincronizar —fusionar es concatenar— y nada choca: las
entidades son ULID, los documentos se llaman `<device>-NNNN.md`, los adjuntos se
llaman por su propio contenido. Lo que cuesta se dice sin rodeos de antemano:
dos listas con el mismo nombre siguen siendo dos listas, porque unirlas por
nombre es una suposición, y ahí una suposición equivocada pasa desapercibida; y
las claves de orden se acuñaron por separado, así que las listas se entrelazan.

**Conservar esta máquina.** La carpeta se respalda, se vacía y se repuebla desde
aquí. La otra máquina será rechazada la próxima vez y enfrentará la misma
pregunta — esa consecuencia se dice por adelantado, porque no es obvia.

**Quedarse con lo que tiene la carpeta.** Esta máquina se respalda, se vacía y
adopta la carpeta. Acuña un identificador de dispositivo nuevo, así que vuelve
como un participante nuevo en vez de arrastrar detrás su propia remoción. Esto
es lo que hace una máquina removida que vuelve, y lo que `--join` hizo siempre.

**Adoptar sin pérdida** — se ofrece cuando la carpeta ya contiene la historia
propia de esta máquina. No es una cuarta opción, es un hecho distinto: una
máquina que quedó atrás antes de una fusión encuentra su historia dentro de la
de la carpeta, y no tiene nada que decidir. Sin eso, una máquina así sería
rechazada por todas las demás puertas y quedaría arrinconada.

Cuál es el caso se lee en los segmentos, no se adivina. Un directorio de
dispositivo presente en los dos lados se compara como la **concatenación
ordenada de sus segmentos** — un solo escritor, que solo agrega al final, así
que un lado tiene que ser prefijo del otro. Se compara entero y no archivo por
archivo porque la rotación renombra lo que cierra: la misma historia puede ser
un archivo acá y dos allá. Un archivo de cero bytes —lo que deja un cliente de
nube antes de llenarlo— no prueba nada en ningún sentido. Un archivo que el
lector **no puede abrir** no es lo mismo y no se trata como tal: ahí la
respuesta es que todavía no se puede saber, y no se ofrece nada hasta que se
pueda, porque las dos respuestas llevan a lados opuestos. Donde no hay
evidencia, la respuesta depende de si algún nombre de dispositivo aparece en los
dos lados: si ninguno aparece, «desconocidos», porque una costura innecesaria es
contabilidad inofensiva; si alguno aparece, se rechaza, porque un nombre
compartido que no se puede probar es el caso fatal. «Nombres», acá, abarca los
directorios **y los ids que nombran los eventos**, así que un id que sobrevive
solo dentro del `device.remove` de otro también cuenta.

Cuando el mismo nombre de dispositivo existe en los dos lados habiendo escrito
cosas **distintas**, no se fusiona nada. Dos escritores bajo un mismo nombre es
justo lo que todo el diseño da por descartado.

**Cuando se fusionan dos historias, el nombre que sobrevive es el de la
carpeta.** No es por comodidad: `.store-id` es el único archivo de la carpeta
compartida que no tiene un único escritor. Acuñar un nombre nuevo —o imponer el
local— lo reescribe, y dos máquinas fusionando a la vez escribirían dos
contenidos distintos en un mismo archivo, que es exactamente la clase de
conflicto que todo lo demás acá está dispuesto para volver imposible. Adoptar el
nombre de la carpeta vuelve ese archivo inmutable en la práctica: las fusiones
concurrentes escriben los mismos bytes.

Una fusión escribe un evento `stores.joined` **antes** de tomar el nombre de la
carpeta, con los dos nombres y qué dispositivos vinieron de cada lado. Ese orden
no es cuestión de gusto: tomar el nombre primero y morir dejaría un almacén cuyo
nombre ya coincide, así que la pregunta no se vuelve a hacer y se pierde el
rastro. Escribir el evento primero significa que una muerte deja los nombres
todavía separados, la pregunta vuelve, y hacerlo dos veces solo registra un
ancestro que ya estaba registrado.

Ese conjunto de ancestros **se escribe pero todavía no se lee**: hoy la pregunta
del linaje se responde solo desde los segmentos, y la constancia existe para que
a una máquina que llegue mucho después se le pueda contar qué pasó, y no solo
qué elegir. Hasta que algo lo lea, esto es una afirmación sobre lo que queda
escrito, no sobre el comportamiento. No toca ninguna tarea ni ningún documento;
como `device.join`, es un evento que proyecta algo que no son datos — el
conjunto de nombres de almacén ancestros, que se acumula, para que una máquina
que llegue mucho después todavía pueda reconocer el suyo entre ellos.

Los nombres de directorio se comparan sin distinguir mayúsculas: en Windows y
macOS `DEV_A` y `dev_a` son un solo directorio, así que la copia de un
desconocido caería encima del único original que tiene esta máquina.

La sincronización corre sola — baja al abrir la ventana y al recuperar el foco,
sube un rato después de cada cambio, y hace las dos cosas cada cierto tiempo.
Nunca bloquea una escritura local ni interrumpe para quejarse: una carpeta
inalcanzable se reintenta en silencio y se reporta en el panel de mantenimiento.

## Cómo se llama un documento, y qué pasa cuando se va

Un archivo de documento es `<device>-NNNN.md`. El prefijo de dispositivo es lo
que permite que dos máquinas creen documentos a la vez sin ponerse de acuerdo en
nada, así que solo la máquina dueña acuña sus propios números.

**Un número no se entrega dos veces.** Tomar el número más alto en disco y
sumarle uno no alcanza: borrar el último documento liberaría su nombre, y una
referencia `tisty:doc/…` que quedara apuntando ahí resolvería en silencio a lo
que tomara ese nombre después. Una marca local de máximo alcanzado, que solo
sube, es lo que lo evita. Es local porque solo esta máquina acuña estos nombres.

**Un cuerpo se rechaza por encima del techo del lector.** Escribir no tenía
límite mientras leer, exportar e imprimir se detenían todos en 500 KB, así que
pegar suficiente texto producía un documento que ya no se podía abrir, exportar
ni transportar — sin aviso hasta que era tarde. Ahora el rechazo ocurre donde
ocurre la escritura.

**Un borrado se ejecuta, no se infiere.** Borrar un documento nombra su archivo
en el registro, y cada máquina que lee ese evento elimina su propia copia — el
mismo trato que recibe un adjunto retirado. Lo que quedó de antes de que esto
existiera, o de una copia que se detuvo a medio camino, no se borra por
suposición: `tisty doctor` y el panel de mantenimiento **cuentan** los archivos
de documento en disco que el registro no conoce, y los dejan donde están.

## Sacar un documento de aquí

Un documento es un archivo Markdown, y el punto entero es que sobreviva sin
nosotros. Dos salidas, y la diferencia no es cosmética.

**Copiar como Markdown** entrega el texto al portapapeles exactamente como está
guardado, referencias incluidas. Rápido, y suficiente para prosa. Pero una
referencia a un adjunto dice `attachments/<shelf>/<file>`, y esos bytes viven en
el almacén: pega el texto en una página o en un ticket y las imágenes no están.

**Exportar como Markdown** escribe una carpeta — el documento junto a un
`attachments/` que contiene solo lo que ese documento nombra. **No se reescribe
ninguna referencia**, y ese es el punto: dentro del almacén un documento vive en
`docs/`, un nivel por debajo de los adjuntos que nombra, así que la ruta
relativa solo resuelve porque la resolvemos nosotros mismos desde la raíz de
datos. Pon el documento *junto* a sus adjuntos y esa misma ruta resuelve como
cualquier otro lector esperaría.

Por eso la exportación no necesita un segundo formato de referencia, y por eso
el almacén no necesita migrarse. La disposición hace el trabajo.

Lo que sigue sin sobrevivir el viaje es una referencia a **otro documento**
(`tisty:doc/…`), que fuera de Tisty no significa nada. Se queda tal como se
escribió, como un trozo de texto en vez de una ruta de archivo rota.

## Respaldar a mano

Un zip con `store/`, `docs/`, `originals/` y `attachments/`, nunca la
configuración: un `device_id` compartido pondría dos máquinas en un mismo
archivo.

Restaurar es **una fotografía**: vuelves a ese momento, y lo posterior se pierde
a propósito. La máquina **toma un identificador de dispositivo nuevo** para que
su directorio empiece vacío y nunca pueda encoger lo que otras máquinas ya
tienen.

No se toca nada tuyo hasta que el respaldo completo se desempacó al lado y se
volvió a leer, y el cambio mueve a un costado cada carpeta vieja antes de que
entre una sola nueva. Un zip que resulta estar corrupto, truncado, ser de otra
persona o no ser un respaldo no te cuesta nada — y media restauración es el
único resultado que vale menos que cualquiera de los dos enteros.

**Respaldar y sincronizar se excluyen mutuamente**, y los botones se
deshabilitan entre sí. La carpeta compartida ya guarda la historia de todas las
máquinas, así que una segunda instantánea al lado sería una verdad rival.
Restaurar es una decisión local con consecuencias globales, y las otras máquinas
nunca se enteran.

El límite honesto: con la sincronización obtienes **redundancia, no una forma de
volver en el tiempo**. Borra una tarea y el borrado viaja. Volver atrás para
todos tendría que ser un evento propio —un `store.rewind` que la proyección
respete—, que está anotado como idea y no construido.

## Dónde vive cada cosa

| | Ubicación | Se sincroniza |
|---|---|---|
| Eventos | `<data>/store/<device>/` | sí |
| Adjuntos | `<data>/attachments/` | sí |
| Documentos | `<data>/docs/` | sí, por tres huellas y ningún reloj |
| Libro de adjuntos | `<data>/attachments.jsonl` | **no** — local y se rehace a demanda |
| Huellas transportadas | `<data>/carried.json` | **no** — lo último que llevó esta máquina |
| Bases de fusión | `<data>/carried/` | **no** — el cuerpo que representa cada huella |
| Nombre más alto entregado | `<data>/docs/.spent-<device>` | **no** — para que un nombre no se reutilice |
| Antes de una conversión | `<data>/originals/` | **no**, pero está en un respaldo |
| Adjuntos retirados | `<data>/bin/` | **no** — treinta días de gracia |
| Configuración e identificador de dispositivo | `<config>/config.toml` | **no** |
| El programa mismo | `%LOCALAPPDATA%\Programs\Tisty` y compañía | **no** |
| Caché de lectura | `<cache>/read.db` | **no** |
| Último listado | `<cache>/selection.json` | **no** |

`<data>`, `<config>` y `<cache>` son los directorios propios de la plataforma.
`TISTY_DATA`, `TISTY_CONFIG` y `TISTY_CACHE` los sobrescriben, y existen para
los tests.

La **configuración** nunca se sincroniza, y eso es lo que importa: si dos
máquinas compartieran un identificador de dispositivo escribirían en el mismo
archivo y todas las garantías de arriba dejarían de valer. Es también la razón
por la que la configuración se queda fuera de un respaldo.

El identificador de dispositivo sí viaja, y tiene que hacerlo — es el nombre del
directorio y el campo `by` de cada evento, que es lo que distingue a los
escritores. Lo que nunca puede viajar es el archivo que dice *«esta máquina es
ese id»*. Ese archivo vive en el directorio de configuración local, nunca en uno
itinerante: un perfil de dominio de Windows copia `%APPDATA%` a un servidor de
la empresa al cerrar sesión, y se llevaría con él el identificador de
dispositivo y la carpeta `private/`.
