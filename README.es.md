# Tisty

[English](README.md) · **Español**

Un gestor de tareas local, privado y minimalista para macOS, Windows y Linux.

Sin cuenta, sin telemetría, sin servidor. Tus tareas son archivos de texto plano
en tu propio disco, legibles con `cat` y buscables con `grep`. Si Tisty
desaparece mañana, tus datos siguen ahí.

> **En desarrollo temprano.** El núcleo funciona y la línea de comandos ya es
> usable, pero el lenguaje natural, la sincronización y la interfaz gráfica aún
> no están. Esto no es un lanzamiento.

---

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

Esto es una herramienta que construyo porque quiero usarla. Es software
deliberadamente personal — sin equipos, sin colaboración, sin plan de
crecimiento. Si a ti también te sirve, mejor.

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

Escribes una frase. Tisty le saca la fecha, deja la frase legible y te dice qué
entendió antes de guardar nada.

```console
$ tisty "entregar el informe mañana a las 10 en la oficina"
  ✓ entregar el informe en la oficina
    mañana 10:00

$ tisty "renovar el dominio antes del 30 de agosto"
  ✓ renovar el dominio
    límite dom 30 ago            # «antes del» vence, no se planifica

$ tisty "revisar el deploy #backend !alto"
  ✓ revisar el deploy
    alto · #backend

$ tisty "reunión el lunes 15"       →  reunión        · sáb 15 ago
$ tisty "revisar la próxima semana" →  revisar        · dom 16 ago
$ tisty "tomar café a las 3"        →  tomar café     · hoy 15:00
```

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

Cada frase de arriba es un test. El parser lleva detrás un contrato de 190 casos
en español e inglés que fija qué debe leer y, con la misma frecuencia, qué debe
dejar en paz.

## Cómo se ve

```console
$ tisty "corregir los timeouts intermitentes al guardar" --prioridad 1

  ✓ corregir los timeouts intermitentes al guardar
    !1
    ae0bvq
```

```console
$ tisty ls todas

  todas                                                 3 tareas

    1  ○ validar las notificaciones de pago
       mañana
    2  ○ corregir los timeouts intermitentes al guardar
       !1
    3  ○ actualizar las dependencias de CI

$ tisty done 3
  ✓ actualizar las dependencias de CI
```

Para referirte a una tarea puedes usar el número de la última lista, un trozo
del título (`tisty done pago`) o su identificador. Un ULID es para los scripts,
no para los dedos.

Y luego la parte que solo se cobra más tarde. Lo que anotas mientras trabajas
queda pegado a la tarea, y completarla no lo pone fuera de alcance:

```console
$ tisty log 1 "el presupuesto de reintentos se agotó antes de que el pool se rellenara"
$ tisty done 1

$ tisty search "presupuesto de reintentos"

  «presupuesto de reintentos»                          1 tarea

    1  ✓ validar las notificaciones de pago
       mañana · ✎1
```

La búsqueda lee el título, la descripción, la bitácora, los pasos y las
etiquetas — tanto lo pendiente como el archivo.

Los filtros se combinan, y las mismas palabras sirven para escribir una tarea y
para buscarla:

```console
$ tisty ls semana #seguridad

  semana #seguridad                                       1 tarea

    1  ○ rotar las llaves de firma
       !1 · mañana · @plataforma · #seguridad
```

`hoy` · `mañana` · `semana` · `vencidas` · `inbox` · `archivo` · `todas` ·
`@lista` · `#etiqueta` · `!1`, o cualquier fecha que sepas escribir —
`tisty ls viernes`. `tisty ls` a secas significa hoy; nombrar cualquier filtro
amplía a todo lo abierto, porque pedir `#seguridad` y recibir solo lo de hoy
escondería justo las tareas por las que preguntabas.

## Pensado para quien vive en una terminal

- **`--json` en todo comando de lectura.** Sin eso, nada de esto sería
  scriptable.
- **stdout es datos, stderr es conversación.** Un pipe nunca arrastra
  decoración, y sin terminal interactivo no hay color ni secuencias de escape.
- **Códigos de salida con significado:** `0` correcto · `1` error · `2` uso
  incorrecto · `4` no encontrado.
