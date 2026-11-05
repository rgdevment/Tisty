# Widgets

Un bloque de código cuyo lenguaje es `widget` se dibuja como una vista fija: HTML y CSS, y nada más. Así un documento puede tener un panel de cifras, un resumen o una tabla chica con los colores de Tisty, algo más que texto, que se queda como lo escribiste.

Cada forma, con sus ejemplos dibujados como los dibuja Tisty, está en la guía de Markdown de Tisty: [diagramas, fórmulas y widgets](https://markdown.tisty.app/es/diagramas-formulas-y-widgets#widgets).

````markdown
```widget title="Paseos"
<div class="stat"><small>Este mes</small><b>42</b></div>
```
````

> [!NOTE]
> GitHub no dibuja widgets, así que esta página muestra el código. En Tisty, el bloque de arriba se dibuja como una tarjeta con su cifra.

## Fijo, e igual en todas partes

- Un widget **no ejecuta nada, nunca**: ni script, ni `onclick`, ni botones o formularios.
- **No nombra nada fuera de sí mismo**: ni enlaces, ni imágenes, fuentes o estilos desde una dirección, ni marcos, sonido o video. Una imagen va dentro de él, como `data:image/…`.
- Se ve igual donde se lea el documento.

## Cuando un bloque no es un widget

Un bloque con cualquier otra cosa no es un widget. Se muestra como su código, con lo que está mal, y no se dibuja nada de él. Un agente que intente escribirlo recibe el motivo, y no se escribe nada.

## Clases listas

Trae los colores de Tisty y clases listas, así que con HTML sencillo ya se ve bien, sin escribir CSS; cada clase, con su ejemplo, está en la [guía de Markdown](https://markdown.tisty.app/es/diagramas-formulas-y-widgets#widgets).

## Ver el código

El código del widget queda plegado bajo el dibujo. Ábrelo con **Ver código**.

## Cuando tiene que ejecutarse: adjunta un `.html`

Un widget nunca ejecuta nada. Si algo tiene que ejecutarse, adjunta un archivo `.html` al documento: Tisty lo dibuja en vivo en su propio marco, sin acceso a la ventana ni a tus otros documentos, y desde su barra puedes volver a verlo como adjunto con **Ver como adjunto**. Solo Tisty lo ejecuta.

## En una ventana aparte

**Abrir aparte**, en un widget o en una página `.html` adjunta, lo abre en una ventana propia. Su barra lo ajusta a **Ajustar**, **Móvil**, **Tablet** o **Escritorio** (`Ctrl`/`⌘` + `1` a `4`), y **Más** tiene **Recargar**, **Ver código**, **Ver en el documento**, **Probar en oscuro** y **Zoom al 100 %**.
