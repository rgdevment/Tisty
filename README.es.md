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

## Cómo se ve

```console
$ tisty "arreglar los timeouts intermitentes al guardar" --prioridad 1

  ✓ arreglar los timeouts intermitentes al guardar
    !1
    z8k4qm
```

```console
$ tisty ls

  hoy                                                   3 tareas

    1  ○ arreglar los timeouts intermitentes al guardar
       !1 · hoy
    2  ○ validar notificaciones de pagos
       !3 · hoy
    3  ○ actualizar las dependencias de CI

$ tisty done 2
  ✓ validar notificaciones de pagos
```

Para referirte a una tarea puedes usar el número de la última lista, un trozo
del título (`tisty done pagos`) o su identificador. Un ULID es para los scripts,
no para los dedos.

## Pensado para quien vive en una terminal

- **`--json` en todo comando de lectura.** Sin eso, nada de esto sería
  scriptable.
- **stdout es datos, stderr es conversación.** Un pipe nunca arrastra
  decoración, y sin terminal interactivo no hay color ni secuencias de escape.
- **Códigos de salida con significado:** `0` correcto · `1` error · `2` uso
  incorrecto · `4` no encontrado.
- **Todo lo que hará la interfaz gráfica se puede hacer desde la terminal.** Lo
  que cambia es cuántas teclas cuesta, no qué es posible.

## Tus datos

Un directorio con archivos de texto:

```
~/Documents/Tisty/
└── store/
    └── dev_a3f1/
        ├── 000001.jsonl      segmento cerrado, ya no cambia
        └── active.jsonl      una línea por evento
```

```jsonl
{"v":1,"ts":"2026-08-05T08:27:49Z","by":"dev_a3f1","op":"task.add","id":"01KZ8G…","d":{"title":"arreglar los timeouts intermitentes al guardar","priority":1,"tags":["backend","db"]}}
```

Un registro de eventos que solo crece. De ahí salen gratis el historial y el
deshacer, y también una sincronización sin conflictos cuando llegue: **cada
máquina escribe únicamente en su propio directorio**, así que fusionar dos
historias es concatenarlas.

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
| ⬜ | Lenguaje natural: `tisty "desplegar la API mañana a las 10"` |
| ⬜ | Bitácora, pasos y listas desde la línea de comandos |
| ⬜ | Sincronización por Git o por la carpeta de tu nube |
| ⬜ | Interfaz gráfica (Tauri) |
| ⬜ | Documentos en Markdown |

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

Ver también [SECURITY.md](SECURITY.md) y [PRIVACY.md](PRIVACY.md) — el resumen
de esta última es que no se recoge nada y no se envía a ninguna parte.
