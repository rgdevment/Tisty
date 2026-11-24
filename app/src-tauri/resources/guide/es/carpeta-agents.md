# Agentes: las reglas de tus agentes

Esta hoja es una página de la guía. Cuenta la carpeta Agentes: qué es traer y qué es sincronizar, sus tres estados en la barra, cómo guardar en Tisty las reglas, las skills y la memoria de tus agentes de IA, cómo llegan a los archivos que lee cada uno, qué hacer cuando un archivo cambia por fuera o en otro equipo, cómo traer un archivo después o quitar el de una regla que sacas, cómo dejar de sincronizar u ocultarla, y cómo aprobar lo que tu agente te propone.

Uso más de un agente de IA, y cada uno lee sus reglas de un archivo distinto, en una carpeta distinta: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Cambiar una regla en todos era abrir cuatro archivos, y tarde o temprano uno quedaba atrás. La carpeta Agentes junta esos archivos en Tisty, como documentos, para que los leas y los ordenes, y si quieres, Tisty escribe por debajo el archivo que lee cada agente.

> [!TIP]
> **En corto:** **Traer** copia los archivos de tus agentes a Tisty, para leerlos y ordenarlos, y no escribe nada en ellos. **Sincronizar** hace que, desde ahí, lo que cambias en Tisty llegue a esos archivos, y que Tisty te avise si alguien los cambia por fuera. Antes de tocar un archivo guarda lo que tenía, nunca pisa un cambio hecho por fuera, y tu agente no escribe tus reglas: te propone el cambio, y solo tú lo apruebas.

## Qué es y cuándo te sirve

Agentes es una carpeta de tu Biblioteca, arriba de todo, con el icono de un robot. Está en tu barra lateral desde el primer día, en gris, hasta que traes algo. Cada documento que tiene adentro es un archivo de un agente: una **regla**, una **skill** o su **memoria**. Trabaja con Claude Code, Codex, GitHub Copilot CLI, Gemini CLI y Cursor.

Te sirve cuando:

- quieres leer y ordenar las reglas que ya tienes, como documentos, con títulos y búsqueda: para eso basta **Traer**;
- usas varios agentes y quieres la misma skill en todos: para eso, **Sincronizar**, con la skill en **Global**;
- quieres poder volver atrás: sincronizando, Tisty guarda el archivo original y los últimos 20 antes de cada sobrescritura;
- trabajas en más de un equipo: cada uno tiene su carpeta, lee y escribe solo la suya, y lo de otro equipo lo lees pero no lo cambias desde este.

No maneja las reglas de cada repositorio: solo los archivos de tu carpeta de usuario, como `~/.claude/CLAUDE.md`.

## Traer o sincronizar

- **Traer** copia a Tisty los archivos que marcas, como documentos. Los archivos de tus agentes quedan como están: Tisty no escribe nada en ellos ni te pide decidir nada cuando cambian. Lo que editas en esos documentos queda en Tisty.
- **Sincronizar** hace que, desde ese momento, lo que cambias en Agentes llegue a los archivos de tus agentes, con un respaldo antes de cada sobrescritura, y que Tisty te avise en la ficha de un documento si su archivo cambió por fuera.
- **Dejar de sincronizar** lo detiene: ningún archivo de tus agentes se toca y ningún documento se borra.

**Un ejemplo.** Jaime solo quiere leer con calma su `CLAUDE.md` y sus skills: elige **Traer**, y Claude Code sigue leyendo sus archivos como siempre. Yisset usa Claude Code y Codex y quiere una regla para los dos: elige **Traer y sincronizar**, escribe la regla en el documento de Claude y en el de Codex, dentro de la carpeta de su equipo, y Tisty la escribe en su `CLAUDE.md` y en su `AGENTS.md`. Una skill para los dos, en cambio, la escribe una vez en **Agentes → Global**.

## Tres estados en la barra

Agentes no lleva ninguna palabra al lado: su color te dice cómo está.

1. **Todavía sin traer**: el robot y el nombre en gris. Un clic abre el aviso de la primera vez. Su botón derecho ofrece **Ocultar de la barra**.
2. **Traída, sin sincronizar**: el robot gris y el nombre como cualquier carpeta; al pasar el puntero, cuántos documentos tiene, como cualquier carpeta. Se abre como cualquier carpeta, y arriba muestra una línea discreta: «No se sincroniza con tus agentes · Sincronizar», donde **Sincronizar** es un enlace. Su botón derecho ofrece **Sincronizar**, **Buscar de nuevo** y **Ocultar de la barra**, nada más.
3. **Sincronizando**: el robot **coral** y el nombre como cualquier carpeta; al pasar el puntero, cuántos documentos tiene. Se abre como cualquier carpeta. Su botón derecho ofrece **Dejar de sincronizar**, **Buscar de nuevo** y **Ocultar de la barra**, que antes te pide dejar de sincronizar.

El coral es solo de Agentes: el morado es de los diagramas. Lo que espera tu decisión, como un archivo cambiado por fuera o una propuesta de tu agente, no aparece en la barra: lo ves en la ficha de cada documento y en **Configuración › Agentes › Carpeta Agentes**. El botón derecho de Agentes misma no tiene ninguna otra acción de carpeta; sus subcarpetas, como **Agentes › Casa (este equipo) › Claude**, tienen un menú solo de agentes: **Documento nuevo**, **Traer otro archivo**, **Pausar la sincronización con Claude** mientras Agentes sincroniza, **Ocultar de la barra** y **Eliminar**. En **Global** y en la carpeta de tu equipo no hay pausa, porque no son de un agente, y una carpeta dentro de **Claude** tiene el mismo menú que **Claude**. La carpeta de otro equipo solo ofrece **Ocultar de la barra**.

