# Diagramas

Un diagrama en Tisty es un bloque `mermaid` dentro de un documento: unas pocas líneas de texto que Tisty dibuja, y que GitHub, Notion, Obsidian o un asistente leen igual. Puedes escribirlas tú, pero no hace falta: la ventana de diagramas lo dibuja contigo y guarda el resultado como ese mismo texto.

Cómo se escribe cada tipo de diagrama está en la guía de Markdown de Tisty: [diagramas, fórmulas y widgets](https://markdown.tisty.app/es/diagramas-formulas-y-widgets).

## Un diagrama nuevo

- **Fuera de un documento:** en el **+** de Documentos, o en el menú de una carpeta, elige **Diagrama nuevo**. Pide un nombre y el tipo: mapa mental, esquema de base de datos, diagrama libre, diagrama de clases u otro (texto Mermaid). Tisty crea un documento normal con un ejemplo corto de ese tipo, o en blanco si es un diagrama libre, y abre la ventana. El documento nace en la carpeta donde estás, o en la que elegiste en su menú; fuera de toda carpeta, en la carpeta **Diagramas**, que se crea si falta.
- **Dentro de un documento:** escribe `/` y elige **Diagrama nuevo**. El ejemplo se escribe donde estás y la ventana se abre sobre él. **Bloque Mermaid como texto** deja un bloque vacío, para escribirlo a mano. El botón **Diagrama** de la barra flotante ofrece lo mismo sin pasar por el menú.
- **Reutilizar uno:** **Diagrama existente…**, en el menú `/` o en el botón **Diagrama**, abre un buscador con los diagramas de todos tus documentos. Cada uno aparece con su documento, la sección donde está y su título, si lo tiene; escribe parte de cualquiera de ellos o del tipo para filtrarlos. El elegido se ve dibujado al lado. Al elegirlo se escribe una copia, y desde ahí cada uno es independiente: cambiar uno no cambia el otro.

## Abrir uno que ya existe

Pasa el puntero sobre un diagrama y pulsa **Abrir en la ventana de diagramas**, o haz doble clic sobre él. **Editar como Markdown**, en la misma barra, abre su texto Mermaid ahí mismo en la página. Un clic simple sigue abriéndolo en grande, para mirarlo. La ventana solo se abre en documentos que puedes escribir, y cada diagrama tiene una sola.

## La ventana

A un lado está el dibujo, y al otro, **Lo que se escribe en el documento**: el texto Mermaid, que cambia en vivo con el dibujo y que puedes escribir tú también.

- La rueda del ratón, o arrastrar el fondo, mueve el dibujo; con `Ctrl`/`⌘` acerca y aleja donde está el puntero. **Acercar**, **Alejar** y **Ajustar** hacen lo mismo con botones.
- **Guardar en «…»**, o `Ctrl`/`⌘` + `S`, escribe el diagrama de vuelta en su bloque y no toca nada más del documento.
- Si el documento cambió mientras la ventana estaba abierta, la ventana te lo dice: **Usar el del documento** o **Guardar el mío encima**.
- Si cierras con cambios sin guardar, la ventana te pregunta antes.

## Dibujar a mano

Cuatro tipos se dibujan y se editan directamente sobre el dibujo. Los demás (secuencias, Gantt, tortas…) se editan en el texto, con el dibujo al lado.

- **Mapa mental.** **+ Hija** y **+ Hermana**, o `Tab` y `Enter`; `Supr` borra la idea elegida; doble clic o `F2` le cambia el nombre; arrastrar una idea sobre otra la cambia de padre, con su rama. El panel cambia su forma.
- **Esquema de base de datos.** Funciona sobre el mismo lienzo que el diagrama libre. La **Tabla** de la paleta se arrastra al lienzo o se pulsa, y nace con su `id` y el nombre listo para escribir; doble clic en el título la renombra. En el panel, **+ Columna**; cada columna tiene tipo, claves PK, FK y UK, y puede ser **Obligatoria**. Cada tabla queda donde la dejas. Arrastra el punto de una columna a otra tabla para relacionarlas, o a un espacio vacío para crear una tabla ya relacionada; la relación va en ángulo recto, con cuántos hay en cada extremo, y doble clic le cambia el texto. La barra deshace y rehace, borra, alinea varias tablas, **Acomodar solo** las devuelve a su cuadrícula, y acerca, aleja o ajusta. **Copiar como SQL** y **Copiar como DBML** llevan el esquema a otra herramienta.
- **Diagrama libre** (el `flowchart` de Mermaid). La paleta de la izquierda trae las formas, agrupadas en Básico, Datos, Personas y sistemas, y Notas: arrastra una al lienzo, o púlsala para ponerla en el medio. Cada forma queda donde la dejas. Arrastra uno de sus puntos a otra forma para unirlas, o a un espacio vacío para crear una forma nueva ya unida; las flechas van en ángulo recto entre los lados que se miran. Doble clic en una forma o en una flecha para cambiarle el texto. Los **Carriles** de la paleta son columnas o filas: se arrastran con todo lo que tienen adentro, se agrandan desde sus bordes, y una forma que sueltas dentro pasa a ser parte de ese carril (y deja de serlo si la sueltas fuera). Con `Ctrl` o `⌘` + clic, o con `Mayús` + arrastrar, eliges varias formas a la vez. La barra de arriba deshace y rehace (`Ctrl`/`⌘` + `Z`), borra, duplica (`Ctrl`/`⌘` + `D`), da color de relleno y de línea, cambia el tipo de la flecha elegida, alinea varias formas, y acerca, aleja o ajusta el diagrama a la ventana. **Acomodar solo** ordena todo de nuevo hacia donde elijas: de arriba abajo, de izquierda a derecha, de abajo arriba o de derecha a izquierda. **Acomodar solo en cada cambio**, apagado de entrada, vuelve a ordenar el diagrama cada vez que cambia; mover una forma a mano lo apaga, y al apagarlo cada forma queda donde está. El panel cambia el texto y la **Forma** de lo elegido, su **Carril**, y la etiqueta y el tipo de una flecha; en un carril, su nombre, o **Deshacer el carril**, que deja sus formas sueltas. `Supr` borra lo elegido.
- **Diagrama de clases.** Funciona como el de flujo, sobre el mismo lienzo. La paleta trae clase, interfaz, clase abstracta y enumeración: arrástralas al lienzo o púlsalas, y cada clase queda donde la dejas. Un **Paquete** (el `namespace` de Mermaid) es un carril: nace con una clase adentro, porque Mermaid no acepta uno vacío; se arrastra con sus clases y se agranda desde sus bordes; una clase que sueltas dentro pasa al paquete, y si lo dejas vacío, el paquete se quita. Arrastra el punto de una clase a otra para relacionarlas, o a un espacio vacío para crear una clase ya relacionada. Cada relación se dibuja con su punta (herencia, realización, composición, agregación, asociación, dependencia) y su cantidad en cada extremo. Doble clic para renombrar una clase o ponerle etiqueta a una relación. La barra cambia el tipo de la relación elegida sin darla vuelta, y además deshace, borra, da color, alinea, acomoda y hace zoom. En el panel, **+ Atributo** y **+ Método** agregan miembros; cada uno tiene su **Visibilidad** (+, -, #, ~). También se elige el **Tipo de clase** y, en una relación, su tipo, la cantidad en cada extremo y la etiqueta.

Lo que solo la ventana necesita recordar, como dónde dejaste cada tabla del esquema o cada forma del flujo, va en líneas `%% tisty:` dentro del mismo bloque. Mermaid las trata como comentarios: en GitHub, Notion u Obsidian el diagrama lo acomoda Mermaid, mientras que en la página de Tisty el mapa mental, el esquema, el diagrama libre y el de clases se ven tal como en la ventana, con un pequeño margen. La ventana lo vuelve a abrir tal como lo dejaste.

Un diagrama que la ventana no puede conservar sin cambiarlo, como un mapa mental con dos raíces, se abre igual, con el dibujo de Mermaid y su texto para editar.

## Exportar

**Exportar ▾**, en la cabecera de la ventana, y **Exportar**, en la barra del diagrama en la página, ofrecen lo mismo:

- **PNG transparente**, **PNG claro** o **PNG oscuro**.
- **JPG claro** o **JPG oscuro**.
- **SVG**, claro y sin fondo.
- **Copiar como imagen**, con el fondo del tema que estés usando, para pegarlo en un chat o en un correo.

Las imágenes salen al doble de su tamaño para que se vean nítidas (una muy grande, algo menos, hasta donde el sistema puede pintarla), y el archivo se llama como el documento, con «-2», «-3»… desde su segundo diagrama. El mapa mental, el esquema, el diagrama libre y las clases salen tal como los ves en la ventana, con tus tablas y tus formas donde las dejaste (uno libre con iconos o imágenes sale como lo dibuja Mermaid); los demás tipos, como los dibuja Mermaid.

## Con un asistente

Un asistente conectado por [[MCP|Asistentes-y-MCP]] escribe diagramas como cualquier otro texto del documento: un bloque `mermaid`. La herramienta `language` le explica qué tipos abre la ventana y que las líneas `%% tisty:` se conservan tal cual, así no deshace lo que acomodaste.
