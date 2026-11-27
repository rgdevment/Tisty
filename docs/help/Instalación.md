# Instalación

Tisty funciona en **Windows** y **macOS**. Hoy no hay versión para Linux ni app móvil.

## Windows

**Microsoft Store**: [Tisty en la Microsoft Store](https://apps.microsoft.com/detail/9PGVWXD8X93N). La Store lo mantiene actualizado, y desde **Configuración › Ayuda y acerca de** puedes pedirle una actualización en el momento.

**winget**: el gestor de paquetes de Windows toma el instalador firmado de GitHub Releases y después se aparta, porque Tisty se actualiza solo:

```console
> winget install rgdevment.Tisty
```

También sirven `winget upgrade rgdevment.Tisty` y `winget uninstall rgdevment.Tisty`, que lo quita como cualquier otra desinstalación.

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

Tisty te pregunta hasta tres cosas:

1. **En qué idioma**: inglés o español. La ventana, la guía y el lenguaje natural lo siguen, y puedes cambiarlo cuando quieras en **Configuración › Aspecto y ventana**.
2. **Dónde dejar tus copias**: una carpeta para [[sincronizar|Sincronizar-y-respaldar]], o ninguna si usas un solo equipo.
3. **¿Cómo quieres que Tisty te llame?**: un alias opcional para firmar lo que compartes; **Ahora no** lo salta, y no se pregunta cuando el Tisty al que te unes ya tiene uno.

Después **te escribe una guía y la abre**, salvo que el lugar que elegiste ya guarde tu Tisty: entonces trae eso en su lugar. La guía es un documento normal en tu propio almacén, en una carpeta propia (tuya para leerla, editarla o tirarla), y te muestra desde dentro lo que cabe en un documento. Puedes volver a ella desde **Configuración › Ayuda y acerca de**, con **Abrir la guía**.

Todo lo demás tiene una respuesta obvia que puedes cambiar después: Tisty arranca con el equipo y espera sin estorbar.

## Configuración

El engranaje al pie de la barra lateral abre Configuración en **Resumen**, que dice cómo está todo y qué pide algo de ti. **Buscar un ajuste** encuentra cualquier ajuste por su nombre, y las once páginas van en cuatro grupos:

- **Este equipo**: **Aspecto y ventana** (idioma, aspecto, arranque, qué hace al cerrar la ventana y las carpetas ocultas), **Tareas y avisos** (los atajos de captura, los avisos y tu calendario) y **Documentos y firma** (tu alias, tu nombre en los PDF y cuánto guarda el historial de cada documento).
- **Tus datos**: **Sincronización** (dónde van tus copias), **Seguridad** (tu frase de 24 palabras, tus equipos y [cifrar tu nube](Sincronizar-y-respaldar#cifrar-tu-nube)), **Copias** (los respaldos, los documentos de otro Tisty, restaurar y cómo irte) y **Almacenamiento** (dónde viven tus archivos, cuánto ocupan, los adjuntos grandes y liberar espacio).
- **Conexiones**: **Agentes**, con la [[puerta MCP|Agentes-y-MCP]], los agentes conectados, la carpeta Agentes y las skills de Tisty.
- **Tisty**: **Salud** (la revisión del almacén, el índice rápido y reportar un problema) y **Ayuda y acerca de**: la versión, las actualizaciones, las actualizaciones automáticas y las versiones candidatas. Al pie, **Avisos de terceros** nombra cada pieza que Tisty lleva dentro, con su versión y su licencia; ahí están también los datos del diccionario español–inglés de Brainsty, con sus autores y la licencia de WordNet completa. **Textos de las licencias** trae el texto completo de cada licencia del núcleo.

Todo, página por página y con ejemplos, en [[Configuración]].

## Actualizaciones

Tisty revisa al abrirse, y una vez al día después, si hay una versión más nueva; no envía nada para preguntar. Una copia de la Microsoft Store le pregunta a la Store al abrirse y cada seis horas. Si la hay, aparece una línea en **Configuración › Ayuda y acerca de**, y **Resumen** la pone en **Por atender**:

- una copia de la Microsoft Store la actualiza la Store, y la línea trae un botón **Actualizar** cuando la Store se la ofrece a este equipo;
- una copia dueña de su propia carpeta recibe un botón.

Tisty no descarga nada hasta que lo pulsas, o hasta que activas **Instalar actualizaciones automáticamente** en **Ayuda y acerca de**: entonces las próximas versiones se descargan solas y se instalan al salir o al abrir Tisty, nunca mientras trabajas. Viene apagado, Tisty te lo ofrece justo después de actualizarse, y no está en la copia de la Store. Si una actualización falla, **Descargar desde la web** abre la página de versiones. Las versiones candidatas se activan con **Recibir versiones candidatas**, en **Ayuda y acerca de**; en la copia de la Microsoft Store esa fila dice **No aplica aquí**, porque la Store mantiene esa copia al día y no ofrece candidatas. Si buscar o instalar una versión falla, el error sale en esa misma fila. Más detalle en [[Privacidad]].

En Windows, cuando la Store o un instalador cierra Tisty para reemplazarlo, Windows lo vuelve a abrir solo: una ventana a la vista vuelve a la vista, y una oculta en la bandeja vuelve oculta. Lo que acababas de escribir en un documento o en la página de una tarea se conserva, y lo mismo cuando Windows se apaga. Un fallo, un cuelgue o un reinicio no lo reabren, ni tampoco una copia que llevaba menos de un minuto abierta.

## Dónde viven tus datos

- **Windows:** `.tisty` en tu carpeta de usuario, oculta y cerrada para las demás cuentas del equipo. Activa **Elementos ocultos** en el menú **Ver** del Explorador de archivos para verla.
- **macOS:** `Library/Application Support/tisty` y `Library/Caches/tisty`, en tu carpeta personal.

**Configuración › Almacenamiento** muestra la ruta, y **Mostrar carpeta** la abre.

## Desinstalar

Desinstalar quita el programa y **deja tus tareas y documentos donde están**, así que al reinstalar los encuentras como los dejaste. Para irte del todo, borra esa carpeta tú mismo.

Si usas Agentes, los hooks o las skills de Tisty, lo que Tisty escribió en las carpetas de tus agentes se queda ahí. Si no lo quieres, quítalo antes de desinstalar, desde **Configuración › Agentes**: las skills con **Apagar en este equipo**, los hooks con su interruptor bajo cada agente, y los archivos de Agentes al eliminar sus documentos con **Quitar también**, o con **Revisar** en la tarjeta Agentes los que dejaste. Una copia que cambiaste se queda hasta que la quitas tú. Lo cuentan [Hooks y skills](Hooks-y-skills) y [la carpeta Agentes](Carpeta-AGENTS#eliminar-una-regla-o-una-skill).