**Agentes se queda en el primer nivel**, con su nombre y su robot: no se puede meter dentro de otra carpeta, cambiarle el nombre, darle otro icono ni archivarla, y mientras tenga algo adentro, tampoco borrarla, ni desde la ventana, ni por un agente, ni desde otro equipo. Por dentro se guarda como `AGENTS`, y así la encuentran las versiones anteriores de Tisty y tus agentes por MCP. Si lo intentas, por ejemplo arrastrándola dentro de otra carpeta, Tisty te dice «Agentes se queda arriba, con su nombre y su robot, para seguir cuidando las reglas de tus agentes. Su color y su lugar entre tus carpetas sí los cambias».

## Empezar

1. En la barra lateral, haz clic en **Agentes**, en gris. Se abre **Tus agentes, en un solo lugar**, con una X arriba para cerrarlo, y lo que encontró en las carpetas de tus agentes: **solo sus nombres y tamaños**. Todavía no abrió ningún archivo.
2. Arriba, una línea te cuenta las dos palabras: «Tisty busca las reglas, skills y memoria de tus agentes y las trae aquí como documentos, para leerlas y ordenarlas. Traer solo las copia; Traer y sincronizar además hace que lo que cambies en Agentes llegue solo a tus agentes.».
3. Abre la fila de un agente, como **Claude · 2 reglas · 1 skill · 1 memoria**, y marca lo que quieres traer. Cada regla es una fila, y cada skill también, entera: la fila **pr · Skill · 3 archivos** trae la skill `pr` con todos sus archivos, con una sola casilla. Las reglas y las skills vienen marcadas; la **Memoria del agente**, sin marcar. Los ajustes nunca aparecen, un archivo de credenciales dentro de una skill no entra, y los archivos ocultos, los que empiezan con un punto, ni siquiera aparecen.
4. Elige uno de los dos botones:
   - **Traer** copia lo que marcaste y nada más: «Entraron 7 documentos a Agentes. Los archivos de tus agentes quedan como están hasta que elijas Sincronizar.». Agentes queda traída, sin sincronizar.
   - **Traer y sincronizar**, el destacado, además sincroniza: «Entraron 7 documentos a Agentes, y desde ahora Tisty los mantiene al día.». Agentes queda con su robot coral.

La X o `Esc` cierran el aviso sin leer ni traer nada, y Agentes sigue en gris; vuelves cuando quieras con otro clic. El aviso no tiene un botón para ocultarla: eso va en el botón derecho, como cuenta **Dejar de sincronizar u ocultar Agentes**, más abajo.

La lista trae solo lo que tu agente lee de verdad. Una skill es una carpeta con su `SKILL.md` en la raíz, como `~/.claude/skills/pr/`, buscada tan abajo como la busca su agente: justo adentro para Claude, Copilot y Gemini, hasta seis carpetas más abajo para Codex, y en cualquier carpeta de categoría para Cursor. De `~/.claude/rules` entran los `.md`, y de `~/.copilot/instructions`, los `.instructions.md`. Nada que empiece con un punto aparece, como `skills/.system/` de Codex, ni `skills/synced/` de Claude Code, la copia de las skills de tu cuenta de claude.ai, que Claude Code puede sobrescribir o borrar. La fila de cada agente cuenta reglas, skills y memorias, no archivos: con nada que traer, dice **Claude · nada para traer**. Un agente que encontró sin ningún archivo todavía tiene su propia línea: «Gemini: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta.».

Si todavía no tienes archivos de agentes, el aviso dice «No encontré archivos de agentes en este equipo. Igual puedes sincronizar: Agentes se llena a medida que escribes tus reglas.». **Traer** queda desactivado, y **Traer y sincronizar** crea igual Agentes con **Global** y la carpeta de este equipo, con una subcarpeta por cada agente que encontró, vacías, listas para tu primera regla, y dice «No entró nada nuevo. La carpeta Agentes ya sincroniza en este equipo: lo que escribas ahí llega a tus agentes.». Lo que tu agente cree después lo traes con **Buscar de nuevo** o **Traer otro archivo**, como cuenta más abajo.

**Pruébalo:** haz clic en **Agentes**, en gris en tu barra, para ver qué agentes encuentra en este equipo. Si solo querías mirar, la X la deja como estaba. Lo mismo está en **Configuración › Agentes › Carpeta Agentes**, en la tarjeta **Agentes**: antes de traer nada dice qué agentes encontró y ofrece **Traer**, **Traer y sincronizar** y **Ocultar**; traída, **Sincronizar**, **Ocultar** y, si este equipo ya te dio permiso, **Traer otro archivo**; sincronizando, **Traer otro archivo**, **Dejar de sincronizar**, **Ocultar** y un interruptor por agente, como **Sincronizar Claude**. Bajo el interruptor de un agente sin archivos dice que todavía no tiene reglas, skills ni memoria.

Después, en la barra lateral, Agentes tiene **Global**, solo para skills, y una carpeta por cada equipo tuyo: la de este lleva su nombre con «(este equipo)» y las de los demás van al final, en gris tenue. Lo que traes va a la carpeta de este equipo, y cada agente que Tisty encontró aquí tiene su subcarpeta en ella, aunque esté vacía. Una carpeta de agente suelta en la raíz, como **Codex**, solo aparece si moviste o creaste algo ahí: es para todos tus equipos. **Global** lleva un globo azul y cada agente un símbolo propio en el color de su etiqueta: un globo de conversación naranja para Claude, una terminal verde azulado para Codex, unas llaves índigo para Copilot, una brújula verde para Gemini y un puntero café para Cursor. Una subcarpeta que borras no vuelve a aparecer. Cada documento se ve con su nombre y un icono: un libro para una regla, una pieza para una skill y un cerebro para la memoria. Pasa el puntero sobre uno y ves de qué archivo es.

