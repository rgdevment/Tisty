# La red de Brainsty

Llevaba años guardando cosas en Tisty y no tenía forma de verlas juntas. Sabía que «Rina» citaba la tarea de su vacuna y que el acta del condominio había creado tres tareas, pero cada relación vivía dentro de su archivo. Brainsty dibuja todo lo que guardas, con lo que lo une, como una sola red: de lejos, un mapa de tus proyectos; de cerca, cada documento y cada tarea con sus vecinos.

Esta página cuenta cómo abrirla, qué significa cada forma y cada línea, cómo moverte por ella y cómo filtrarla. Las uniones que dibuja las haces tú al escribir: [[Enlazar documentos|Enlazar-documentos]] y [[Tareas en documentos|Tareas-en-documentos]]. Tu agente lee la misma red como un índice: [[Cómo trabaja un agente|Cómo-trabaja-un-agente]].

![La ventana de Tisty con la barra lateral a la izquierda y Brainsty marcado en ella; en el panel de contenido, la red alejada: arriba la cuenta, «589 nodos · 1252 conexiones», cada carpeta y cada lista de su color, los documentos como cuadrados, las tareas como círculos y las etiquetas como puntos morados, y abajo a la izquierda la Leyenda, plegada](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/es/red.png)

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
- **Cada vez que entras, empieza de nuevo.** Se lee otra vez, con lo que escribiste entretanto, y arranca sin nodo elegido, sin búsqueda y sin filtros. Lo único que recuerda es si dejaste abierta la [leyenda](#la-leyenda).
- **Sigue a Tisty**, como cualquier vista: el tema claro u oscuro y el idioma. En oscuro, la red toma los colores oscuros.

## Qué es cada cosa

| En la red | Es | Cómo se ve |
| :-- | :-- | :-- |
| Cuadrado | un documento o una página de un libro | del color de su carpeta; una página, más pequeña junto a su documento |
| Cuadrado grande | una carpeta | en medio de sus documentos, unida a cada uno |
| Rombo | una lista | en medio de sus tareas, unida a cada una |
| Círculo | una tarea | del color de su lista; en curso, viva; terminada, más tenue; descartada o archivada, tenue y pequeña |
| Punto morado | una etiqueta | unido a todo lo que la lleva |
| Punto gris | una imagen o un archivo | unido a lo que lo usa; de cerca, con su miniatura |

Cada carpeta y cada lista que tiene algo en la red es un nodo, en medio de lo suyo. No empuja a nadie: la red se acomoda por sus otras líneas y la carpeta o la lista se pone donde quedó lo suyo, así que la red conserva su forma. Además, cada una tiñe su zona con su color. Solo existen en la vista: tu agente no las ve como nodos en su índice. La carpeta [[Agentes|Carpeta-AGENTS]] tiene un color propio, el coral, que ninguna otra carpeta puede elegir: su zona se tiñe de coral.

| Línea | Une | Cuándo existe |
| :-- | :-- | :-- |
| Sólida | una cita | una tarjeta de otro documento o de una tarea, sola en su párrafo |
| Punteada | una mención | un enlace dentro de una frase a otro documento o a una tarea |
| Fina | la estructura | una página y su libro, una parte y su tarea, una tarea y el documento donde nació, una idea y su documento, una carpeta y sus documentos, una lista y sus tareas |
| Fina y tenue | una etiqueta | lo que lleva `#etiqueta` y su punto |
| Fina, hacia un archivo | un uso | lo que muestra una imagen o adjunta un archivo |
| Curva ámbar | un parecido | dos cosas que comparten palabras poco comunes, una palabra con un error de tipeo o una traducción, y que nadie unió; más opaca cuanto más se parecen |

Cada línea recta está escrita en alguna parte, y se puede explicar. La curva ámbar no: es lo que Tisty adivina, nunca es recta, sólida ni punteada, para no confundirse con nada escrito, y la ficha del nodo dice por qué. Lo cuenta [Parecidos](#parecidos).

## La leyenda

No hace falta aprenderse estas tablas: **Leyenda**, abajo a la izquierda de la red, dice qué significa cada forma, cada color y cada línea de tu red. Viene plegada, para no tapar la red; un clic en su título la abre y otro la pliega. Cada equipo recuerda cómo la dejaste.

Abierta, arriba lleva dos interruptores, **Viajeros** y **Girar**, que cuenta [La red viva](#la-red-viva). Debajo tiene cuatro partes. Cada muestra se dibuja con el mismo pincel que la red, así que lo que ves en la leyenda es exactamente lo que ves en la red, también en tema oscuro:

| Parte | Qué muestra |
| :-- | :-- |
| **Qué es** | la forma y el tono de cada clase que hay en tu red: **Documento**, **Página**, **Tarea**, **Etiqueta**, **Archivo**, **Carpeta** y **Lista**. Una clase que no tienes no aparece |
| **Color: su carpeta o lista** | las carpetas y listas con color que tienen algo en la red, primero las más llenas. Muestra seis; **N más** abre el resto y **Menos** las vuelve a esconder. Al pie: «Sin color, toma el tono de su clase.» |
| **Estado** | cómo se ve una tarea **En curso**, una de las **Terminadas** y una de las **Archivadas** |
| **Líneas, de cerca** | **Cita**, **Menciona**, **Parte de (página, parte, carpeta o lista)**, **Etiquetada** y **Usa un archivo**, solo las que hay en tu red; y **Parecidos**, la curva ámbar, si Tisty adivinó alguno. Con el puntero sobre **Parecidos**, te dice qué son |

Por ejemplo, Marisol tiene once carpetas y listas con color. Su leyenda muestra las seis más llenas, **Casa**, **Trabajo**, **Mascotas**, **Viajes**, **Compras** y **Salud**, y debajo **5 más**. Una carpeta sin color, como **Recetas**, no está en la lista: sus documentos se ven con el tono de los documentos.

![La Leyenda abierta en la esquina inferior izquierda de la red: arriba los interruptores Viajeros y Girar, con su punto encendido; Qué es con Documento, Página, Tarea, Etiqueta, Archivo, Carpeta y Lista; Color: su carpeta o lista con Casa, Trabajo, Mascotas, Viajes, Compras y Salud, «5 más» y «Sin color, toma el tono de su clase.»; Estado con En curso, Terminadas y Archivadas; Líneas, de cerca con Cita, Menciona, Parte de (página, parte, carpeta o lista), Etiquetada, Usa un archivo y Parecidos en ámbar](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/es-leyenda.png)

## La red viva

Mientras la miras, la red se mueve: gira y la recorren viajeros por sus líneas. Si la prefieres quieta, la apagas desde la leyenda.

- **Gira como un globo**, sola, una vuelta por minuto. Lo que queda atrás se ve más chico y más tenue, y con la red girada las zonas de color se ocultan y se ven menos títulos.
- **Se detiene cuando la necesitas quieta:** mientras el puntero está sobre la forma de la red, un momento después de acercarte o alejarte con la rueda y mientras tienes un nodo elegido. En el espacio vacío alrededor sigue girando.
- **La giras tú** arrastrando con el botón derecho, o con `Shift` y el izquierdo. El arrastre normal sigue moviendo la vista, como siempre.
- **Los viajeros** son puntos de luz que recorren las conexiones en viajes largos: pasan de un nodo al siguiente cada cinco segundos, dejan una estela corta y destellan al pasar por una etiqueta o por un nodo muy conectado. Prefieren los documentos y las páginas, y nunca pasan por lo que los filtros esconden.
- **Cuántos viajeros** depende del tamaño de la red:

| Nodos en la red | Viajeros |
| :-- | :-- |
| menos de 60 | 2 |
| de 60 a 99 | 4 |
| de 100 a 149 | 6 |
| de 150 a 249 | 8 |
| 250 o más | 10 |

- **Dos interruptores**, arriba en la [leyenda](#la-leyenda) abierta: **Viajeros** y **Girar**, cada uno con un punto que se enciende. Sin **Girar**, la red queda plana; con los dos apagados, queda como una red quieta. Cada equipo recuerda cómo los dejaste. Si tu sistema pide reducir el movimiento, arrancan apagados.
- **Solo corre mientras la ves.** Al pasar a otra vista se detiene, igual que el resto de la red.
- **Lo que cuesta**, si te interesa: `Ctrl` + `Shift` + `M` muestra junto a la leyenda cuántos milisegundos toma cada cuadro y cuántos cuadros por segundo dibuja, por ejemplo «2.10 ms por cuadro · 60 cuadros por segundo». Otro `Ctrl` + `Shift` + `M` lo esconde.

Por ejemplo, Lucas abre Brainsty con 180 nodos: ve ocho viajeros cruzando la red mientras gira. Deja el puntero sobre la zona de **Trabajo** para leer un título y el globo se detiene; lo saca al espacio vacío y vuelve a girar. Para presentar su pantalla prefiere la red quieta: abre la **Leyenda** y apaga **Viajeros** y **Girar**, y la próxima vez en ese equipo abre igual.

## De lejos y de cerca

La red cambia lo que muestra según la distancia, y cada detalle aparece de a poco al acercarte:

- **De lejos**, con la rueda hacia atrás: los nodos y sus uniones forman la silueta de un cerebro visto de lado, sin títulos y con líneas tenues. Sirve para ver dónde hay mucho y dónde hay poco.
- **A media distancia**, como se abre: cada carpeta y cada lista tiñen su zona, y llevan título solo los nodos más conectados.
- **De cerca**: todos los títulos, cada tipo de línea, las etiquetas como `#etiqueta`, junto a cada imagen su miniatura y, dentro de cada nodo, el ícono de lo que es: documento, página, tarea (con un tic si está hecha), etiqueta, archivo, carpeta o lista. El ícono se atenúa con lo que no está elegido y no cuesta nada de lejos. Las curvas ámbar aparecen aquí, de a poco, entre dos nodos que están a la vista; junto al nodo que eliges se ven desde cualquier distancia.

![Brainsty en el panel de contenido, junto a la barra lateral, con la misma red de lejos: los nodos forman la silueta de un cerebro visto de lado, sin títulos y con líneas tenues, y la Leyenda plegada abajo a la izquierda](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/es-red-lejos.png)

Con la rueda acercas y alejas; arrastrando te mueves, y arrastrando con el botón derecho, o con `Shift`, giras el globo. Al primer toque la red deja de seguir el punto donde arrancó.

## Elegir, ver y abrir

**Un clic en un nodo** lo resalta junto con sus vecinos, atenúa el resto y abre su **ficha**, a la derecha. La ficha no muestra el texto del documento: muestra sus datos, para que sepas qué es, de cuándo es y con qué se une antes de abrirlo. Arriba van su título y una línea con qué es, su carpeta o su lista y cuántas conexiones tiene: «Documento · Mascotas · 6 conexiones».

Debajo vienen sus datos, según lo que elegiste:

| Eliges | La ficha muestra |
| :-- | :-- |
| Un documento | **Escrito por última vez**, **Creado**, **Tamaño**, con su peso y cuántas palabras tiene, e **Idioma**, como «ES» |
| Una página de un libro | lo mismo, y primero **Página de**, con el título de su libro |
| Una tarea | **Estado**, como «En curso · Por vencer»; **Día**, **Plazo**, **Parte de**, con su tarea madre, **Partes**, con cuántas tiene, **Pasos**, como «1 de 3 hechos», y **Origen**, si llegó con uno |
| Una etiqueta, un archivo, una carpeta o una lista | solo la línea de arriba y sus conexiones |

El estado de una tarea suma, después de un punto, cómo va su fecha: **Por venir**, **Para hoy**, **Atrasada**, **En curso**, **Por vencer** o **Vencida**. Una fila sin dato no aparece: una tarea sin plazo no muestra **Plazo**.

Después, en este orden, lo que hay:

- **Etiquetas**, cada una como `#rina`.
- **Conexiones**, contadas por tipo: «2 citas 3 menciones».
- **Apuntan aquí**: los documentos y las tareas que lo citan o lo mencionan, hasta seis, y «N más» si hay otros. Un clic en uno lo elige en la red y abre su ficha; uno que no está en la red, como lo que quedó fuera del tope de nodos, se lee, pero no se elige.
- **Parecidos**, en ámbar, si Tisty le adivina alguno: cada uno con su título y su porqué, «Tisty lo adivina: comparten *presupuesto* y *techo*». Un clic en el título lo elige en la red. Lo cuenta [Parecidos](#parecidos).
- **Enlace**, en un documento, una página o una tarea: su dirección y **Copiar enlace**, que pasa a decir «Enlace copiado». Es el mismo enlace que copias desde el panel de un documento, y sirve para pegarlo en un chat, en tu navegador o dárselo a tu agente. El de un documento y el de una tarea se ven así:

```text
tisty://doc/mascotas/rina?id=…
tisty://task?id=…
```

Al pie, **Abrir**. La cruz de arriba, **Cerrar la vista previa**, o `Esc`, suelta el nodo y la red vuelve a verse entera.

- **Abrir**, `Enter` o **un doble clic** en el nodo lo abre en la misma ventana, en el lugar de la red: un documento en la Biblioteca, una tarea junto a su lista, una etiqueta en su vista, una carpeta en **Documentos** y una lista en **Listas**. Para volver a la red, otro clic en **Brainsty**; abre de nuevo entera, y lo que abriste queda como punto de partida de las flechas.
- **Una imagen o un archivo** tienen ficha, pero no **Abrir** ni **Enlace**: se abren desde el documento o la tarea que los usa, su vecino en la red.

![«Rina» elegida de cerca en Brainsty, en el panel de contenido junto a la barra lateral: sus vecinos con título, el resto atenuado y, a la derecha, su ficha: «Documento · Mascotas · 6 conexiones», las filas Escrito por última vez, Creado, Tamaño con sus palabras e Idioma ES, la etiqueta #rina, Conexiones por tipo, Apuntan aquí con lo que la cita, Parecidos con «Paseos del sábado» y «Tisty lo adivina: comparten rina y parque», el Enlace con Copiar enlace y el botón Abrir; en la red, una curva ámbar une a Rina con «Paseos del sábado»](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/es/red-cerca.png)

## Buscar

El buscador está arriba a la izquierda, **Buscar en la red**; `/` lo enfoca desde cualquier parte de la red. Escribe parte de un título: no importan los acentos ni las mayúsculas. Debajo aparece la lista de lo que calza, cada uno con lo que es («Documento», «Tarea», «Etiqueta»…): primero lo que empieza igual, después lo que tiene una palabra que empieza así y al final lo demás. `Enter` elige el primero, o un clic el que quieras: la red se centra en él y abre su ficha. Si nada se llama así, dice «Nada con ese nombre».

## Filtrar

El embudo junto al buscador, **Filtros**, abre lo que se puede dejar fuera. Cada opción es un botón que se enciende y se apaga, y la red se redibuja con lo que queda:

- **Mostrar**: **Documentos**, **Páginas**, **Tareas**, **Etiquetas**, **Archivos**, **Carpetas**, **Listas** y **Parecidos**. **Parecidos** viene encendido; apagado, la red no dibuja ninguna curva ámbar y la ficha no los lista. Una carpeta o una lista se esconde sola cuando los filtros esconden todo lo suyo: con solo **Tareas**, las carpetas se van y las listas quedan.
- El estado de las tareas: **En curso**, **Terminadas** y **Archivadas**.
- **Tocado**: **Cuando sea**, **Últimos 7 días**, **Últimos 30 días** o **Último año**, según el último día en que cambió cada cosa.
- **Etiqueta**: elige una de la lista y deja solo lo que la lleva; **Cualquier etiqueta** la quita.

Con algún filtro puesto, el embudo se tiñe y aparece **Mostrar todo**, que devuelve la red entera.

![Brainsty en el panel de contenido con Filtros abierto: Documentos, Tareas y Parecidos encendidos, En curso, Últimos 30 días y la etiqueta rina, el embudo teñido, Mostrar todo a la vista y la red con lo que queda](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/es-filtros.png)

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
2. `/`, `rina`, `Enter`. El globo se detiene, la red se centra en «Rina», el documento de la carpeta **Mascotas**, y abre su ficha: «Documento · Mascotas · 6 conexiones», escrito por última vez hace un mes, en español.
3. Te acercas con la rueda: aparecen sus vecinos con título, «Veterinaria» unida por una línea punteada (la mencionas en una frase), la tarjeta sólida de «Vacuna anual de Rina», la foto del paseo con su miniatura, el cuadrado grande de **Mascotas**, en medio de sus documentos, y una curva ámbar hacia «Paseos del sábado», que nadie enlazó. La ficha de Rina dice por qué, en **Parecidos**: «Tisty lo adivina: comparten *rina* y *parque*».
4. Un clic en «Vacuna anual de Rina»: su ficha dice «En curso · Por vencer», **Plazo** el viernes y **Pasos** «1 de 3 hechos», y en **Apuntan aquí** está «Rina», que la cita. **Copiar enlace** te deja su enlace para pasárselo a Yisset, que la va a llevar.
5. **Abrir**: la tarea se abre junto a su lista, en el lugar de la red, y ahí marcas «Pedir hora».
6. Otro clic en **Brainsty**: la red abre de nuevo, entera; la tarea queda como punto de partida de las flechas. **Filtros**, **Etiqueta** `rina` y **Tocado** **Últimos 30 días**: quedan sus dos documentos y las tareas en curso que se movieron este mes.
7. `Esc` hasta volver: primero suelta lo que elegiste y el último te devuelve a la tarea. Los filtros no se guardan: la próxima vez la red arranca entera.

## Parecidos

Dos cosas pueden hablar de lo mismo sin que nadie las haya unido: la tarea «Pedir presupuesto del techo» y el documento «Arreglos de la casa», que habla del techo y de su presupuesto. Brainsty las anota como **parecidas**, aparte de las líneas que escribiste.

Tisty busca parecidos con tres pistas, en este orden. Cada pista solo llena los lugares que la anterior dejó libres, hasta tres parecidos por cosa, y cada parecido guarda lo que dice por qué.

### Palabras en común

Dos cosas se parecen cuando comparten al menos dos **palabras de peso**: de esas que aparecen en pocas de las cosas que guardas, en el 5 % o menos. *Presupuesto* y *techo* pesan; *semana* o *hacer*, que están en todas partes, no. Las palabras se comparan sin tildes ni mayúsculas, así que *Techo* y *techo* son la misma.

Sirve también entre idiomas cuando lo que comparten son nombres: una tarea en inglés y un documento en español que hablan de *Folio* y de *Glama* se parecen.

### Un error de tipeo

Escribes rápido y anotas «La imrpesora nueva no imprime en color». Una semana después, Jaime te pide «Instalar la impresora nueva». Solo comparten *nueva*, pero *imrpesora* está a un cambio de *impresora*, y Tisty las une: «Tisty lo adivina: comparten *impresora ≈ imrpesora* y *nueva*». En inglés funciona igual, con palabras de seis letras o más.

- **Un cambio** es una letra de más, una de menos, una distinta o dos letras vecinas que cambiaron de lugar, como *rp* por *pr*.
- **Solo palabras de seis letras o más.** *Casa* y *cosa* son dos palabras, no un error.
- **Nunca palabras con dígitos.** *capitulo1* y *capitulo2* se escribieron distintas a propósito.
- De las dos formas, la que más usas en lo que guardas es la que cuenta como la buena.

### Una traducción

Anotas «Pagar la cuenta del agua» y, otro día, tu agente anota en inglés «Pay the water bill». Hablan de lo mismo y no comparten ni una palabra. Tisty las une con un diccionario español–inglés: «Tisty lo adivina: comparten *pagar ≈ pay* y *agua ≈ water*». Lo mismo pasa con «Arreglar la gotera del techo» y «Fix the roof leak»: *gotera ≈ leak* y *techo ≈ roof*.

- **No es IA.** Es una lista fija de 41.140 parejas de palabras, hecha por lingüistas: sale del [Open Multilingual Wordnet 2.0](https://github.com/omwn/omw-data), con el Multilingual Central Repository 3.0 en español y Princeton WordNet 3.0 en inglés. Es un archivo de unos 760 KB (777.917 bytes) que viene dentro del instalador; comprimido ahí, suma unos 0,3 MB a la descarga. Nunca se busca en internet. Mira [Sin IA](Privacidad#sin-ia).
- **Una sola pareja por palabra**, la de su sentido más usado. Por eso a veces no es la que esperas: *cuenta* da *tally*, no *bill*. En el ejemplo de arriba, lo que une es *pagar* y *agua*.
- **Una palabra que se escribe igual en los dos idiomas**, como *hotel*, no está en la lista: ya se une como palabra en común.
- **Solo une dos idiomas distintos:** nunca dos cosas en español ni dos en inglés, aunque digan lo mismo con otras palabras.
- **El idioma de cada cosa** es el que declara el documento, si lo dice. Si no, Tisty lo adivina por palabras como «el», «la», «de» o «the», «of», «to», y por las tildes y la ñ. Una cosa en inglés nunca se lee como español.
- **Si no sabe adivinarlo, no la une por diccionario.** Una tarea que solo dice «Router», o «Rina's yearly vaccine», no trae ninguna de esas palabras. Las palabras en común y el tipeo sí la siguen uniendo.
- **Solo la app lee el diccionario**, en su pasada de baja prioridad. La puerta de tu agente no lo carga: tu agente ve en la ficha los parecidos que la app ya guardó.

### Lo que vale para las tres

Un tipeo o una traducción cuenta como una palabra en común: dos cosas se parecen solo si comparten al menos dos palabras de peso, y al menos una escrita distinta en cada lado. En la traducción, una palabra que cada lado escribió distinto, como *tarjeta* y *card*, cuenta aunque aparezca en hasta el 25 % de lo que guardas, no solo en el 5 %.

- **Como mucho tres parecidos por cosa**, y cada punta tiene que estar entre las tres más cercanas de la otra.
- **El umbral sale de lo que guardas**, no de una cifra fija: el parecido tiene que superar lo que alcanza un par de cada mil en tu base.
- **Lo que ya está unido no se adivina**: si una línea de la red ya los une (una cita, una mención, una parte y su tarea, el documento donde nació una tarea), o si son una página y su libro, un documento y su versión en otro idioma, o una tarea y su página.
- **Un documento en otro idioma** tiene su propio lugar entre los tres, para que lo que comparte con otro idioma no quede tapado por parientes del mismo.
- **De cada cosa se guardan hasta 256 palabras**, las más dichas. Los números solos y las cadenas largas de letras y números, como un identificador o un hash, no cuentan.

**Cuándo.** Una pasada de baja prioridad, en segundo plano, mira cada 30 segundos y espera a que dejes de escribir, cinco minutos como mucho, antes de comparar. Las palabras de lo que cambió se guardan en el momento. Cuando una versión nueva de Tisty cambia la base de Brainsty, esta se rehace una vez, en el primer arranque: mientras tanto la red dice «Brainsty todavía está leyendo lo que guardas. Vuelve a abrirlo en un momento.».

**Lo que nunca pasa por aquí.** Las tareas ocultas y las descartadas, y todo tu [Diario](Documentos#diario), no guardan palabras, aunque abras el Diario a los agentes. Ocultar una tarea saca sus parecidos en la misma ronda.

**Dónde se ven:**

- **En la red**, como una curva ámbar: de cerca entre dos nodos a la vista, y junto al nodo que eliges desde cualquier distancia. Qué tan opaca es se mide entre los parecidos de tu base: la más opaca es la que más se parece.
- **En la ficha**, en su sección **Parecidos**, aparte de las conexiones. Se muestran como las guarda la base, sin tildes y en minúscula. Una palabra en común va sola, *presupuesto*; un tipeo o una traducción, con lo que escribió cada lado, *impresora ≈ imrpesora* o *pagar ≈ pay*.
- **En la ficha que lee tu agente**, en `guessed`, aparte de todo lo que alguien escribió. Cada parecido trae su título, su puntaje y lo que comparten, y las reglas le piden leer los dos antes de unir nada. Una pareja llega como `a≈b`, sin espacios: es una traducción o un tipeo, nunca una palabra que los dos escribieron:

```json
> brain {"task": "…Pedir presupuesto del techo…"}
{ "guessed": [
    { "id": "…", "kind": "doc", "title": "Arreglos de la casa", "score": 0.42, "shared": ["presupuesto", "techo"] } ] }
"Guessed alike by Tisty, linked by no one: \"Arreglos de la casa\" […] (shares presupuesto, techo)."

> brain {"task": "…Pagar la cuenta del agua…"}
{ "guessed": [
    { "id": "…", "kind": "task", "title": "Pay the water bill", "score": 0.61, "shared": ["pagar≈pay", "agua≈water"] } ] }
"Guessed alike by Tisty, linked by no one: \"Pay the water bill\" […] (shares pagar≈pay, agua≈water)."
```

![La ficha de «Pagar la cuenta del agua» de cerca en Brainsty: «Tarea · Casa» con sus conexiones, Estado «En curso · Para hoy», y en Parecidos «Pay the water bill» con «Tisty lo adivina: comparten pagar ≈ pay y agua ≈ water»; en la red, una curva ámbar une las dos tareas](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/es-gemela.png)

Un parecido aparece solo cuando sus dos puntas están a la vista: en la red, las dos tienen que estar dibujadas; para un agente, las dos tienen que pasar su puerta, así que una tarea oculta nunca se nombra a través de un parecido. Un parecido no suma conexiones a la cuenta de arriba ni cuenta para el tope de 12.000 nodos.

## Lo que conviene saber

- **Se arma sola, en este equipo.** Tisty lee cómo se unen tus cosas en segundo plano, sin frenar lo que haces. Un enlace recién escrito llega a la base a los pocos segundos.
- **La red se lee al entrar.** Lo que escribiste en otra vista ya está la próxima vez que entras a **Brainsty**. Un parecido nuevo tarda un poco más: espera la pasada de baja prioridad.
- **Recién instalado**, mientras lee todo por primera vez, la red dice «Brainsty todavía está leyendo lo que guardas. Vuelve a abrirlo en un momento.». Con nada guardado: «Todavía no hay nada que dibujar: escribe un documento o una tarea y aparecerá aquí.».
- **Hasta 12.000 nodos y 40.000 conexiones.** Más allá, la red guarda lo último que tocaste, sus vecinos, tus documentos y las tareas en curso, y dice arriba cuánto quedó fuera: «312 nodos y 940 conexiones quedaron fuera». Pasados los 6000 nodos, los nodos se achican para que la red entera ocupe lo mismo en el panel.
- **Lo oculto no aparece.** Una tarea que ocultaste no se dibuja, ni como nodo ni como punta de una línea, y una etiqueta o un archivo que solo ella llevaba se va con ella. Una tarea descartada sí aparece, tenue y pequeña.
- **Nada sale del equipo.** La red y los parecidos se calculan en cada equipo y se rehacen solos: no se sincronizan, ni van a Folio, a un `.tistyx` o a una exportación. Lo que viaja son tus documentos y tareas, y cada equipo vuelve a dibujarla.
- **Sin tarjeta gráfica dedicada** la red se dibuja igual; acercarse y alejarse cuesta un poco más.

## Para seguir

- [[Enlazar documentos|Enlazar-documentos]]: las líneas punteadas y sólidas entre documentos.
- [[Tareas en documentos|Tareas-en-documentos]]: las líneas entre un documento y sus tareas.
- [[Cómo trabaja un agente|Cómo-trabaja-un-agente]]: la misma red, leída por tu agente.
- [[Documentos]]: carpetas y etiquetas, que dan los colores y las zonas.
