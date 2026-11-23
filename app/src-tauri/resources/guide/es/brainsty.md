# Brainsty: todo unido

Esta hoja es una página de la guía. Cuenta Brainsty, el cerebro de Tisty: cómo unir un documento con otro y con sus tareas mientras escribes, y cómo ver todo lo que guardas como una sola red. Casi todo se prueba aquí mismo, sobre esta guía.

Lo que guardas ya estaba unido: un documento cita otro, una tarea nació de un acta, un libro tiene páginas. Brainsty hace que esas uniones se vean, se recorran y le sirvan también a tu agente.

## Enlazar un documento con otro

Escribe `[[` donde quieras el enlace y sigue con unas palabras del título. Tisty te ofrece hasta ocho documentos que calzan, sin mirar acentos ni mayúsculas, primero los que empiezan así. Elige uno con `Enter` y queda un enlace con su título, que apunta al documento y no a su nombre: si lo renombras, el enlace sigue llegando.

![El menú que abren dos corchetes en la ficha de Rina, antes de escribir nada más: los documentos que puedes enlazar, uno por línea](enlazar.png)

**Pruébalo:** al final de este párrafo escribe `[[rina` y elige **Rina**, la página de ejemplo de esta guía. Queda un enlace igual a este: [Rina](tisty:page).

Para llegar a una sección, sigue el título con `#`: `[[qué es#tareas` te ofrece las secciones de la portada de esta guía que llevan esa palabra, y el enlace abre la guía justo en **Las tareas**. Si esa sección deja de existir, el enlace abre el documento al comienzo.

Deja el puntero sobre un enlace a otro documento y aparece su vista previa: sus primeras líneas, o la sección a la que apunta. Para abrirlo mientras escribes, `Ctrl` + clic (`⌘` + clic en un Mac); un clic solo deja el cursor ahí.

## Quién lo enlaza

El panel junto a un documento tiene la fila **Enlazado desde**: cada documento y cada tarea que lo cita o lo menciona, con la sección donde lo hace. Un clic te lleva ahí.

**Pruébalo:** abre la página de Rina con el enlace de arriba y, en el panel de al lado, abre **Enlazado desde**: ahí está esta hoja, y la portada de la guía. Tisty arma esa lista en segundo plano, así que un enlace recién escrito aparece a los pocos segundos.

## Citar una tarea

Lo que sabes de un tema vive en su documento, y el trabajo que hiciste sobre él, en sus tareas. Cita la tarea en el documento y la encuentras donde la vas a buscar. En el menú `/`, **Una tarea** busca entre las abiertas y las archivadas, y deja su tarjeta sola en su párrafo.

![Las tarjetas de «Renovar el seguro del auto» y «Cambiar los neumáticos del auto» en el documento «Auto», cada una con su lista, Finanzas, y «1 de 3 pasos»; la segunda, además, con su fecha](tarjeta.png)

La tarjeta lee la tarea cada vez que se dibuja: su lista, sus fechas, cuántos pasos lleva hechos, cuántos esperan que confirmes si un agente dice que los hizo («1 de 3 pasos · 1 por confirmar») y, si se cerró, el día. Un clic la abre; su círculo la marca hecha; su contador despliega los pasos para marcarlos ahí mismo. Una tarea cerrada se lee, pero ya no cambia desde el documento.

**Pruébalo:** en una línea vacía de esta hoja escribe `/`, elige **Una tarea** y escribe una palabra de una tarea tuya. Si todavía no tienes ninguna, la captura rápida está a un `Ctrl` + `Shift` + `Space`.

## Crear una tarea sin salir del documento

En un acta o en un plan aparece un «hay que…». En el menú `/`, **Una tarea nueva** abre ahí mismo el campo de la captura rápida: escribe la tarea como la dirías y `Enter` la crea con su fecha, su lista, sus etiquetas y su prioridad, y deja su tarjeta en ese lugar.

