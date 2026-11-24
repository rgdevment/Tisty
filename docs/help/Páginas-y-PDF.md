# Páginas, saltos de página y PDF

## Un documento con páginas

Un año de actas, un libro por capítulos, la vida de un perro contada año a año. Eso no cabe en una sola hoja, y tampoco en una carpeta de papeles sueltos que nada ata.

Una **página** es un documento como cualquier otro (el mismo archivo `.md` en el disco, la misma salida) con una sola diferencia: dice de qué documento es. Hay un nivel y no más: una página no tiene páginas.

### Añadir una página

Escribe `/` donde quieras que empiece ese capítulo y elige **Una página nueva**, o **Una página que ya existe** si ya está escrita. Queda nombrada ahí mismo, dibujada como una tarjeta con su título, sus primeras palabras y su lugar en el libro.

En el árbol, un documento que sueltas encima de otro se vuelve página suya.

> [!IMPORTANT]
> El orden en que las páginas están nombradas en el texto es el orden en que van: en el árbol, en la exportación y al imprimir. Para mover un capítulo, abre **Páginas**, bajo la hoja, y usa las flechas de su fila, o `Alt` y una flecha sobre la fila, o corta y pega su bloque.

### El índice

Bajo la hoja del documento hay una fila **Páginas**: un clic la abre y muestra el índice, las páginas que el texto nombra, numeradas, y tras una raya las que no, cada una con **Ponerla en el texto** para darle su lugar. Dentro de una página ves arriba de qué documento es y en qué lugar va, flechas a sus hermanas, y abajo el paso a la siguiente.

Un enlace a una página en medio de un párrafo la señala sin darle lugar, como una nota al pie señala un capítulo sin serlo. Una página de la que solo hablaste así espera bajo la raya hasta que le pongas su tarjeta.

### Archivar una página

Una página se archiva sola con el botón derecho, y no deja su documento: se queda donde vive, atenuada y con el icono del archivo; se lee pero no se escribe. Archiva el documento entero y todas las páginas se van con él; desarchívalo y cada una vuelve como estaba: la que habías apartado tú sigue apartada.

## Salto de página

Enciende **Hojas** en el panel de un documento para elegir su papel (**A4**, **Carta** u **Oficio**, de 216 × 330 mm) y leerlo en esas hojas, verticales o, con **Horizontal** encendido, apaisadas: una hoja que se llena pasa sola a la siguiente, en pantalla igual que en papel. Un documento que ya pedía **Tabloide** lo conserva. Las hojas vienen apagadas hasta que las enciendes; el papel sigue elegido al apagarlas, su PDF sale en él, y lo que eliges viaja con el archivo en sus [propiedades](https://markdown.tisty.app/es/propiedades). En una ventana de solo lectura, **Hojas** cambia solo cómo lo ves y su PDF: el archivo queda como estaba.

Una raya sola en su línea (`---`) corta siempre la hoja, con hojas o sin ellas, y `Ctrl`/`⌘` + `Enter` la escribe mientras escribes. Con hojas encendidas, se marca como **Salto de página**. Cómo se escribe, y qué entra como uno desde LaTeX, Word o una página web, está en la [guía de Markdown](https://markdown.tisty.app/es/saltos-de-pagina).

## Diseños de documento

La receta que cocinas y el acta de una reunión no tienen por qué verse iguales. La fila de líneas de color del panel del documento, bajo el papel, cambia cómo se ve ese documento sin tocar ni una palabra. Cada línea es un diseño: deja el cursor sobre una, o llega con `Tab`, para leer su nombre, y la más gruesa es la que lleva.

| Diseño | Cómo se ve |
| --- | --- |
| **Default** | El que trae todo documento al nacer |
| **Bosque** | Verdes, con una línea bajo el título |
| **Coral** | Cálido, con una barra bajo el título |
| **Fiordo** | Azules y grises |
| **Violeta** | Violetas, con una barra bajo el título |
| **Propio** | Fondo, títulos, cuerpo y letra, a tu gusto |

Cada uno tiene sus colores claros y los oscuros, y sigue a la ventana. Sirve con hojas o sin ellas, y cambia la letra, el título, los encabezados y la cabecera de las tablas. Los diagramas conservan sus colores.

Por ejemplo, abre **Vacunas de Rina**, presiona la línea de **Bosque** y pon la ventana en oscuro: el papel queda verde oscuro y los títulos, verde claro. Presiona **Default**, la primera, y vuelve a como estaba.

- **Propio** se arma en el panel: presiona su línea, la última, y elige un fondo, el color de los títulos, el del cuerpo y una de nueve letras. Por ejemplo, **Arena**, **Vino**, **Café** y **Lora** para un recetario. Cada color trae su claro y su oscuro, cualquiera de ellos se lee bien con los demás, y los tamaños no cambian. Las letras vienen con Tisty, así que no se descarga nada.
- **Desde el texto:** un enlace a `tisty:design/coral`, o a cualquiera de los otros, le pone ese diseño al documento al presionarlo. La guía inicial los trae para probar; `tisty:design/tisty` vuelve.
- **Las notas, el código, los diagramas y los widgets** conservan sus colores en todos los diseños.
- **En papel y en el PDF** sale con los colores claros sobre papel blanco, también desde una ventana en oscuro.
- **En el archivo** es una línea de sus [propiedades](https://markdown.tisty.app/es/propiedades), `design: bosque` o `design: own/arena/vino/cafe/lora`, y con la primera línea del panel, **Default**, no hay ninguna. Un nombre que Tisty no conoce queda escrito y el documento se ve como **Default**. Si un documento tuyo ya tenía una propiedad propia con uno de esos cuatro nombres, como `design: coral`, ahora surte efecto: presiona la primera línea del panel o ponle otro nombre a la propiedad. Una propiedad tuya con cualquier otro valor queda como está hasta que elijas otro diseño en ese documento.
- **Con páginas**, un documento lleva un solo diseño y sus páginas también lo llevan, en pantalla, impresas y compartidas. Se elige en el documento; en una página no hay nada que elegir.
- **En varios idiomas**, el diseño es el mismo en todos: elegir uno, o unir un idioma, se lo pone a los demás. Una propiedad tuya llamada `design` queda como la escribiste.
- **En su propia ventana**, la que solo lee, la hoja también lleva el diseño; ahí no hay nada que elegir.

## El PDF propio de Tisty

Tisty hace su propio PDF y te lo muestra antes de exportarlo:

- Papel **A4**, **Carta** u **Oficio**, en vertical u horizontal.
- Sus propios márgenes.
- Sus imágenes van dentro; cualquier otro adjunto queda nombrado donde está.
- Si quieres, **quién lo escribió y cuándo**: un PDF sale de Tisty para siempre, así que lo que va dentro lo decides tú.
- La página con todo lo que dibuja en pantalla (el color del código, los avisos, los diagramas, las fórmulas y los iconos), con la letra que tiene con **Hojas** encendido.

Desde **Exportar**, en el panel del documento: **Vista previa del PDF** lo muestra y **Exportar a PDF…** lo guarda.
