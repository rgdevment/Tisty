# Markdown de Tisty

Un documento de Tisty es un archivo `.md` normal. Lo que lleva es Markdown: CommonMark con lo que le agrega GitHub, más unas pocas palabras cortas propias de Tisty para lo que Markdown no sabe decir: el color de un rotulador, un subrayado, un icono, una sección que se pliega. Esta página las reúne todas: para qué sirve cada una, cómo queda escrita en el archivo, cómo se lee y las reglas chicas que deciden cuándo cuenta. La ayuda sobre las tareas, los asistentes por MCP y el resto de Tisty está en [www.tisty.app/es/ayuda](https://www.tisty.app/es/ayuda/).

Nada de esto hay que aprenderlo de memoria. El menú `/` y la barra del editor lo escriben todo por ti; esto es lo que encuentras si abres el archivo en otra parte, o si quieres escribirlo a mano.

## De un vistazo

| Qué | Se escribe | GitHub | CommonMark |
| :-- | :-- | :-: | :-: |
| [Propiedades](#propiedades) | `---` `title:` `tags:` … `---` | como tabla | como raya y texto |
| [Títulos](#títulos) | `# Título` … `###### Título` | sí | sí |
| [Negrita, cursiva, tachado, código](#negrita-cursiva-tachado-y-código) | `**n**` `*c*` `~~t~~` `` `c` `` | sí | salvo `~~t~~` |
| [Subrayado](#subrayado) | `++palabras++` | no | no |
| [Rotuladores](#rotuladores) | `==palabras==` `==palabras=={green}` | no | no |
| [Enlaces](#enlaces) | `[palabras](https://…)` `[palabras][uno]` `https://…` | sí | salvo una `https://…` suelta |
| [Saltos de línea](#párrafos-y-saltos-de-línea) | una línea por línea | las junta | las junta |
| [Listas y etapas](#listas) | `- elemento` `1. elemento` `- [ ] etapa` | sí | listas, no etapas |
| [Citas y avisos](#citas-y-avisos) | `> palabras` `> [!NOTE]` | sí | citas, no avisos |
| [Secciones plegables](#secciones-plegables) | `:::details Título` … `:::` | no | no |
| [Etiquetas](#etiquetas) | `#contrato` | como texto | como texto |
| [Iconos](#iconos) | `::dog orange::` | no | no |
| [Emojis](#emojis) | `🎉` o `:tada:` | sí | solo `🎉` |
| [Otros documentos y páginas](#otros-documentos-y-páginas) | `![Título](tisty:doc/…)` | no | no |
| [Adjuntos](#adjuntos) | `![Etiqueta](archivo-1a2b3c4d.pdf)` | no | no |
| [Referencias](#referencias) | `[[Nombre]]` `[[#2]]` | no | no |
| [Notas al pie](#notas-al-pie) | `[^1]` … `[^1]: nota` | sí | no |
| [Código](#bloques-de-código) | ` ```lenguaje title="nombre" ` | sí | sí |
| [Diagramas](#diagramas-fórmulas-y-widgets) | ` ```mermaid ` | sí | como código |
| [Fórmulas](#diagramas-fórmulas-y-widgets) | ` ```math ` `$x$` `$$x$$` | sí | no |
| [Widgets](#diagramas-fórmulas-y-widgets) | ` ```widget ` | como código | como código |
| [Tablas](#tablas) | `\| a \| b \|` | sí | no |
| [Saltos de página](#saltos-de-página) | `---` | como raya | como raya |
| [Comentarios y entidades](#comentarios-y-entidades) | `<!-- nota -->` `&copy;` | sí | sí |

Donde la tabla dice *no*, un lector que sigue solo GitHub o solo CommonMark muestra las palabras tal como están escritas, así que no se pierde nada: `++palabras++` se sigue leyendo, y `::dog::` sigue siendo un perro.

## Propiedades

Un bloque de propiedades al principio del archivo, entre dos líneas `---`, como lo escriben muchas herramientas de Markdown (también se le dice front matter):

```markdown
---
title: Vacunas de Rina
tags: [rina, veterinaria]
fuente: la libreta de la veterinaria
---
```

- Se guarda tal como está escrito. La página no lo muestra: las propiedades aparecen en el panel de al lado, bajo **Sobre este documento**, donde **Editar** abre el bloque arriba de la página tal como está escrito.
- `title:` es el [título](#el-título) del documento.
- `tags:` (o `tag:`) guarda el documento bajo cada una, igual que escribir `#rina` en una frase: es la misma etiqueta de las dos maneras. Sirve una lista entre corchetes, palabras separadas por comas o un `- elemento` por línea, con `#` o sin él.
- El bloque solo cuenta al principio: la primera línea es `---`, la de abajo tiene palabras, y lo cierra un `---` o `...`. En cualquier otra parte, `---` es un [salto de página](#saltos-de-página).
- `paper:` es la hoja en que el documento se dibuja y se imprime: `letter` (Carta) o `tabloid` (Tabloide), y A4 cuando no está. Elegir **Papel** en el panel la escribe, y A4 la quita.
- Fuera de `paper:`, Tisty nunca escribe propiedades por su cuenta: una etiqueta que agrega va en el texto como `#etiqueta`.
- Un bloque de más de 4 KiB, con más de 32 propiedades o con un valor de más de 500 caracteres se guarda tal cual, pero no se lee nada de él. Solo se leen propiedades simples: `clave: valor`, una lista entre corchetes o líneas `- elemento`.

## El título

La primera línea con palabras es el título del documento, salvo que sus [propiedades](#propiedades) digan otro con `title:`. No hay que ponerle nombre en ninguna parte. Para encontrarla, Tisty se salta la marca de un aviso y las vallas de código.

```markdown
# Vacunas de Rina
```

## Títulos

Seis niveles, de `#` a `######`, siempre seguidos de un espacio. El menú `/` ofrece los tres primeros.

```markdown
## Vacunas

### Antirrábica
```

- `# Hola` es un título; `#Hola`, sin espacio, es una [etiqueta](#etiquetas).
- Para empezar una línea con un `#` que no sea ninguna de las dos, escribe `\#`.

## Párrafos y saltos de línea

Una línea en blanco empieza un párrafo nuevo. Un salto de línea simple se queda como salto: lo que escribes en la línea siguiente se dibuja en la línea siguiente, en el mismo párrafo.

```markdown
Una línea termina donde tú la terminas.
Esta línea es del mismo párrafo, una más abajo.

Este ya es un párrafo nuevo.
```

GitHub junta esas dos líneas en una cuando muestra un archivo; Tisty las deja separadas. Una barra invertida al final de la línea la corta en todas partes.

Para escribir una línea que si no empezaría otra cosa (un título, una lista, una cita), pon una barra invertida delante: `\#`, `\-`, `1\.`, `\>`.

## Negrita, cursiva, tachado y código

```markdown
**Negrita**, *cursiva*, ***las dos a la vez***, ~~tachado~~ y `código suelto`.
```

- Una marca que toca un espacio por dentro no es marca: `** negrita **` queda tal cual, y `5 * 3` también.
- Una palabra unida con guiones bajos, como `snake_case` o `__dunder__`, no es cursiva ni negrita.
- Entre comillas invertidas no cuenta nada más: una etiqueta, un rotulador o un emoji ahí dentro es solo texto.

## Subrayado

Markdown no tiene subrayado, así que Tisty usa dos signos más.

```markdown
++subrayado++
```

- Exactamente dos `+` a cada lado. Las palabras de dentro no pueden empezar ni terminar con un espacio.
- Por fuera, las paredes no pueden tocar una letra, un dígito ni otro `+`, así que `C++`, `i++` y `a ++ b ++ c` nunca son subrayados.
- Un subrayado no puede cruzar el borde de una negrita ni de un enlace: las dos paredes quedan dentro del mismo.
- Los demás lectores muestran `++subrayado++` tal cual. Cuando un documento sale de Tisty como Markdown llano (**Copiar como Markdown**, o escrito en una carpeta), las paredes se quitan y las palabras quedan.

## Rotuladores

Un destacado, en un color. El amarillo es el de siempre; cualquier otro color se nombra entre llaves justo después de la pared que cierra.

```markdown
==amarillo==, ==verde=={green}, ==azul=={blue} y ==rosa=={pink}.
```

La barra ofrece cuatro rotuladores (**amarillo**, **verde**, **azul** y **rosa**) y el archivo acepta ocho colores más: `red`, `orange`, `amber`, `teal`, `indigo`, `purple`, `brown` y `gray`. Los colores se escriben en inglés.

- Exactamente dos `=` a cada lado: `===` y más largos son otra cosa. Las palabras de dentro no pueden empezar ni terminar con un espacio.
- El color va pegado al `==` que cierra, sin espacio: `==palabras=={green}`. `=={yellow}` es lo mismo que no poner color.
- Un color que la paleta no tiene, como `{chartreuse}`, deja un rotulador amarillo con `{chartreuse}` escrito detrás, a la vista.
- Un rotulador no puede cruzar el borde de una negrita ni de un enlace.
- Cuando el documento sale de Tisty como Markdown llano, los rotuladores se escriben como `<mark>`, para que otros lectores los sigan pintando.

## Enlaces

```markdown
Un [enlace a una web](https://github.com/rgdevment/Tisty), una dirección
entre ángulos <https://github.com/rgdevment/Tisty>, y un correo
igual: <rina@example.com>.
```

Un enlace solo en su propio párrafo se muestra como una tarjeta, con la letra y el color de su sitio:

```markdown
https://github.com/rgdevment/Tisty
```

- Una dirección web escrita sola, `https://github.com/rgdevment/Tisty`, también es un enlace. Un punto, una coma o un paréntesis que cierra justo después no son parte de ella.
- Un enlace también se puede escribir por referencia, con su dirección dicha una vez en una línea propia, en cualquier parte del documento:

  ```markdown
  El [mapa del paseo][parque] y la [veterinaria][vet].

  [parque]: https://example.com/parque
  [vet]: https://example.com/vet "Abre los sábados"
  ```

- La página muestra cada definición donde está escrita, como una línea pequeña con su etiqueta y su dirección; un clic en ella la edita.
- Un enlace todavía sin dirección, `[palabras]()`, se lee como sus palabras.
- Solo se siguen las direcciones web, `mailto:` y el `tisty:` propio. Cualquier otro esquema se lee como texto.
- Una dirección con espacios va entre ángulos: `[clip](<C:/Users/yo/un clip.mkv>)`.
- Un enlace abre tu navegador solo cuando haces clic en él.

## Listas

```markdown
- Correa
- Bolsas
  - Las verdes
- Agua

1. Ponerle la correa
2. Caminar hasta el parque

3. Una lista numerada conserva el número con que empieza
```

- Las viñetas son `-`, `*` o `+`; los números, `1.` o `1)`. Sangra un elemento para anidarlo.
- Un elemento puede empezar con palabras o con otro bloque (una cita, un título, una imagen, una valla), tal como lo dice Markdown.

### Etapas

Una lista de casillas que se marcan. No son tareas de Tisty: no se cierran, no tienen fecha y no salen en *Hoy*.

```markdown
- [x] Llevar la cartilla de Rina
- [ ] Preguntar por la pastilla de las garrapatas
```

`[ ]` está abierta, `[x]` (o `[X]`) está marcada. Un clic en la casilla la marca, en la página y mientras escribes.

## Citas y avisos

Una cita es una línea que empieza con `>`.

```markdown
> Lo que no está escrito, no ocurrió.
```

Un aviso es una cita cuya primera línea nombra su tipo entre corchetes. Cinco tipos, cada uno con su color:

```markdown
> [!NOTE]
> Para lo que conviene saber, sin urgencia ninguna.

> [!TIP]
> Para el atajo que descubriste por las malas.

> [!IMPORTANT]
> Para lo que no se puede pasar por alto.

> [!WARNING]
> Para lo que puede salir mal si no miras.

> [!CAUTION]
> Para lo que no tiene vuelta atrás.
```

- El tipo se escribe en inglés y se lee en mayúsculas o minúsculas: `[!note]` es una nota. En pantalla se llaman Nota, Consejo, Importante, Aviso y Cuidado.
- GitHub quiere la marca sola en su línea; Tisty también acepta palabras detrás, `> [!TIP] Lleva el rollo de repuesto.`, y esas palabras pasan a ser la primera línea del aviso.
- Un tipo que Tisty no conoce, como `[!FOO]`, deja una cita normal.
- Dentro de un aviso cabe casi todo (listas, código, rayas, una sección) siempre que cada línea lleve su `>`.
- Los mismos cinco tipos también se pueden escribir como lo hacen otras herramientas, como un bloque entre `:::tip` y un `:::` solo; Tisty lo dibuja como el mismo aviso.

## Secciones plegables

Un título en el que haces clic para abrir o plegar lo que hay debajo. Elige **Sección plegable** en el menú `/`, o escríbela:

```markdown
:::details La cartilla completa de Rina

Nació en 2023. Séxtuple y antirrábica cada marzo.

- Desparasitación cada tres meses

:::
```

- La línea que abre, `:::details` y su título, va como párrafo propio: una línea en blanco antes y otra después. Lo mismo el `:::` que cierra.
- Sin su `:::` de cierre no es sección: las dos líneas quedan tal como están escritas.
- Una sección empieza plegada. Sin título después de `:::details`, muestra uno vacío.
- Las secciones se anidan. Cualquier línea `:::palabra` abre un contenedor que cierra el siguiente `:::` solo, así que una sección dentro de otra, o un bloque `:::tip` escrito por otra herramienta, nunca se roba la valla de cierre. `details` se pliega y los cinco tipos de aviso dibujan un aviso; cualquier otro bloque `:::palabra` muestra sus líneas tal como están escritas.
- También funcionan dentro de un aviso.
- GitHub muestra las dos vallas como texto y todo lo de en medio desplegado. Tisty también dibuja un bloque `<details>` escrito en HTML, como los escribe GitHub, pero eso es HTML, que queda [fuera del lenguaje](#lo-que-queda-fuera-del-lenguaje).

## Etiquetas

Una palabra después de `#` archiva el documento bajo esa etiqueta. Vive en la frase, así que viaja con el archivo.

```markdown
Refuerzo en marzo #rina #veterinario
```

- `#Hola` es una etiqueta; `# Hola`, con espacio, es un título.
- Se quitan los acentos y las mayúsculas: `#Camión` y `#camion` son una sola etiqueta, `camion`. Un guion bajo pasa a ser guion: `#mi_etiqueta` es `mi-etiqueta`.
- Dos caracteres como mínimo, uno de ellos una letra: `#1234` es un número de ticket, no una etiqueta, y `#a` tampoco. `#2026x` sí lo es.
- Un `#` pegado a lo que va antes no es etiqueta: `C#`, `bg-#fff`, `pagina.html#arriba` y el ancla en la dirección de un enlace no cuentan.
- Nada dentro de código es etiqueta, ni tampoco el `#2` de una [referencia](#referencias) como `[[#2]]`.
- Un documento guarda sus primeras 64 etiquetas.
- Mientras escribes, al teclear `#` se te ofrecen las etiquetas que ya usas, para que una etiqueta nunca se escriba de dos maneras.

## Iconos

Un icono marca una línea sin gastar un título en ella. Elige **Un icono** en el menú `/` y búscalo por lo que dibuja, en español: «perro», «moto», «puño». Se escribe entre dos puntos dobles, con un color opcional después de un espacio:

```markdown
::dog orange:: Vacunas de Rina
::star:: Listo
```

- El nombre es uno del juego de Tisty, en inglés: en minúsculas y con guiones (`dog`, `bone`, `heart`, `rocket-thing`). El buscador los conoce todos.
- Los colores son `red`, `orange`, `amber`, `green`, `teal`, `blue`, `indigo`, `purple`, `pink`, `brown` y `gray`.
- Un icono no puede tocar una letra ni un dígito: `a::star::b` queda tal cual.
- Un nombre o un color que Tisty no tiene queda tal como está escrito, `::nada::` incluido.
- Los demás lectores muestran `::dog orange::` como texto. Cuando el documento sale de Tisty como Markdown llano, un icono se escribe como un pedacito de HTML cuyas palabras son su nombre entre dos puntos, `:dog:`.

## Emojis

Un emoji conviene escribirlo tal cual: 🎉. Un código que GitHub conoce, como `:tada:`, `:+1:` o `:dog:`, también se dibuja como su emoji, y queda escrito como código.

- Un código no puede tocar una letra, un dígito ni otros dos puntos: `10:30:00` y `a:tada:b` quedan tal cual.
- Uno que Tisty no conoce queda como texto.

## Otros documentos y páginas

Otro documento se nombra por su identificador después de `tisty:doc/`. El menú `/` los escribe por ti (**Un documento**, **Un documento nuevo**, **Una página que ya existe**, **Una página nueva**).

```markdown
![La cartilla de Rina](tisty:doc/01JC4M7Q3B8YH2A9K6TN5VXZRD)

Los [paseos de octubre](tisty:doc/01JC4M9F2W7DQ1E3S8RB6PKMHT) fueron mejor.
```

- Con `!` delante y sola en su párrafo, es una **tarjeta**: el título actual del documento, sus primeras palabras y, si es una página, su lugar en el libro. Las palabras entre corchetes son solo lo que muestran otros lectores.
- Lo mismo dentro de una frase es una palabra en la que puedes hacer clic.
- En una página, solo la tarjeta le da su lugar en el libro; un enlace en una frase la señala sin colocarla. Mira Páginas y PDF.
- Un `[`, `]` o `\` en un título se escribe con una barra invertida delante.

## Adjuntos

Un archivo que arrastras queda guardado dentro de Tisty y se nombra con un sello de sus bytes. Se escribe como una imagen, con la etiqueta que ves:

```markdown
![Rina en su silla](rina-en-su-silla-3f9a1c2be8d47a60.jpg)

![La cartilla de Rina](cartilla-9d02e4b7c1f3a85e.pdf)
```

- Una imagen se dibuja; cualquier otro archivo es una ficha con su etiqueta que lo abre.
- El sello son de 8 a 16 caracteres hexadecimales en minúscula después del último guion, y el nombre entero va en minúsculas.
- Los documentos escritos por un Tisty anterior que dicen `attachments/ab/nombre` se siguen leyendo; cuando el documento se reescribe, queda solo el nombre.
- Cuando un documento se escribe en una carpeta, sus adjuntos van al lado y los enlaces apuntan a ellos.

## Referencias

Dos corchetes dobles nombran algo sin enlazarlo.

```markdown
Mira [[La cartilla de Rina]] y el paso [[#2]].
```

- `[[nombre]]` se dibuja como una ficha con el nombre.
- En la descripción o el diario de una tarea, `[[#2]]` muestra las palabras del segundo paso de esa tarea.
- En un documento, `[[#2]]` muestra las palabras de la segunda [etapa](#listas) del documento, contadas en el orden en que se leen, y un clic te lleva a ella.
- Un nombre vacío, o con un `[` o un salto de línea, no es referencia.

## Notas al pie

Una nota al pie del texto se llama con una etiqueta corta entre corchetes y con acento circunflejo, y se dice una vez en una línea propia:

```markdown
El veterinario lo confirmó[^vet].

[^vet]: Visita del 3 de marzo, con el examen de sangre.
```

- La llamada se dibuja como una etiqueta pequeña y elevada que lleva a la nota; la nota queda donde está escrita, con su etiqueta al lado.
- La etiqueta es cualquier palabra o número corto: `[^1]`, `[^vet]`.
- GitHub las lee igual.

## Bloques de código

Una valla de tres o más comillas invertidas (o virgulillas), el lenguaje detrás y, si quieres, un nombre:

````markdown
```bash title="respaldo.sh"
rsync -a ~/Tisty/ /Volumes/Respaldo/Tisty/
```
````

Cada lenguaje se colorea a su manera:

````markdown
```python title="paseos.py"
def kilometros(paseos):
    # Suma lo caminado en el mes
    return sum(p["km"] for p in paseos if p["hecho"])
```
````

````markdown
```javascript
const vacunas = ["antirrábica", "óctuple"];
for (const una of vacunas) console.log(`Pendiente: ${una}`);
```
````

````markdown
```json title="rina.json"
{ "nombre": "Rina", "edad": 4, "vacunas": ["antirrábica"], "chip": true }
```
````

````markdown
```sql
SELECT nombre, fecha FROM vacunas WHERE mascota = 'Rina' ORDER BY fecha DESC;
```
````

Dentro de una frase, el código va entre comillas invertidas:

```markdown
Corre `npm run guide` y abre `target/guide`.
```

- El nombre va en `title="…"`; una `"` dentro se escribe `\"`. Los demás lectores lo ignoran.
- Lo demás que otras herramientas escriben después del lenguaje, como `showLineNumbers`, se guarda tal cual; Tisty lee el lenguaje y el nombre.
- Para mostrar una valla dentro de un bloque, rodéalo con una valla más larga.
- Un bloque sangrado cuatro espacios también es código.
- Nada en el código es etiqueta, rotulador, icono ni emoji.

## Diagramas, fórmulas y widgets

Tres lenguajes se dibujan en vez de mostrarse como código: `mermaid` para diagramas, `math` para fórmulas y `widget` para una pequeña página propia.

### Diagramas

Un bloque `mermaid` (o `mmd`) dibuja un diagrama: flujos, secuencias, estados, tortas y cronogramas.

````markdown
```mermaid
flowchart LR
  A{¿Llueve?} -->|sí| B[Llevar paraguas]
  A -->|no| C[Salir a caminar]
```
````

````markdown
```mermaid
sequenceDiagram
  Ana->>Luis: ¿Pagaste la cuota?
  Luis-->>Ana: Sí, ayer
```
````

````markdown
```mermaid
pie title Gastos del mes
  "Comida" : 40
  "Casa" : 35
  "Paseos" : 25
```
````

````markdown
```mermaid
gantt
  title Mudanza
  dateFormat YYYY-MM-DD
  Cajas  :a1, 2026-11-01, 5d
  Camión :a2, after a1, 1d
```
````

- Uno demasiado grande para acomodarlo (más de 32 KiB, o mil cajas y flechas) o que no se entiende, se muestra como código.

### Fórmulas

Una fórmula dentro de una frase va entre dólares simples:

```markdown
La energía es $E=mc^2$ y el área de un círculo, $\pi r^2$.
```

Un bloque `math`, o una fórmula entre dólares dobles en una línea propia, va aparte:

````markdown
```math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```
````

```markdown
$$
\int_0^1 x\,dx = \tfrac{1}{2}
$$
```

````markdown
```math
\begin{pmatrix} a & b \\ c & d \end{pmatrix} \qquad |x| = \begin{cases} x & x \ge 0 \\ -x & x < 0 \end{cases}
```
````

- Un dígito justo después del dólar que cierra la vuelve un precio, no una fórmula: `$5 y $10` queda tal cual.
- `\$` es un signo de dólar.
- Pasados 8 KiB una fórmula se muestra como código; un comando que no conoce se salta.

### Widgets

Un bloque `widget` dibuja una pequeña página aislada, sin acceso a la red; su nombre va en `title`. Trae estilos listos para usar: `stat`, `card`, `grid`, `two`, `head`, `badge` (con `ok`, `warn` y `bad`), `list`, `box`, `up` y `down`, además de tablas y botones.

````markdown
```widget title="Rina este mes"
<div class="grid">
  <div class="stat"><small>Paseos</small><b>42</b><em class="up">+6</em></div>
  <div class="stat"><small>Kilómetros</small><b>61</b><em class="down">−3</em></div>
  <div class="stat"><small>Vacunas</small><b>al día</b><span class="badge ok">listo</span></div>
</div>
```
````

````markdown
```widget title="Vasos de agua"
<div class="card">
  <div class="head"><h3>Hoy</h3><span class="badge" id="n">0</span></div>
  <div class="row">
    <button class="primary" onclick="n.textContent = +n.textContent + 1">Uno más</button>
    <button onclick="n.textContent = 0">De nuevo</button>
  </div>
</div>
```
````

- Puede llevar su propio `<script>`, que corre dentro del widget y nunca en el documento.

GitHub dibuja las tres vallas salvo `widget`, y también `$…$` y `$$…$$`.

## Tablas

```markdown
| Vacuna         | Puesta | Toca otra vez |
| :------------- | :----: | ------------: |
| Séxtuple       | marzo  |      12 meses |
| Antirrábica    | marzo  |      12 meses |
```

- Los dos puntos en la raya bajo el encabezado alinean una columna: `:---` a la izquierda, `:---:` al centro, `---:` a la derecha.
- El ancho de una columna viaja en esa raya: cuatro guiones o más lo fijan, diez píxeles por guion; con tres lo decide la página. Los demás lectores ignoran cuántos guiones hay. Mira Tablas.
- Un `|` dentro de una celda se escribe `\|`.
- Una celda lleva una línea de palabras y marcas, no bloques.

## Saltos de página

Una raya sola en su línea corta la hoja: en pantalla se ve como un corte, y al imprimir o en el PDF lo que sigue empieza en la hoja siguiente.

```markdown
Lo que sigue empieza en la hoja siguiente.

---

Esto ya está en la otra.
```

Deja una línea en blanco encima: justo debajo de una línea con palabras, `---` convierte esa línea en un título.

Una hoja que se llena pasa sola a la siguiente, en pantalla igual que en papel, y `Ctrl`/`⌘` + `Enter` escribe un salto mientras escribes. Un `\newpage`, `\pagebreak` o `\clearpage` solo en su línea, y un salto de página escrito en una página web o en un documento de Word, entran como una raya.

## Comentarios y entidades

Una nota para ti que la página no muestra va entre `<!--` y `-->`, en una línea o en varias. Una entidad nombra un carácter difícil de escribir: `&copy;` es ©, `&amp;` es &, `&nbsp;` es un espacio que no se corta.

```markdown
Refuerzo en marzo. <!-- preguntar por la pastilla de garrapatas -->

&copy; 2026 Veterinaria de Rina
```

- Los dos quedan en el archivo tal como se escribieron; la página muestra lo que significan.
- Dentro de código son solo texto.

## Lo que queda fuera del lenguaje

Lo que llega de afuera se convierte en estas palabras al entrar: una página web pegada o importada, un documento de Word, una nota de Obsidian, el Markdown que manda un asistente. Sus títulos, listas, enlaces, tablas, colores e imágenes pasan a las formas de esta página; un color fuera de la paleta pasa al rotulador más cercano, y lo que no tiene ninguna palabra queda como texto. El archivo de donde vino no se toca.

Solo el HTML que alguien escribe a mano en un documento queda tal como está escrito. El documento se sigue abriendo, editando y guardando entero, sin que se corte nada; la página dibuja esa parte como buenamente puede, y **Configuración › Mantenimiento** te lista los documentos que lo llevan.

### Palabras de un Tisty anterior

Las versiones anteriores escribían los rotuladores como `<mark data-pen="green">`, los iconos como `<span data-ico="dog">` y los subrayados como `<u>`. Todo eso se convierte en las palabras de esta página cuando los documentos llegan, sin tocar ningún otro byte.
