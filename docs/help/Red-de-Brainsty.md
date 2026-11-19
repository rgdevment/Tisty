# La red de Brainsty

Llevaba años guardando cosas en Tisty y no tenía forma de verlas juntas. Sabía que «Rina» citaba la tarea de su vacuna y que el acta del condominio había creado tres tareas, pero cada relación vivía dentro de su archivo. Brainsty dibuja todo lo que guardas, con lo que lo une, como una sola red: de lejos, un mapa de tus proyectos; de cerca, cada documento y cada tarea con sus vecinos.

Esta página cuenta cómo abrirla, qué significa cada forma y cada línea, cómo moverte por ella y cómo filtrarla. Las uniones que dibuja las haces tú al escribir: [[Enlazar documentos|Enlazar-documentos]] y [[Tareas en documentos|Tareas-en-documentos]]. Tu agente lee la misma red como un índice: [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

![La ventana de Brainsty, con su barra de título del sistema, junto a la ventana principal: arriba dice «Brainsty» y la cuenta, «589 nodos · 1252 conexiones», y abajo la red alejada, con cada carpeta y cada lista de su color, los documentos como cuadrados, las tareas como círculos y las etiquetas como puntos morados](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/es/red.png)

## Cuándo te sirve

- **Volver a un proyecto después de semanas.** Abres la red, buscas el proyecto y ves de un vistazo sus documentos, sus tareas abiertas y las que ya cerraste.
- **Encontrar lo que quedó suelto.** Un documento sin líneas es uno que nada nombra: quizás le falta un enlace, o ya no sirve.
- **Ver qué toca una etiqueta.** Filtra por `#rina` y quedan sus vacunas, sus paseos, el veterinario y las tareas de este mes.
- **Ver lo último que tocaste.** El filtro **Tocado** con **Últimos 7 días** deja solo lo que se movió esta semana.

## Abrirla y cerrarla

La red vive en su propia ventana, aparte de la principal. Así gasta solo mientras está abierta, y la ventana principal sigue libre al lado: puedes escribir mientras los nodos se acomodan.

- **Brainsty**, en la barra lateral bajo las vistas de tareas, la abre. Con la barra plegada, es el icono de la red en el riel.
- Arranca centrada en el documento o la tarea en que estabas; si no estabas en ninguno, en lo último que escribiste.
- Lleva la barra de título del sistema, con el nombre «Brainsty», y se mueve, se maximiza y se minimiza como cualquier otra ventana. Arriba, dentro, la cuenta: «589 nodos · 1252 conexiones».
- La primera vez los nodos se acomodan unos segundos, como si se soltaran, y se detienen solos en cuatro segundos como mucho; una red de unos 5600 nodos queda quieta en dos o tres.
- **Nunca hay más de una.** Con la red ya abierta, otro clic en **Brainsty** la trae al frente tal como la dejaste: el mismo zoom, el mismo nodo elegido, los mismos filtros. No se vuelve a centrar en lo que miras ahora; para eso, ciérrala y ábrela de nuevo.
- **Se cierra** con el botón de cerrar de su barra o con el último `Esc` (mira [Con el teclado](#con-el-teclado)). Al cerrarla no queda nada corriendo, ni el dibujo ni el acomodo, y la ventana principal sigue como estaba.

### Dónde aparece

- La primera vez se abre centrada, de 1280 × 820. Se puede achicar hasta 640 × 480, no menos.
- Después recuerda su tamaño, su lugar y si estaba maximizada. Lo recuerda **cada equipo por separado**: en el portátil puede vivir a la derecha y en el equipo de la oficina, maximizada en la segunda pantalla. No viaja con la sincronización.
- Si su lugar quedó en una pantalla que ya no está conectada, se abre centrada en la que tienes.
- Minimizarla no cambia el lugar que recuerda.
- **Sigue a Tisty**: el tema claro u oscuro y el idioma valen también aquí, aunque los cambies en **Configuración** con la red abierta. En oscuro, la red toma los colores oscuros.

## Qué es cada cosa

| En la red | Es | Cómo se ve |
| :-- | :-- | :-- |
| Cuadrado | un documento o una página de un libro | del color de su carpeta; una página, más pequeña junto a su documento |
| Círculo | una tarea | del color de su lista; en curso, viva; terminada, más tenue; descartada o archivada, tenue y pequeña |
| Punto morado | una etiqueta | unido a todo lo que la lleva |
| Punto gris | una imagen o un archivo | unido a lo que lo usa; de cerca, con su miniatura |

Las listas y las carpetas no son nodos: son el color y la zona donde se juntan sus cosas. Si fueran nodos, todo quedaría unido a «Casa» o a la bandeja y la red no diría nada.

| Línea | Une | Cuándo existe |
| :-- | :-- | :-- |
| Sólida | una cita | una tarjeta de otro documento o de una tarea, sola en su párrafo |
| Punteada | una mención | un enlace dentro de una frase a otro documento o a una tarea |
| Fina | la estructura | una página y su libro, una parte y su tarea, una tarea y el documento donde nació, una idea y su documento |
| Fina y tenue | una etiqueta | lo que lleva `#etiqueta` y su punto |
| Fina, hacia un archivo | un uso | lo que muestra una imagen o adjunta un archivo |

Cada línea de la red está escrita en alguna parte, y se puede explicar. Lo que Tisty anota por parecido no se dibuja como línea: lo cuenta [Parecidos](#parecidos).

## De lejos y de cerca

La red cambia lo que muestra según la distancia, y cada detalle aparece de a poco al acercarte:

- **De lejos**, con la rueda hacia atrás: los nodos y sus uniones forman la silueta de un cerebro visto de lado, sin títulos y con líneas tenues. Sirve para ver dónde hay mucho y dónde hay poco.
- **A media distancia**, como se abre: cada carpeta y cada lista tiñen su zona, y llevan título solo los nodos más conectados.
- **De cerca**: todos los títulos, cada tipo de línea, las etiquetas como `#etiqueta` y, junto a cada imagen, su miniatura.

![La ventana de Brainsty con la misma red de lejos: los nodos forman la silueta de un cerebro visto de lado, sin títulos y con líneas tenues](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/es-red-lejos.png)

Con la rueda acercas y alejas; arrastrando te mueves. Al primer toque la red deja de seguir el punto donde arrancó.

## Elegir, ver y abrir

- **Un clic en un nodo** lo resalta junto con sus vecinos, atenúa el resto y abre su vista previa, a la derecha. Arriba dice qué es, dónde vive y cuántas conexiones tiene: «Documento · Mascotas · 6 conexiones». Debajo:
  - de un documento o una página, sus primeras líneas, con sus fotos;
  - de una tarea, su tarjeta, la misma que dibuja un documento que la cita: su lista, su fecha o su estado y cuántos pasos lleva. Aquí se lee, no se cambia: un clic en la tarjeta la abre en la ventana principal, y ahí marcas sus pasos;
  - de una etiqueta, su nombre y cuántas conexiones tiene.
- **Abrir**, en la vista previa, o **un doble clic** en el nodo, trae al frente la ventana principal y lo abre ahí: un documento en la Biblioteca, una tarea junto a su lista, una etiqueta en su vista. La red queda abierta detrás, con el mismo nodo elegido; otro clic en **Brainsty** te devuelve a ella.
- **Una imagen o un archivo** tienen vista previa, pero no **Abrir**: se abren desde el documento o la tarea que los usa, su vecino en la red.
- **Cerrar la vista previa**, o `Esc`, suelta el nodo y la red vuelve a verse entera.

![«Rina» elegida de cerca en la ventana de Brainsty: sus vecinos con título, el resto atenuado y, a la derecha, la vista previa del documento con lo que es, su carpeta y cuántas conexiones tiene, sus primeras líneas, la foto de Rina y el botón Abrir](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/es/red-cerca.png)

## Buscar

El buscador está arriba a la izquierda, **Buscar en la red**; `/` lo enfoca desde cualquier parte de la red. Escribe parte de un título: no importan los acentos ni las mayúsculas. Debajo aparece la lista de lo que calza, cada uno con lo que es («Documento», «Tarea», «Etiqueta»…): primero lo que empieza igual, después lo que tiene una palabra que empieza así y al final lo demás. `Enter` elige el primero, o un clic el que quieras: la red se centra en él y abre su vista previa. Si nada se llama así, dice «Nada con ese nombre».

## Filtrar

El embudo junto al buscador, **Filtros**, abre lo que se puede dejar fuera. Cada opción es un botón que se enciende y se apaga, y la red se redibuja con lo que queda:

- **Mostrar**: **Documentos**, **Páginas**, **Tareas**, **Etiquetas** y **Archivos**.
- El estado de las tareas: **En curso**, **Terminadas** y **Archivadas**.
- **Tocado**: **Cuando sea**, **Últimos 7 días**, **Últimos 30 días** o **Último año**, según el último día en que cambió cada cosa.
- **Etiqueta**: elige una de la lista y deja solo lo que la lleva; **Cualquier etiqueta** la quita.

Con algún filtro puesto, el embudo se tiñe y aparece **Mostrar todo**, que devuelve la red entera.

![La ventana de Brainsty con Filtros abierto: Documentos y Tareas encendidos, En curso, Últimos 30 días y la etiqueta rina, el embudo teñido, Mostrar todo a la vista y la red con lo que queda](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/es-filtros.png)

## Con el teclado

| Tecla | Qué hace |
| :-- | :-- |
| `Tab` | recorrer el buscador, **Filtros**, la vista previa y la red, sin salir de la ventana |
| Flechas | elegir un nodo y caminar: primero a un vecino en esa dirección, si no, al más cercano |
| `Enter` | abrir el nodo elegido en la ventana principal |
| `/` | buscar |
| `Esc` | soltar por capas: primero la búsqueda, aunque esté vacía; después el nodo; y el último cierra la ventana |

Después de elegir un resultado de la búsqueda, o de dejarla con `Esc`, las flechas y `Enter` vuelven a caminar por la red, sin tocar el mouse. Así, `/`, `rina`, `Enter` y otra vez `Enter` abren la ficha de Rina sin soltar el teclado.

## Un ejemplo: volver a lo de Rina

Hace un mes que no tocas nada de Rina y quieres saber cómo va.

1. **Brainsty** en la barra lateral. La red se abre en su ventana, en lo último que escribiste: el acta del condominio.
2. `/`, `rina`, `Enter`. La red se centra en «Rina», la ficha de la carpeta **Mascotas**, y su vista previa muestra sus primeras líneas.
3. Te acercas con la rueda: aparecen sus vecinos con título, «Veterinaria» unida por una línea punteada (la mencionas en una frase), la tarjeta sólida de «Vacuna anual de Rina» y la foto del paseo con su miniatura.
4. Un clic en «Vacuna anual de Rina»: la vista previa muestra su tarjeta, «1 de 3 pasos», vence el viernes. **Abrir** trae al frente la ventana principal, con la tarea junto a su lista, y ahí marcas «Pedir hora».
5. Otro clic en **Brainsty** te devuelve a la red, que seguía detrás. **Filtros**, **Etiqueta** `rina` y **Tocado** **Últimos 30 días**: quedan sus dos documentos y las tareas en curso que se movieron este mes.
6. **Mostrar todo**, y `Esc` hasta que la ventana se cierra. La ventana principal sigue en la tarea que abriste.

## Parecidos

Dos cosas pueden hablar de lo mismo sin que nadie las haya unido: la tarea «Pedir presupuesto del techo» y el documento «Arreglos de la casa», que habla del techo y de su presupuesto. Brainsty las anota como **parecidas**, aparte de las líneas que escribiste.

Dos cosas se parecen cuando comparten al menos dos **palabras de peso**: de esas que aparecen en pocas de las cosas que guardas, en el 5 % o menos. *Presupuesto* y *techo* pesan; *semana* o *hacer*, que están en todas partes, no. Cada parecido guarda las palabras que dicen por qué. Las palabras se comparan sin tildes ni mayúsculas, así que *Techo* y *techo* son la misma.

Sirve también entre idiomas cuando lo que comparten son nombres: una tarea en inglés y un documento en español que hablan de *Folio* y de *Glama* se parecen; «Vacuna anual de Rina» y «Rina's yearly vaccine», que solo comparten *Rina*, no.

Las reglas:

- **Como mucho tres parecidos por cosa**, y cada punta tiene que estar entre las tres más cercanas de la otra.
- **El umbral sale de lo que guardas**, no de una cifra fija: el parecido tiene que superar lo que alcanza un par de cada mil en tu base.
- **Lo que ya está unido no se adivina**: si una línea de la red ya los une (una cita, una mención, una parte y su tarea, el documento donde nació una tarea), o si son una página y su libro, un documento y su versión en otro idioma, o una tarea y su página.
- **Un documento en otro idioma** tiene su propio lugar entre los tres, para que lo que comparte con otro idioma no quede tapado por parientes del mismo.
- **De cada cosa se guardan hasta 256 palabras**, las más dichas. Los números solos y las cadenas largas de letras y números, como un identificador o un hash, no cuentan.

**Cuándo.** Una pasada de baja prioridad, en segundo plano, mira cada 30 segundos y espera a que dejes de escribir, cinco minutos como mucho, antes de comparar. Las palabras de lo que cambió se guardan en el momento. Cuando una versión nueva de Tisty cambia la base de Brainsty, esta se rehace una vez, en el primer arranque: mientras tanto la red dice «Brainsty todavía está leyendo lo que guardas. Vuelve a abrirlo en un momento.».

**Lo que nunca pasa por aquí.** Las tareas ocultas y las descartadas, y todo tu [Diario](Documentos#diario), no guardan palabras, aunque abras el Diario a los agentes. Ocultar una tarea saca sus parecidos en la misma ronda.

**Dónde se ven.** Hoy, en ninguna parte: la red dibuja solo lo escrito, así que un parecido no es una línea de la red, y la ficha que lee tu agente con `brain` tampoco lo trae. Los parecidos viven en la base de Brainsty de este equipo.

## Lo que conviene saber

- **Se arma sola, en este equipo.** Tisty lee cómo se unen tus cosas en segundo plano, sin frenar lo que haces. Un enlace recién escrito llega a la base a los pocos segundos.
- **La red se lee al abrirse.** Lo que escribes con la ventana de Brainsty abierta aparece la próxima vez que la abras: ciérrala y ábrela otra vez.
- **Recién instalado**, mientras lee todo por primera vez, la red dice «Brainsty todavía está leyendo lo que guardas. Vuelve a abrirlo en un momento.». Con nada guardado: «Todavía no hay nada que dibujar: escribe un documento o una tarea y aparecerá aquí.».
- **Hasta 12.000 nodos y 40.000 conexiones.** Más allá, la red guarda lo último que tocaste, sus vecinos, tus documentos y las tareas en curso, y dice arriba cuánto quedó fuera: «312 nodos y 940 conexiones quedaron fuera». Pasados los 6000 nodos, los nodos se achican para que la red entera ocupe lo mismo en la ventana.
- **Lo oculto no aparece.** Una tarea que ocultaste no se dibuja, ni como nodo ni como punta de una línea, y una etiqueta o un archivo que solo ella llevaba se va con ella. Una tarea descartada sí aparece, tenue y pequeña.
- **Nada sale del equipo.** La red y los parecidos se calculan en cada equipo y se rehacen solos: no se sincronizan, ni van a Folio, a un `.tistyx` o a una exportación. Lo que viaja son tus documentos y tareas, y cada equipo vuelve a dibujarla.
- **Sin tarjeta gráfica dedicada** la red se dibuja igual; acercarse y alejarse cuesta un poco más.

## Para seguir

- [[Enlazar documentos|Enlazar-documentos]]: las líneas punteadas y sólidas entre documentos.
- [[Tareas en documentos|Tareas-en-documentos]]: las líneas entre un documento y sus tareas.
- [[Cómo trabaja un agente|Cómo-trabaja-un-agente]]: la misma red, leída por tu agente.
- [[Documentos]]: carpetas y etiquetas, que dan los colores y las zonas.
