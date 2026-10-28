# Convertir a y desde Markdown

Tisty convierte Word, HTML, Google Docs, texto enriquecido, texto plano y notas de Obsidian a Markdown, y convierte Markdown a Word, HTML, PDF y Markdown estándar. No hay que instalar ningún conversor: todo lo que entra termina en [Markdown de Tisty](https://markdown.tisty.app/es/), y todo lo que sale parte de él. El archivo que traes no se toca.

## De Word a Markdown (`.docx` a `.md`)

Importa el `.docx` desde **Importar un documento**, en el árbol, o copia desde Word y pega en un documento. Entran los títulos, listas, tablas, enlaces, colores e imágenes: un color pasa al rotulador más cercano de la paleta, y las imágenes entran como adjuntos. Las líneas en una letra de máquina como Courier o Consolas entran como código.

## De HTML a Markdown

Importa una página `.html` o `.htm`, o pega lo que copiaste de una página web o de un correo. Se conservan los títulos, listas, negritas, enlaces, tablas y palabras de color; lo que el HTML dice y Markdown no sabe decir pasa a Markdown o a texto.

## De Google Docs a Markdown

Copia desde Google Docs y pega en un documento. Entra como una página web: títulos, listas, enlaces, tablas y colores.

## De texto enriquecido a Markdown

El texto que copias con su formato (de una página web, un correo o un procesador de texto) conserva al pegarlo sus títulos, listas, negritas, enlaces, tablas y palabras de color. Un color pasa al rotulador más cercano de la paleta. Las filas copiadas de Excel, Numbers o Google Sheets, o un CSV, entran como tabla.

## De texto plano a Markdown

Importa un archivo `.txt`, o pega. Una tabla que una terminal dibujó con líneas (`┌─┬─┐`) pasa a ser una tabla de Markdown, y las líneas que se leen como código, copiadas de un editor de código o de una terminal, entran como bloque de código, salvo que el cursor ya esté dentro de uno.

## De Obsidian a Markdown

Importa la nota. Sus formas propias pasan a las estándar: sus avisos al más cercano de los cinco tipos, sin plegado; los `%%comentarios%%` a `<!-- -->`; `![[archivo]]` a una imagen o un enlace al archivo; y una nota nombrada por su título (`[[Plan]]`, `[[Plan|el plan]]`, `![[Plan]]`) pasa a un enlace o una tarjeta al documento de aquí que lleva ese título. Si ningún documento lo lleva, o lo llevan varios, queda como referencia con su nombre y Tisty te lo dice; importa primero la otra nota y sus enlaces la encuentran. El front matter, las notas al pie y todo lo demás que el lenguaje acepta queda tal como venía.

## Lo que entra con él

Las imágenes y archivos que el documento nombra a su lado (en su misma carpeta o en una de más adentro) entran con él y quedan guardados en Tisty, y el texto apunta a esas copias. Uno que no está, o que queda fuera de esa carpeta, se queda como texto, y Tisty te dice cuántos no se pudieron traer. Otra nota que enlaza no se copia como archivo: queda enlazada si un documento de aquí ya lleva su título, y si no, su nombre queda como referencia y Tisty te recuerda importarla por separado. Un enlace a una página web, un correo u otra app (`tel:`, `obsidian://`…) sigue siendo enlace.

Lo que no tiene palabra en Markdown de Tisty se convierte al entrar; la [guía](https://markdown.tisty.app/es/lo-que-queda-fuera-del-lenguaje) lo detalla.

## De Markdown a Word (`.md` a `.docx`)

**Exportar como Word…**, en el menú del documento, hace un `.docx` con sus títulos, listas, tablas, enlaces, rotuladores e imágenes dentro, y el código en Consolas, para que vuelva a entrar como código si lo importas de nuevo. Cada página y cada salto de página empieza una hoja nueva.

## De Markdown a HTML

**Exportar como HTML…** hace una sola página web que viaja sola: el documento y sus páginas en orden, con el aspecto de la página y sus imágenes dentro.

## De Markdown a PDF

Tisty imprime su propio PDF, con sus páginas, saltos de página y tamaño de papel. Mira [Páginas y PDF](Páginas-y-PDF#el-pdf-propio-de-tisty).

## De Markdown a Markdown estándar

**Copiar como Markdown** da Markdown estándar, como lo leen GitHub y CommonMark, listo para cualquier otro editor. Las formas propias de Tisty pasan a las estándar (`:::tip` pasa a `> [!TIP]`, `:::details` a `<details>`, y un salto de línea simple sigue siendo salto), un enlace a otro documento queda como su texto, y todas las etiquetas del documento (las de `tags:` y los `#hashtags`) quedan en el front matter. **Escribirlo en una carpeta** hace lo mismo como archivos, con sus páginas y adjuntos al lado.

Más formas de sacar documentos, como un `.tistyx` para otro Tisty, están en [Documentos](Documentos#sacar-documentos).
