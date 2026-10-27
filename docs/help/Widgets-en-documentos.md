# Widgets

Un bloque de código cuyo lenguaje es `widget` se dibuja como una pequeña página: HTML, estilos y, si quieres, algo de script. Así un documento puede tener un panel, un contador o una calculadora chica, algo más que texto.

Cada forma, con sus ejemplos dibujados como los dibuja Tisty, está en la guía de Markdown de Tisty: [diagramas, fórmulas y widgets](https://markdown.tisty.app/es/diagramas-formulas-y-widgets).

````markdown
```widget title="Rina este mes"
<div class="grid">
  <div class="stat"><small>Paseos</small><b>42</b><em class="up">+6 frente a agosto</em></div>
  <div class="stat"><small>Kilómetros</small><b>61</b></div>
  <div class="stat"><small>Próxima vacuna</small><b>12 nov</b><span class="badge warn">pendiente</span></div>
</div>
```
````

> [!NOTE]
> GitHub no ejecuta widgets, así que esta página muestra el código. En Tisty, el bloque de arriba se dibuja como tres tarjetas de cifras con los colores de Tisty.

## Aislado

- Un widget corre en un marco que **no tiene red**: no puede descargar nada, cargar una imagen desde una dirección ni enviar a ninguna parte lo que muestra.
- No puede alcanzar la ventana que lo rodea ni tus otros documentos.
- Un enlace dentro de él abre tu navegador solo cuando haces clic.

## Clases listas

Trae los colores de Tisty y algunas clases, así que con HTML sencillo ya se ve bien: `grid`, `stat`, `card` y `badge`, entre otras (`badge warn`, `em.up`…). No hace falta escribir CSS para que quede presentable.

## Ver el código

El código del widget queda plegado bajo el dibujo. Ábrelo con **Ver código**.

## Cuando crece: adjunta un `.html`

Si el widget se vuelve grande, adjúntalo: un archivo `.html` adjunto a un documento se dibuja igual, igual de aislado, y desde su barra puedes volver a verlo como adjunto con **Ver como adjunto**.

## En una ventana aparte

**Abrir aparte**, en un widget o en una página `.html` adjunta, lo abre en una ventana propia, igual de aislado. Su barra lo ajusta a **Ajustar**, **Móvil**, **Tablet** o **Escritorio** (`Ctrl`/`⌘` + `1` a `4`), y **Más** tiene **Recargar**, **Ver código**, **Ver en el documento**, **Probar en oscuro** y **Zoom al 100 %**.
