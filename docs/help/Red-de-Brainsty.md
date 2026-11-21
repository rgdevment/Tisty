# La red de Brainsty

Llevaba años guardando cosas en Tisty y no tenía forma de verlas juntas. Sabía que «Rina» citaba la tarea de su vacuna y que el acta del condominio había creado tres tareas, pero cada relación vivía dentro de su archivo. Brainsty dibuja todo lo que guardas, con lo que lo une, como una sola red: de lejos, un mapa de tus proyectos; de cerca, cada documento y cada tarea con sus vecinos.

Esta página cuenta cómo abrirla, qué significa cada forma y cada línea, cómo moverte por ella y cómo filtrarla. Las uniones que dibuja las haces tú al escribir: [[Enlazar documentos|Enlazar-documentos]] y [[Tareas en documentos|Tareas-en-documentos]]. Tu agente lee la misma red como un índice: [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

![La ventana de Tisty con la barra lateral a la izquierda y Brainsty marcado en ella; en el panel de contenido, la red alejada: arriba la cuenta, «1940 nodos · 2847 conexiones», cada documento y cada tarea del color de su carpeta o su lista, los documentos como cuadrados, las tareas como círculos y las etiquetas como puntos morados, más tenues, la red un poco girada como un globo, con sus viajeros como puntos de luz sobre las líneas, y abajo a la izquierda la Leyenda, plegada](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/es/red.png)

## Cuándo te sirve

- **Volver a un proyecto después de semanas.** Abres la red, buscas el proyecto y ves de un vistazo sus documentos, sus tareas abiertas y las que ya cerraste.
- **Encontrar lo que quedó suelto.** Un documento sin líneas es uno que nada nombra: quizás le falta un enlace, o ya no sirve.
- **Ver qué toca una etiqueta.** Filtra por `#rina` y quedan sus vacunas, sus paseos, el veterinario y las tareas de este mes.
- **Ver lo último que tocaste.** El filtro **Tocado** con **Últimos 7 días** deja solo lo que se movió esta semana.

## Abrirla y salir

La red se abre en el panel de contenido de la ventana principal, como cualquier otra vista, con la barra lateral a la vista. Así la miras sin perder de vista tus listas, y sales de ella con un clic en otra vista.

- **Brainsty**, en la barra lateral bajo las vistas de tareas, la abre, y queda marcado mientras la miras. Con la barra plegada, es el icono de la red en el riel.
- Abre con la red entera a la vista, girando como un globo (mira [La red viva](#la-red-viva)). El documento o la tarea en que estabas queda como punto de partida de las flechas; si no estabas en ninguno, lo último que escribiste. Arriba, la cuenta: «589 nodos · 1252 conexiones».
- La primera vez los nodos se acomodan unos segundos, como si se soltaran, y se detienen solos en cuatro segundos como mucho; una red de unos 5600 nodos queda quieta en dos o tres.
- **El teclado ya es suyo** apenas se dibuja: las flechas, `/` y `Enter` funcionan sin hacer clic antes en la red.
- **Se ajusta al panel.** Si el panel cambia de ancho, por ejemplo al plegar la barra lateral, la red se vuelve a dibujar a su medida.
- **Para salir**, elige otra vista en la barra lateral, abre algo con **Abrir** o usa `Esc` hasta que te devuelva a donde estabas (mira [Con el teclado](#con-el-teclado)). Al salir se detiene todo lo que la red corría: el dibujo, el acomodo y su trabajo en segundo plano. No queda nada gastando.
- **Cada vez que entras, empieza de nuevo.** Se lee otra vez, con lo que escribiste entretanto, y arranca sin nodo elegido, sin búsqueda y sin filtros. Lo que cada equipo sí recuerda: si dejaste abierta la [leyenda](#la-leyenda), sus interruptores **Viajeros** y **Girar**, y **Mostrar AGENTS**.
- **Mientras la miras, no se rehace sola.** Si llega algo nuevo, como un paso que marca tu agente o una ronda de sincronización, la red conserva la vista, el acomodo y tus interruptores; lo nuevo aparece la próxima vez que entras. Solo un cambio en tus carpetas o listas (una nueva, otro nombre) la dibuja de nuevo, y un color nuevo se ve al momento.
- **Sigue a Tisty**, como cualquier vista: el tema claro u oscuro y el idioma. En oscuro, la red toma los colores oscuros.

## Qué es cada cosa

La tabla dice cómo se ve cada clase de lejos y a media distancia. De cerca, cada nodo pasa a ser su ícono: lo cuenta [De lejos y de cerca](#de-lejos-y-de-cerca).

| En la red | Es | Cómo se ve |
| :-- | :-- | :-- |
| Cuadrado | un documento o una página de un libro | del color de su carpeta; una página, más pequeña junto a su documento |
| Círculo | una tarea | del color de su lista; en curso, viva; terminada, más tenue |
| Punto morado | una etiqueta | unido a todo lo que la lleva, tenue |
| Punto gris | una imagen o un archivo | unido a lo que lo usa; de cerca, con su miniatura |

**Las carpetas y las listas no son nodos.** Dan el color a lo suyo, tiñen su zona, tienen su lugar en la [leyenda](#la-leyenda) y la ficha de cada documento o tarea dice en cuál está, pero no hay un punto para «Mascotas» ni una línea hacia él: una carpeta no une nada que no hayas unido tú al escribir. Las etiquetas sí son nodos, porque `#rina` junta cosas de carpetas y listas distintas.

Las etiquetas se ven tenues a propósito: dicen de qué trata algo, no qué escribiste. Lo que manda en la red es lo que alguien escribió, tú o tu agente (mira [Lo que escribiste pesa más](#lo-que-escribiste-pesa-más)).

**Un documento archivado** se ve tenue y pequeño. Una tarea no llega a ese estado en la red: mientras está en curso o terminada se dibuja, y la que ocultas o descartas sale de ella (mira [Lo que conviene saber](#lo-que-conviene-saber)).

**Los documentos de [[Agentes|Carpeta-AGENTS]]**, las reglas, skills y memorias de tus agentes, vienen ocultos. No se dibujan, no salen en la búsqueda ni en la leyenda y los viajeros no pasan por ellos, hasta que enciendes **Mostrar AGENTS** en los [filtros](#filtrar). Mostrados, van más chicos y en un coral tenue, el color propio de esa carpeta, con sus líneas también más tenues. Así no compiten con lo tuyo: en una red con veinte reglas de Claude y Codex, lo que salta a la vista sigue siendo lo que escribiste tú.

| Línea | Une | Cuándo existe | Cómo se ve |
| :-- | :-- | :-- | :-- |
| Sólida | una cita | una tarjeta de otro documento o de una tarea, sola en su párrafo | la más gruesa y marcada |
| Punteada | una mención | un enlace dentro de una frase a otro documento o a una tarea | marcada |
| Sólida, algo más fina | la estructura | una página y su libro, una parte y su tarea, una tarea y el documento donde nació, una idea y su documento | marcada |
| Fina, hacia un archivo | un uso | lo que muestra una imagen o adjunta un archivo | tenue |
| Fina | una etiqueta | lo que lleva `#etiqueta` y su punto | más tenue |
| Curva ámbar | un parecido | dos cosas que comparten palabras poco comunes, una palabra con un error de tipeo o una traducción, y que nadie unió | más opaca cuanto más se parecen |

Cada línea recta está escrita en alguna parte, y se puede explicar. La curva ámbar no: es lo que Tisty cree que se parece, nunca es recta, sólida ni punteada, para no confundirse con nada escrito, y la ficha del nodo dice por qué. Lo cuenta [Parecidos](#parecidos).

### Lo que escribiste pesa más

Una cita, una mención o una parte las escribió alguien a propósito; una etiqueta solo dice de qué trata algo. Por eso la red no las trata igual:

- **Al acomodarse**, una cita, una mención o una parte tiran con fuerza 3; una línea a un archivo, con 1,5, y una etiqueta, con 0,3. Los parecidos no tiran: no mueven a nadie.
- **Al dibujarse**, las citas, las menciones y las partes van más gruesas y marcadas; los archivos, más tenues, y las etiquetas, las más tenues de todas.
- **Los viajeros** también prefieren lo escrito (mira [La red viva](#la-red-viva)).

Por ejemplo, Yisset cita la tarea «Vacuna anual de Rina» en la ficha de Rina. La tarea vive en la lista **Casa** y la ficha en la carpeta **Mascotas**, pero en la red quedan juntas, unidas por una línea sólida bien visible. Las dos llevan `#rina`, y esa etiqueta casi no las acerca: lo que las junta es la cita.

## La leyenda

No hace falta aprenderse estas tablas: **Leyenda**, abajo a la izquierda de la red, dice qué significa cada forma, cada color y cada línea de tu red. Viene plegada, para no tapar la red; un clic en su título la abre y otro la pliega. Cada equipo recuerda cómo la dejaste.

Abierta, arriba lleva dos interruptores, **Viajeros** y **Girar**, que cuenta [La red viva](#la-red-viva). Debajo tiene cuatro partes. Cada muestra se dibuja con el mismo pincel que la red, así que lo que ves en la leyenda es exactamente lo que ves en la red, también en tema oscuro:

| Parte | Qué muestra |
| :-- | :-- |
| **Qué es** | la forma y el tono de cada clase que hay en tu red: **Documento**, **Página**, **Tarea**, **Etiqueta** y **Archivo**. Una clase que no tienes no aparece |
| **Color: su carpeta o lista** | las carpetas y listas con color que tienen algo en la red, primero las más llenas. Muestra seis; **N más** abre el resto y **Menos** las vuelve a esconder. Con **Mostrar AGENTS** encendido, debajo va **AGENTS**, con su cuadrado chico en coral tenue; con el puntero encima dice «Los archivos de los agentes que Tisty mantiene al día, chicos y tenues para no tapar los tuyos». Al pie: «Sin color, toma el tono de su clase.» |
| **Estado** | cómo se ve lo que está **En curso**, una de las tareas **Terminadas** y uno de los documentos **Archivados** (la fila dice **Archivadas**) |
| **Líneas, de cerca** | **Cita**, **Menciona**, **Parte de (página, parte)**, **Etiquetada** y **Usa un archivo**, solo las que hay en tu red; y **Parecidos**, la curva ámbar, si Tisty cree que algo se parece. Con el puntero sobre **Parecidos**, te dice qué son: «Curvas ámbar: lo que Tisty cree que se parece, de cerca o junto al nodo que eliges. Nadie las escribió.» |

Por ejemplo, Marisol tiene once carpetas y listas con color, y la carpeta Agentes con sus reglas de Claude. Su leyenda muestra las seis más llenas, **Casa**, **Trabajo**, **Mascotas**, **Viajes**, **Compras** y **Salud**, y debajo **5 más**. Un día quiere ver dónde quedaron sus reglas: enciende **Mostrar AGENTS** en los filtros y la leyenda suma, aparte, **AGENTS**; la carpeta Agentes no compite por un lugar entre sus colores. Una carpeta sin color, como **Recetas**, no está en la lista: sus documentos se ven con el tono de los documentos.

![La Leyenda abierta en la esquina inferior izquierda de la red: arriba los interruptores Viajeros y Girar, con su punto encendido; Qué es con Documento, Página, Tarea, Etiqueta y Archivo; Color: su carpeta o lista con Trabajo, Casa, Finanzas, Proyecto Tisty, Salud y Familia, «35 más» y «Sin color, toma el tono de su clase.»; Estado con En curso, Terminadas y Archivadas; Líneas, de cerca con Cita, Menciona, Parte de (página, parte), Etiquetada y Usa un archivo](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/es-leyenda.png)

## La red viva

Mientras la miras, la red se mueve: gira y la recorren viajeros por sus líneas. Viene así de fábrica; si la prefieres quieta, la apagas desde la leyenda.

![La ventana de Tisty con Brainsty abierto, en el tema oscuro y en inglés: la barra lateral con las tareas de hoy y la Biblioteca y, al lado, la red viva, 1.931 nodos y 2.811 conexiones, girando despacio como un globo mientras viajeros de luz recorren sus líneas, con la Leyenda plegada abajo a la izquierda](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/app/en-brainsty-living-dark.webp)

- **Gira como un globo**, sola, una vuelta por minuto. Lo que queda atrás se ve más chico y más tenue, y con la red girada las zonas de color se ocultan y se ven menos títulos.
- **De cerca, se queda quieta.** Apenas acercas la vista, con la rueda, al elegir un resultado de la búsqueda o al caminar con las flechas, el globo deja de girar, y no vuelve a hacerlo hasta que ves otra vez la red entera. Así el nodo que miras no se te va de lado. Alejarte más allá de la red entera no lo detiene.
- **También descansa** mientras el puntero está sobre la forma de la red y mientras tienes un nodo elegido. En el espacio vacío alrededor sigue girando.
- **Un doble clic en un espacio vacío** te devuelve a la red entera, y el globo vuelve a girar. Un doble clic sobre un nodo, en cambio, lo abre.
- **La giras tú** arrastrando con el botón derecho, o con `Shift` y el izquierdo. El arrastre normal sigue moviendo la vista, como siempre.
- **Los viajeros** son puntos de luz que recorren las conexiones en viajes largos: dejan una estela corta y destellan al pasar por una etiqueta o por un nodo muy conectado. Siguen caminando aunque el globo descanse, y nunca pasan por lo que los filtros esconden ni por los archivos de Agentes mientras están ocultos. Si no queda ninguna línea por la que caminar, no sale ninguno.
- **Prefieren lo escrito.** Al elegir por dónde seguir, miran dos cosas: adónde van, con los documentos y las páginas primero, y por qué línea. Una cita, una mención o una parte les pesa el doble que una línea a un archivo y diez veces más que una etiqueta.
- **Cuántos viajeros, y a qué paso**, depende del tamaño de la red: cuentan todos sus nodos, también los que un filtro esconde, los mismos que dice la cuenta de arriba. En una red chica van pocos y rápidos; en una grande, más y con calma, para que se puedan seguir con la vista:

| Nodos en la red | Viajeros | Pasan de un nodo al siguiente cada |
| :-- | :-- | :-- |
| menos de 60 | 4 | 3 segundos |
| de 60 a 99 | 8 | 4 segundos |
| de 100 a 149 | 10 | 5 segundos |
| de 150 a 349 | 12 | 6 segundos |
| 350 o más | 14 | 7 segundos |

- **Dos interruptores**, arriba en la [leyenda](#la-leyenda) abierta: **Viajeros** y **Girar**, cada uno con un punto que se enciende. Vienen encendidos, también si tu sistema pide reducir el movimiento: la decisión es tuya. Sin **Girar**, la red queda plana; con los dos apagados, queda como una red quieta. Cada equipo recuerda cómo los dejaste.
- **Solo corre mientras la ves.** Al pasar a otra vista se detiene, igual que el resto de la red.
- **Lo que cuesta**, si te interesa: `Ctrl` + `Shift` + `M` muestra junto a la leyenda cuántos milisegundos toma cada cuadro y cuántos cuadros por segundo dibuja, por ejemplo «2.10 ms por cuadro · 60 cuadros por segundo». Otro `Ctrl` + `Shift` + `M` lo esconde.

Por ejemplo, Lucas abre Brainsty con 180 nodos: ve doce viajeros cruzando la red mientras gira, cada uno pasando de un nodo al siguiente cada seis segundos. Deja el puntero sobre la zona de **Trabajo** para leer un título y el globo se detiene; lo saca al espacio vacío y vuelve a girar. Se acerca con la rueda a «Presupuesto 2027» y el globo se queda quieto mientras lo lee, aunque mueva el puntero; los viajeros siguen pasando. Un doble clic en un espacio vacío y está otra vez frente a la red entera, girando. Para presentar su pantalla prefiere la red quieta: abre la **Leyenda** y apaga **Viajeros** y **Girar**, y la próxima vez en ese equipo abre igual.

## De lejos y de cerca

La red cambia lo que muestra según la distancia, y cada detalle aparece de a poco al acercarte:

- **De lejos**, como se abre, con la red entera a la vista, o con la rueda hacia atrás: los nodos y sus uniones forman la silueta de un cerebro visto de lado, sin títulos y con líneas tenues. Sirve para ver dónde hay mucho y dónde hay poco.
- **A media distancia**, en cuanto te acercas un poco: cada carpeta y cada lista tiñen su zona, y llevan título solo los nodos más conectados.
- **De cerca**: todos los títulos, cada tipo de línea, las etiquetas como `#etiqueta` y junto a cada imagen su miniatura. Y cada nodo deja de ser un cuadrado o un círculo para ser su ícono, del color de su carpeta o lista y con su estado: la forma simple se desvanece al acercarte y el ícono toma su lugar. Las curvas ámbar aparecen aquí, de a poco, entre dos nodos que están a la vista; junto al nodo que eliges se ven desde cualquier distancia.

| Nodo | Su ícono, de cerca |
| :-- | :-- |
| Documento | una hoja |
| Página de un libro | unas líneas de texto |
| Tarea | una casilla; una terminada, un tic |
| Etiqueta | un `#` |
| Imagen o archivo | un clip |

Así, de cerca, la tarea «Vacuna anual de Rina» es una casilla del color de la lista **Casa**, y «Comprar el antiparasitario de Rina», que terminaste la semana pasada, un tic del mismo color. El ícono se atenúa con lo que no está elegido, como el resto, y un clic sobre él elige el nodo igual que antes. De lejos no cuesta nada: ahí vuelven las formas simples, que se dibujan de a miles.

Con la rueda acercas y alejas; arrastrando te mueves, y arrastrando con el botón derecho, o con `Shift`, giras el globo. Al primer toque la red deja de seguir el punto donde arrancó. Un doble clic en un espacio vacío te devuelve de un salto a la red entera, desde cualquier distancia.

## Elegir, ver y abrir

**Un clic en un nodo** lo resalta junto con sus vecinos, atenúa el resto y abre su **ficha**, a la derecha. La ficha no muestra el texto del documento: muestra sus datos, para que sepas qué es, de cuándo es y con qué se une antes de abrirlo. Arriba van su título y una línea con qué es, su carpeta o su lista y cuántas conexiones tiene: «Documento · Mascotas · 6 conexiones».

Debajo vienen sus datos, según lo que elegiste:

| Eliges | La ficha muestra |
| :-- | :-- |
| Un documento | **Escrito por última vez**, **Creado**, **Tamaño**, con su peso y cuántas palabras tiene, e **Idioma**, como «ES» |
| Una página de un libro | lo mismo, y primero **Página de**, con el título de su libro |
| Una tarea | **Estado**, como «En curso · Por vencer»; **Día**, **Plazo**, **Parte de**, con su tarea madre, **Partes**, con cuántas tiene, **Pasos**, como «1 de 3 hechos», y **Origen**, si llegó con uno |
| Una etiqueta | solo la línea de arriba y sus conexiones |
| Una imagen o un archivo | su miniatura, si es una imagen, o el ícono de su tipo; **Tipo**, **Tamaño** y **Modificado**, y **Se usa en**, con cada documento o tarea que lo usa: un clic en uno lo elige en la red. Un archivo que nada a la vista usa muestra solo su nombre |

El estado de una tarea suma, después de un punto, cómo va su fecha: **Por venir**, **Para hoy**, **Atrasada**, **En curso**, **Por vencer** o **Vencida**. Una fila sin dato no aparece: una tarea sin plazo no muestra **Plazo**.

Después, en este orden, lo que hay:

- **Etiquetas**, cada una como `#rina`.
- **Conexiones**, contadas por tipo: «2 citas 3 menciones». Los tipos van en este orden: **citas**, **menciones**, **partes**, **etiquetas** y **archivos**, solo los que tiene. Su carpeta o su lista no cuenta como conexión: va en la línea de arriba.
- **Apuntan aquí**: los documentos y las tareas que lo citan o lo mencionan, hasta seis, y «N más» si hay otros. Un clic en uno lo elige en la red y abre su ficha; uno que no está en la red, como lo que quedó fuera del tope de nodos, se lee, pero no se elige.
- **Parecidos**, en ámbar, si Tisty cree que algo se le parece: cada uno con su título y su porqué, «Tisty cree que se parecen: comparten *presupuesto* y *techo*». Un clic en el título lo elige en la red. Lo cuenta [Parecidos](#parecidos).
- **Enlace**, en un documento, una página o una tarea: su dirección y **Copiar enlace**, que pasa a decir «Enlace copiado». El **?** junto al título, **¿Para qué sirve un enlace?**, lo explica al pasar el puntero y lo deja escrito si lo pulsas: «Un enlace lleva a esto mismo. Pégalo en un documento para llegar aquí con un clic, o dáselo a un agente para que lea esto. Solo funciona dentro de Tisty.» Más en [[el enlace de un documento|Documentos#el-enlace-de-un-documento]]. Es el mismo enlace que copias desde el panel de un documento, y sirve para pegarlo en un chat, en tu navegador o dárselo a tu agente. El de un documento y el de una tarea se ven así:

```text
tisty://doc/mascotas/rina?id=…
tisty://task?id=…
```

Al pie, **Abrir**. La cruz de arriba, **Cerrar la vista previa**, `Esc` o un clic en un espacio vacío sueltan el nodo: nada queda atenuado, y la vista se queda donde estaba. Para volver a la red entera, un doble clic en un espacio vacío.

- **Abrir**, `Enter` o **un doble clic** en el nodo lo abre en la misma ventana, en el lugar de la red: un documento en la Biblioteca, una tarea junto a su lista y una etiqueta en su vista. Para volver a la red, otro clic en **Brainsty**; abre de nuevo entera, y lo que abriste queda como punto de partida de las flechas.
- **Una imagen o un archivo** no tienen **Enlace**. Su **Abrir** lo abre con el programa de tu sistema, fuera de Tisty, y **Mostrar en la carpeta** te lleva a él en el explorador de archivos. Por ejemplo, eliges el clip de «presupuesto-techo.pdf»: su ficha dice **Tipo** PDF, **Tamaño** 240 kB, **Modificado** el martes, y en **Se usa en**, la tarea «Pedir presupuesto del techo» y el documento «Arreglos de la casa».

![«Ficha de Rina» elegida de cerca en Brainsty, en el panel de contenido junto a la barra lateral: a su alrededor, cada nodo dibujado como su ícono del color de su carpeta o lista (la hoja de un documento, la casilla de una tarea, el clip de un archivo) y la foto de Rina en miniatura; abajo a la izquierda, la Leyenda abierta; a la derecha, su ficha: «Documento · Salud · 7 conexiones», las filas Escrito por última vez, Creado y Tamaño con sus palabras, la etiqueta rina, Conexiones con 6 menciones y 1 etiqueta, Apuntan aquí con «Ir a la veterinaria por la vacuna de Rina» y «Paseos del sábado», el Enlace con Copiar enlace y el botón Abrir](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/es/red-cerca.png)

## Buscar

El buscador está arriba a la izquierda, **Buscar en la red**; `/` lo enfoca desde cualquier parte de la red. Escribe parte de un título: no importan los acentos ni las mayúsculas. Debajo aparece la lista de lo que calza, cada uno con lo que es («Documento», «Tarea», «Etiqueta»…): primero lo que empieza igual, después lo que tiene una palabra que empieza así y al final lo demás. `Enter` elige el primero, o un clic el que quieras: la red se centra en él y abre su ficha. Si nada se llama así, dice «Nada con ese nombre».

## Filtrar

El embudo junto al buscador, **Filtros**, abre lo que se puede dejar fuera. Cada opción es un botón que se enciende y se apaga, y la red se redibuja con lo que queda:

- **Mostrar**: **Documentos**, **Páginas**, **Tareas**, **Etiquetas**, **Archivos** y **Parecidos**. **Parecidos** viene encendido; apagado, la red no dibuja ninguna curva ámbar y la ficha no los lista.
- **Mostrar AGENTS**, en esa misma fila, solo si tu red trae archivos de la carpeta [[Agentes|Carpeta-AGENTS]]. Viene apagado: encendido, sus documentos aparecen chicos y en coral tenue, entran en la búsqueda y la leyenda los nombra **AGENTS**. Cada equipo recuerda cómo lo dejaste, y **Mostrar todo** no lo cambia. Con el puntero encima dice «Los archivos de los agentes que Tisty mantiene al día, chicos y tenues para no tapar los tuyos».
- El estado: **En curso**, **Terminadas** y **Archivadas**. Vale para tareas, documentos y páginas, no para etiquetas ni archivos: un documento cuenta como en curso mientras no lo archives, así que apagar **En curso** esconde también tus documentos abiertos, y **Archivadas** deja ver los que archivaste.
- **Tocado**: **Cuando sea**, **Últimos 7 días**, **Últimos 30 días** o **Último año**, según el último día en que cambió cada cosa.
- **Etiqueta**: elige una de la lista y deja solo lo que la lleva; **Cualquier etiqueta** la quita.

Con algún filtro puesto, el embudo se tiñe y aparece **Mostrar todo**, que devuelve la red entera. **Mostrar AGENTS** queda aparte: ni tiñe el embudo ni se apaga con **Mostrar todo**.

![Brainsty en el panel de contenido con Filtros abierto: en Mostrar, todo encendido, de Documentos a Archivadas; en Tocado, Últimos 30 días; en Etiqueta, Cualquier etiqueta; el embudo teñido y Mostrar todo; debajo, la Leyenda abierta, con Parecidos en ámbar, y al lado la red con lo que queda](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/es-filtros.png)

## Con el teclado

| Tecla | Qué hace |
| :-- | :-- |
| `Tab` | recorrer el buscador, **Filtros**, la ficha y la red |
| Flechas | elegir un nodo y caminar: primero a un vecino en esa dirección, si no, al más cercano; una curva ámbar no es un camino. Sin nodo elegido, la primera sale del documento o la tarea en que estabas |
| `Enter` | abrir el nodo elegido, en el lugar de la red |
| `/` | buscar |
| `Esc` | soltar por capas: primero la búsqueda, aunque esté vacía; después el nodo; y el último te devuelve a la vista donde estabas |
| `Ctrl` + `Shift` + `M` | mostrar o esconder lo que cuesta cada cuadro |

La línea de teclas al pie de la red lo resume: «Flechas para moverte entre nodos, Enter abre, / busca, Esc suelta, arrastre derecho o Shift gira».

Después de elegir un resultado de la búsqueda, o de dejarla con `Esc`, las flechas y `Enter` vuelven a caminar por la red, sin tocar el mouse. Así, `/`, `rina`, `Enter` y otra vez `Enter` abren el documento de Rina sin soltar el teclado.

Un `Esc` que escribes en un campo de la barra lateral, como su buscador, es de ese campo: no suelta nada en la red ni te saca de ella.

## Un ejemplo: volver a lo de Rina

Hace un mes que no tocas nada de Rina y quieres saber cómo va. Estás en el acta del condominio.

1. **Brainsty** en la barra lateral. La red se abre en el panel, en lugar del acta, entera y girando, con sus viajeros.
2. `/`, `rina`, `Enter`. La red se acerca y se centra en «Rina», el documento de la carpeta **Mascotas**, y el globo se queda quieto mientras la miras de cerca. Abre su ficha: «Documento · Mascotas · 6 conexiones», escrito por última vez hace un mes, en español. **Mascotas** va en esa línea de arriba y no cuenta entre las seis.
3. Te acercas un poco más con la rueda: cada vecino pasa a ser su ícono, con su título. La hoja de «Veterinaria», unida por una línea punteada (la mencionas en una frase); la casilla de «Vacuna anual de Rina», del color de la lista **Casa**, unida por la línea sólida y marcada de su tarjeta; el clip de la foto del paseo, con su miniatura; el `#` de la etiqueta `rina`, tenue, y una curva ámbar hacia «Paseos del sábado», que nadie enlazó. La ficha de Rina dice por qué, en **Parecidos**: «Tisty cree que se parecen: comparten *rina* y *parque*».
4. Un clic en «Vacuna anual de Rina»: su ficha dice «En curso · Por vencer», **Plazo** el viernes y **Pasos** «1 de 3 hechos», y en **Apuntan aquí** está «Rina», que la cita. **Copiar enlace** te deja su enlace para pasárselo a Yisset, que la va a llevar.
5. Quieres ver qué más hay antes de seguir: un doble clic en un espacio vacío y vuelves a la red entera, girando. `/`, `vacuna`, `Enter` te lleva de vuelta a la tarea. **Abrir**: la tarea se abre junto a su lista, en el lugar de la red, y ahí marcas «Pedir hora».
6. Otro clic en **Brainsty**: la red abre de nuevo, entera; la tarea queda como punto de partida de las flechas. **Filtros**, **Etiqueta** `rina` y **Tocado** **Últimos 30 días**: quedan sus dos documentos y las tareas en curso que se movieron este mes.
7. `Esc` hasta volver: primero suelta lo que elegiste y el último te devuelve a la tarea. Los filtros no se guardan: la próxima vez la red arranca entera. Solo **Mostrar AGENTS** queda como lo dejaste.

## Parecidos

Dos cosas pueden hablar de lo mismo sin que nadie las haya unido: la tarea «Pedir presupuesto del techo» y el documento «Arreglos de la casa», que habla del techo y de su presupuesto. Brainsty las anota como **parecidas**, aparte de las líneas que escribiste.

Tisty busca parecidos con tres pistas, en este orden. Cada pista solo llena los lugares que la anterior dejó libres, hasta tres parecidos por cosa, y cada parecido guarda lo que dice por qué.

### Palabras en común

Dos cosas se parecen cuando comparten al menos dos **palabras de peso**: de esas que aparecen en pocas de las cosas que guardas, en el 5 % o menos. *Presupuesto* y *techo* pesan; *semana* o *hacer*, que están en todas partes, no. Las palabras se comparan sin tildes ni mayúsculas, así que *Techo* y *techo* son la misma.

Sirve también entre idiomas cuando lo que comparten son nombres: una tarea en inglés y un documento en español que hablan de *Folio* y de *Glama* se parecen.

### Un error de tipeo

Escribes rápido y anotas «La imrpesora nueva no imprime en color». Una semana después, Jaime te pide «Instalar la impresora nueva». Solo comparten *nueva*, pero *imrpesora* está a un cambio de *impresora*, y Tisty las une: «Tisty cree que se parecen: comparten *impresora ≈ imrpesora* y *nueva*». En inglés funciona igual, con palabras de seis letras o más.

- **Un cambio** es una letra de más, una de menos, una distinta o dos letras vecinas que cambiaron de lugar, como *rp* por *pr*.
- **Solo palabras de seis letras o más.** *Casa* y *cosa* son dos palabras, no un error.
- **Nunca palabras con dígitos.** *capitulo1* y *capitulo2* se escribieron distintas a propósito.
- De las dos formas, la que más usas en lo que guardas es la que cuenta como la buena.

### Una traducción

Anotas «Pagar la cuenta del agua» y, otro día, tu agente anota en inglés «Pay the water bill». Hablan de lo mismo y no comparten ni una palabra. Tisty las une con un diccionario español–inglés: «Tisty cree que se parecen: comparten *pagar ≈ pay* y *agua ≈ water*». Lo mismo pasa con «Arreglar la gotera del techo» y «Fix the roof leak»: *gotera ≈ leak* y *techo ≈ roof*.

- **No es IA.** Es una lista fija de 41.140 parejas de palabras, hecha por lingüistas: sale del [Open Multilingual Wordnet 2.0](https://github.com/omwn/omw-data), con el Multilingual Central Repository 3.0 en español y Princeton WordNet 3.0 en inglés. Es un archivo de unos 760 KB (777.917 bytes) que viene dentro del instalador; comprimido ahí, suma unos 0,3 MB a la descarga. Nunca se busca en internet. Mira [Sin IA](Privacidad#sin-ia).
- **Una sola pareja por palabra**, la de su sentido más usado. Por eso a veces no es la que esperas: *cuenta* da *tally*, no *bill*. En el ejemplo de arriba, lo que une es *pagar* y *agua*.
- **Una palabra que se escribe igual en los dos idiomas**, como *hotel*, no está en la lista: ya se une como palabra en común.
- **Solo une dos idiomas distintos:** nunca dos cosas en español ni dos en inglés, aunque digan lo mismo con otras palabras.
- **El idioma de cada cosa** es el que declara el documento, si lo dice. Si no, Tisty lo deduce de palabras como «el», «la», «de» o «the», «of», «to», y por las tildes y la ñ. Una cosa en inglés nunca se lee como español.
- **Si no puede deducirlo, no la une por diccionario.** Una tarea que solo dice «Router», o «Rina's yearly vaccine», no trae ninguna de esas palabras. Las palabras en común y el tipeo sí la siguen uniendo.
- **Solo la app lee el diccionario**, en su pasada de baja prioridad. La puerta de tu agente no lo carga: tu agente ve en la ficha los parecidos que la app ya guardó.

### Lo que vale para las tres

Un tipeo o una traducción cuenta como una palabra en común: dos cosas se parecen solo si comparten al menos dos palabras de peso, y al menos una escrita distinta en cada lado. En la traducción, una palabra que cada lado escribió distinto, como *tarjeta* y *card*, cuenta aunque aparezca en hasta el 25 % de lo que guardas, no solo en el 5 %.

- **Como mucho tres parecidos por cosa**, y cada punta tiene que estar entre las tres más cercanas de la otra.
- **El umbral sale de lo que guardas**, no de una cifra fija: el parecido tiene que superar lo que alcanza un par de cada mil en tu base.
- **Lo que ya está unido no se marca como parecido**: si una línea de la red ya los une (una cita, una mención, una parte y su tarea, el documento donde nació una tarea), o si son una página y su libro, un documento y su versión en otro idioma, o una tarea y su página.
- **Un documento en otro idioma** tiene su propio lugar entre los tres, para que lo que comparte con otro idioma no quede tapado por parientes del mismo.
- **De cada cosa se guardan hasta 256 palabras**, las más dichas, y **solo de su prosa**: lo que la página muestra como texto. No cuentan el código, en bloque o en línea, el HTML, las propiedades, las direcciones de los enlaces, las rutas, los nombres de archivo, las etiquetas, los íconos, los números solos ni las cadenas largas de letras y números, como un identificador o un hash. Tampoco las palabras que solo dicen cómo se arma un texto, como *negrita*, *cursiva*, *tabla* o *enlace*, en español o en inglés. Antes, dos documentos con el mismo trozo de HTML se unían por *span* y *div*; ahora solo los une lo que dicen.
- **De una tarea cuentan** su título, su descripción, sus pasos y su página. Sus notas de avance no: cuentan cómo te fue, muchas veces con las mismas palabras para cualquier tarea, como «Quedé de llamar el lunes», y unían tareas que no tenían nada que ver.

**Cuándo.** Una pasada de baja prioridad, en segundo plano, mira cada 30 segundos y espera a que dejes de escribir, cinco minutos como mucho, antes de comparar. Las palabras de lo que cambió se guardan en el momento. Cuando una versión nueva de Tisty cambia la base de Brainsty, esta se rehace una vez, en el primer arranque: mientras tanto la red dice «Brainsty todavía está leyendo lo que guardas. Vuelve a abrirlo en un momento.».

**Lo que nunca pasa por aquí.** Las tareas ocultas y las descartadas, todo tu [Diario](Documentos#diario), aunque lo abras a los agentes, y los documentos de la carpeta [[Agentes|Carpeta-AGENTS]] no guardan palabras: nunca son parecidos de nada. El aviso que Tisty pone sobre algo que parece una credencial tampoco cuenta. Ocultar o descartar una tarea saca sus parecidos en la misma ronda.

**Dónde se ven:**

- **En la red**, como una curva ámbar: de cerca entre dos nodos a la vista, y junto al nodo que eliges desde cualquier distancia. Qué tan opaca es se mide entre los parecidos de tu base: la más opaca es la que más se parece.
- **En la ficha**, en su sección **Parecidos**, aparte de las conexiones. Se muestran como las guarda la base, sin tildes y en minúscula. Una palabra en común va sola, *presupuesto*; un tipeo o una traducción, como pareja, *impresora ≈ imrpesora* o *pagar ≈ pay*. El orden de la pareja es el que guardó la base, el mismo desde las dos fichas.
- **En la ficha que lee tu agente**, en `guessed`, aparte de todo lo que alguien escribió. Cada parecido trae su título, su estado, su puntaje, lo que comparten y `method`, que dice cómo se unieron: `words`, `typo` o `dictionary`. La frase que lo acompaña lo dice con palabras: «shares» para las palabras en común, «spelled almost the same:» para un tipeo o un plural, como `vacuna≈vacunas`, y «alike by translation:» para el diccionario. Las reglas le piden leer los dos antes de unir nada. Una pareja llega como `a≈b`, sin espacios, y nunca es una palabra que los dos escribieron igual:

```json
> brain {"task": "…Pedir presupuesto del techo…"}
{ "guessed": [
    { "id": "…", "kind": "doc", "title": "Arreglos de la casa", "state": "open", "score": 0.42, "shared": ["presupuesto", "techo"], "method": "words" } ] }
"Guessed alike by Tisty, linked by no one: \"Arreglos de la casa\" […] (open) (shares presupuesto, techo)."

> brain {"task": "…La imrpesora nueva no imprime en color…"}
{ "guessed": [
    { "id": "…", "kind": "task", "title": "Instalar la impresora nueva", "state": "open", "score": 0.55, "shared": ["impresora≈imrpesora", "nueva"], "method": "typo" } ] }
"Guessed alike by Tisty, linked by no one: \"Instalar la impresora nueva\" […] (open) (spelled almost the same: impresora≈imrpesora, nueva)."

> brain {"task": "…Pagar la cuenta del agua…"}
{ "guessed": [
    { "id": "…", "kind": "task", "title": "Pay the water bill", "state": "open", "score": 0.61, "shared": ["pagar≈pay", "agua≈water"], "method": "dictionary" } ] }
"Guessed alike by Tisty, linked by no one: \"Pay the water bill\" […] (open) (alike by translation: pagar≈pay, agua≈water)."
```

Un parecido aparece solo cuando sus dos puntas están a la vista: en la red, las dos tienen que estar dibujadas; para un agente, las dos tienen que pasar su puerta, así que una tarea oculta o descartada nunca se nombra a través de un parecido. Un parecido no suma conexiones a la cuenta de arriba ni cuenta para el tope de 12.000 nodos. Y `read`, que trae el texto, no trae parecidos: van solo en la ficha de `brain`.

## Lo que conviene saber

- **Se arma sola, en este equipo.** Tisty lee cómo se unen tus cosas en segundo plano, sin frenar lo que haces. Un enlace recién escrito llega a la base a los pocos segundos.
- **La red se lee al entrar.** Lo que escribiste en otra vista ya está la próxima vez que entras a **Brainsty**. Un parecido nuevo tarda un poco más: espera la pasada de baja prioridad.
- **Recién instalado**, mientras lee todo por primera vez, la red dice «Brainsty todavía está leyendo lo que guardas. Vuelve a abrirlo en un momento.». Con nada guardado: «Todavía no hay nada que dibujar: escribe un documento o una tarea y aparecerá aquí.».
- **Hasta 12.000 nodos y 40.000 conexiones.** Más allá, la red guarda lo último que tocaste, sus vecinos, tus documentos y las tareas en curso, y dice arriba cuánto quedó fuera: «312 nodos y 940 conexiones quedaron fuera». Pasados los 6000 nodos, los nodos se achican para que la red entera ocupe lo mismo en el panel.
- **Lo que apartas no aparece.** Una tarea que ocultaste o descartaste no se dibuja, ni como nodo ni como punta de una línea, y una etiqueta o un archivo que solo ella llevaba se va con ella. Tampoco las páginas que cuelgan de ella, ni tu Diario con sus páginas, aunque lo abras a los agentes. Nada apunta a ellos: ni un parecido, ni un viajero, ni **Apuntan aquí**, y la red no arranca desde uno de ellos aunque sea lo último que tocaste. Lo archivado sí está: **Archivadas** deja ver los documentos que archivaste. Por ejemplo, Jaime descarta «Cambiar de compañía de internet» con **No la voy a hacer**: la próxima vez que entra a Brainsty, la tarea, su página de notas y la curva ámbar que la unía a «Router» ya no están. Para tu agente vale lo mismo: donde enlazaste una de esas tareas lee «a task out of sight» (mira [[Cómo trabaja un agente|Cómo-trabaja-un-agente]]).
- **Nada sale del equipo.** La red y los parecidos se calculan en cada equipo y se rehacen solos: no se sincronizan, ni van a Folio, a un `.tistyx`, a una exportación o a un respaldo. Lo que viaja son tus documentos y tareas, y cada equipo vuelve a dibujarla. Mira [[Privacidad|Privacidad#qué-se-guarda]].
- **Sin tarjeta gráfica dedicada** la red se dibuja igual; acercarse y alejarse cuesta un poco más.

## Para seguir

- [[Enlazar documentos|Enlazar-documentos]]: las líneas punteadas y sólidas entre documentos.
- [[Tareas en documentos|Tareas-en-documentos]]: las líneas entre un documento y sus tareas.
- [[Cómo trabaja un agente|Cómo-trabaja-un-agente]]: la misma red, leída por tu agente.
- [[Documentos]]: carpetas y etiquetas, que dan los colores y las zonas; [[el enlace de un documento|Documentos#el-enlace-de-un-documento]], el que copias desde la ficha, y sus [[versiones anteriores|Documentos#versiones-anteriores]].
- [[La carpeta Agentes|Carpeta-AGENTS]]: los archivos de tus agentes, ocultos en la red hasta **Mostrar AGENTS**.
- [[Privacidad|Privacidad#sin-ia]]: por qué los parecidos no son IA, y qué guarda Brainsty en este equipo.