![La barra lateral con la carpeta Agentes abierta y sincronizando, con el robot en coral: dentro, Global con su globo azul y Claude con su globo de conversación naranja; en Claude, la regla Reglas, elegida, con el icono de un libro, y debajo las skills, cada una con el icono de una pieza y cuántos archivos lleva](agents-barra.png)

## Una regla: en Tisty y en `CLAUDE.md`

Tu `CLAUDE.md` entra a Agentes como un documento. Arriba lleva sus propiedades; la línea `agent:` dice para quién es, qué es y dónde va: `nivel/tipo/nombre`, y una marca más, `on-` y el código de tu equipo, que pone Tisty. La carpeta y la línea tienen que estar de acuerdo; si no, ningún equipo lo escribe y la ficha ofrece **Según su carpeta**.

```markdown title="El documento en Tisty, en Agentes / Casa / Claude"
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

En la barra se llama **Cómo trabajo**, por su primer título. Y esto es lo que Tisty escribe en el archivo, byte a byte, sin las propiedades:

```markdown title="~/.claude/CLAUDE.md"
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

Para cambiar una regla, abre el documento, pulsa **Editar** en su ficha, escribe y pulsa **Listo**. Si Agentes sincroniza, en un instante el archivo dice lo mismo.

### Un documento es un archivo, y Global es para skills

Un documento es un archivo, palabra por palabra: lo que dice el documento es lo que dice el archivo, y nada se junta con otra cosa. **Global** es solo de skills, las de Tisty y las tuyas, para todos los agentes. Las reglas son de cada agente, y viven en la carpeta de tu equipo.

Para sumar reglas a un agente, haz otro archivo que ese agente lea: Claude lee cada archivo de `rules/`, y Copilot cada uno de `instructions/`. Codex y Gemini leen un solo archivo de reglas.

Una regla de Global de la 1.49 o la 1.50 se queda ahí para que la leas, la copies o la borres, pero ya no se escribe en ningún agente.

> [!IMPORTANT]
> Trae el archivo principal de cada agente. Si `~/.codex/AGENTS.md` ya existe y nunca lo trajiste, Tisty no sabe qué parte es tuya y no lo pisa: te pregunta **Existe en los dos lados**, y ahí **Traer a Agentes** lo trae con sus palabras. Lo cuenta **Traer otro archivo**, más abajo.

### Una regla nueva, sin escribir la línea `agent:`

Botón derecho sobre la carpeta de un agente de tu equipo, y **Documento nuevo**: el documento ya empieza como una regla de ese agente, con su línea `agent:` puesta.

- En la carpeta de un agente, su archivo principal mientras ningún documento lo tenga: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md` o, en Gemini, `tisty-rules.md`. Después, un archivo de su carpeta de reglas: `rules/rule.md` en Claude, `instructions/rule.instructions.md` en Copilot.
- Codex y Gemini leen un solo archivo de reglas: cuando un documento ya lo tiene, **Documento nuevo** en su carpeta abre ese mismo, para que sumes ahí lo nuevo.
- En **Global**, en **Cursor**, que no tiene archivo de reglas, en Agentes misma y en una subcarpeta tuya, **Documento nuevo** crea un documento común, sin línea `agent:`.

**Un ejemplo.** Marisol instala Gemini CLI, y en su barra aparece **Agentes › Casa (este equipo) › Gemini**, vacía. Botón derecho sobre **Gemini** y **Documento nuevo**; en su ficha pulsa **Editar**, escribe «Responde en español.» y pulsa **Listo**. Como Agentes sincroniza, Tisty lo escribe en `~/.gemini/tisty-rules.md` y agrega la línea `@tisty-rules.md` a su `GEMINI.md`, para que Gemini lo lea.

## Una skill con su `SKILL.md`

Una skill es una carpeta con un `SKILL.md` y, a veces, scripts. Cada archivo es un documento con la misma carpeta de skill en su línea `agent:`. El `SKILL.md` lleva además sus propias propiedades: la ficha toma su `name` como nombre, pone su `description` debajo y el resto en su cuadro de datos, así que el texto empieza donde empieza lo que lee tu agente:

```markdown title="El SKILL.md de la skill pr"
---
agent: [claude/skill/pr/SKILL.md, on-dev_ab12cd34]
title: claude · pr/SKILL.md
---
---
name: pr
description: Abre un PR con el título y el cuerpo en el formato del repositorio.
---

# Abrir un PR