- **Todo lo que hará la interfaz gráfica se puede hacer desde la terminal.** Lo
  que cambia es cuántas teclas cuesta, no qué es posible.
- **`tisty export` te devuelve los datos**, como JSON o como un documento
  Markdown que se lee sin Tisty. Acepta los mismos filtros que `ls`, así que
  puedes exportar una lista, una etiqueta o el archivo entero.

## Tus datos

Un directorio con archivos de texto:

```
<datos de aplicación>/tisty/data/
└── store/
    └── dev_a3f1/
        ├── 000001.jsonl      segmento cerrado, ya no cambia
        └── active.jsonl      una línea por evento
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

Tu configuración nunca viaja con ellos. El identificador de dispositivo vive en
el fichero de configuración precisamente para quedarse en esta máquina: si
viajara, dos equipos lo compartirían, escribirían en el mismo fichero y la
garantía de abajo se vendría abajo.

Un registro de eventos que solo crece. De ahí salen gratis el historial y el
deshacer, y también una sincronización sin conflictos cuando llegue: **cada
máquina escribe únicamente en su propio directorio**, así que fusionar dos
historias es concatenarlas.

Eso es una sola lista, no una por máquina. Cada dispositivo lee todos los
directorios y los reproduce en orden; solo escribe en el suyo. Por eso una
carpeta sincronizada nunca produce uno de esos `fichero (copia en conflicto)`:
dos escritores jamás tocan el mismo fichero.

## Dos máquinas

Apunta Tisty a una carpeta que tus dos equipos ya alcancen —la que mantiene al
día tu cliente de Google Drive, OneDrive o iCloud; un NAS montado; un disco
externo que enchufas los viernes— y el resto lo hace solo: baja al abrir la ventana, sube al
rato de que cambies algo, y ambas cada cierto tiempo. Nunca bloquea una
escritura ni te interrumpe; si la carpeta no está, lo reintenta en silencio.

No hay fusión ni «¿cuál es más nueva?», porque el directorio de un dispositivo
tiene un único escritor: subiendo manda la tuya, bajando manda la suya.

Quién mantenga esa carpeta no es asunto de Tisty, y no hay nada nuestro en medio
—ni cuenta, ni servidor, ni proceso residente—. Si no sincronizas nada, Tisty no
abre un socket en su vida.

**O respalda a mano.** Un zip, guardado donde quieras. Restaurar es una foto:
vuelves a ese momento y lo posterior se pierde a propósito. Las dos cosas se
excluyen — la carpeta compartida ya contiene la historia de todas las máquinas,
así que una foto al lado solo sería una verdad rival.

## Instalación

Todavía no hay binarios publicados. Con Rust 1.97 o superior:

```sh
git clone https://github.com/rgdevment/Tisty
cd Tisty
cargo install --path crates/tisty-cli
```

## En qué punto está

| | |
|---|---|
| ✅ | Núcleo: modelo, registro de eventos, almacenamiento, proyección |
| ✅ | CLI: capturar, listar, completar, ver detalle |
| ✅ | Lenguaje natural: `tisty "desplegar la API mañana a las 10"` |
| ✅ | Bitácora, pasos, listas, etiquetas, búsqueda y deshacer desde la terminal |
| ✅ | Filtros compuestos de `ls`, `config`, `export`, `--json`, códigos de salida |
| ✅ | Interfaz gráfica (Tauri): tres columnas, arrastrar y soltar, adjuntos |
| ✅ | Markdown en descripciones y bitácora |
| ✅ | Sincronización por una carpeta que alcancen los dos equipos, y respaldo a mano |
| ✅ | Bandeja y barra de menú, con captura rápida por atajo global |
| ◐ | Uso diario, que es lo que saca los fallos que los tests no |
| ⬜ | Binarios empaquetados para Windows y macOS |

## Qué nunca va a hacer

Tan importante como la lista anterior. Permanentemente fuera del alcance:
colaboración en tiempo real, tableros kanban, diagramas de Gantt, control de
tiempo, métricas de productividad, bases de datos con propiedades tipadas y
fórmulas, y IA en el camino crítico de cualquier operación.

El intérprete de lenguaje natural será determinista y local. Nada se envía nunca
a ningún modelo.

## Contribuir

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