**Pruébalo:** en una línea vacía, aquí abajo, escribe `/tarea`, `Enter`, y después `Pasear a Rina y Roger el sábado #rina`. Abre la tarea con un clic en su tarjeta: junto a su página, **Nació en** nombra esta hoja, y un clic te trae de vuelta. `Esc` en el campo la deja ir sin crear nada.

En la página de una tarea, **Una tarea nueva** crea una parte de esa tarea, en su misma lista.

## La red

**Brainsty**, en la barra lateral bajo las vistas de tareas, abre todo lo que guardas como una sola red. Se abre en el panel de contenido, como una vista más, con la barra lateral a la vista y **Brainsty** marcado en ella. Abre con la red entera a la vista; el documento o la tarea en que estabas queda como punto de partida de las flechas.

![La ventana de Tisty con la barra lateral a la izquierda y Brainsty marcado en ella; en el panel de contenido, la red alejada: arriba la cuenta, «1940 nodos · 2847 conexiones», cada documento y cada tarea del color de su carpeta o su lista, los documentos como cuadrados, las tareas como círculos y las etiquetas como puntos morados, más tenues, la red un poco girada como un globo, con sus viajeros como puntos de luz sobre las líneas, y abajo a la izquierda la Leyenda, plegada](red.png)

- **Gasta solo mientras la miras.** Cuando pasas a otra vista, se detiene todo lo que la red corría: el dibujo, el acomodo de los puntos y su trabajo en segundo plano.
- **Se lee de nuevo cada vez que entras**, así que lo que escribiste entretanto ya está. Arranca limpia: sin el punto que habías elegido ni los filtros de la vez anterior. Lo que cada equipo sí recuerda es cómo dejaste la leyenda, **Viajeros**, **Girar** y **Mostrar AGENTS**. Si mientras la miras llega algo nuevo, como un paso que marca tu agente o una ronda de sincronización, la red se queda como estaba: la misma vista, el mismo acomodo y tus **Viajeros** y **Girar** como los dejaste.
- **Leyenda**, abajo a la izquierda, dice qué significa cada cosa que ves. Viene plegada y un clic la abre. Cada equipo recuerda si la dejaste abierta.

**La red está viva.** Gira sola como un globo, una vuelta por minuto, y lo que queda atrás se ve más chico y tenue. Por sus líneas caminan **viajeros**: puntos de luz con una estela corta, que destellan al cruzar una etiqueta o un nodo muy conectado. Prefieren lo que alguien escribió: para seguir camino, una cita, una mención o una parte les pesa el doble que una línea a un archivo y diez veces más que una etiqueta. Cuántos son, y cada cuánto pasan de un nodo al siguiente, depende de cuántos nodos tiene tu red, contando también los que un filtro esconde:

| Nodos | Viajeros | Un nodo cada |
| --- | --- | --- |
| menos de 60 | 4 | 3 segundos |
| de 60 a 99 | 8 | 4 segundos |
| de 100 a 149 | 10 | 5 segundos |
| de 150 a 349 | 12 | 6 segundos |
| 350 o más | 14 | 7 segundos |

**De cerca, el globo se queda quieto.** Apenas acercas la vista, con la rueda, una búsqueda o las flechas, deja de girar, y no vuelve a hacerlo hasta que ves otra vez la red entera; los viajeros siguen caminando. También descansa mientras el puntero está sobre la red y mientras tienes un nodo elegido; en el espacio vacío alrededor sigue girando. **Un doble clic en un espacio vacío** te devuelve a la red entera, y el globo vuelve a girar. Para girarlo tú, arrastra con el botón derecho o con `Shift`; el arrastre normal sigue moviendo la vista.

Arriba, dentro de la **Leyenda** abierta, hay dos interruptores, **Viajeros** y **Girar**, cada uno con un punto que se enciende. Vienen encendidos, también si tu sistema pide reducir el movimiento. Sin giro, la red queda plana; con los dos apagados, queda quieta. Cada equipo recuerda cómo los dejaste.