1. Revisa que la rama esté al día.
2. Corre scripts/abrir-pr.sh.
```

Un script se guarda dentro de un bloque de código, y Tisty lo escribe tal cual, sin el bloque:

````markdown title="El script de la skill"
---
agent: [claude/skill/pr/scripts/abrir-pr.sh, on-dev_ab12cd34]
title: claude · pr/scripts/abrir-pr.sh
---
```sh
#!/bin/sh
gh pr create --fill
```
````

Tisty escribe la skill en `~/.claude/skills/pr/`. En la barra lateral, la skill es **una sola fila**, **pr**, con **3 archivos** a la derecha: un clic la abre y muestra `SKILL.md`, `scripts/abrir-pr.sh` y el resto. Una skill de **Global** se escribe para todos: en `~/.claude/skills/`, en `~/.cursor/skills/` y una sola copia en `~/.agents/skills/`, que leen Codex, Copilot y Gemini.

**Las skills de Tisty.** Tisty trae tres skills propias, `tisty`, `consolidate` y `next-agent`, que no son documentos de Agentes: viven dentro de la app y se encienden con su propio sí, en **Configuración › Agentes › Conectados**, aunque nunca abras esta carpeta. En **Agentes › Global** las ves como una fila fija, **Skills de Tisty**, que abre la misma ficha; aparece solo si este equipo tiene algún agente, si las skills están encendidas o si quedan copias suyas. **Traer** no las trae: sus copias salen como **propio de Tisty**. Una skill tuya con el mismo nombre deja fuera la de Tisty, la pausa de un agente no las alcanza y la fila dice **Encendidas** o **Apagadas**, y abre la ficha. Nunca se eliminan. Lo cuenta [Paso a paso con tu agente de IA](tisty:agent).

## Gemini y la línea `@tisty-rules.md`

Gemini escribe su propio `GEMINI.md` cuando le pides que recuerde algo, así que ese archivo es suyo y Tisty nunca lo sobrescribe. Tus reglas van a un archivo aparte, `~/.gemini/tisty-rules.md`, y Tisty agrega **una sola línea** al final de `GEMINI.md` para que Gemini lo lea:

```markdown title="~/.gemini/GEMINI.md"
## Gemini Added Memories
- La perra de Mario se llama Rina.
@tisty-rules.md
```

No toca nada más de ese archivo, y si la línea ya está, no la repite.

**Si borras esa línea**, Tisty lo toma como tu decisión y no la vuelve a poner; solo cuenta si la había puesto Tisty. Gemini queda **En pausa**, y Tisty deja de escribir `tisty-rules.md`. La ficha de cada regla que Gemini lee te lo dice: «Gemini no lee esta regla: falta la línea @tisty-rules.md en GEMINI.md, y Tisty no la vuelve a poner sola.». También lo dicen **Configuración › Agentes › Carpeta Agentes**, bajo **Sincronizar Gemini**, y el aviso **Falta la línea de Tisty** al entrar en Agentes. **Volver a ponerla**, en cualquiera de los tres, agrega otra vez la línea al final de `GEMINI.md` y Gemini vuelve a recibir tus reglas.

## La ficha de un documento de agente

Un documento de Agentes no se ve como un documento más: se abre para leer, en una sola **ficha**, sin columna al lado. Arriba, el camino de carpetas, como «Agentes / Casa (este equipo) / Claude», y después:

- **las etiquetas**: **Regla**, **Skill** o **Memoria del agente**; **Global** o el agente; y el estado, como **Al día**;
- **el nombre**, **Cómo trabajo**; en una skill, su `description` va debajo, en gris;
- **el cuadro de datos del archivo**: **Lo lee**, **Se escribe en** con su botón para copiar la ruta, una fila por archivo en una skill de Global, **Tamaño**, **Respaldos** con **Ver**, **Formato**, **Última escritura**, **Original guardado**, **Sincroniza en este equipo**, **Compartido** cuando el documento tiene un enlace compartido activo y, en filas chicas, las demás propiedades del archivo, como `disable-model-invocation`;
- **el texto del agente**, tal como lo escribiste, con encabezados compactos y sin hoja aparte. Tisty no le impone secciones ni convierte su primera línea en un título, y el archivo sigue siendo el mismo, byte a byte;
- **los avisos y las propuestas**, dentro de la misma ficha, bajo el cuadro de datos: un archivo cambiado por fuera, un documento que viene de otro equipo, dos documentos que van al mismo archivo, un texto que espera tu aprobación o el cambio que te propone un agente;
- **al pie, los botones**, fijos abajo mientras toda la ficha se desplaza: **Editar**, **Respaldos**, **Historial**, **Copiar enlace** y **Pausar la sincronización con Claude**; en una skill de Global, un botón por agente, como **Pausar Claude**, y sus respaldos se abren con **Ver**, en la fila de cada archivo. Al editar, el texto se corre a la derecha para dejar sitio a la barra del editor, como en cualquier documento. Un documento de otro equipo no tiene **Editar**, **Aplicar aquí**, **Eliminar** ni pausa: la ficha dice «Es de otro de tus equipos: en este solo se lee.».

Codex lee su `AGENTS.md` solo hasta 32 KiB. Cuando el archivo se acerca a ese tope, **Tamaño** lo dice en ámbar, «cerca de los 32 KiB que lee Codex», y si lo pasa, en rojo: es el momento de pasar lo largo a una skill.

![La ficha de la regla Reglas sin columna al lado: arriba el camino Agentes / Claude; las etiquetas Regla, Claude y Al día; el nombre en grande; el cuadro de datos con Lo lee Claude, Se escribe en con la ruta de CLAUDE.md y su botón de copiar, Tamaño, Respaldos con Ver, Formato Markdown · LF, Última escritura, Original guardado y Sincroniza en este equipo Sí; debajo, el texto de la regla con encabezados compactos y sin hoja aparte; al pie, los botones Editar, Respaldos, Historial, Copiar enlace y Pausar la sincronización con Claude; en la barra lateral, Agentes abierta, con Global, Claude, la regla y sus skills](agents-ficha.png)

## Un cambio por fuera: traer o sobrescribir

Puedes seguir editando los archivos de tus agentes con tu editor, o dejar que el agente los cambie. Sincronizando, Tisty nunca pisa esos cambios; traída sin sincronizar, no escribe esos archivos ni te pide decidir nada.

**Un ejemplo.** Abres `~/.claude/CLAUDE.md` en tu editor y agregas «- Corre los tests antes de cada commit.». La próxima vez que Tisty mira, la ficha de **Cómo trabajo** dice **Cambiado por fuera**:

- **Ver el archivo** muestra lo que tiene ahora.
- **Traer el cambio** lo escribe en el documento. Lo que decía antes queda en su historial: **Historial**, al pie de la ficha, lo muestra y te deja volver.
- **Sobrescribir con Tisty** pregunta antes y vuelve a escribir el archivo desde el documento. Lo que tenía el archivo queda en un respaldo.

Mientras no eliges, Tisty no escribe ese archivo. Si cambiaste el archivo y también el documento, la ficha dice **Cambiado en los dos lados**, y eliges cuál conservar.

## Recuperar un respaldo

Antes de sobrescribir un archivo, Tisty guarda lo que tenía. El **respaldo n.º 1** es el original, el de antes de que Tisty lo tocara, y nunca se borra; además guarda los últimos 20, así que un archivo puede tener hasta 21. Viven en este equipo y no viajan con la sincronización.

**Respaldos** e **Historial** guardan cosas distintas. Un respaldo es el *archivo* de tu agente, como estaba en el disco antes de que Tisty lo sobrescribiera. El historial son las versiones anteriores del *documento* en Tisty, como las de cualquier otro, y lo cuenta [la portada de esta guía](tisty:guide), en **Volver a lo que decía antes**. La comparación con lo **Aprobado** lee de ese historial: si lo vacías, la ficha te pide leer el texto entero antes de aprobarlo.

1. Al pie de la ficha, pulsa **Respaldos**, o **Ver** en la fila **Respaldos** de su cuadro de datos, junto a «3 respaldos».
2. Elige uno, como **Respaldo n.º 2 · 1,1 kB**. **Ver** lo muestra entero.
3. **Recuperar** pregunta «¿Devolver este respaldo a `~/.claude/CLAUDE.md`? Lo que tiene ahora el archivo queda en un respaldo.» Di que sí.
4. El archivo vuelve a decir lo del respaldo, y la ficha pasa a **Cambiado por fuera**: tu documento no cambió. **Traer el cambio** lo deja igual al respaldo; **Sobrescribir con Tisty**, si te arrepentiste.

Así nunca pierdes ninguno de los dos lados.

## Pausar un agente

Si un día quieres que Tisty no toque los archivos de Claude, por ejemplo mientras pruebas otras reglas a mano:

- en **Configuración › Agentes › Carpeta Agentes**, en la tarjeta **Agentes**, apaga el interruptor **Sincronizar Claude**;
- o al pie de la ficha de un documento, **Pausar la sincronización con Claude**; en una skill de Global, el botón **Pausar Claude**;
- o en la barra, botón derecho sobre **Agentes › Casa (este equipo) › Claude** y **Pausar la sincronización con Claude**. En pausa, el mismo menú ofrece **Reanudar la sincronización con Claude**. Este botón aparece solo mientras Agentes sincroniza: sin sincronizar, no hay nada que pausar.

Para detener todos a la vez, **Dejar de sincronizar**, como cuenta la sección siguiente.

En pausa, Tisty no abre ni escribe esos archivos, y tú sigues editando tus documentos. **Reanudar la sincronización con Claude** escribe lo que cambiaste, salvo un archivo que cambió por fuera mientras tanto, que queda marcado para que decidas. La pausa es solo de este equipo.

## Dejar de sincronizar u ocultar Agentes

**Dejar de sincronizar.** Botón derecho sobre **Agentes** y **Dejar de sincronizar**, o el mismo botón en su tarjeta de **Configuración › Agentes › Carpeta Agentes**. Ningún archivo de tus agentes se toca y ningún documento se borra: tus agentes siguen leyendo lo que Tisty escribió la última vez, y tus documentos siguen en Agentes, para leerlos y editarlos. Agentes vuelve al robot gris, y al abrirla dice arriba «No se sincroniza con tus agentes · Sincronizar».

**Sincronizar otra vez.** El enlace **Sincronizar** de esa línea, **Sincronizar** en su botón derecho o en su tarjeta. Como este equipo ya te había dejado leer los archivos de tus agentes, sincroniza de una vez, sin el aviso de la primera vez. Escribe lo que cambiaste mientras tanto, salvo un archivo que cambió por fuera, que queda marcado para que decidas. Un archivo que solo habías traído y sigue tal cual se toma como está.

**Un ejemplo.** Lucas quiere probar a mano unas reglas nuevas en su `CLAUDE.md` durante una semana. Elige **Dejar de sincronizar**, edita el archivo con su editor y sigue ordenando sus documentos en Agentes. Al final de la semana pulsa **Sincronizar**: la ficha de **Cómo trabajo** dice **Cambiado por fuera**, y elige **Traer el cambio** para quedarse con lo que probó, o **Sobrescribir con Tisty** para volver a lo de sus documentos.

**Sacarla de la barra.** Botón derecho sobre **Agentes** y **Ocultar de la barra**, o **Ocultar** en su tarjeta de **Configuración › Agentes › Carpeta Agentes**, en cualquiera de sus tres estados. Agentes nunca queda oculta y sincronizando, escribiendo archivos que no ves: si sincroniza, antes te avisa «Para ocultar Agentes, primero deja de sincronizar. Tus archivos de agente no se tocan.», con **Dejar de sincronizar y ocultar** y **Cancelar**. **Cancelar** la deja como estaba, a la vista y sincronizando. Lo mismo vale para sus subcarpetas, como **Global**: ocultar una mientras Agentes sincroniza pasa por el mismo aviso y deja de sincronizar Agentes entera. Se va de la barra en este equipo sin dejar rastro, y la encuentras en **Configuración › Aspecto y ventana**, en **Carpetas ocultas**, con **Volver a mostrar**, que la devuelve a la barra en el estado que tenía. **Sincronizar**, en la tarjeta, la sincroniza y también la vuelve a mostrar: sincronizando, siempre se ve.

Oculta, Agentes no sincroniza: no escribe ningún archivo de tus agentes, y nada de esos archivos espera tu decisión. Tus documentos siguen ahí, y si un agente te propone un cambio, la propuesta queda en la ficha de su documento.

**Si venías de la versión anterior**, donde la carpeta se veía en mayúsculas y, mientras no escribía los archivos de tus agentes, atenuada, en gris, con una palabra al lado, no tienes que hacer nada. Si se veía con su color y escribía esos archivos, la encuentras sincronizando, con su robot coral; si se veía atenuada y tenía documentos, traída sin sincronizar. Si la habías ocultado mientras escribía archivos, ahora la encuentras otra vez en la barra, sincronizando: desde ahí puedes dejar de sincronizarla y ocultarla.

Nada de lo que llega por MCP trae, sincroniza, deja de sincronizar, oculta o muestra Agentes: eso lo haces tú, en la ventana de Tisty.

## Traer otro archivo

La primera vez no tiene que ser la única. Si instalas un agente nuevo o escribes a mano un archivo de reglas que antes no existía, lo traes después. La lista es la misma de la primera vez: una skill es una sola fila que entra entera, y lo oculto o lo que mantiene el propio agente no aparece.

**Un ejemplo.** Escribes en tu editor una regla nueva para Claude Code:

```markdown title="~/.claude/rules/pruebas.md"
# Pruebas

