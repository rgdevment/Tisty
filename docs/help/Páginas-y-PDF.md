# Páginas, saltos de página y PDF

## Un documento con páginas

Un año de actas, un libro por capítulos, la vida de un perro contada año a año. Eso no cabe en una sola hoja, y tampoco en una carpeta de papeles sueltos que nada ata.

Una **página** es un documento como cualquier otro (el mismo archivo `.md` en el disco, la misma salida) con una sola diferencia: dice de qué documento es. Hay un nivel y no más: una página no tiene páginas.

### Añadir una página

Escribe `/` donde quieras que empiece ese capítulo y elige **Una página nueva**, o **Una página que ya existe** si ya está escrita. Queda nombrada ahí mismo, dibujada como una tarjeta con su título, sus primeras palabras y su lugar en el libro.

En el árbol, un documento que sueltas encima de otro se vuelve página suya.

> [!IMPORTANT]
> El orden en que las páginas están nombradas en el texto es el orden en que van: en el árbol, en la exportación y al imprimir. Para mover un capítulo, usa las flechas de su fila en el índice del final del documento, o `Alt` y una flecha sobre la fila, o corta y pega su bloque.

### El índice

Al final del documento, en su propia hoja, va el índice: las páginas que el texto nombra, numeradas, y tras una raya las que no, a un clic de tener su lugar en el texto. Dentro de una página ves arriba de qué documento es y en qué lugar va, flechas a sus hermanas, y abajo el paso a la siguiente.

Un enlace a una página en medio de un párrafo la señala sin darle lugar, como una nota al pie señala un capítulo sin serlo. Una página de la que solo hablaste así espera bajo la raya hasta que le pongas su tarjeta.

### Archivar una página

Una página se archiva sola con el botón derecho, y no deja su documento: se queda donde vive, atenuada y con el icono del archivo; se lee pero no se escribe. Archiva el documento entero y todas las páginas se van con él; desarchívalo y cada una vuelve como estaba: la que habías apartado tú sigue apartada.

## Salto de página

Cada documento elige su papel (**A4**, **Carta** u **Oficio**, de 216 × 330 mm) en **Papel**, en su panel; un documento que ya pedía **Tabloide** lo conserva. Enciende **Hojas** ahí para leerlo en esas hojas, en **Vertical** u **Horizontal**: una hoja que se llena pasa sola a la siguiente, en pantalla igual que en papel. Las hojas vienen apagadas hasta que las enciendes, y lo que eliges viaja con el archivo en sus [propiedades](https://markdown.tisty.app/es/propiedades). En una ventana de solo lectura, **Hojas** cambia solo cómo lo ves y su PDF: el archivo queda como estaba.

Una raya sola en su línea (`---`) corta siempre la hoja, con hojas o sin ellas, y `Ctrl`/`⌘` + `Enter` la escribe mientras escribes. Con hojas encendidas, se marca como **Salto de página**. Cómo se escribe, y qué entra como uno desde LaTeX, Word o una página web, está en la [guía de Markdown](https://markdown.tisty.app/es/saltos-de-pagina).

## El PDF propio de Tisty

Tisty hace su propio PDF y te lo muestra antes de exportarlo:

- Papel **A4**, **Carta** u **Oficio**, en vertical u horizontal.
- Sus propios márgenes.
- Sus imágenes van dentro; cualquier otro adjunto queda nombrado donde está.
- Si quieres, **quién lo escribió y cuándo**: un PDF sale de Tisty para siempre, así que lo que va dentro lo decides tú.
- La página tal como se ve en pantalla (el color del código, los avisos, los diagramas, las fórmulas y los iconos), porque el PDF es la misma página puesta en papel.

Imprime, o usa **Exportar a PDF…**, desde el panel del documento.
