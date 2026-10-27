# Instalación

Tisty funciona en **Windows** y **macOS**. Hoy no hay versión para Linux ni app móvil.

## Windows

**Microsoft Store**: [Tisty en la Microsoft Store](https://apps.microsoft.com/detail/9PGVWXD8X93N). La Store lo mantiene actualizado, y desde *Acerca de* puedes pedirle una actualización en el momento.

## macOS

**Homebrew**: el tap se añade y se marca como confiable una vez; Homebrew carga un tap que no es suyo solo cuando confías en él. Después Tisty se actualiza solo:

```console
$ brew tap rgdevment/tap
$ brew trust rgdevment/tap
$ brew install --cask tisty
```

La fórmula `tisty-cli`, que instalaba solo la línea de comandos, ya no se publica desde la 1.31: mira [[Línea de comandos|Línea-de-comandos]].

## Descarga directa

[GitHub Releases](https://github.com/rgdevment/Tisty/releases/latest) tiene los instaladores firmados:

- Windows: `.exe`
- macOS: `.dmg`, uno por chip: `aarch64` para Apple Silicon y `x86_64` para Intel. Menú Apple › *Acerca de esta Mac* te dice cuál es el tuyo. Homebrew lo elige solo.

## ¿Las descargas están firmadas?

Sí. Las imágenes de macOS están firmadas y notarizadas por Apple, y el instalador de Windows también está firmado. Además, Tisty se niega a instalar una actualización que no esté firmada con una clave compilada dentro de la copia que estás usando. Qué licencia aplica a cada versión se explica en [DISTRIBUTION.md](https://github.com/rgdevment/Tisty/blob/main/docs/DISTRIBUTION.md).

## La primera vez que se abre

Tisty te pregunta dos cosas:

1. **En qué idioma**: inglés o español. La ventana, la guía y el lenguaje natural lo siguen, y puedes cambiarlo cuando quieras en Configuración.
2. **Dónde dejar tus copias**: una carpeta para [[sincronizar|Sincronizar-y-respaldar]], o ninguna si usas un solo equipo.

Después **te escribe una guía y la abre**. La guía es un documento normal en tu propio almacén, en una carpeta propia (tuya para leerla, editarla o tirarla), y te muestra desde dentro lo que cabe en un documento. Puedes volver a ella desde Configuración.

Todo lo demás tiene una respuesta obvia que puedes cambiar después: Tisty arranca con el equipo y espera sin estorbar.

## Configuración

Configuración ocupa la barra lateral con sus secciones:

- **General**: cómo se comporta Tisty contigo: idioma, aspecto, arranque, la captura rápida, avisos y la bienvenida.
- **Sincronizar**: dónde van tus copias, tus equipos y dónde se quedan los adjuntos grandes.
- **Seguridad**: tu frase de 24 palabras y [cifrar tu nube](Sincronizar-y-respaldar#cifrar-tu-nube).
- **Tus datos**: dónde viven, los respaldos, los documentos de otro Tisty y cómo irte.
- **Asistentes**: la [[puerta MCP|Asistentes-y-MCP]].
- **Mantenimiento**: la revisión del almacén, los adjuntos sueltos, liberar espacio y reportar un problema.
- **Acerca de**: la versión, las actualizaciones y las versiones candidatas.

## Actualizaciones

Tisty revisa al abrirse, y una vez al día después, si hay una versión más nueva; no envía nada para preguntar. Si la hay, aparece una línea en *Acerca de*:

- una copia de la Microsoft Store la actualiza la Store;
- una copia dueña de su propia carpeta recibe un botón.

No se descarga nada hasta que lo pulsas. Las versiones candidatas se activan en *Acerca de*. Más detalle en [[Privacidad]].

## Dónde viven tus datos

- **Windows:** `.tisty` en tu carpeta de usuario, oculta y cerrada para las demás cuentas del equipo. Activa **Elementos ocultos** en el menú **Ver** del Explorador de archivos para verla.
- **macOS:** `Library/Application Support/tisty` y `Library/Caches/tisty`, en tu carpeta personal.

**Configuración › Tus datos** muestra la ruta, y *Mostrar carpeta* la abre.

## Desinstalar

Desinstalar quita el programa y **deja tus tareas y documentos donde están**, así que al reinstalar los encuentras como los dejaste. Para irte del todo, borra esa carpeta tú mismo.