En la **Leyenda** hay además cuatro partes. **Qué es** muestra la forma de cada clase que hay en tu red. **Color: su carpeta o lista** muestra las carpetas y listas de color, primero las más llenas; si son más de seis, **N más** muestra el resto, y lo que no tiene color toma el tono de su clase. Si encendiste **Mostrar AGENTS** en los filtros, ahí mismo aparece **AGENTS**, aparte de tus colores; con el puntero encima, te dice que son los archivos de tus agentes que Tisty mantiene al día. **Estado** muestra cómo se ve lo que sigue en curso, una tarea terminada y un documento archivado. **Líneas, de cerca** muestra cada tipo de línea que hay en tu red, y también **Parecidos** cuando Tisty cree que algo se parece. Esta tabla dice lo mismo:

| En la red | Es |
| --- | --- |
| Cuadrado | un documento o una página, del color de su carpeta |
| Cuadrado chico, coral tenue | un documento de **Agentes**, solo con **Mostrar AGENTS**: los archivos de tus agentes, que no tapan los tuyos |
| Círculo | una tarea, del color de su lista: viva si sigue en curso, tenue si se terminó |
| Punto morado, tenue | una etiqueta |
| Punto gris | una imagen o un archivo |
| Línea sólida y marcada | una cita, una tarjeta |
| Línea punteada | una mención, un enlace en una frase |
| Línea sólida | **Parte de**: una página y su libro, una parte y su tarea, el documento donde nació una tarea |
| Línea fina y tenue | una etiqueta, o lo que usa un archivo |
| Curva ámbar | un parecido: dos cosas que Tisty cree que se parecen porque comparten palabras poco comunes, una con un error de tipeo o una traducción, y que nadie unió |

Las carpetas y las listas no son puntos de la red: dan el color a lo suyo, tiñen su zona, tienen su lugar en la leyenda y la ficha de cada documento o tarea dice en cuál está. **Agentes** viene oculta: sus archivos no se dibujan, no salen en la búsqueda ni en la leyenda, y los viajeros no pasan por ellos, hasta que enciendes **Mostrar AGENTS** en los filtros. Ese botón aparece solo si tienes archivos ahí, cada equipo recuerda cómo lo dejaste y **Mostrar todo** no lo toca. Qué es esa carpeta lo cuenta [Agentes: las reglas de tus agentes](tisty:agents).

**Lo que escribiste manda.** Al acomodarse, la red junta lo que se cita, se menciona o es parte de lo mismo; una línea a un archivo acerca la mitad, y una etiqueta compartida, apenas. Los parecidos no mueven a nadie. Por eso las etiquetas se ven tenues: dicen de qué trata algo, no qué escribiste. Si citas la tarea «Vacuna anual de Rina» en la ficha de Rina, las dos quedan juntas, aunque la tarea viva en la lista **Casa** y la ficha en la carpeta **Mascotas**.

La red cambia con la distancia. De lejos, como abre o con la rueda hacia atrás, sus puntos forman un cerebro visto de lado, sin títulos, y cada clase tiene la forma de la tabla. A media distancia llevan título los nodos más conectados y, con **Girar** apagado, aparecen las zonas de color. De cerca aparecen todos los títulos, cada tipo de línea, las etiquetas y la miniatura de cada imagen, y cada nodo pasa a ser su ícono, del color de su carpeta o lista: una hoja para un documento, unas líneas de texto para una página, una casilla para una tarea y un tic cuando está hecha, un `#` para una etiqueta y un clip para un archivo. Con **Girar** encendido, como viene, las zonas no se ven, ni siquiera mientras el globo está quieto, y se ven menos títulos.

![«Ficha de Rina» elegida de cerca en Brainsty, en el panel de contenido junto a la barra lateral: a su alrededor, cada nodo dibujado como su ícono del color de su carpeta o lista (la hoja de un documento, la casilla de una tarea, el clip de un archivo) y la foto de Rina en miniatura; abajo a la izquierda, la Leyenda abierta; a la derecha, su ficha: «Documento · Salud · 7 conexiones», las filas Escrito por última vez, Creado y Tamaño con sus palabras, la etiqueta rina, Conexiones con 6 menciones y 1 etiqueta, Apuntan aquí con «Ir a la veterinaria por la vacuna de Rina» y «Paseos del sábado», el Enlace con Copiar enlace y el botón Abrir](red-cerca.png)