- Corre los tests antes de cada commit.
```

1. En la barra, botón derecho sobre una subcarpeta de Agentes, como **Agentes › Casa (este equipo) › Claude**, y **Traer otro archivo**. Con Agentes o una de sus subcarpetas abierta, también está en el botón **+** junto a **Biblioteca** y en la paleta de comandos, en **Crear**; y siempre en **Configuración › Agentes › Carpeta Agentes**, en la tarjeta **Agentes**. No cambia si Agentes sincroniza: traída sin sincronizar, el archivo de tu agente sigue como está.
2. El aviso dice «Estos son los archivos de agentes de este equipo que todavía no están en Agentes. Elige qué traer:». Esta vez **no viene nada marcado**, y lo que ya tiene documento dice **ya está en Agentes**. **Ahora no** lo cierra sin traer nada.
3. Marca `rules/pruebas.md` y pulsa **Traer**. En **Agentes › Casa (este equipo) › Claude** aparece **Pruebas**, y desde ahí la lees y la cambias en Tisty.

**Buscar de nuevo.** Si instalaste un agente o tu agente creó archivos nuevos, botón derecho sobre **Agentes** misma, traída o sincronizando, y **Buscar de nuevo**. Tisty vuelve a buscar tus agentes en este equipo, crea la subcarpeta del que falte y abre **Traer otro archivo** con **solo lo nuevo ya marcado**; la memoria, como siempre, sin marcar. Si no queda nada por traer, ni siquiera una memoria sin marcar, dice «No encontré nada nuevo. Esto es lo que hay en cada agente:» y una línea por agente, como «Claude · ya está en Agentes». En un equipo que todavía no te dio permiso, abre en cambio el aviso de la primera vez, con sus nombres y tamaños, y no abre ningún archivo ni crea ninguna carpeta antes de tu sí.

**Traer no toca nada**, ni siquiera la cuenta de lo que Tisty escribió: un archivo que solo trajiste nunca se ofrece para quitar, aunque después saques su documento de Agentes.

**Un archivo que nunca trajiste.** Si `~/.codex/AGENTS.md` ya tenía tus reglas de Codex y la primera vez lo dejaste sin marcar, y después escribes un documento de Codex que va a ese mismo archivo, Tisty no sabe qué parte es tuya y no lo pisa: te pregunta **Existe en los dos lados**, con «Tisty nunca trajo este archivo. Si lo traes a Agentes, queda como está, como un documento de este equipo.». **Traer a Agentes** lo trae como documento, con sus palabras tal como están, y lo que tenía antes queda en sus respaldos. Un documento es un archivo: Tisty no le suma nada.

`AGENTS.override.md` es el archivo que Codex lee en vez de `AGENTS.md` cuando existe, y para Tisty es otro archivo y otro documento: si lo creas después, tráelo con **Traer otro archivo**, y queda junto al `AGENTS.md`.

## Eliminar una regla o una skill

Cuando eliminas de Agentes una regla o una skill que Tisty escribió en la carpeta de un agente, con **Eliminar** en su menú, Tisty te pregunta ahí mismo si también la quita de tus agentes. Es un solo aviso: arriba, lo de siempre (lo que se va con el documento, sus idiomas y sus páginas, y que no se puede deshacer); debajo, en negrita, «¿Quitar también de Claude, Codex y Copilot?», con los agentes que la tienen:

- **Quitar también** elimina el documento y saca sus archivos de las carpetas de tus agentes, con un respaldo de cada uno en este equipo.
- **Solo de Tisty** elimina el documento y deja los archivos donde están: tus agentes los siguen leyendo.
- **Cancelar** no elimina nada.

**Un ejemplo.** Yisset ya no quiere su skill de Global **voz**, que Tisty puso en `~/.claude/skills/voz/` y en `~/.agents/skills/voz/`, la que leen Codex y Copilot. Elige **Eliminar**, y el aviso dice «¿Borrar «voz»? El documento y su contenido se van para siempre. Esto no se puede deshacer.» y «¿Quitar también de Claude, Codex y Copilot?». Con **Quitar también**, los dos archivos se van, cada uno con su respaldo, y ninguno de sus agentes vuelve a leer esa skill.

El aviso dice aparte lo que no se va, y nada se va sin que lo veas:

- **Una copia que cambió fuera de Tisty**: «Cambió fuera de Tisty en Codex y Copilot: esa copia se queda y la ves después, aparte.». Después de eliminar, Tisty te pregunta por ella con **¿Quitar también el archivo?**, que cuenta la sección de abajo.
- **La copia que lee un agente en pausa**: «En pausa: Copilot. Su copia se queda y Tisty vuelve a preguntar cuando sincronice.».

**Lo que dejas con Solo de Tisty sigue siendo tuyo.** La tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes** lo lista aparte, como «2 archivos que dejaste en tus agentes al eliminarlos de Agentes.», con **Revisar**, por si cambias de idea; si eliminaste el último documento de Agentes y la carpeta se fue, la lista sigue ahí. Tisty no te pregunta solo por esos archivos y nunca les escribe encima: si una regla o skill nueva cae otra vez en el mismo archivo, te pregunta antes. Esa lista es solo de este equipo.

## Sacar una regla y su archivo

Cuando mueves una regla fuera de Agentes, la mueves a la carpeta de otro equipo, dejas sin palabras su texto o vuelves a una versión sin ficha, Tisty deja de escribirla. Su archivo sigue en el disco y tu agente lo sigue leyendo, así que alrededor de un segundo después Tisty te pregunta **¿Quitar también el archivo?**, con la ruta del archivo, por ejemplo `~/.claude/rules/pruebas.md`, y «Si lo quitas, queda un respaldo en este equipo.»:

- **Quitar el archivo** guarda un respaldo en este equipo y lo borra: Claude deja de leer esa regla.
- **Dejarlo** lo deja donde está, y Tisty no lo vuelve a escribir ni a preguntar. Si un día lo quieres de vuelta, aparece en **Traer otro archivo**.

Si lo editaste a mano después de que Tisty lo escribió, el aviso te lo dice antes de que elijas. Nunca pregunta por la memoria de tu agente, ni por un agente en pausa, ni mientras Agentes no sincroniza: entonces Tisty no escribía ningún archivo.

**Eliminar una subcarpeta.** Botón derecho sobre una subcarpeta, como **Agentes › Casa (este equipo) › Claude**, y **Eliminar**. Tisty pregunta antes, y sus documentos pasan a **Sin carpeta**: como salen de Agentes, Tisty deja de escribirlos y te pregunta por cada archivo que escribían, igual que al sacar una regla, siempre con su respaldo. La carpeta no vuelve sola.

**Un ejemplo.** Marisol guardaba sus reglas de pruebas en una carpeta suya, **Agentes › Casa (este equipo) › Claude › Pruebas**, y ya no las quiere. Botón derecho sobre **Pruebas** y **Eliminar**; un segundo después, Tisty le pregunta **¿Quitar también el archivo?** por `~/.claude/rules/pruebas.md`. Elige **Quitar el archivo**: el respaldo queda en este equipo y Claude deja de leer esas reglas.

**Vaciar Agentes.** Si sacas todos sus documentos con Tisty abierto, primero te pregunta por cada archivo que escribía, y después la carpeta desaparece. En la barra queda Agentes en gris, como al principio: un clic en ella abre otra vez el aviso de la primera vez, con tus reglas y skills ya marcadas.

## Un segundo equipo

Tus documentos de Agentes viajan con tu sincronización a tus otros equipos, pero **cada equipo tiene su propia carpeta y escribe solo la suya**. Lo que hay en la carpeta de otro equipo lo lees, lo mueves, lo copias como texto y lo sacas; no lo editas, no lo aplicas ni lo eliminas desde este.

**Un ejemplo.** En el equipo de la casa elegiste **Traer y sincronizar**: tu `CLAUDE.md`, **Cómo trabajo**, quedó en **Agentes › Casa (este equipo) › Claude**. En el equipo del trabajo:

1. En la barra, Agentes llega traída, sin sincronizar: el robot gris y, al final y en gris tenue, la carpeta **Casa** con sus documentos, que ya puedes leer. Arriba dice «No se sincroniza con tus agentes · Sincronizar».
2. Pulsa **Sincronizar**. Como este equipo nunca te dejó leer los archivos de sus agentes, se abre el aviso de la primera vez: marca lo que quieras traer de este equipo y pulsa **Traer y sincronizar**. Se crea la carpeta de este equipo, con una subcarpeta por cada agente que encontró, y Agentes queda con su robot coral.
3. Abre **Cómo trabajo**, el de **Casa**. La ficha dice «Es de otro de tus equipos: en este solo se lee.», sin **Editar** ni **Aplicar aquí**: en el trabajo no se escribe en ningún archivo.
4. Si quieres esa misma regla en los dos equipos, muévela a la raíz: botón derecho sobre el documento, **Mover a** y **Agentes**. En el equipo donde la mueves cuenta como aplicada; en el otro, la ficha dice «Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este?». **Aplicar aquí** lo escribe en los archivos de ese equipo, con su respaldo antes, y **No en este equipo** lo deja fuera, con **Aplicar aquí** a mano si cambias de idea.

Desde ahí, lo que ya aplicas se pone al día solo cuando lo cambias en el otro equipo. Lo que escribes o traes en un equipo que sincroniza se aplica en ese mismo equipo sin ningún clic. Mover un documento a la carpeta de otro equipo deja de aplicarlo en este, y Tisty te pregunta por su archivo. Un documento con una propuesta de un agente esperando no se mueve a otro equipo hasta que la apruebes o la descartes. Y la memoria de un agente es del equipo que la trajo: en los demás solo se lee.

**Dos documentos al mismo archivo.** Si el documento de tu carpeta y uno de la raíz escriben el mismo archivo, o la misma skill, en este equipo no cambia nada hasta que elijas: la ficha dice «En este equipo «X» ya va al mismo archivo. ¿Cuál manda aquí?», con **Este** y **El otro**. Una skill se elige entera, y se elige por archivo: si eliges para Claude, una skill de Global sigue funcionando en los demás agentes. El que no manda se queda en su carpeta, y puedes cambiar de idea con **Este**. Si el que mandaba se va, Tisty no escribe el otro por su cuenta: vuelve a preguntar.

**Aprobar no es aplicar.** Si en el equipo del trabajo apruebas una propuesta, pulsas **Aprobar este texto** o vuelves a una versión anterior de un documento de la raíz que espera, cambias su texto en todos tus equipos, pero ahí sigue esperando. La ficha dice «Aprobaste su texto en este equipo, pero aplicarlo aquí es aparte. ¿Lo aplicas a los agentes de este?», con los mismos dos botones. Solo **Aplicar aquí** escribe sus archivos en ese equipo.

**Dos carpetas Agentes.** Si la casa y el trabajo crearon Agentes antes de sincronizarse, al encontrarse aparecen dos por un momento. Tisty las une solo, en la más antigua: junta sus subcarpetas por nombre (Global, la de cada equipo…), pasa los documentos y borra la que quedó vacía. No toca ningún archivo de tus agentes, no te ofrece quitar ninguno ni pierde una aprobación. Si la segunda Agentes tiene algo que puso un agente, Tisty la deja como está hasta que muevas tú ese documento a una carpeta tuya.

**Si venías de la 1.50.1.** Lo que cada equipo trajo o escribió pasa solo a su carpeta cuando todos tus equipos tienen la 1.50.2 o una más nueva, y ningún archivo de un agente cambia ni en un byte. Un `CLAUDE.md` que la 1.50 había juntado con una regla de Global pasa a ser un documento que dice exactamente lo que dice el archivo. Mientras otro equipo siga en una versión anterior, nada se mueve. Una regla de Global de esa época se queda ahí para que la leas, la copies o la borres.

Dos avisos más que puedes ver en una ficha:

- **«Su texto aún no tiene tu aprobación, así que no llega a ningún agente hasta que lo apruebes.»**: un aviso ámbar, no una etiqueta. El texto del documento cambió sin pasar por tus manos, por ejemplo con un paquete importado. Antes del botón **Aprobar este texto**, el aviso te dice quién lo escribió, como «Se escribió en otro equipo.» o «Cambió fuera de Tisty.», y qué cambió: las líneas comparadas, en rojo con `−` lo **Aprobado** y en verde con `+` lo de **Ahora**. Si nunca se aprobó, dice «Nunca se aprobó: léelo entero antes de aprobarlo.»; si lo último aprobado no está en este equipo, que no hay con qué compararlo y que lo leas entero. El botón pregunta «¿Apruebas lo que dice ahora este documento? Léelo antes: llega a los archivos de tus agentes en cada equipo donde Agentes sincroniza y lo aplicaste.», y si el texto cambió entre que se mostró y tu clic, no aprueba nada. Si ese documento ya se escribía en un archivo, el archivo queda **En espera**, como estaba.
- **Escrito por un Tisty anterior**: una versión anterior de Tisty escribió el archivo y hoy no dice lo mismo que tus documentos. **Ver el archivo** y, si te quedas con el documento, **Sobrescribir con Tisty**.

## Lo que tu agente puede y no puede

En la red de Brainsty, que cuenta la página **Brainsty: todo unido** de esta guía, tus documentos de Agentes vienen ocultos, para que no tapen lo tuyo: aparecen, chicos y en coral tenue, cuando enciendes **Mostrar AGENTS** en sus filtros, y Tisty nunca los marca como parecidos de nada.

Tu agente **lee** tus documentos de Agentes, como cualquier otro, sincronice o no: le pides «¿qué reglas mías lees?» y te responde con sus títulos. Pero **no los escribe por su cuenta**: cuando cambia uno, su cambio queda aparte, como una propuesta, y solo llega a tus agentes si lo apruebas. Sincronizando, una regla de Agentes llega a todos tus agentes, en cada equipo donde Agentes sincroniza y la aplicaste, y nadie más que tú debería decidir qué dice. Un documento de la carpeta de otro de tus equipos lo lee como datos, no como una regla suya para este equipo, y no puede proponer sobre él; en los documentos de este equipo propone como siempre.

Cualquier otra herramienta que nombre esos documentos o la carpeta `AGENTS` se rechaza: no puede moverlos, archivarlos ni marcarlos, proponer sobre una memoria, ni mover, renombrar o archivar Agentes. Tampoco puede agregar ni cambiar una línea `agent:` en ningún documento, de ninguna carpeta, ni con `edit_doc`, ni crear un documento nuevo con esa línea. Si lo intenta, Tisty se lo rechaza sin tocar nada y le dice cómo proponer el cambio o que te cuente qué cambiaría.

## Una propuesta de tu agente

**Un ejemplo.** Le pides a tu agente «agrega a mis reglas que corra los tests antes de cada commit».

1. El agente corrige **Cómo trabajo** con `edit_doc`, como cualquier documento. Tisty guarda su cambio aparte, byte a byte, y le responde que quedó propuesto: el documento y `~/.claude/CLAUDE.md` siguen como estaban. Su guía le pide decírtelo así, «te lo propuse», y no darlo por hecho.
2. Abre **Cómo trabajo**. Si Tisty estaba abierto, la propuesta aparece sola en uno o dos segundos, aunque la ventana no esté al frente.
3. La ficha suma la etiqueta **Propuesta** y un recuadro morado: «Un agente (Claude Code) propone un cambio», y las líneas comparadas, en rojo con `−` lo **Aprobado** y en verde con `+` lo **Propuesto**.
4. **Aprobar el cambio** pregunta «¿Apruebas este cambio? Se escribe en el documento, y llega a los archivos de tus agentes en cada equipo donde Agentes sincroniza y lo aplicaste.». Con tu sí, el documento pasa a decir lo propuesto y Tisty lo escribe en `CLAUDE.md`, con su respaldo antes. **Descartar** la quita y el documento queda como lo aprobaste.

Cada botón actúa solo sobre lo que la ficha te mostró: si el agente volvió a escribir mientras mirabas, Tisty te pide revisarla de nuevo. Si vuelve a escribir, su cambio va encima de su propia propuesta, así que hay una sola por documento. Y mientras una propuesta espera, lo que ya aprobaste sigue llegando a tus agentes.

## Más en la ayuda

Cada estado, cada agente con sus archivos y sus variables, lo que nunca se lee y qué hacer cuando algo no cuadra están en [la carpeta Agentes](https://www.tisty.app/es/ayuda/carpeta-agents/), en la ayuda. Qué guarda Tisty de todo esto, y qué viaja, en [privacidad](https://www.tisty.app/es/ayuda/privacidad/). Y cómo conectar tu agente a Tisty, en [Paso a paso con tu agente de IA](tisty:agent).
