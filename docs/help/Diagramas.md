# Diagramas

Un diagrama en Tisty es un bloque `mermaid` dentro de un documento: unas pocas líneas de texto que Tisty dibuja, y que GitHub, Notion, Obsidian o un asistente leen igual. Puedes escribirlas tú, pero no hace falta: la ventana de diagramas lo dibuja contigo y guarda el resultado como ese mismo texto.

Cómo se escribe cada tipo de diagrama está en la guía de Markdown de Tisty: [diagramas, fórmulas y widgets](https://markdown.tisty.app/es/diagramas-formulas-y-widgets).

## Un diagrama nuevo

- **Fuera de un documento:** en el **+** de Documentos, o en el menú de una carpeta, elige **Diagrama nuevo**. Pide un nombre y el tipo: mapa mental, esquema de base de datos, diagrama de flujo, diagrama de clases u otro (texto Mermaid). Tisty crea un documento normal con un ejemplo corto de ese tipo y abre la ventana. Sin una carpeta elegida, el documento queda en la carpeta **Diagramas**, que se crea si falta.
- **Dentro de un documento:** escribe `/` y elige **Diagrama nuevo**. El ejemplo se escribe donde estás y la ventana se abre sobre él. **Bloque Mermaid como texto** deja un bloque vacío, para escribirlo a mano.

## Abrir uno que ya existe

Pasa el puntero sobre un diagrama y pulsa **Abrir en la ventana de diagramas**, o haz doble clic sobre él. Un clic simple sigue abriéndolo en grande, para mirarlo. La ventana solo se abre en documentos que puedes escribir, y cada diagrama tiene una sola.

## La ventana

A un lado está el dibujo, y al otro, **Lo que se escribe en el documento**: el texto Mermaid, que cambia en vivo con el dibujo y que puedes escribir tú también.

- La rueda del ratón, o arrastrar el fondo, mueve el dibujo; con `Ctrl`/`⌘` acerca y aleja donde está el puntero. **Acercar**, **Alejar** y **Ajustar** hacen lo mismo con botones.
- **Guardar en «…»**, o `Ctrl`/`⌘` + `S`, escribe el diagrama de vuelta en su bloque y no toca nada más del documento.
- Si el documento cambió mientras la ventana estaba abierta, la ventana te lo dice: **Usar el del documento** o **Guardar el mío encima**.
- Si cierras con cambios sin guardar, la ventana te pregunta antes.

## Dibujar a mano

Cuatro tipos se dibujan y se editan directamente sobre el dibujo. Los demás (secuencias, Gantt, tortas…) se editan en el texto, con el dibujo al lado.

- **Mapa mental.** **+ Hija** y **+ Hermana**, o `Tab` y `Enter`; `Supr` borra la idea elegida; doble clic o `F2` le cambia el nombre; arrastrar una idea sobre otra la cambia de padre, con su rama. El panel cambia su forma.
- **Esquema de base de datos.** **+ Tabla** y **+ Columna**; cada columna tiene tipo, claves PK, FK y UK, y puede ser **Obligatoria**. Arrastra una tabla por su título para moverla, y el punto de una columna a otra tabla para relacionarlas. **Copiar como SQL** y **Copiar como DBML** llevan el esquema a otra herramienta.
- **Diagrama de flujo.** **+ Nodo** y **+ Grupo**; arrastra el punto de un nodo a otro para unirlos. El panel cambia el texto y la **Forma** de un nodo, su grupo, la **Dirección** del diagrama y la etiqueta y el tipo de una flecha. `Supr` borra lo elegido.
- **Diagrama de clases.** **+ Clase**, **+ Atributo** y **+ Método**; arrastra el punto de una clase a otra para relacionarlas, y elige el **Tipo** de relación, la cantidad en cada extremo y su etiqueta.

Lo que solo la ventana necesita recordar, como dónde dejaste cada tabla del esquema, va en líneas `%% tisty:` dentro del mismo bloque. Mermaid las trata como comentarios, así que en la página, en el PDF o en GitHub el diagrama lo acomoda Mermaid, y la ventana lo vuelve a abrir tal como lo dejaste.

Un diagrama que la ventana no puede conservar sin cambiarlo, como un mapa mental con dos raíces o un diagrama de clases con `namespace`, se abre igual, con el dibujo de Mermaid y su texto para editar.

## Exportar

**Exportar ▾**, en la cabecera de la ventana, y **Exportar**, en la barra del diagrama en la página, ofrecen lo mismo:

- **PNG transparente**, **PNG claro** o **PNG oscuro**.
- **JPG claro** o **JPG oscuro**.
- **SVG**, claro y sin fondo.
- **Copiar como imagen**, con el fondo del tema que estés usando, para pegarlo en un chat o en un correo.

Las imágenes salen al doble de su tamaño para que se vean nítidas, y el archivo se llama como el documento, con «-2», «-3»… desde su segundo diagrama. El mapa mental, el esquema, el flujo y las clases salen tal como los ves en la ventana, con tus tablas donde las dejaste; los demás tipos, como los dibuja Mermaid.

## Con un asistente

Un asistente conectado por [[MCP|Asistentes-y-MCP]] escribe diagramas como cualquier otro texto del documento: un bloque `mermaid`. La herramienta `language` le explica qué tipos abre la ventana y que las líneas `%% tisty:` se conservan tal cual, así no deshace lo que acomodaste.