- **Un clic** en un punto lo resalta con sus vecinos y abre su ficha, a la derecha. La ficha no muestra el texto del documento: muestra sus datos, para que sepas qué es antes de abrirlo.
- **Abrir**, `Enter` o un doble clic abre el documento, la tarea o la etiqueta en esta misma ventana, en el lugar de la red. Para volver a la red, otro clic en **Brainsty**. Una imagen o un archivo se abre con el programa de tu sistema, y **Mostrar en la carpeta** te lleva a él.
- **Buscar en la red**, arriba, o `/`: escribe parte de un título y `Enter` centra la red en él.
- **Filtros**, el embudo junto al buscador: qué tipos mostrar, **Parecidos** para las curvas ámbar, **Mostrar AGENTS** si tienes archivos de tus agentes, el estado (**En curso**, **Terminadas** y **Archivadas**, que vale para tareas, documentos y páginas), una etiqueta y **Tocado**, para ver solo lo que cambió en los últimos días.
- **Con el teclado**: `Tab` recorre el buscador, los filtros y la red; las flechas caminan de un punto a su vecino, y la primera sale del documento o la tarea en que estabas; `Enter` abre y `Esc` suelta por capas: primero la búsqueda, después el punto, y el último te devuelve a donde estabas. Un `Esc` que escribes en un campo de la barra lateral se queda en ese campo. `Ctrl`/`⌘` + `Shift` + `M` muestra, junto a la leyenda, cuánto cuesta dibujar cada cuadro.

La ficha cambia según lo que eliges:

| Eliges | La ficha muestra |
| --- | --- |
| Un documento o una página | **Página de**, si es página de un libro; **Escrito por última vez**, **Creado**, **Tamaño** con sus palabras e **Idioma** |
| Una tarea | **Estado**, como «En curso · Por vencer»; **Día**, **Plazo**, **Parte de**, **Partes**, **Pasos**, como «1 de 3 hechos», o «1 de 3 hechos · 1 por confirmar» si un agente dice que hizo uno tuyo, y **Origen** |
| Una etiqueta | qué es y cuántas conexiones tiene |
| Una imagen o un archivo | su miniatura, o el ícono de su tipo; **Tipo**, **Tamaño** y **Modificado**; **Se usa en**, con cada documento o tarea que lo usa, a un clic, y los botones **Abrir** y **Mostrar en la carpeta** |

Debajo vienen sus **Etiquetas**; sus **Conexiones** por tipo, como «2 citas 3 menciones»; **Apuntan aquí**, con lo que cita o menciona lo que elegiste, y un clic en uno lo elige en la red; sus **Parecidos**, cada uno con su porqué, y, en un documento, una página o una tarea, su **Enlace**, con **Copiar enlace** y un **?** que dice para qué sirve: pegarlo en un documento para llegar ahí con un clic, o dárselo a tu agente para que lea eso mismo, siempre dentro de Tisty. El de un documento es el mismo que copias desde su panel con **Copiar enlace**, para pegarlo en un chat o dárselo a tu agente; lo cuenta [la portada de esta guía](tisty:guide), en **Su enlace**:

```text
tisty://doc/mascotas/rina?id=…
```

**Pruébalo:** abre **Brainsty** ahora. La red abre entera, girando despacio, con sus viajeros caminando. Deja el puntero sobre ella y el globo se detiene. Busca `brainsty`, `Enter`: la red se acerca a esta hoja, la elige con sus vecinos alrededor, cada uno con su ícono, y abre su ficha; en **Apuntan aquí** está la portada de la guía, que la cita con su tarjeta. Mientras la miras de cerca, el globo no se mueve. Cierra la ficha con su cruz y haz doble clic en un espacio vacío: vuelves a la red entera y el globo sigue girando. Abre la **Leyenda** y apaga **Girar**: la red queda plana. Después busca `uno de cada`, `Enter`, y **Abrir**: **Uno de cada cosa** se abre en el lugar de la red. Vuelve con otro clic en **Brainsty** y, esta vez, sal con `Esc`: el último te devuelve a **Uno de cada cosa**, donde estabas.

## Lo que lee tu agente

Tu agente lee la misma red, pero como un índice. Con una sola herramienta, `brain`, busca tareas y documentos por sus palabras, el Archivo junto a lo demás; trae la ficha de un documento, con sus secciones, quién lo cita y sus decisiones vigentes primero, sin leerlo entero; y ve un proyecto agrupado en bloqueado, en curso, esperándote y sin empezar. En la ficha, lo que Tisty cree que se parece le llega aparte, en `guessed`, con las palabras en común o la pareja de palabras que los une, como `pagar≈pay`, y con cómo las unió: por palabras, por una letra de diferencia o por traducción. Así no lo confunde con lo que alguien escribió. Una tarea que ocultaste o descartaste nunca le llega: si la enlazaste en un documento, ahí lee solo «a task out of sight», y cuando reescribe ese documento tu enlace vuelve intacto. Cuando nombra una tarea o un documento por su título exacto entre comillas, Tisty lo enlaza al guardar, como cuando lo haces tú. Lo cuenta, paso a paso, [la página de tu agente](tisty:agent).

## Lo que conviene saber

- **Todo queda en este equipo.** La red se calcula aquí y se rehace sola: no se sincroniza ni sale con lo que compartes o exportas. Cada equipo la dibuja de nuevo con tus documentos y tareas.
- **Lo que apartas no aparece.** Una tarea oculta o descartada, las páginas que cuelgan de ella y tu Diario con sus páginas no son puntos de la red: nada apunta a ellos, ni una línea, ni un parecido, ni un viajero, ni **Apuntan aquí**, y tampoco le llegan a tu agente. **Archivadas** sigue mostrando los documentos que archivaste.
- **Fuera de Tisty**, un enlace a otro documento y la tarjeta de una tarea quedan solo como sus palabras: en Markdown estándar, en Word, en el PDF y en Folio.
- **Hasta 12.000 puntos.** Más allá, la red guarda lo último que tocaste, sus vecinos, tus documentos y las tareas en curso, y te dice arriba cuánto quedó fuera.
- **Cada línea recta la escribiste tú; la curva ámbar, no.** Tisty une como parecidos lo que nadie unió, con tres pistas y en este orden:
  - **Palabras en común:** al menos dos poco comunes, como *presupuesto* y *techo*.
  - **Un error de tipeo:** «La imrpesora nueva no imprime en color» con «Instalar la impresora nueva».
  - **Una traducción:** una tarea en español y su gemela en inglés, como «Pagar la cuenta del agua» con «Pay the water bill».

  La curva aparece de cerca o junto al punto que eliges, y la ficha dice por qué, con lo que escribió cada lado: «Tisty cree que se parecen: comparten *pagar ≈ pay* y *agua ≈ water*». No hay IA en esto: la traducción sale de una lista fija de palabras español–inglés, hecha por lingüistas, y todo corre en tu equipo. Solo lee la prosa que escribiste: el código, el HTML, las direcciones, las rutas y los nombres de archivo no cuentan, ni las notas de avance de una tarea, ni las palabras que solo dicen cómo se arma un texto, como *negrita*, *tabla* o *enlace*. Los archivos de **Agentes**, las tareas ocultas o descartadas y tu Diario nunca pasan por ahí. Lo cuenta [la red de Brainsty](https://www.tisty.app/es/ayuda/red-de-brainsty/), en la ayuda.

Cada detalle, con un ejemplo de principio a fin, está en la ayuda: [enlazar documentos](https://www.tisty.app/es/ayuda/enlazar-documentos/), [tareas en documentos](https://www.tisty.app/es/ayuda/tareas-en-documentos/) y [la red de Brainsty](https://www.tisty.app/es/ayuda/red-de-brainsty/).
