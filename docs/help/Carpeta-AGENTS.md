# Agentes: las reglas, skills y memoria de tus agentes

Uso más de un agente de IA, y cada uno lee sus reglas de un archivo distinto, en una carpeta distinta: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Cambiar una regla en todos era abrir cuatro archivos, y tarde o temprano uno quedaba atrás. La carpeta **Agentes** junta esos archivos en Tisty, como documentos, para que los leas y los ordenes: un documento es un archivo, palabra por palabra. Y si quieres, Tisty escribe por debajo el archivo que lee cada agente.

> [!TIP]
> **En corto:** la carpeta **Agentes** guarda como documentos las reglas, las skills y la memoria de Claude Code, Codex, GitHub Copilot CLI, Gemini CLI y Cursor. Hay dos palabras, y cada una significa una sola cosa. **Traer** copia los archivos de tus agentes a Tisty, para leerlos y ordenarlos, y no escribe nada en ellos. **Sincronizar** hace que, desde ese momento, lo que cambias en Tisty llegue a esos archivos, y que Tisty te avise si alguien los cambia por fuera. Sincronizando, cada equipo escribe solo lo de su propia carpeta: las skills de **Global** llegan a todos tus agentes, y lo que escribes en la carpeta de un agente de tu equipo, solo a él. Antes de sobrescribir un archivo Tisty guarda lo que tenía, y un cambio hecho por fuera nunca se pisa. Y un agente no escribe tus reglas: te propone el cambio, y solo tú lo apruebas.

![La primera vez de Agentes en español: el aviso Tus agentes, en un solo lugar, con la X para cerrar arriba a la derecha, la línea Tisty busca las reglas, skills y memoria de tus agentes y las trae aquí como documentos, para leerlas y ordenarlas. Traer solo las copia; Traer y sincronizar además hace que lo que cambies en Agentes llegue solo a tus agentes., sus tres líneas con candado, flechas y copias (Queda en tu Tisty…, Si sincronizas, llega solo a Claude, Codex, Copilot, Gemini., Antes de cambiar un archivo de un agente, Tisty guarda un respaldo en este equipo.), y debajo Encontré esto. Elige qué traer:, la línea Gemini: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta., y tres filas plegadas, Claude · 2 reglas · 29 skills · 49 memorias, Codex · 1 regla · 4 skills y Copilot · 1 regla; abajo a la derecha, solo dos botones, Traer y Traer y sincronizar, este destacado](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-01-primera-vez.png)

## Qué es y cuándo te sirve

**Agentes** es una carpeta de tu Biblioteca, arriba de todo, con el icono de un robot. Está en tu barra lateral desde el primer día, en gris, hasta que traes algo. Cada documento que tiene adentro es un archivo de un agente: una regla, una skill o su memoria. Lo lees y lo editas en Tisty, con su buscador y sus versiones anteriores, pero no se ve como un documento más: se abre en su propia [ficha](#la-ficha-de-un-documento-de-agente), que te dice quién lo lee, dónde se escribe y cuántos respaldos tiene, con el texto del agente tal como lo escribiste. Si la sincronizas, Tisty escribe el archivo que ese agente lee, byte a byte.

Te sirve cuando:

- **Quieres leer y ordenar las reglas que ya tienes.** Con **Traer**, tu `CLAUDE.md`, tus skills y tus reglas de Copilot entran como documentos, con títulos y búsqueda, en vez de archivos planos en carpetas ocultas. Tus agentes siguen leyendo sus archivos como estaban.
- **Usas varios agentes y quieres la misma skill en todos.** Con **Sincronizar**, escribes la skill una vez en **Global**, y llega a Claude Code, a Cursor y a la carpeta que comparten Codex, Copilot, Gemini y DeepSeek Harness. Las reglas, en cambio, son de cada agente: cada uno lee su propio archivo, y lo que dice ese archivo es lo que dice tu documento.
- **Trabajas en más de un equipo.** Cada equipo tiene su propia carpeta dentro de Agentes y escribe solo la suya: lo del trabajo no pisa lo de la casa, y lo de otro equipo lo lees, pero no lo cambias desde este.
- **Tienes skills con scripts.** Cada archivo de la skill es un documento, y un script se guarda en un bloque de código que se escribe tal cual.
- **Quieres poder volver atrás.** Sincronizando, Tisty guarda el archivo original antes de tocarlo por primera vez, y los últimos 20 antes de cada sobrescritura.
- **Quieres ver lo que tu agente anotó de ti.** La memoria que escribe Claude Code en cada proyecto, o el `GEMINI.md` de Gemini, se puede traer para leerla en Tisty, sin que Tisty la cambie nunca.

No te sirve para los archivos de cada repositorio: Agentes maneja los archivos de tu usuario, los que están en tu carpeta personal, no el `AGENTS.md` o el `CLAUDE.md` que vive dentro de un proyecto.

## Traer o sincronizar

Son dos pasos distintos, y puedes quedarte en el primero todo lo que quieras.

| | **Traer** | **Sincronizar** |
| :-- | :-- | :-- |
| Qué hace | Copia los archivos que marcas a Tisty, como documentos | Desde ese momento, lo que cambias en Tisty llega a los archivos de tus agentes |
| Los archivos de tus agentes | Quedan como están: Tisty no escribe nada en ellos | Tisty los escribe con lo que dicen tus documentos, con un respaldo antes de cada sobrescritura |
| Un cambio hecho por fuera | Queda solo en el archivo: Tisty no te pide decidir nada | Tisty te avisa en la ficha del documento y espera que elijas |
| Tus documentos | Quedan en Agentes, y los editas como cualquier documento | Lo mismo |
| Qué pide | Que este equipo te deje leer los archivos que marcas | Lo mismo, una vez por equipo |
| Cómo se deshace | Borras o mueves los documentos, como cualquier otro | **Dejar de sincronizar**: ningún archivo de tus agentes se toca y ningún documento se borra |

**Dos ejemplos.**

- Jaime usa solo Claude Code y quiere leer sus reglas y sus skills con calma, sin que nada cambie en `~/.claude`. Elige **Traer**. Sus documentos quedan en **Agentes › Casa (este equipo) › Claude**, y Claude Code sigue leyendo su `CLAUDE.md` como siempre. Si edita uno de esos documentos, el cambio queda en Tisty y nada más.
- Yisset usa Claude Code y Codex, y quiere que los dos sepan que sus ramas van con la forma `tipo/nombre`. Elige **Traer y sincronizar**, escribe la regla en el documento de Claude y la misma en el de Codex, dentro de la carpeta de su equipo, y Tisty las escribe en su `CLAUDE.md` y en su `AGENTS.md`. Si lo que quiere compartir es una skill, esa sí la escribe una vez en **Agentes › Global**.

Si un día Jaime quiere que sus cambios lleguen a Claude Code, pulsa **Sincronizar** y Agentes pasa a sincronizar, sin volver a traer nada. Si Yisset quiere dejar todo quieto un tiempo, **Dejar de sincronizar**: sus archivos quedan como Tisty los dejó y sus documentos siguen en Agentes.

## Tres estados en la barra

La barra te dice cómo está Agentes con su color, y al lado no aparece ninguna palabra. Lo que espera tu decisión lo ves en la ficha de cada documento y en **Configuración › Agentes › Carpeta Agentes**.

| Estado | Cómo se ve | Un clic | Su botón derecho |
| :-- | :-- | :-- | :-- |
| **1. Todavía sin traer** | El robot y el nombre **Agentes** en gris | Abre el aviso de la primera vez, [Tus agentes, en un solo lugar](#la-primera-vez) | **Ocultar de la barra** |
| **2. Traída, sin sincronizar** | El robot en gris y el nombre como cualquier carpeta; al pasar el puntero, cuántos documentos tiene, como cualquier carpeta | La abre como cualquier carpeta; arriba, una línea discreta: «No se sincroniza con tus agentes · Sincronizar» | **Sincronizar**, [**Buscar de nuevo**](#buscar-de-nuevo) y **Ocultar de la barra**, nada más |
| **3. Sincronizando** | El robot **coral** y el nombre como cualquier carpeta; al pasar el puntero, cuántos documentos tiene, como cualquier carpeta | La abre como cualquier carpeta | **Dejar de sincronizar**, [**Buscar de nuevo**](#buscar-de-nuevo) y **Ocultar de la barra**, que antes te pide dejar de sincronizar |

- **El coral es solo de Agentes**, y solo mientras sincroniza: el morado es de los diagramas, y ninguna carpeta tuya puede elegir el coral. En la red de Brainsty, los documentos de Agentes vienen ocultos, sincronice o no, para no tapar lo que escribes tú. Aparecen cuando enciendes **Mostrar AGENTS** en los filtros: más chicos que los tuyos, en un coral tenue y con sus líneas más tenues, y la leyenda los nombra **AGENTS**. Nunca son parecidos de nada. Mira [[Filtrar|Red-de-Brainsty#filtrar]].
- **La línea del estado 2** solo aparece en Agentes, nunca en sus subcarpetas. Su enlace **Sincronizar** hace lo mismo que el del botón derecho.
- **Con la barra plegada**, el robot de Agentes tiene el mismo color. Abierta o plegada, un lector de pantalla dice el estado en palabras: «Agentes — todavía sin traer» o «Agentes — No se sincroniza con tus agentes».
- **El botón derecho** también se abre con la tecla de menú o con `Shift` + `F10`, con Agentes seleccionada.
- **El botón derecho de Agentes misma** solo ofrece sincronizar, buscar de nuevo y ocultar, según su estado: ninguna otra acción de carpeta. Sus subcarpetas, como **Agentes › Claude**, tienen un menú solo de agentes, que cuenta [El botón derecho de una subcarpeta](#el-botón-derecho-de-una-subcarpeta). Por qué Agentes no se mueve ni cambia de nombre lo cuenta [Agentes se queda arriba](#agentes-se-queda-arriba).

## La primera vez

Agentes está en tu barra lateral aunque nunca la hayas usado, en gris. Así sabes que existe, y nada de tus agentes se lee hasta que tú lo dices.

1. En la barra, haz clic en **Agentes**. Se abre **Tus agentes, en un solo lugar**, con una X arriba para cerrarlo. Tisty miró qué agentes tienes en este equipo y qué archivos hay en sus carpetas, **solo sus nombres y tamaños**: todavía no abrió ninguno.
2. Arriba, una línea te cuenta las dos palabras: «Tisty busca las reglas, skills y memoria de tus agentes y las trae aquí como documentos, para leerlas y ordenarlas. Traer solo las copia; Traer y sincronizar además hace que lo que cambies en Agentes llegue solo a tus agentes.». Debajo, tres más:
   - 🔒 «Queda en tu Tisty: en tus equipos y donde tú sincronices. Nadie más lo ve.»
   - ↻ Si sincronizas, a quién llega, con los agentes que encontró: «Si sincronizas, llega solo a Claude, Codex, Gemini.».
   - ⧉ «Antes de cambiar un archivo de un agente, Tisty guarda un respaldo en este equipo.»
3. Debajo, **Encontré esto. Elige qué traer:**, una fila por agente, con lo que tiene para traer, como **Claude · 2 reglas · 1 skill · 1 memoria**. Ábrela con un clic: cada regla y cada memoria es una fila con su casilla, y cada skill es **una sola fila**, entera, como **pr · Skill · 3 archivos**. Pasa el puntero sobre una fila para ver su ruta completa.
4. Marca lo que quieres traer. Una skill entra entera con su casilla: su `SKILL.md`, sus scripts y sus referencias. Las reglas y las skills vienen marcadas; **la memoria del agente viene sin marcar**, porque una vez en Tisty viaja con tu sincronización, y la marcas tú si la quieres.
5. Elige uno de los dos botones:
   - **Traer** copia lo que marcaste y nada más. Los archivos de tus agentes quedan como están, y Agentes queda en el [estado 2](#tres-estados-en-la-barra): traída, sin sincronizar. El aviso dice, por ejemplo, «Entraron 7 documentos a Agentes. Los archivos de tus agentes quedan como están hasta que elijas Sincronizar.». No se activa hasta que marcas algo.
   - **Traer y sincronizar**, el destacado, copia lo que marcaste y además sincroniza: desde ahora, lo que cambias en Agentes llega a los archivos de tus agentes. Agentes queda en el estado 3, con su robot coral, y el aviso dice «Entraron 7 documentos a Agentes, y desde ahora Tisty los mantiene al día.».
6. Tisty abre solo los archivos que marcaste y los guarda como documentos. Lo que no pudo entrar aparece bajo **Esto no entró:**, con el motivo. **Listo** cierra el aviso.

**Cerrar sin elegir.** La X o `Esc` cierran el aviso sin leer ni traer nada, y Agentes sigue en gris en la barra. Vuelves cuando quieras con otro clic en ella, o desde la tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes**, donde **Traer** y **Traer y sincronizar** abren este mismo aviso. Lo que entra en la lista, y lo que no, lo cuenta [Qué entra en la lista](#qué-entra-en-la-lista).

**Si no la quieres ver.** El aviso no tiene un botón para ocultarla: botón derecho sobre **Agentes**, en gris, y **Ocultar de la barra**. Lo cuenta [Ocultarla de la barra](#ocultarla-de-la-barra).

### Qué entra en la lista

La lista trae solo lo que tu agente lee de verdad. Lo demás no aparece, ni atenuado.

**Skills.** Una skill es una carpeta con un `SKILL.md` en su raíz. Tisty la busca tan abajo como la busca cada agente, así que una skill que guardas en una carpeta de categoría, como `~/.agents/skills/trabajo/pr/`, aparece cuando su agente la lee desde ahí:

| Agente | Carpeta de skills | Dónde puede estar una skill |
| :-- | :-- | :-- |
| Claude Code | `~/.claude/skills` | justo adentro, como `skills/pr/` |
| Codex | `~/.agents/skills`, que comparte con Copilot, Gemini y DeepSeek Harness | hasta seis carpetas más abajo, como `skills/trabajo/pr/` |
| DeepSeek Harness | `~/.agents/skills`, que comparte con Codex, Copilot y Gemini | hasta seis carpetas más abajo, como `skills/trabajo/pr/` |
| GitHub Copilot CLI | `~/.copilot/skills` | justo adentro |
| Gemini CLI | `~/.gemini/skills` | justo adentro |
| Cursor | `~/.cursor/skills` | en cualquier carpeta de categoría |

Una carpeta sin `SKILL.md` que su agente no lee más abajo no es una skill y no aparece. Tisty nunca busca una skill dentro de otra: un `SKILL.md` más adentro de `pr/` es un archivo más de **pr**. Cada skill es **una fila**, como **pr · Skill · 3 archivos**, con una casilla que la trae entera: cada archivo de la skill queda como un documento, y en la barra lateral se ve como una sola fila, **pr**, con **3 archivos** a la derecha. Un archivo de credenciales dentro de una skill, como una llave `.pem`, sigue sin entrar.

**Reglas.** De una carpeta de reglas entra solo lo que el agente lee:

- Claude Code: los `.md` de `~/.claude/rules`, también en sus subcarpetas.
- GitHub Copilot CLI: los `.instructions.md` de `~/.copilot/instructions`, también en sus subcarpetas. Un `.md` común ahí no aparece, porque Copilot tampoco lo lee.
- Una carpeta dentro de una carpeta de reglas que tiene su propio `manifest.json` queda fuera entera: es de otro programa, no tuya.

**Memoria del agente.** Solo sus archivos `.md`.

**Lo que nunca aparece:**

- **Un archivo o una carpeta oculta**, cuyo nombre empieza con un punto, a cualquier profundidad. Eso deja fuera, por ejemplo, `skills/.trash/` de Claude Code y `skills/.system/` de Codex, sus skills de sistema, que Codex reescribe solo.
- **`skills/synced/` de Claude Code**, con cualquier mezcla de mayúsculas: es la copia de las skills de tu cuenta de claude.ai, que Claude Code puede sobrescribir o borrar, así que no es tuya para editar.

**Un ejemplo.** Tu carpeta de Claude tiene:

```text
~/.claude/
├── CLAUDE.md
├── rules/
│   ├── estilo.md
│   └── notas.txt
└── skills/
    ├── pr/
    │   ├── SKILL.md
    │   ├── scripts/abrir-pr.sh
    │   └── plantilla.md
    ├── notas-sueltas/
    │   └── ideas.md
    ├── .trash/
    │   └── …
    └── synced/
        └── …
```

La fila de Claude dice **Claude · 2 reglas · 1 skill**: `CLAUDE.md` y `rules/estilo.md` como **Regla**, y **pr · Skill · 3 archivos**. `notas.txt` no aparece, porque no es un `.md`; `notas-sueltas`, porque no tiene `SKILL.md`; `.trash`, porque es oculta, y `synced`, porque es la copia que Claude Code mantiene de tu cuenta de claude.ai.

**La fila de cada agente** cuenta reglas, skills y memorias, no archivos: una skill de diez archivos suma una, como en **Claude · 2 reglas · 3 skills · 1 memoria**. Si no queda nada que traer, dice **Claude · nada para traer**, y si todo ya está en Agentes, **Claude · ya está en Agentes**.

**Un agente sin archivos todavía** no queda fuera, como si no estuviera: Tisty lo nombra en su propia línea, sobre la lista. Si instalaste Gemini ayer y aún no le pediste que recuerde nada, ves «Gemini: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta.». Lo mismo dicen **Traer otro archivo**, [Buscar de nuevo](#buscar-de-nuevo) y su interruptor en **Configuración › Agentes › Carpeta Agentes**.

### Lo que dice la lista junto a un archivo

| En la lista | Qué significa |
| :-- | :-- |
| **Regla**, **Skill**, **Memoria del agente** | Se puede traer. La memoria viene sin marcar |
| **ya está en Agentes** | Un documento de Agentes ya lo representa, así que se compara en vez de traerlo de nuevo |
| **credenciales, no entra** | Su nombre es el de un archivo de credenciales, como `credentials.json`, `auth.json`, o una llave `.pem`, `.key` o `id_rsa`. Los que empiezan con un punto, como `.env` o `.credentials.json`, ni siquiera aparecen |
| **propio de Tisty** | Un archivo que dejó Tisty o uno temporal, como `tisty.json` o uno que termina en `.tmp`, o la copia de una de las [skills de Tisty](#las-skills-de-tisty-en-global) |
| **es un enlace, no entra** | Es un enlace simbólico: Tisty no lo sigue |
| **demasiado grande** | Pasa de 512 kB, lo que cabe en un documento |
| **demasiados archivos** | Tisty ya había visto 2.000 archivos en las carpetas de tus agentes y dejó de mirar |

Lo que aparece atenuado no se puede marcar. Los archivos y las carpetas ocultos, los que empiezan con un punto, nunca aparecen. Tisty tampoco entra en `node_modules`, ni baja más de ocho niveles.

### Si un archivo no entra

Después de traer, un archivo marcado puede quedar fuera:

- **Parece llevar una credencial, como una clave o un token, así que no entra.** Tisty lo revisó en este equipo y encontró algo con forma de secreto. Saca el secreto del archivo y vuelve a intentarlo.
- **No es texto.** Un archivo binario no puede ser un documento.
- **Su nombre no se puede guardar como documento de agente.** Un nombre con `#`, `:`, `,`, comillas, corchetes o una barra invertida no cabe en la línea que dice dónde va el archivo.

### Si todavía no tienes archivos de agentes

El aviso dice «No encontré archivos de agentes en este equipo. Igual puedes sincronizar: Agentes se llena a medida que escribes tus reglas.». Debajo, cada agente que encontró sin archivos tiene su línea, como «Codex: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta.». **Traer** queda desactivado, porque no hay nada que traer, y **Traer y sincronizar** sigue disponible: pulsarlo deja que este equipo sincronice y crea igual la carpeta **Agentes**, con **Global** y la carpeta de este equipo, con una subcarpeta vacía por cada agente que encontró, como **Codex** y **Gemini**. El aviso dice «No entró nada nuevo. La carpeta Agentes ya sincroniza en este equipo: lo que escribas ahí llega a tus agentes.», y en la barra Agentes queda con su robot coral. Entrar en ella no vuelve a abrir el aviso.

Desde ahí tienes dos caminos:

- **Escribir tu primera regla.** Botón derecho sobre la carpeta de un agente de tu equipo, como **Casa (este equipo) › Gemini**, y **Documento nuevo**: el documento ya empieza como una regla de ese agente, con su línea `agent:` puesta, como cuenta [Un documento nuevo ya es una regla](#un-documento-nuevo-ya-es-una-regla). Tisty escribe el archivo apenas escribes algo en él.
- **Traer lo que aparezca después.** Cuando tu agente cree sus archivos, por ejemplo después de usarlo por primera vez, tráelos con [Traer otro archivo](#traer-otro-archivo).

## Traer otro archivo

La primera vez no tiene que ser la única. Si instalas un agente nuevo, si creas un archivo de reglas que antes no existía o si dejaste algo sin marcar, lo traes después:

- **Desde la barra lateral:** botón derecho sobre una subcarpeta de Agentes, como **Agentes › Casa (este equipo) › Claude**, y **Traer otro archivo**. El botón derecho de Agentes misma no lo ofrece: ahí está [Buscar de nuevo](#buscar-de-nuevo), que hace lo mismo y además marca lo nuevo por ti. En el estado 1 todavía no hay nada traído, y un clic en Agentes abre la primera vez.
- **Desde el menú de la Biblioteca:** con Agentes o una de sus subcarpetas abierta, el botón **+** junto al título **Biblioteca** ofrece **Traer otro archivo**, entre las opciones para crear e importar.
- **Desde la paleta de comandos:** con Agentes o una de sus subcarpetas abierta, escribe «traer» y elige **Traer otro archivo**, en el grupo **Crear**.
- **Desde Configuración › Agentes › Carpeta Agentes:** en la tarjeta **Agentes**, el botón **Traer otro archivo**.

**Traer otro archivo no cambia si Agentes sincroniza.** Traída sin sincronizar, el archivo nuevo entra como documento y el archivo de tu agente queda como está; sincronizando, desde ese momento Tisty también lo mantiene al día. Lo que traes, aquí o en la primera vez, va a la carpeta de este equipo, y cada agente que Tisty encuentra aquí tiene su carpeta vacía en ella.

Se abre **Traer otro archivo**, con «Estos son los archivos de agentes de este equipo que todavía no están en Agentes. Elige qué traer:» y la misma lista de la primera vez, con sus mismas reglas: una skill es una sola fila que entra entera, y lo oculto o lo que mantiene el propio agente no aparece. Con una diferencia: **no viene nada marcado**, ni siquiera las reglas. Marca lo que quieres y pulsa **Traer**: el botón no se activa hasta que marcas algo. **Ahora no** lo cierra sin traer nada. Lo que ya tiene documento dice **ya está en Agentes**, y si no queda nada por traer, el aviso dice «Todos los archivos de agentes de este equipo ya están en Agentes.».

**Un ejemplo.** Un día escribes a mano en tu editor una regla nueva para Claude Code, `~/.claude/rules/pruebas.md`:

```markdown title="~/.claude/rules/pruebas.md"
# Pruebas

- Corre los tests antes de cada commit.
```

1. Botón derecho sobre **Agentes › Casa (este equipo) › Claude** y **Traer otro archivo**.
2. Abre la fila de Claude: `CLAUDE.md` aparece atenuado, con **ya está en Agentes**, y `rules/pruebas.md`, sin marcar, como **Regla**.
3. Márcalo y pulsa **Traer**. Si Agentes sincroniza, el aviso dice «Entró 1 documento a Agentes, y desde ahora Tisty lo mantiene al día.»; si está traída sin sincronizar, «Entró 1 documento a Agentes. Los archivos de tus agentes quedan como están hasta que elijas Sincronizar.». **Listo** lo cierra.
4. En **Agentes › Casa (este equipo) › Claude** aparece **Pruebas**, con el icono de un libro. Desde ahora lo lees y lo cambias en Tisty, como cualquier regla.

Si este equipo todavía no te dejó leer los archivos de tus agentes, por ejemplo porque Agentes llegó traída desde [otro equipo](#en-varios-equipos), **Traer otro archivo** abre el aviso de la primera vez, con sus tres líneas, lo que encontró ya marcado y sus dos botones, **Traer** y **Traer y sincronizar**.

**Traer no toca nada.** Ni los archivos de tus agentes ni la cuenta que Tisty lleva de lo que escribió: un archivo que solo trajiste nunca queda anotado como escrito por Tisty, así que, si después sacas su documento de Agentes, Tisty no te ofrece quitarlo con [¿Quitar también el archivo?](#sacar-una-regla-y-su-archivo). Si más adelante eliges **Sincronizar**, un archivo que sigue tal cual lo trajiste se toma como está.

### Buscar de nuevo

**Buscar de nuevo** está en el botón derecho de Agentes misma, en el estado 2, traída sin sincronizar, y en el 3, sincronizando. Vuelve a buscar en este equipo tus agentes y sus archivos, como la primera vez: si instalaste uno nuevo, le crea su subcarpeta, y abre **Traer otro archivo** con **solo lo nuevo ya marcado**. Lo que ya está en Agentes aparece atenuado con **ya está en Agentes**, y la memoria del agente viene sin marcar, como siempre. Revisa las casillas y pulsa **Traer**.

**Un ejemplo.** Marisol tenía Agentes sincronizando con Claude y Codex. Una tarde instala Gemini CLI, le pide que recuerde el nombre de su perra, y escribe una skill nueva para Claude en `~/.claude/skills/informe/`. Botón derecho sobre **Agentes** y **Buscar de nuevo**:

1. En la barra aparece **Agentes › Casa (este equipo) › Gemini**, con su brújula verde, vacía.
2. Se abre **Traer otro archivo**. En la fila de Claude, **informe · Skill · 2 archivos** viene marcada; en la de Gemini, `GEMINI.md` aparece como **Memoria del agente**, sin marcar.
3. Marisol deja la memoria fuera y pulsa **Traer**: «Entraron 2 documentos a Agentes, y desde ahora Tisty los mantiene al día.», uno por cada archivo de la skill.

La memoria que Marisol dejó fuera sigue ahí: la próxima vez que busque de nuevo, `GEMINI.md` vuelve a aparecer en la lista, sin marcar.

**Si no queda nada por traer**, ni siquiera una memoria sin marcar, el aviso no muestra casillas: dice «No encontré nada nuevo. Esto es lo que hay en cada agente:», y debajo una línea por agente, con su icono. Por ejemplo, en el equipo de Lucas, que tiene Claude, Codex y Cursor:

- «Claude · ya está en Agentes»
- «Codex · ya está en Agentes»
- «Cursor: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta.»

**Ahora no** lo cierra.

![El aviso Traer otro archivo abierto con Buscar de nuevo, sin nada que traer: arriba la línea No encontré nada nuevo. Esto es lo que hay en cada agente:, y debajo tres líneas, cada una con el icono de su agente: el globo de conversación naranja con Claude · ya está en Agentes, la terminal verde azulado con Codex · ya está en Agentes y la brújula verde con Gemini · ya está en Agentes; ninguna casilla; abajo a la derecha, Ahora no y Traer](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-17-buscar-de-nuevo.png)

**En un equipo que todavía no te dio permiso**, como un segundo equipo donde Agentes llegó traída, **Buscar de nuevo** abre en cambio el aviso de la primera vez, con sus nombres y tamaños y sus dos botones. No abre ningún archivo ni crea ninguna subcarpeta antes de tu **Traer** o tu **Traer y sincronizar**.

### Un archivo que nunca trajiste

Esto pasa sincronizando: traída sin sincronizar, Tisty no escribe ningún archivo y no tiene nada que preguntar.

**Un ejemplo.** Tienes `~/.codex/AGENTS.md` con tus reglas de Codex, y la primera vez lo dejaste sin marcar. Después escribes en **Agentes › Casa (este equipo) › Codex** un documento que va a ese mismo archivo. Tisty no sabe qué parte del archivo es tuya, así que no lo pisa: te pregunta **Existe en los dos lados**, al entrar en Agentes o con **Revisar** en Configuración › Agentes › Carpeta Agentes. Como nunca trajo ese archivo, el aviso suma «Tisty nunca trajo este archivo. Si lo traes a Agentes, queda como está, como un documento de este equipo.» y el botón **Traer a Agentes**.

Con **Traer a Agentes**, el archivo entra como un documento en **Agentes › Casa (este equipo) › Codex**, con sus palabras tal como están. Un documento es un archivo, palabra por palabra: Tisty no le suma nada. Lo que tenía antes queda en sus [respaldos](#respaldos).

**Traer otro archivo** hace lo mismo: ese `AGENTS.md` aparece en su lista, sin marcar, y al traerlo queda igual que el archivo.

### El `AGENTS.override.md` de Codex

Codex lee `~/.codex/AGENTS.override.md` en vez de `AGENTS.md` cuando existe, y no lee los dos a la vez. Para Tisty son dos archivos y dos documentos, cada uno con lo que dice su archivo: no se pasa nada de uno al otro. Si creas el override después de la primera vez, tráelo con **Traer otro archivo**: queda en **Agentes › Casa (este equipo) › Codex**, junto al `AGENTS.md`. Mientras exista el override, Codex solo lee ese.

## Cómo queda la carpeta

Después de traer, la Biblioteca tiene una carpeta **Agentes** con **Global**, una carpeta por cada equipo tuyo y, a veces, una carpeta de agente suelta en la raíz. En la barra lateral, cada documento se ve con su nombre y un icono que dice qué es:

```text
Agentes
├── Global                      lo de todos: reglas por agente, skills y hooks
├── Codex                       solo si moviste o creaste algo aquí: es para todos tus equipos
├── Casa (este equipo)
│   ├── Claude
│   │   ├── [libro]    Cómo trabajo
│   │   ├── [libro]    Estilo de código
│   │   ├── [pieza]    pr                3 archivos
│   │   └── [cerebro]  MEMORY.md
│   └── Codex
│       └── [libro]    Reglas de Codex
└── Trabajo                     otro equipo tuyo: en gris tenue
    └── Claude
```

- **Una carpeta por equipo.** Cada equipo lee y sincroniza solo la suya. Lo que traes (**Traer**, **Traer otro archivo**, **Buscar de nuevo**) va a la carpeta de este equipo, y cada agente que Tisty encuentra aquí tiene su subcarpeta en ella, aunque esté vacía. Se llama como el equipo se llama en la sincronización, con «(este equipo)» detrás cuando es el tuyo de ahora; los demás van por su nombre, al final y en gris tenue. Si dos equipos se llaman igual, un código corto los distingue.
- **La raíz es para todos los equipos.** Una carpeta de agente suelta en **Agentes**, como **Codex** en el dibujo, solo existe si tú moviste algo ahí o creaste algo ahí; no la crea Tisty. Si no tiene nada adentro, no se ve. Qué significa tener un documento en la raíz lo cuenta [En varios equipos](#en-varios-equipos).
- **Global guarda lo de todos: reglas por agente, skills y hooks.** Las skills, las de Tisty y las tuyas, llegan a todos los agentes. Lo cuenta [Global y la carpeta de tu equipo](#global-y-la-carpeta-de-tu-equipo).
- **El icono de cada agente**: **Global** lleva un globo azul, y cada agente un símbolo propio, que no imita su logo, en el mismo color de su etiqueta en la ficha: un globo de conversación naranja para Claude, una terminal verde azulado para Codex, unas llaves de código índigo para Copilot, una brújula verde para Gemini y un puntero café para Cursor. Los mismos iconos aparecen en la lista de la primera vez y en **Configuración › Agentes › Carpeta Agentes**. Una subcarpeta a la que le pusiste un icono tuyo lo conserva.
- **El icono de cada documento**: un libro para una **regla**, una pieza de rompecabezas para una **skill** y un cerebro para la **memoria del agente**.
- **El nombre**: el primer título del documento, como «Cómo trabajo» en un `CLAUDE.md` que empieza con `# Cómo trabajo`. Una skill toma el `name` de su `SKILL.md`. Un script, o un archivo sin títulos, se llama como el archivo: `MEMORY.md`, `extraer.py`.
- **La ruta**: pasa el puntero sobre una fila y aparece, con el agente delante: `claude · CLAUDE.md`, `claude · rules/estilo.md`.
- **Una skill es una sola fila**, con cuántos archivos tiene a la derecha, como **pr** y **3 archivos**. Un clic la abre y muestra sus archivos por su ruta dentro de la skill: `SKILL.md`, `scripts/abrir-pr.sh`. Así, una skill de 200 archivos ocupa una línea hasta que la abres. Una skill de un solo archivo se ve como un documento más.

Las subcarpetas son para que encuentres las cosas, pero **dónde está un documento sí cuenta**: la carpeta dice de qué equipo es (o que es de todos), y la línea `agent:` dice qué archivo escribe. Las dos tienen que estar de acuerdo, como cuenta [La línea `agent:`](#la-línea-agent). Dentro de la carpeta de un agente de tu equipo puedes mover un documento entre tus propias carpetas o crear las tuyas, y se sigue escribiendo en el mismo archivo.

**Una subcarpeta por cada agente de este equipo, aunque esté vacía.** Tisty la crea al traer, al [buscar de nuevo](#buscar-de-nuevo) o cuando aparece un agente nuevo, para que tengas dónde escribirle aunque todavía no tenga ningún archivo. Crear una carpeta no toca ningún archivo de tus agentes. Si borras una de esas subcarpetas, por ejemplo **Cursor** porque no lo usas, Tisty no la vuelve a crear. Y si dos equipos la crean a la vez antes de sincronizarse, queda una sola.

**Un equipo sin agentes** no escribe nada con Agentes: los documentos de agentes que tienen tus otros equipos los ves ahí solo como documentos.

**La barra no dice nada junto a Agentes**: ninguna palabra, ni cuántos archivos esperan. Lo que espera tu decisión está en tres lugares:

| Dónde | Qué ves |
| :-- | :-- |
| **La ficha de cada documento** | Su estado, siempre, en la etiqueta de color: **Cambiado por fuera**, **Cambiado en los dos lados**, **Escrito por un Tisty anterior** o **En espera**; la etiqueta **Propuesta**, si un agente te propone un cambio; o un aviso dentro de la ficha, cuando el documento [viene de otro equipo](#en-varios-equipos), cuando [dos documentos van al mismo archivo](#si-dos-documentos-van-al-mismo-archivo) o cuando su texto [espera tu aprobación](#por-aprobar). La tabla de [cada estado](#cada-estado) dice qué hacer con cada uno |
| **Configuración › Agentes › Carpeta Agentes**, en la tarjeta **Agentes** | Los archivos que piden tu decisión: «1 archivo dice algo distinto en Tisty y en el agente.», o «1 archivo ya no es de ninguna regla ni skill de Agentes.», cada uno con **Revisar**; y bajo **Sincronizar Gemini**, si falta [su línea](#gemini-en-pausa) |
| **Al entrar en Agentes** | Si Agentes sincroniza y un archivo existe en los dos lados, ya no es de ninguna regla ni skill o perdió la línea de Gemini, Tisty te pregunta por él apenas entras, uno por uno |

![La barra lateral con la carpeta Agentes abierta y sincronizando, con el robot en coral: dentro, Global con su globo azul y Claude con su globo de conversación naranja; en Claude, la regla Reglas, elegida, con el icono de un libro, y debajo las skills, cada una con el icono de una pieza y cuántos archivos lleva](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-02-barra.png)

### El botón derecho de una subcarpeta

**Global**, la carpeta de tu equipo, **Claude**, **Codex** y cualquier carpeta dentro de Agentes tienen un menú solo de agentes. Sobre **Agentes › Casa (este equipo) › Claude**, mientras Agentes sincroniza, ofrece esto, y nada más:

| Opción | Qué hace |
| :-- | :-- |
| **Documento nuevo** | Crea un documento en esa carpeta que ya nace con su línea `agent:`: en **Claude**, una regla de Claude. En **Global** crea un documento común, sin línea `agent:`, porque Global no es la carpeta de un solo agente. Mira [Un documento nuevo ya es una regla](#un-documento-nuevo-ya-es-una-regla) |
| **Traer otro archivo** | Abre la lista de los archivos de tus agentes que todavía no están en Agentes. Mira [Traer otro archivo](#traer-otro-archivo) |
| **Pausar la sincronización con Claude** | Deja de escribir los archivos de Claude en este equipo, como el botón del mismo nombre en la ficha. En pausa, dice **Reanudar la sincronización con Claude**. Mira [Pausar, dejar de sincronizar y vaciar](#pausar-dejar-de-sincronizar-y-vaciar) |
| **Ocultar de la barra** | La saca de la barra en este equipo. Si Agentes sincroniza, antes te pide dejar de sincronizar. Mira [Ocultarla de la barra](#ocultarla-de-la-barra) |
| **Eliminar** | Pregunta antes; sus documentos pasan a **Sin carpeta**, y Tisty te pregunta por los archivos que escribían, con su respaldo. Mira [Eliminar una subcarpeta](#eliminar-una-subcarpeta) |

- **La carpeta de otro equipo solo ofrece Ocultar de la barra.** Lo que hay en ella es de ese equipo: aquí se lee, se mueve y se copia como texto, pero no se escribe.
- **Global y la carpeta de tu equipo no tienen pausa**: no son de un agente. Para pausar uno, usa la subcarpeta de ese agente.
- **La pausa solo aparece mientras Agentes sincroniza.** Traída sin sincronizar, Tisty no escribe ningún archivo, y no hay nada que pausar. Tampoco aparece para un agente que no está instalado en este equipo.
- **Una carpeta dentro de una subcarpeta**, como **Agentes › Casa (este equipo) › Claude › Pruebas**, tiene el mismo menú que **Claude**, con la pausa de Claude.
- **No hay Diagrama nuevo, Carpeta nueva, Cambiar nombre, Mover a, Importar ni Desempaquetar.** Esas son acciones de tus carpetas; dentro de Agentes, el nombre de cada carpeta es el que Tisty usa para saber a quién le escribe.
- **Una subcarpeta archivada** muestra además **Desarchivar**, para que vuelva.

**Un ejemplo.** Jaime prueba a mano unas reglas nuevas de Claude esta tarde, pero quiere que Codex siga recibiendo lo suyo. Botón derecho sobre **Agentes › Casa (este equipo) › Claude** y **Pausar la sincronización con Claude**. Codex sigue sincronizando; Claude queda como lo dejó la última escritura. Al terminar, botón derecho otra vez y **Reanudar la sincronización con Claude**.

### La línea `agent:`

Cada documento de Agentes empieza con sus propiedades, y una de ellas dice qué archivo es:

```markdown
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

El primer valor tiene tres partes, separadas por `/`:

- **El nivel**: `global`, o un agente: `claude`, `codex`, `copilot`, `gemini` o `cursor`.
- **El tipo**: `rule`, `skill` o `memory`.
- **El nombre**: la ruta del archivo dentro de la carpeta del agente, como `CLAUDE.md`, `rules/estilo.md` o `pr/SKILL.md`.

**Un documento de un equipo lleva una marca más**: `on-` y el código de ese equipo, como `on-dev_ab12cd34`. La pone Tisty, no tú. Un documento de Global o de la raíz no la lleva, porque es de todos.

**La línea y la carpeta tienen que estar de acuerdo.** Si no lo están, por ejemplo porque restauraste una versión anterior del documento o lo moviste con un Tisty más viejo, ningún equipo lo escribe: la ficha dice «Está en la carpeta de un equipo y su texto dice otro, así que ningún equipo lo escribe hasta que lo resuelvas.» y ofrece **Según su carpeta**, que deja la línea como dice la carpeta.

A veces lleva marcas más, que Tisty pone solo al traer un archivo para escribirlo igual que estaba: `crlf` (líneas que terminan como en Windows), `bom` (una marca invisible al principio), `as-is` (finales de línea mezclados) y `open-end` (el archivo no termina en salto de línea). Por ejemplo, `agent: [copilot/rule/copilot-instructions.md, crlf, bom, on-dev_ab12cd34]`. La [guía de propiedades](https://markdown.tisty.app/es/propiedades) explica el resto.

La página no muestra esa línea: el documento se abre en su [ficha](#la-ficha-de-un-documento-de-agente).

## Global y la carpeta de tu equipo

**Un documento es un archivo, palabra por palabra.** Nada se junta con otra cosa: lo que dice el documento es lo que dice el archivo que lee tu agente. **Global** guarda lo de todos: reglas por agente, skills y hooks. Una regla de un agente puesta en Global se escribe en cada equipo cuando pulsas **Aplicar aquí** en él; en el equipo donde la aprobaste se aplica, y en los demás espera ese botón.

### Un documento es un archivo

Escribe en **Agentes › Casa (este equipo) › Claude** un documento con estas propiedades:

```markdown
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

Tisty escribe `~/.claude/CLAUDE.md` con esas líneas, y nada más. Si mañana cambias el documento, cambia el archivo; si alguien cambia el archivo por fuera, la ficha te lo dice. El cuadro de datos del documento lo confirma: **Lo lee** Claude y **Se escribe en** `~/.claude/CLAUDE.md`, sin ningún «junto con otro documento».

Codex lee un solo archivo de reglas, `AGENTS.md`, y Gemini también, `tisty-rules.md`. Cursor no recibe reglas: las suyas viven en sus ajustes, no en un archivo.

> [!IMPORTANT]
> **Una regla con nivel `global` (`agent: global/rule/…`), de la 1.49 o la 1.50, se queda para que la leas.** No tiene archivo en ningún agente, así que no se escribe: puedes leerla, copiarla o borrarla. Si quieres que siga llegando a un agente, copia su texto a un documento de la carpeta de ese agente, que también puede estar en Global (por ejemplo, una regla de Claude en Global).

### Sumar reglas a un agente

Para sumar reglas a un agente, haz otro archivo que ese agente lea, no más texto en el mismo:

- **Claude Code** lee cada archivo de `~/.claude/rules`: escribe un documento con `agent: claude/rule/rules/pruebas.md`, en la carpeta **Claude** de tu equipo, y Tisty escribe `~/.claude/rules/pruebas.md`.
- **Copilot** hace lo mismo con su carpeta `instructions`: `agent: copilot/rule/instructions/rust.instructions.md`.
- **Codex y Gemini** leen un solo archivo de reglas, así que lo que quieras sumarles va en ese mismo documento.

Un nombre que el agente no lee no se escribe en ninguna parte: `agent: claude/rule/notas.md` deja la ficha diciendo «Ningún agente de este equipo lee este documento todavía.».

### Un documento nuevo ya es una regla

No tienes que escribir la línea `agent:` a mano. Un documento que creas con **Documento nuevo** dentro de la carpeta de un agente de tu equipo empieza como una regla de ese agente: sus propiedades ya traen su línea `agent:`, y tú solo escribes la regla.

| Dónde lo creas | Con qué línea empieza |
| :-- | :-- |
| **Claude** | `agent: claude/rule/CLAUDE.md` mientras ningún documento lo tenga; después, `claude/rule/rules/rule.md`, `rules/rule-2.md`… |
| **Codex** | `agent: codex/rule/AGENTS.md` mientras ningún documento lo tenga |
| **Copilot** | `agent: copilot/rule/copilot-instructions.md` mientras ningún documento lo tenga; después, `copilot/rule/instructions/rule.instructions.md`, `rule-2.instructions.md`… |
| **Gemini** | `agent: gemini/rule/tisty-rules.md` mientras ningún documento lo tenga |
| **Global**, **Cursor**, Agentes misma o una carpeta tuya dentro de Agentes | Ninguna: es un documento común |

En cada caso Tisty le suma la marca del equipo. **Codex y Gemini leen un solo archivo de reglas.** Cuando un documento ya lo tiene, **Documento nuevo** en su carpeta no crea otro: abre ese mismo documento, para que sumes ahí lo nuevo. **Cursor** no tiene un archivo de reglas propio, así que un documento nuevo en su carpeta también empieza común; sus skills las traes con **Traer otro archivo**.

**Un ejemplo.** Marisol tiene Agentes sincronizando y acaba de instalar Gemini CLI. En la barra aparece **Agentes › Casa (este equipo) › Gemini**, con su brújula verde, vacía.

1. Botón derecho sobre **Gemini** y **Documento nuevo**. El documento se abre en su ficha, con las etiquetas **Regla** y **Gemini**, todavía sin texto.
2. Pulsa **Editar**, escribe «Responde en español.» y pulsa **Listo**.
3. Tisty escribe `~/.gemini/tisty-rules.md` con esa línea y agrega la línea `@tisty-rules.md` al final de `~/.gemini/GEMINI.md`, para que Gemini lo lea. La ficha pasa a **Al día**, y en la barra el documento se llama `tisty-rules.md`, porque no tiene un título; si le escribes `# Reglas de Gemini` arriba, se llama así.

Sus propiedades, que la ficha no muestra, quedaron así:

```markdown
---
agent: [gemini/rule/tisty-rules.md, on-dev_ab12cd34]
title: gemini · tisty-rules.md
---
Responde en español.
```

Si después pulsa otra vez **Documento nuevo** en **Gemini**, se abre ese mismo documento, y Marisol suma ahí la regla nueva.

### Una skill en Global

Una skill es una carpeta con un `SKILL.md` y, a veces, scripts. Cada archivo es un documento, con el mismo nivel y la misma carpeta de skill:

```text
agent: global/skill/pdf/SKILL.md
agent: global/skill/pdf/scripts/extraer.py
```

Una skill de Global se escribe una sola vez donde la leen varios a la vez:

- en `~/.claude/skills/pdf/`, para Claude Code;
- en `~/.cursor/skills/pdf/`, para Cursor;
- en `~/.agents/skills/pdf/`, una sola copia que leen Codex, Copilot, Gemini y DeepSeek Harness.

Las propiedades propias del `SKILL.md` no ocupan el texto: su `name` es el nombre de la ficha, su `description` va debajo en una línea discreta, y el resto, como `disable-model-invocation: true`, va en filas chicas del cuadro de datos. El script se guarda en un bloque de código y se escribe tal cual, sin el bloque: la fila **Formato** del cuadro de datos dice el tipo del script, como SH · LF, y en un documento Markdown, Markdown · LF.

**Una skill de un agente** va en la carpeta de ese agente, y se escribe solo en su carpeta de skills.

### Las skills de Tisty en Global

Tisty trae tres skills propias, `tisty`, `consolidate` y `next-agent`, que les enseñan a tus agentes a trabajar con él. No son documentos de Agentes: viven dentro de la app y tienen su propio sí, **Encender en este equipo**, que no necesita esta carpeta. Lo cuenta entero [[Hooks y skills|Hooks-y-skills]].

Aquí las ves como una fila fija al final de **Agentes › Global**, **Skills de Tisty**, con su estado: **Apagadas** o **Encendidas**. La fila aparece solo si este equipo tiene algún agente, si las skills están encendidas o si quedan copias suyas. Un clic abre su ficha, la misma de **Configuración › Agentes › Conectados**. Lo que cambia por tener Agentes:

- **No se traen.** En la lista de la primera vez, en **Traer otro archivo** y en **Buscar de nuevo**, sus copias en las carpetas de tus agentes salen como **propio de Tisty**, y no entran como documentos.
- **Una skill tuya con el mismo nombre gana.** Si en Agentes tienes tu propia `consolidate`, en cualquier nivel, la de Tisty no se escribe en ningún agente, y su ficha dice «Ya tienes en Agentes una skill llamada consolidate, así que la de Tisty no se agrega.».
- **La pausa no las alcanza.** Pausar Claude o dejar de sincronizar Agentes vale para tus documentos; las skills de Tisty siguen su propio interruptor.
- **Se ocultan, no se eliminan.** Al pasar el puntero sobre la fila aparece **Ocultar**, que la saca de Agentes en este equipo sin apagarlas. No tiene menú, ni **Eliminar**: **Mostrar en Agentes**, en Configuración, la devuelve.
- **Los dos sí van aparte.** Dejar de sincronizar Agentes no apaga las skills, y encenderlas no trae nada a Agentes.

**Un ejemplo.** Lucas encendió las skills de Tisty al conectar Claude Code, sin abrir nunca Agentes. Un mes después hace clic en **Agentes** y elige **Traer y sincronizar**: en la fila de Claude, la skill `tisty` aparece atenuada, como **propio de Tisty**, y sus propias skills, como **pr**, con su casilla. Al terminar, **Agentes › Global** muestra **Skills de Tisty · En este equipo** al final.

## Qué archivo lee cada agente

`~` es tu carpeta de usuario: `C:\Users\tú` en Windows y `/Users/tú` en macOS. Tisty solo trabaja con un agente cuya carpeta existe en este equipo.

| Agente | Su carpeta | La mueve | Reglas que escribe Tisty | Skills | Memoria, solo para leer |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Claude Code** | `~/.claude` | `CLAUDE_CONFIG_DIR` | `CLAUDE.md` y lo que va en `rules/` | `skills/` en su carpeta | `projects/*/memory/`, la memoria de cada proyecto |
| **Codex** | `~/.codex` | `CODEX_HOME` | `AGENTS.md`, y `AGENTS.override.md` si tienes un documento para él | `~/.agents/skills/` | — |
| **GitHub Copilot CLI** | `~/.copilot` | `COPILOT_HOME` | `copilot-instructions.md` y lo que va en `instructions/` | `skills/` en su carpeta; las de Global, en `~/.agents/skills/` | — |
| **Gemini CLI** | `~/.gemini` | `GEMINI_CLI_HOME`, la carpeta que contiene `.gemini` | `tisty-rules.md`, y la línea `@tisty-rules.md` en `GEMINI.md` | `skills/` en su carpeta; las de Global, en `~/.agents/skills/` | `GEMINI.md` |
| **Cursor** | `~/.cursor` | — | Ninguna: sus reglas viven en sus ajustes | `skills/` en su carpeta | — |

Lo que hace Tisty con cada uno:

- **Claude Code.** Trae `CLAUDE.md`, cada archivo de `rules/`, cada skill y, si la marcas, la memoria de cada proyecto, solo la carpeta `memory` y nunca las conversaciones guardadas. Escribe `CLAUDE.md` y `rules/`, y las skills.
- **Codex.** Codex lee `AGENTS.override.md` en vez de `AGENTS.md` cuando el primero existe, y cada uno es su propio documento. Si creas el override después de la primera vez, [tráelo](#el-agentsoverridemd-de-codex). Sus skills van a `~/.agents/skills`, que `CODEX_HOME` no mueve.
- **GitHub Copilot CLI.** Trae y escribe `copilot-instructions.md` y cada archivo de `instructions/`, como `rust.instructions.md`.
- **Gemini CLI.** Gemini escribe su propio `GEMINI.md` cuando le pides que recuerde algo, así que ese archivo es suyo: Tisty lo puede traer como memoria y nunca lo sobrescribe. Tus reglas van a un archivo aparte, `~/.gemini/tisty-rules.md`, y Tisty agrega **una sola línea** al final de `GEMINI.md`, `@tisty-rules.md`, para que Gemini lo lea. No toca nada más de ese archivo, y si la línea ya está, no la repite. Si después la borras, Tisty no la vuelve a poner: Gemini queda [en pausa](#gemini-en-pausa) hasta que tú lo decides.
- **Cursor.** Solo skills, propias o de Global.

Un agente cuenta en cada equipo por separado: Tisty solo escribe lo de un agente cuya carpeta existe en ese equipo.

**Las variables que mueven una carpeta** se leen del entorno con que arrancó Tisty. Si usas `CLAUDE_CONFIG_DIR` para guardar Claude Code en otra parte, defínela como variable de tu usuario y vuelve a abrir Tisty, para que la vea.

## La ficha de un documento de agente

Un documento de Agentes no se ve como un documento más. Se abre **para leer**, en una sola ficha: arriba, qué es y dónde va; en el medio, el texto del agente; abajo, lo que puedes hacer. No hay columna al lado: todo lo útil está en la ficha.

![La ficha de la regla Reglas en Agentes / Claude: las etiquetas Regla, Claude y Al día; el cuadro de datos con Lo lee Claude, Se escribe en con su botón de copiar, Tamaño, Respaldos con Ver, Formato Markdown · LF, Última escritura, Original guardado y Sincroniza en este equipo Sí; debajo, el texto de la regla, y al pie Editar, Respaldos, Historial, Copiar enlace y Pausar la sincronización con Claude](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/app/es-agentes-regla-light.webp)

**Arriba de todo, el camino de carpetas**: «Agentes / Casa (este equipo) / Claude». Un clic en una carpeta la abre.

**La cabecera:**

- **Las etiquetas**: el tipo (**Regla**, **Skill** o **Memoria del agente**), el nivel (**Global** o el agente) y el estado, con su color.
- **El nombre**, el mismo que en la barra lateral. En una skill, su `description` va debajo, en una línea discreta.

**El cuadro de datos del archivo**, el único lugar donde aparecen:

| Fila | Qué dice | Ejemplo |
| :-- | :-- | :-- |
| **Lo lee** | Qué agentes de este equipo leen el archivo | Claude, Codex |
| **Se escribe en** | La ruta completa del archivo, con un botón para copiarla. Una skill de Global tiene una por agente: **Archivo de Claude**, **Archivo de Codex**. Es siempre un solo documento por archivo: nada se escribe «junto con otro» | `~/.claude/CLAUDE.md` |
| **Tamaño** | Cuánto pesa el archivo que lee el agente | 12,7 kB |
| **Respaldos** | Cuántos guarda este equipo, y **Ver** para abrirlos | 3 respaldos · Ver, o Ninguno todavía |
| **Formato** | Markdown, o el tipo del script; cómo terminan sus líneas, y las marcas que conserva | Markdown · LF, o PY · CRLF · BOM |
| **Última escritura** | Cuándo cambió el archivo por última vez | hoy, 19:37 |
| **Original guardado** | Si ya está el respaldo n.º 1, el archivo de antes de que Tisty lo tocara | Sí, el de antes de la primera escritura de Tisty |
| **Sincroniza en este equipo** | Si Tisty lo mantiene al día aquí | Sí; Sí, salvo Codex (en pausa); Tisty solo lo lee; No, Agentes no se sincroniza; No: espera «Aplicar aquí»; No: su texto espera tu aprobación; No: es de otro equipo; Espera que elijas, o No: aquí manda otro, cuando [dos documentos van al mismo archivo](#si-dos-documentos-van-al-mismo-archivo); No: su carpeta y su texto dicen equipos distintos. Con una propuesta esperando, se suma «un cambio propuesto por un agente espera tu aprobación» |
| **Compartido** | Solo si el documento tiene un enlace compartido activo: si se puede leer o editar, y **Ver** muestra los enlaces con su botón para copiarlos y las acciones de siempre, como **Dejar de compartir**. Lo cuenta [Compartir](Documentos#compartir) | Solo leer · Ver |
| **Las propiedades del archivo** | Las otras claves de su encabezado, en filas chicas de clave y valor. Si son demasiadas o muy largas para leerlas como campos, van tal como están escritas | `disable-model-invocation` true; `applyTo` `**/*.rs` |

**El tope de Codex.** Codex lee su `AGENTS.md` hasta 32 KiB y deja sin leer lo que sigue. Cuando el archivo se acerca a ese tope, la fila **Tamaño** lo dice en ámbar: «29 kB · cerca de los 32 KiB que lee Codex». Si lo pasa, lo dice en rojo: «33,8 kB · pasa los 32 KiB que lee Codex: lo demás no lo lee». Es buen momento para pasar las reglas largas a una skill.

**El texto del agente**, dentro de la misma ficha y sin hoja aparte. Es lo que escribiste, tal cual, sin secciones ni plantilla impuestas: Tisty lo dibuja con encabezados compactos, a todo el ancho de la ficha, y sin convertir su primera línea en un título. El encabezado del archivo no se repite ahí, porque ya está en el cuadro de datos. Lo que lee tu agente no cambia: el archivo sigue siendo el mismo, byte a byte.

**Los botones, al pie**, fijos abajo mientras la ficha entera se desplaza junto con el texto:

- **Editar** abre el editor de Tisty sobre el texto, con su encabezado y la barra del editor de siempre para títulos y listas, y **Listo** lo vuelve a cerrar.
- **Respaldos**, cuando el documento se escribe en un solo archivo y ese archivo tiene alguno. Abre la misma lista que **Ver** en el cuadro de datos. Una skill de **Global**, que se escribe en varios archivos, no lo tiene al pie: los respaldos de cada archivo se abren con **Ver**, en su fila.
- **Historial**: las versiones anteriores del documento, no del archivo. Lo cuenta [[Versiones anteriores|Documentos#versiones-anteriores]]. La comparación con lo **Aprobado** lee de ahí: si vacías el historial, o la versión aprobada pasa el plazo, la ficha te pide leer el texto entero antes de aprobar.
- **Copiar enlace** copia su [enlace](Documentos#el-enlace-de-un-documento).
- **Pausar la sincronización con Claude**, cuando el documento se escribe en un solo archivo. Una skill de Global tiene un botón por agente: **Pausar Claude**, **Pausar Codex**…
- **Los de otro equipo no tienen Editar, ni Aplicar, ni Eliminar, ni pausa**: la ficha dice «Es de otro de tus equipos: en este solo se lee.». Lo que sí puedes es leerlo, moverlo, copiarlo como texto y sacarlo. Un cambio que otro equipo aprobó para este espera aquí cada vez, como cuenta [En varios equipos](#documentos-de-otros-equipos).

Un documento de memoria no tiene **Editar**: lo escribe el agente, y la ficha dice «Este archivo lo escribe el agente: Tisty lo muestra y lo trae, y nunca lo sobrescribe.».

**Un ejemplo.** Abres la regla **Horario laboral** de Codex, que va en `~/.codex/AGENTS.md`. Arriba ves «Agentes / Casa (este equipo) / Codex» y las etiquetas **Regla**, **Codex** y **Al día**. En el cuadro, «Se escribe en `~/.codex/AGENTS.md`», y «Tamaño 29 kB · cerca de los 32 KiB que lee Codex» en ámbar. Lees el texto, decides pasar la parte larga a una skill y pulsas **Editar** al pie.

### Cada estado

| Estado | Qué significa | Qué hacer |
| :-- | :-- | :-- |
| **Al día** | El archivo dice lo mismo que el documento | Nada |
| **Por escribir** | Cambiaste el documento y Tisty lo escribe en un instante | Nada |
| **Cambiado por fuera** | Alguien cambió el archivo fuera de Tisty, o lo borró | Elegir: [traer o sobrescribir](#un-cambio-por-fuera-traer-o-sobrescribir) |
| **Cambiado en los dos lados** | Cambiaron el archivo y el documento desde la última escritura | Elegir cuál conservar |
| **Escrito por un Tisty anterior** | Una versión anterior de Tisty escribió el archivo sin anotar de qué documentos venía, y hoy no dice lo mismo que ellos | **Ver el archivo**, y después **Sobrescribir con Tisty** si te quedas con el documento |
| **En espera** | El texto de este documento [cambió sin tu aprobación](#por-aprobar), o el documento salió de Agentes por mano de un agente, así que el archivo queda como estaba. La ficha lo nombra | Aprobar el documento, o volver a ponerlo y sacarlo tú |
| **En pausa** | Pausaste la sincronización de ese agente, o Agentes no sincroniza en este equipo; entonces la ficha suma «La carpeta Agentes no se sincroniza con tus agentes. Elige Sincronizar en la barra o en Configuración › Agentes › Carpeta Agentes.». En Gemini, también cuando falta en `GEMINI.md` la línea `@tisty-rules.md` que Tisty había puesto | **Reanudar** cuando quieras, o **Sincronizar** la carpeta; en Gemini, [**Volver a ponerla**](#gemini-en-pausa) |
| **Sin permiso en este equipo** | Este equipo todavía no te dejó leer los archivos de tus agentes, por ejemplo porque Agentes llegó traída desde otro equipo | **Sincronizar**, en la barra o en Configuración › Agentes › Carpeta Agentes, y elegir en el aviso de la primera vez |
| **Nada que escribir** | El documento quedó vacío, y Tisty nunca vacía un archivo de un agente | Escribir algo, o sacar el documento de Agentes |
| **El archivo es un enlace** | El archivo, o una carpeta en su camino, es un enlace simbólico; escribirlo reemplazaría el enlace | Cambiar el enlace por el archivo real |
| **El bloque de código está roto** | El bloque que guarda un script ya no está entero | Arreglar el bloque en el documento |
| **No se puede leer** | El archivo existe, pero Tisty no pudo leerlo | Revisar sus permisos |

Dos avisos más pueden aparecer bajo el cuadro de datos:

- **«Ningún agente de este equipo lee este documento todavía.»** El agente no está en este equipo, o el nombre no es un archivo que ese agente lea.
- **«En este equipo otro documento escribe este archivo.»** Dos documentos, el de tu carpeta y uno de la raíz, van al mismo archivo: solo uno se escribe, y la ficha del otro lo dice.

Y si el documento todavía no se aplica en este equipo, un aviso dice por qué, con el botón que lo resuelve: **Aplicar aquí** y **No en este equipo** si espera tu sí; un botón **Usar** por cada documento, con su título y de quién es, si [dos documentos van al mismo archivo](#si-dos-documentos-van-al-mismo-archivo); **Según su carpeta** si su carpeta y su línea `agent:` no se ponen de acuerdo. Todo eso lo cuenta [En varios equipos](#en-varios-equipos).

## Un cambio por fuera: traer o sobrescribir

Puedes seguir editando los archivos de tus agentes con tu editor, o dejar que el agente los cambie. Tisty nunca pisa esos cambios. Esto pasa sincronizando: traída sin sincronizar, Tisty no escribe esos archivos ni te pide decidir nada, y lo que cambies ahí queda solo ahí.

**Un ejemplo.** Abres `~/.claude/CLAUDE.md` en tu editor y agregas «- Corre los tests antes de cada commit.». Si Agentes sincroniza, la próxima vez que Tisty mira (al iniciar, al volver a la ventana o al abrir el documento), la ficha de **Cómo trabajo**, el documento de ese `CLAUDE.md`, dice **Cambiado por fuera**, con un aviso:

- **Ver el archivo** muestra lo que el archivo tiene ahora.
- **Traer el cambio** escribe en el documento lo que dice el archivo. Lo que decía antes el documento queda en sus [versiones anteriores](Documentos#versiones-anteriores). Si alguien cambió el documento después de que se dibujó la ficha, Tisty no lo pisa: te pide leerlo de nuevo y volver a elegir.
- **Sobrescribir con Tisty** pregunta «¿Escribir este documento sobre `~/.claude/CLAUDE.md`? Lo que tiene ahora el archivo queda en un respaldo.» y, si dices que sí, vuelve a escribir el archivo desde el documento.

Mientras no eliges, Tisty no escribe ese archivo, aunque cambies el documento.

**Traer el cambio** aparece cuando el documento es de un solo agente, no de Global, y no lo estás editando: como un documento es el archivo entero, lo que cambió en el archivo es lo que pasa al documento. Si estás editando, pulsa **Listo** primero.

**Si borraron el archivo**, la ficha dice **Cambiado por fuera** sin **Ver el archivo**: **Sobrescribir con Tisty** lo vuelve a crear.

### Cambiado en los dos lados

Si cambiaron el archivo y también el documento, la ficha dice **Cambiado en los dos lados**. Aparece **Quedarse con el archivo**, que pregunta «¿Dejar este documento como está ahora …? Lo que dice ahora queda en su historial.», junto a **Sobrescribir con Tisty**.

Lo mismo puede pasar la primera vez que sincronizas, cuando ya tenías en Agentes un documento de un archivo que entretanto cambió. Entonces aparece **Existe en los dos lados**, con las líneas que difieren: en rojo, con `−`, lo que hay **En Tisty**; en verde, con `+`, lo que hay **En el archivo**. Eliges **Conservar el de Tisty**, **Conservar el archivo** o **Decidir después**, y lo que no conservas queda guardado: el archivo en sus [respaldos](#respaldos), el documento en sus versiones anteriores. Si Tisty nunca trajo ese archivo, el aviso suma **Traer a Agentes**, como cuenta [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste). Desde **Configuración › Agentes › Carpeta Agentes**, «1 archivo dice algo distinto en Tisty y en el agente.» y **Revisar** te traen de vuelta a ese aviso.

## Respaldos

**Antes de sobrescribir un archivo de un agente, Tisty guarda lo que tenía.** Los respaldos viven en este equipo, en `data/agents/` dentro de la carpeta de Tisty, nunca en las carpetas de tus agentes, y no viajan con la sincronización ni con los paquetes.

- **El respaldo n.º 1 es el original**: el archivo como estaba antes de que Tisty lo sobrescribiera por primera vez. Ese no se borra nunca.
- **Además, los últimos 20.** Cuando llega el 21, se va el más antiguo después del original.
- **Quitar un archivo también deja uno.** Si [quitas el archivo](#sacar-una-regla-y-su-archivo) de una regla que sacaste, sus respaldos siguen en este equipo, y si una regla vuelve a escribir esa misma ruta, aparecen en su **Respaldos**.

Así, un archivo que Tisty sobrescribió muchas veces guarda hasta 21: el original y los últimos 20. El cuadro dice «Se guardan los últimos 20.», y el n.º 1 se queda además de ellos.

Para verlos, pulsa **Respaldos** al pie de la ficha, o **Ver** en la fila **Respaldos** del cuadro de datos, junto a «3 respaldos»:

- **Ver** muestra el respaldo completo.
- **Recuperar** pregunta «¿Devolver este respaldo a …? Lo que tiene ahora el archivo queda en un respaldo.» y lo escribe en el archivo.

**Un respaldo recuperado no cambia tu documento.** El archivo vuelve a decir lo que decía el respaldo, y la ficha pasa a **Cambiado por fuera**: eliges **Traer el cambio**, para que el documento diga lo mismo, o **Sobrescribir con Tisty**, si te arrepentiste. Así nunca pierdes ninguno de los dos lados.

Las versiones anteriores del **documento**, en cambio, son las de cualquier documento: están en su [historial](Documentos#versiones-anteriores).

## Pausar, dejar de sincronizar y vaciar

**Pausar un agente.** Mientras Agentes sincroniza, el lugar de siempre es **Configuración › Agentes › Carpeta Agentes**: en la tarjeta **Agentes**, apaga el interruptor **Sincronizar Claude**, que aparece solo mientras la carpeta sincroniza. También en la barra: botón derecho sobre **Agentes › Casa (este equipo) › Claude** y **Pausar la sincronización con Claude**; en pausa, el mismo menú dice **Reanudar la sincronización con Claude**. También desde la ficha de un documento, con los botones del pie: **Pausar la sincronización con Claude**, en un documento que se escribe en un solo archivo, o un botón por agente, **Pausar Claude**, **Pausar Codex**…, en una skill de Global. En pausa, Tisty no abre ni escribe los archivos de ese agente: quedan como los dejó la última escritura, y tú sigues editando tus documentos. **Reanudar la sincronización con Claude** escribe lo que cambiaste, salvo un archivo que cambió por fuera mientras tanto, que queda marcado para que decidas.

**Dejar de sincronizar toda la carpeta.** Botón derecho sobre **Agentes** y **Dejar de sincronizar**, o el botón **Dejar de sincronizar** en su tarjeta de **Configuración › Agentes › Carpeta Agentes**. Pasa esto, y nada más:

- **Ningún archivo de tus agentes se toca.** Tus agentes siguen leyendo sus archivos tal como Tisty los dejó, y Tisty deja de escribirlos y de pedirte que decidas cuando cambian por fuera.
- **Ningún documento se borra.** Siguen en Agentes, y los lees y los editas como siempre; lo que cambias queda en Tisty.
- **Agentes pasa al estado 2.** Su robot vuelve al gris y, al abrirla, arriba aparece «No se sincroniza con tus agentes · Sincronizar». La ficha de cada documento dice «La carpeta Agentes no se sincroniza con tus agentes. Elige Sincronizar en la barra o en Configuración › Agentes › Carpeta Agentes.», y su cuadro de datos, en **Sincroniza en este equipo**, «No, Agentes no se sincroniza».

**Sincronizar otra vez.** Con Agentes en el estado 2, cualquiera de estos tres hace lo mismo: el enlace **Sincronizar** de la línea de arriba, **Sincronizar** en su botón derecho, o **Sincronizar** en su tarjeta de Configuración › Agentes › Carpeta Agentes. Si este equipo ya te había dejado leer los archivos de tus agentes, por ejemplo porque elegiste **Traer** o **Dejar de sincronizar** aquí mismo, sincroniza de una vez, sin el aviso de la primera vez. Si nunca lo hizo, como un [segundo equipo](#un-segundo-equipo-paso-a-paso), abre antes el aviso de la primera vez.

Al sincronizar, Tisty escribe lo que cambiaste en tus documentos mientras tanto, con su respaldo antes, salvo un archivo que cambió por fuera, que queda marcado para que decidas. Lo que encuentra distinto en los dos lados, o que ya no es de ninguna regla ni skill, te lo pregunta en ese mismo momento.

**La primera vez que sincronizas algo que solo habías traído**, un archivo que sigue tal cual lo trajiste se toma como está: no se pregunta nada por él. Un archivo que cambió desde que lo trajiste, en cambio, no se pisa: aparece **Existe en los dos lados** y eliges.

**Un ejemplo.** Lucas quiere probar a mano unas reglas nuevas en su `CLAUDE.md` durante una semana, sin que Tisty las toque. Elige **Dejar de sincronizar**. Edita el archivo con su editor y, en paralelo, sigue ordenando sus documentos en Agentes. Al final de la semana pulsa **Sincronizar**: la ficha de **Cómo trabajo** dice **Cambiado por fuera**, y Lucas elige **Traer el cambio** para quedarse con lo que probó, o **Sobrescribir con Tisty** para volver a lo de sus documentos.

La pausa, la sincronización y el permiso son de este equipo: dejar de sincronizar aquí no cambia nada en tus otros equipos.

### Gemini en pausa

Sincronizando, Tisty pone la línea `@tisty-rules.md` en tu `GEMINI.md` una vez. Si después la borras, o borras `GEMINI.md` entero, Tisty lo toma como tu decisión y no la vuelve a poner: Gemini queda **En pausa**, deja de recibir tus reglas, y Tisty deja de escribir `tisty-rules.md`. Solo cuenta una línea que puso Tisty: si nunca la puso ahí, por ejemplo porque todavía no tenías reglas para Gemini, que falte no deja a Gemini en pausa. Lo ves en tres lugares:

- **En la ficha** de cada regla que Gemini lee: la etiqueta **En pausa** y «Gemini no lee esta regla: falta la línea @tisty-rules.md en GEMINI.md, y Tisty no la vuelve a poner sola.», con el botón **Volver a ponerla**.
- **En Configuración › Agentes › Carpeta Agentes**, bajo **Sincronizar Gemini**: «En pausa: falta la línea @tisty-rules.md en GEMINI.md.», con **Volver a ponerla**.
- **Al entrar en Agentes**, el aviso **Falta la línea de Tisty**: «La línea @tisty-rules.md ya no está en GEMINI.md, así que Gemini no lee tus reglas. Tisty no la vuelve a poner sola: Gemini queda en pausa hasta que tú lo hagas.», con **Dejarlo en pausa** y **Volver a ponerla**.

**Volver a ponerla** agrega otra vez la línea al final de `GEMINI.md`, sin tocar lo demás, y Tisty vuelve a escribir tus reglas en `tisty-rules.md`. Mientras no lo haces, la ficha y la tarjeta lo siguen diciendo.

### Eliminar una regla o una skill

Cuando eliminas de Agentes una regla o una skill tuya, con **Eliminar** en su menú, y Tisty la escribió en la carpeta de un agente, Tisty te pregunta ahí mismo si también la quita de tus agentes. Es un solo aviso: arriba dice lo mismo que la pregunta de siempre (lo que se va con el documento, sus idiomas y sus páginas, y que no se puede deshacer), y debajo nombra a cada agente cuya copia se puede quitar:

- **Quitar también** elimina el documento y saca cada uno de esos archivos de las carpetas de tus agentes. Antes queda un respaldo de cada archivo en este equipo.
- **Solo de Tisty** elimina el documento y deja esos archivos donde están, y tus agentes los siguen leyendo.
- **Cancelar** no elimina nada.

**Un ejemplo.** Yisset escribió en Agentes la skill de Global **voz**, y Tisty la puso en `~/.claude/skills/voz/SKILL.md` y en `~/.agents/skills/voz/SKILL.md`, que leen Codex y Copilot. Ya no la quiere y elige **Eliminar**. El aviso dice «¿Borrar «voz»? El documento y su contenido se van para siempre. Esto no se puede deshacer.» y, en negrita, «¿Quitar también de Claude, Codex y Copilot?». Elige **Quitar también**: los dos archivos se van, cada uno con su respaldo, y ninguno de sus agentes vuelve a leer esa skill. Solo se van los archivos que el aviso nombró: lo que quedó suelto por otro camino mientras estaba abierto espera su propia pregunta.

Un documento de otro equipo no se elimina desde este: aquí se lee, se mueve y se saca, y eliminarlo es cosa de ese equipo.

Algunas copias el aviso las dice aparte, y nunca se van sin que las veas:

- **Una copia que cambió fuera de Tisty**, por ejemplo una que editaste a mano: «Cambió fuera de Tisty en Codex y Copilot: esa copia se queda y la ves después, aparte.». Después de eliminar se abre por ella el aviso [¿Quitar también el archivo?](#sacar-una-regla-y-su-archivo), con su ruta y «Cambió después de la última vez que Tisty lo escribió.». Si cambia entre la pregunta y tu respuesta, también se queda. **Solo de Tisty** no la anota como dejada: queda para esa pregunta.
- **Una copia que lee un agente en pausa**: «En pausa: Copilot. Su copia se queda y Tisty vuelve a preguntar cuando sincronice.». Vale para cualquier agente que la lea, también uno que solo comparte `~/.agents/skills` con otro, y para Gemini mientras falta la línea `@tisty-rules.md` en su `GEMINI.md`. Tisty no la toca. Cuando el agente vuelve a sincronizar, esa copia cuenta como archivo suelto: la tarjeta **Agentes** de Configuración lo dice con **Revisar**, y Tisty hace la pregunta de siempre la próxima vez que entras en Agentes.

Si todas las copias cambiaron o están en pausa, el aviso igual aparece, con **Solo de Tisty** y **Cancelar**. Si no queda suelto nada que Tisty escribió (el documento solo se trajo y nunca se sincronizó, Agentes nunca sincronizó en este equipo, o cada archivo también lo escribe otro documento), toda la pregunta es el «¿Borrar «voz»?» de siempre. Si dejaste de sincronizar, el aviso nombra en pausa a cada agente que lee una copia, con **Solo de Tisty** y **Cancelar**. Eliminar varios documentos a la vez hace un solo aviso para todos, con la misma lista que muestra la pregunta de siempre y todos los agentes nombrados juntos.

Si un documento no se puede eliminar, por ejemplo porque está bloqueado, ninguno de sus archivos se toca, y Tisty dice por qué. Si un archivo no se puede quitar, por ejemplo porque su carpeta es un enlace, el documento igual se elimina y Tisty nombra ese archivo y el motivo; queda para la pregunta de siempre.

#### Archivos que dejaste en tus agentes

Lo que dejas con **Solo de Tisty** sigue siendo tuyo: Tisty lo toma como archivo tuyo. La tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes** los lista aparte: «2 archivos que dejaste en tus agentes al eliminarlos de Agentes.», con **Revisar**, que pregunta por cada uno con [¿Quitar también el archivo?](#sacar-una-regla-y-su-archivo). Tisty nunca pregunta por ellos solo y nunca escribe encima, y una carpeta de Agentes que vacías igual se va. Si más adelante escribes una regla o skill que cae otra vez en el mismo archivo, Tisty pregunta antes, como con [un cambio por fuera](#un-cambio-por-fuera-traer-o-sobrescribir), con **Conservar el de Tisty** (con respaldo antes), o **Conservar el archivo** cuando el documento es de un agente de este equipo. Si traes ese mismo archivo con [Traer otro archivo](#traer-otro-archivo), vuelve a ser de Tisty y sale de esa lista. Si el documento que eliminaste era el último de Agentes, la carpeta se va, pero esta lista sigue en la tarjeta con **Revisar**, para que puedas cambiar de idea; nada más de lo que escribió Agentes se ofrece ahí. Esa lista es solo de este equipo: no viaja a tus otros equipos.

### Sacar una regla y su archivo

Sincronizando, Tisty deja de escribir un documento cuando lo sacas de Agentes: cuando lo archivas o lo mueves fuera, cuando cambias su línea `agent:` por la de otro archivo, cuando vacías todas sus palabras, cuando lo mueves a la carpeta de otro equipo, y cuando vuelves a una [versión anterior](Documentos#versiones-anteriores) que no tenía ficha, o tenía otra. Eliminarlo pregunta [en ese momento](#eliminar-una-regla-o-una-skill); esta pregunta es para todos los demás caminos, y para las copias que esa pregunta deja para después.

Como un documento es un archivo entero, ninguna regla ni skill escribe ya ese archivo, pero sigue en el disco y tu agente lo sigue leyendo. Por eso, alrededor de un segundo después, Tisty te pregunta qué hacer con él.

**Vaciar un documento.** Dejar sin ninguna palabra una regla es lo mismo que sacarla: Tisty nunca vacía el archivo de un agente, así que en vez de eso te pregunta por él con **¿Quitar también el archivo?**, como si lo hubieras sacado.

**Moverlo a otro equipo.** Un documento que mueves a la carpeta de otro de tus equipos deja de aplicarse aquí, y Tisty te pregunta por su archivo de este equipo con la misma pregunta. Si tiene una propuesta de un agente esperando, no se deja mover: la ficha dice «Un agente propuso un cambio en este documento. Apruébalo o descártalo antes de moverlo a otro equipo». Lo que mueves a la carpeta de tu equipo, o a la raíz, cuenta como aplicarlo aquí; en los demás equipos queda esperando su **Aplicar aquí**.

**Un ejemplo.** Tenías la regla **Pruebas**, de `~/.claude/rules/pruebas.md`, y la moviste fuera de Agentes, a **Notas**. Se abre **¿Quitar también el archivo?**: «Ninguna regla ni skill de Agentes escribe ya este archivo de Claude, pero sigue en el disco y Claude lo sigue leyendo.», con la ruta debajo y «Si lo quitas, queda un respaldo en este equipo.».

- **Quitar el archivo** guarda un respaldo y lo borra: Claude deja de leer esa regla. Si era el último archivo de la carpeta de una skill, esa carpeta se va con él; las carpetas propias del agente, como `rules`, quedan.
- **Dejarlo** lo deja donde está, y Tisty deja de responder por él: no lo escribe ni te vuelve a preguntar. Si un día lo quieres de vuelta en Agentes, aparece en [Traer otro archivo](#traer-otro-archivo).

Si el archivo cambió después de que Tisty lo escribió, por ejemplo porque lo editaste a mano, el aviso suma «Cambió después de la última vez que Tisty lo escribió.». El respaldo guarda el archivo como está ahora, con ese cambio.

Si quitas `tisty-rules.md`, el archivo de reglas de Gemini, Tisty también saca la línea `@tisty-rules.md` de `GEMINI.md`, y nada más.

Mientras no respondes, la tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes** dice «1 archivo ya no es de ninguna regla ni skill de Agentes.», con **Revisar** para volver a la pregunta.

Tisty no pregunta, y deja el archivo como está:

- por la memoria del agente, que escribe el propio agente: la de cada proyecto de Claude Code ni el `GEMINI.md` de Gemini;
- por un archivo de un agente que pausaste, ni por ninguno mientras Agentes no sincroniza: traída sin sincronizar, sacar un documento no toca ningún archivo;
- por un archivo que solo trajiste y Tisty nunca escribió: **Traer** no lo anota como escrito por Tisty, así que no es de Tisty ofrecer quitarlo;
- por un archivo cuyo documento [espera tu aprobación](#por-aprobar);
- por un archivo que ya no existe.

En tus otros equipos pasa lo mismo: cuando la regla borrada les llega con la sincronización, cada uno te pregunta por su propio archivo con esta pregunta la próxima vez que Tisty lo mira, y el respaldo queda en ese equipo. Un documento eliminado en otro equipo nunca quita un archivo en este hasta que respondes aquí.

### Eliminar una subcarpeta

Botón derecho sobre una subcarpeta de Agentes, como **Agentes › Casa (este equipo) › Claude › Pruebas**, y **Eliminar**. Tisty pregunta antes «¿Borrar «Pruebas»? Sus documentos pasan a Sin clasificar y sus subcarpetas suben un nivel. No se pierde nada, pero no se puede deshacer.». Con tu sí:

1. Sus documentos pasan a **Sin carpeta**. Como salen de Agentes, Tisty deja de escribirlos.
2. Alrededor de un segundo después, Tisty te pregunta por cada archivo que armaban, igual que en [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo): **Quitar el archivo** guarda un respaldo en este equipo y lo borra; **Dejarlo** lo deja donde está.
3. La carpeta no vuelve sola: Tisty no rehace una carpeta de Agentes que borraste.

Como al sacar una regla, no pregunta por un agente en pausa ni mientras Agentes no sincroniza: entonces Tisty no escribía ningún archivo.

**Un ejemplo.** Marisol guardaba sus reglas de pruebas en una carpeta suya, **Agentes › Casa (este equipo) › Claude › Pruebas**, y ya no las quiere. Botón derecho sobre **Pruebas** y **Eliminar**; un segundo después, Tisty le pregunta **¿Quitar también el archivo?** por `~/.claude/rules/pruebas.md`. Elige **Quitar el archivo**: el respaldo queda en este equipo, Claude deja de leer esas reglas y sus documentos siguen en **Sin carpeta**, por si un día los quiere de vuelta.

### Vaciar Agentes

Si sacas todos los documentos de Agentes mientras Tisty está abierto y la carpeta sincroniza, primero te pregunta por los archivos que escribían sus últimas reglas, uno por uno, como en [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo). Cuando respondiste el último, la carpeta Agentes y sus subcarpetas desaparecen de Tisty, y los archivos que elegiste dejar quedan como estaban. Traída sin sincronizar, no hay nada que preguntar: Tisty no escribía ningún archivo, y la carpeta se va apenas sale el último documento.

Solo se va una Agentes que tenía documentos y vaciaste con la ventana abierta. Una Agentes vacía de otra forma se queda: la que se creó con **Traer y sincronizar** sin encontrar archivos, o la que ya estaba vacía al abrir Tisty.

Vaciada, Agentes vuelve a la barra como al principio, en el estado 1: el robot y el nombre en gris. Si la quieres otra vez, un clic en ella abre de nuevo el aviso de la primera vez, con las reglas y las skills que encuentre ya marcadas, también las que dejaste, y la memoria sin marcar, como siempre.

### Ocultarla de la barra

Si no usas Agentes y no quieres verla en la barra, ni siquiera en gris, ocúltala. **Nunca queda oculta y sincronizando**, para que no escriba archivos que no ves.

1. Botón derecho sobre Agentes y **Ocultar de la barra**, en cualquiera de sus tres estados. También sirve **Ocultar** en la tarjeta **Agentes** de Configuración › Agentes › Carpeta Agentes.
2. Si Agentes sincroniza, antes te avisa: «Para ocultar Agentes, primero deja de sincronizar. Tus archivos de agente no se tocan.», con **Dejar de sincronizar y ocultar** y **Cancelar**.
3. **Dejar de sincronizar y ocultar** hace las dos cosas, en ese orden: Agentes pasa al estado 2 y sale de la barra. **Cancelar** la deja como estaba, a la vista y sincronizando. Si Tisty no logra saber si Agentes sincroniza, te avisa igual.

Agentes sale de la barra lateral en este equipo sin dejar rastro, y queda en **Configuración › Aspecto y ventana › Carpetas ocultas**, junto a cualquier otra carpeta que hayas ocultado: mira [Ocultar una carpeta de la barra](Documentos#ocultar-una-carpeta-de-la-barra). Su tarjeta en Configuración › Agentes › Carpeta Agentes dice «Oculta de la barra en este equipo y sin sincronizar: no escribe ningún archivo de agente.», con **Volver a mostrar**.

- **Volver a mostrar**, en Carpetas ocultas o en la tarjeta, la devuelve a la barra en el estado que tenía: en gris, sin traer o traída sin sincronizar.
- **Sincronizar**, en la tarjeta de una Agentes traída y oculta, la sincroniza y también la vuelve a mostrar: sincronizando, Agentes siempre se ve.

**Oculta, Agentes no sincroniza.** No escribe ningún archivo de tus agentes, y nada de esos archivos espera tu decisión. Tus documentos siguen ahí: la búsqueda y **Mover a** los encuentran, y si un agente te propone un cambio, la propuesta queda en la ficha de su documento hasta que la apruebas o la descartas.

**Si en la versión anterior la habías ocultado mientras escribía los archivos de tus agentes**, ahora la encuentras otra vez en la barra, sincronizando. Desde ahí, si no la quieres ver, **Dejar de sincronizar** y **Ocultar de la barra**.

**Las subcarpetas siguen la misma regla.** **Ocultar de la barra** está siempre en el botón derecho de **Global**, de la carpeta de un equipo, de **Claude** o de cualquier carpeta dentro de Agentes; en la de otro equipo es lo único que ofrece. Mientras Agentes sincroniza, pasa por el mismo aviso, y **Dejar de sincronizar y ocultar** deja de sincronizar Agentes entera antes de ocultar esa subcarpeta: lo que tiene no debe quedar escribiendo sin que lo veas. Y si una subcarpeta oculta vuelve a sincronizar, por ejemplo con **Sincronizar** en la tarjeta, vuelve sola a la barra.

### Agentes se queda arriba

Tisty reconoce la carpeta porque está arriba, en el primer nivel de tu Biblioteca y no dentro de otra carpeta, lleva su icono de robot y guarda el nombre `AGENTS`. En la ventana la ves como **Agentes**, o **Agents** con Tisty en inglés, pero por dentro sigue llamándose `AGENTS`: así la encuentran las versiones anteriores de Tisty en tus otros equipos y los agentes que la leen por MCP.

Por eso Agentes no se puede meter dentro de otra carpeta, cambiarle el nombre, darle otro icono ni archivarla, y mientras tenga algo adentro, tampoco borrarla. Vale igual si lo intentas desde la ventana, si lo pide un agente por MCP o si llega de otro de tus equipos: Tisty no lo hace, y tus reglas siguen llegando a tus agentes como antes. Si lo intentas desde la ventana, por ejemplo arrastrándola dentro de **Trabajo**, Tisty te dice «Agentes se queda arriba, con su nombre y su robot, para seguir cuidando las reglas de tus agentes. Su color y su lugar entre tus carpetas sí los cambias».

Lo que sí es tuyo: su color y su orden entre tus carpetas. Para que deje de escribir los archivos de tus agentes, la forma es **Dejar de sincronizar**; para no verla, **Ocultar de la barra**; y para que desaparezca, [vaciarla](#vaciar-agentes).

## La tarjeta Agentes en Configuración › Agentes › Carpeta Agentes

La tarjeta **Agentes** usa las mismas palabras y hace lo mismo que la barra. Arriba, siempre, «Las reglas, skills y memoria de tus agentes como documentos de Tisty. Traer las copia aquí; Sincronizar además escribe lo que cambies en el archivo que lee cada agente.». Debajo, según el estado:

| Estado | Lo que dice | Sus botones |
| :-- | :-- | :-- |
| **1. Todavía sin traer** | Qué agentes vio Tisty, como «Encontré Claude y Codex en este equipo.», o «No encontré ningún agente en este equipo.», y «Solo miré si existen sus carpetas; no leí nada de adentro.» | **Traer** y **Traer y sincronizar**, que abren los dos el aviso de la primera vez, y **Ocultar** |
| **2. Traída, sin sincronizar** | «No se sincroniza con tus agentes» | **Sincronizar**; **Traer otro archivo**, cuando este equipo ya te dejó leer los archivos de tus agentes; y **Ocultar** |
| **3. Sincronizando** | Lo que espera tu decisión, como «1 archivo dice algo distinto en Tisty y en el agente.», con **Revisar** | **Traer otro archivo**, **Dejar de sincronizar** y **Ocultar**, y debajo un interruptor por agente: **Sincronizar Claude**, **Sincronizar Codex**… |

Con Agentes sincronizando y un agente sin sincronizar, su interruptor dice «Sin sincronizar: sus archivos quedan como están y no se borra nada en Tisty.». Un agente que sincroniza pero todavía no tiene ningún archivo lo dice bajo su interruptor: «Gemini: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta.». Si Tisty no encuentra ningún agente en este equipo, la tarjeta dice «No encontré agentes en este equipo.». **Ocultar** aparece en los tres estados mientras no está oculta ya; sincronizando, antes te pide dejar de sincronizar, con el mismo aviso de la barra.

## Si venías de la versión anterior

En la versión anterior, la carpeta se veía en mayúsculas en la barra y, mientras no escribía los archivos de tus agentes, atenuada, en gris, con una palabra al lado. Al actualizar no tienes que hacer nada:

| Cómo la veías | Cómo la encuentras |
| :-- | :-- |
| Con su color, escribiendo los archivos de tus agentes | En el estado 3, **sincronizando**, con su robot coral. Sigue escribiendo los mismos archivos |
| Atenuada, con documentos | En el estado 2, **traída, sin sincronizar**. Ningún archivo de tus agentes cambió |
| Atenuada, sin documentos ni carpeta | En el estado 1, **todavía sin traer**, en gris |
| Oculta, sin escribir archivos | Sigue oculta, en Configuración › Aspecto y ventana › Carpetas ocultas |
| Oculta mientras escribía archivos | De vuelta en la barra, sincronizando |

Desde ahora, el primer paso es **Traer** o **Traer y sincronizar**, también cuando no marcas nada; con la carpeta ya traída, **Sincronizar**; y para detenerla, **Dejar de sincronizar**, en su botón derecho o en su tarjeta. Tus documentos, sus versiones anteriores y los respaldos de cada archivo quedan como estaban.

## En varios equipos

Si usas Tisty en más de un equipo, por ejemplo el computador de la casa y el del trabajo, tus documentos de Agentes viajan con tu sincronización como cualquier otro. Pero **cada equipo tiene su propia carpeta y escribe solo la suya**: que la casa sincronice no hace que el trabajo sincronice, y lo que está en la carpeta de otro equipo lo lees aquí, pero no lo escribes aquí.

### Cada equipo, su carpeta

```text
Agentes
├── Global                      lo de todos: reglas por agente, skills y hooks
├── Codex                       solo si moviste o creaste algo aquí: es para todos tus equipos
├── Casa (este equipo)
│   ├── Claude
│   └── Codex
└── Trabajo                     otro equipo: en gris tenue
    └── Claude
```

- **Tu carpeta** lleva el nombre que tu equipo tiene en la sincronización, con «(este equipo)» detrás. Tisty solo la lee y la sincroniza a ella: lo que traes, y cada agente que encuentra aquí, va a esa carpeta.
- **Las de los demás equipos** van al final, en gris tenue, por su nombre. Si dos equipos se llaman igual, un código corto los distingue. Ahí lees, mueves, copias como texto y sacas; no editas, no aplicas, no eliminas ni pausas.
- **La raíz es para todos los equipos.** Un documento queda ahí solo si lo mueves ahí o lo creas ahí, con botón derecho sobre el documento, **Mover a** y **Agentes** o la carpeta suelta, como **Codex**. Es para el caso en que quieres la misma regla en todos tus equipos. En cada equipo sigue esperando su **Aplicar aquí**, como cuenta el paso a paso.
- **Mover aplica o deja de aplicar.** Mover algo a la carpeta de tu equipo o a la raíz cuenta como aplicarlo en este equipo; en los demás espera su **Aplicar aquí**. Mover un documento a la carpeta de otro equipo deja de aplicarlo aquí, y Tisty te pregunta por su archivo, como cuenta [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo). Un documento con una propuesta de un agente esperando no se deja mover a otro equipo hasta que la apruebes o la descartes, y tampoco una carpeta que tenga uno.
- **Tisty pregunta antes de un movimiento que cambia de quién es.** Cuando mueves un documento o una carpeta entre la raíz, la carpeta de tu equipo y la de otro equipo, arrastrándolo o con **Mover a**, primero una pregunta dice qué cambia, y nada se mueve hasta que pulsas **Mover**. **Cancelar** deja todo como estaba. Un movimiento dentro de la misma carpeta, o que se queda en el mismo equipo, no pregunta nada.

**Un ejemplo.** Yisset mueve **Ramas**, su regla de Claude en la raíz, a **Casa (este equipo) › Claude**, porque en el trabajo las ramas van de otra forma. Tisty pregunta:

```text
¿Mover «Ramas» a la carpeta de este equipo?

En este equipo, se sigue escribiendo en CLAUDE.md.
Tus otros equipos conservan su archivo como está, como un documento propio.
```

Cada línea dice una cosa que va a pasar:

| La línea | Qué significa |
| :-- | :-- |
| **Tisty lo escribe en …** | Este equipo empieza a escribir ese archivo, con un respaldo antes |
| **Se sigue escribiendo en …** | Ya se escribía aquí, y se sigue escribiendo |
| **Otro documento también escribe …** | Algo en la raíz o en tu carpeta escribe el mismo archivo: Tisty pregunta cuál manda aquí, como cuenta [Si dos documentos van al mismo archivo](#si-dos-documentos-van-al-mismo-archivo) |
| **Tisty deja de escribir … y te pregunta** | Sale de este equipo, y Tisty te pregunta qué hacer con el archivo, con un respaldo |
| **Su texto aún no está aprobado** | Ningún equipo lo escribe hasta que lo apruebes |
| **Todavía ningún agente lo lee** | Su agente no está en este equipo, o su sincronización está en pausa aquí: no se escribe nada |
| **En tus otros equipos, espera «Aplicar aquí»** | Va a la raíz: cada uno de tus otros equipos elige |
| **En …, un Tisty anterior** | Ese equipo lo lee como un documento para todos tus equipos hasta que se actualice a 1.50.2 |

Cuando mueves una carpeta, la pregunta cuenta sus documentos y da una línea por cada uno que cambia algo en este equipo.

### Un segundo equipo, paso a paso

**Un ejemplo.** En el equipo de la casa, Mario eligió **Traer y sincronizar**: su `CLAUDE.md`, **Cómo trabajo**, quedó en **Agentes › Casa (este equipo) › Claude**, y escribió allí otra regla, **Estilo de código**. Ahora instala Tisty en el equipo del trabajo y sincroniza:

1. En la barra, Agentes llega en el **estado 2**, traída sin sincronizar. Ve **Casa** en gris tenue, al final, con sus documentos: son los del otro equipo, y los puede leer desde ya. Arriba dice «No se sincroniza con tus agentes · Sincronizar». Ningún archivo de los agentes del trabajo cambió.
2. Pulsa **Sincronizar**, en esa línea, en el botón derecho o en la tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes**. Como este equipo nunca te dejó leer los archivos de sus agentes, primero se abre el aviso de la primera vez, con los archivos de este equipo. Los que ya tienen documento dicen **ya está en Agentes**.
3. Marca lo que quieras traer de este equipo, como la memoria de sus proyectos, y pulsa **Traer y sincronizar**. Se crea **Trabajo (este equipo)**, con una carpeta por cada agente que encontró, y Agentes pasa al estado 3, con su robot coral. Si en cambio eliges **Traer**, entra lo que marcaste y este equipo sigue sin sincronizar.
4. **Cómo trabajo** y **Estilo de código** se quedan en **Casa**: en el trabajo no se escriben en ningún archivo. Su ficha dice «Es de otro de tus equipos: en este solo se lee.», sin **Editar**, **Aplicar aquí** ni **Eliminar**.
5. Si Mario quiere **Estilo de código** también en el trabajo, lo copia como texto a la carpeta de su agente del trabajo, o lo mueve a la raíz: botón derecho sobre el documento, **Mover a** y **Agentes**. Mover a la raíz cuenta como aplicarlo en el equipo donde lo hace; en el otro, la ficha muestra un aviso ámbar, «Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este?», con dos botones:
   - **Aplicar aquí** lo escribe en los archivos de los agentes de este equipo, con su respaldo antes, como siempre. Si un archivo de este equipo ya decía otra cosa, Tisty no lo pisa: la ficha queda en **Cambiado en los dos lados** y eliges.
   - **No en este equipo** lo deja fuera aquí. La ficha dice «Elegiste no aplicarlo en este equipo.», y **Aplicar aquí** sigue a mano por si cambias de idea.

Lo que eliges vale para ese documento en ese equipo, y queda:

- **Un documento que ya aplicas se pone al día solo.** Si mañana cambias el documento de la raíz en la casa, el trabajo lo escribe en cuanto llega, sin volver a preguntar.
- **Lo que escribes o traes en un equipo se aplica en ese equipo sin ningún clic.** Solo lo que llega a la raíz desde otro espera.
- **Solo espera lo que este equipo leería.** Un documento de Codex no pregunta nada en un equipo sin Codex.
- **Una vez aplicado, la ficha no ofrece deshacerlo.** Para que un agente de este equipo deje de recibirlo, pausa ese agente aquí, o deja de sincronizar Agentes en este equipo; para que ningún equipo lo reciba, sácalo de Agentes.

### Si dos documentos van al mismo archivo

Pasa cuando el documento de tu carpeta y uno de la raíz escriben el mismo archivo, o la misma skill. Mientras no elijas, en este equipo no cambia nada: ninguno de los dos se escribe. Lo pregunta la ficha de los dos, sin importar cuál llegó primero: «En este equipo «CLAUDE.md» de todos tus equipos ya va al mismo archivo. ¿Cuál manda aquí?». Ofrece un botón por cada uno, con su título y de quién es, como **Usar «CLAUDE.md» de este equipo** y **Usar «CLAUDE.md» de todos tus equipos**, porque los dos suelen llamarse igual.

**Un ejemplo.** Marisol tiene en **Casa (este equipo) › Claude** su `CLAUDE.md`, y desde la casa mueve a la raíz otro documento que también va a `CLAUDE.md`. Al llegar, el trabajo no sabe cuál debe mandar. Marisol abre cualquiera de los dos y pulsa el botón **Usar** del que quiere: ese se escribe en `~/.claude/CLAUDE.md`, y el otro sigue en su carpeta, con la ficha «En este equipo manda …».

- **Una skill se elige entera**, con todos sus archivos.
- **Se elige por archivo.** Si eliges para Claude, una skill de Global sigue funcionando en los demás agentes.
- **Puedes cambiar de idea**: el que no manda sigue en su carpeta, y su botón **Usar** lo pone a mandar.
- **Si el que mandaba se va**, Tisty no escribe el otro por su cuenta: vuelve a preguntar.

### Documentos de otros equipos

Los de la carpeta de otro equipo se leen, se mueven, se copian como texto y se sacan. No tienen **Editar**, **Aplicar aquí**, **Eliminar** ni pausa, y la ficha dice «Es de otro de tus equipos: en este solo se lee.». Un agente que intente escribir uno por MCP recibe un rechazo. Las palabras que otro equipo aprobó para este esperan aquí cada vez, hasta que pulses **Aplicar aquí**.

La **memoria** es igual, y [más estricta todavía](#la-memoria-es-del-equipo-que-la-trajo).

### Aprobar en otro equipo no es aplicar

Aprobar cambia el texto; **Aplicar aquí** decide si un equipo escribe el archivo. Son dos cosas aparte, y en un equipo donde un documento espera, solo la segunda lo escribe.

**Un ejemplo.** Jaime tiene **Cómo trabajo** en la raíz. En el trabajo todavía espera su **Aplicar aquí**. Mientras tanto, Claude Code propone cambiarlo, y como Jaime está en el trabajo, lo aprueba ahí mismo:

1. **Aprobar el cambio** pregunta «¿Apruebas este cambio? Se escribe en el documento, y llega a los archivos de tus agentes en cada equipo donde Agentes sincroniza y lo aplicaste.». Con tu sí, el documento dice lo nuevo en todos tus equipos.
2. En la casa, que ya lo aplicaba, Tisty escribe el cambio en sus archivos como siempre, con su respaldo antes.
3. En el trabajo no se escribe nada todavía. La ficha cambia su aviso ámbar a «Aprobaste su texto en este equipo, pero aplicarlo aquí es aparte. ¿Lo aplicas a los agentes de este?», y siguen **Aplicar aquí** y **No en este equipo**. El cuadro de datos sigue diciendo «No: espera «Aplicar aquí»».
4. Pulsa **Aplicar aquí** cuando quieras que el trabajo lo reciba.

Pasa lo mismo con **Aprobar este texto** y con volver a una versión anterior desde el historial del documento: el texto cambia en todos tus equipos, y un equipo donde el documento esperaba sigue esperando. Lo que tú escribiste o trajiste en un equipo se aplica en ese equipo sin ningún clic, como siempre.

### Dos carpetas Agentes que se unen solas

Si dos equipos crearon Agentes antes de sincronizarse entre sí, por ejemplo porque instalaste Tisty en el trabajo sin conectar todavía la sincronización y elegiste **Traer y sincronizar** en los dos, al encontrarse aparecen **dos carpetas Agentes** en la barra por un momento.

Tisty las une solo, sin preguntar:

- **Queda la más antigua.** La misma en todos tus equipos, aunque los dos unan a la vez.
- **Las subcarpetas se juntan por nombre.** Lo que estaba en **Global** del otro equipo pasa a tu **Global**, la carpeta de cada equipo a la suya, y así. Una subcarpeta que solo tenía el otro pasa entera.
- **Los documentos pasan a la carpeta que queda,** con su texto, su aprobación y su historial: ninguna aprobación se pierde. La carpeta que quedó vacía se borra.
- **Ningún archivo de tus agentes cambia** por la unión, y por ninguno te pregunta «¿Quitar también el archivo?». Lo que ya se aplicaba en un equipo sigue aplicándose ahí.
- **Una carpeta archivada pasa entera** y sigue archivada: lo que guardaba no vuelve a escribirse.

Un caso raro que queda como está: si en la carpeta que se iría hay algo que puso un agente, Tisty no las une, porque al moverlo parecería puesto por ti. Mueve tú ese documento a una carpeta tuya y la unión se hace sola en la siguiente vuelta. Mientras haya dos carpetas Agentes sin unir, las dos quedan protegidas: tu agente solo puede proponer en cualquiera de ellas.

### La memoria es del equipo que la trajo

La memoria de un agente habla de lo que pasó en ese equipo, así que nunca se escribe en otro. En los demás equipos, su ficha dice «Memoria del agente de otro equipo: en este solo se lee.», sin botones. Cada equipo puede traer su propia memoria del mismo proyecto: aparece sin marcar en el aviso de la primera vez, aunque otro equipo ya haya traído la suya.

### Si venías de la 1.50.1

Antes, un documento vivía en **Claude**, **Codex** o **Global** para todos los equipos a la vez, y Tisty juntaba en un mismo archivo lo de Global con lo de tu agente. Desde la 1.50.2 es al revés: cada equipo tiene su carpeta, y un documento es un archivo.

- **Lo que cada equipo trajo o escribió pasa solo a su carpeta**, cuando todos tus equipos ya tienen la 1.50.2 o una más nueva. Ningún archivo de un agente cambia ni en un byte.
- **Un `CLAUDE.md` que la 1.50 había juntado con una regla con nivel `global`** pasa a ser un documento que dice exactamente lo que dice el archivo.
- **Mientras otro equipo siga en una versión anterior**, nada se mueve y los archivos quedan como están.
- **Si el archivo cambió por fuera**, se deja y se pregunta como siempre.
- **En un equipo que todavía tiene la 1.50.1**, puedes editar un documento de otro equipo, pero ese texto espera tu aprobación en el equipo más nuevo.
- **Una regla con nivel `global` (`agent: global/rule/…`) de esa época** se queda en Global para que la leas, la copies o la borres; ya no se escribe en ningún agente.

### Por aprobar

**Solo las palabras que tú escribiste o aprobaste llegan al archivo de un agente.** Lo que escribes en la ventana de Tisty, lo que traes la primera vez y lo que traes con **Traer el cambio** ya cuenta como aprobado. Un texto que cambió sin pasar por tus manos no: por ejemplo, si alguien editó el archivo del documento dentro de la carpeta de datos de Tisty, o lo trajo un paquete importado.

Entonces, dentro de la ficha del documento, un aviso ámbar dice «Su texto aún no tiene tu aprobación, así que no llega a ningún agente hasta que lo apruebes.». No es una etiqueta de estado: es esa frase, y debajo te muestra, antes del botón, lo que necesitas para decidir:

1. **Quién escribió estas palabras**, en una línea: «Se escribió en este equipo.», «Se escribió en otro equipo.», «Lo escribió un agente.», «Volvió una versión anterior.» o «Cambió fuera de Tisty.».
2. **Qué cambió desde lo último que aprobaste:**
   - Si lo último aprobado está guardado en este equipo, las líneas comparadas: en rojo, con `−`, lo **Aprobado** que se va; en verde, con `+`, lo que dice **Ahora**.
   - Si el documento nunca se aprobó: «Nunca se aprobó: léelo entero antes de aprobarlo.».
   - Si lo último aprobado no está guardado aquí: «Lo último aprobado no está guardado en este equipo, así que no hay con qué compararlo: léelo entero.».
3. El botón **Aprobar este texto**.

**Aprobar este texto** pregunta «¿Apruebas lo que dice ahora este documento? Léelo antes: llega a los archivos de tus agentes en cada equipo donde Agentes sincroniza y lo aplicaste.». Si el documento cambió entre que se mostró y tu clic, Tisty no aprueba nada: «Este documento cambió desde que se mostró su ficha. Léelo de nuevo y vuelve a intentarlo.».

**Un ejemplo.** Un programa de Jaime que ordena archivos tocó, sin que él lo supiera, el archivo de la regla **Cómo trabajo** dentro de la carpeta de datos de Tisty, y le sumó una línea. Su ficha muestra el aviso ámbar, «Cambió fuera de Tisty.», y las líneas comparadas: `+ - Usa siempre la rama main.` en verde, bajo **Ahora**. Jaime no escribió eso: no lo aprueba, pulsa **Editar**, borra la línea y pulsa **Listo**. Lo que escribe en la ventana ya cuenta como aprobado, y su `CLAUDE.md` vuelve a recibirlo.

Si ese documento ya se escribía en un archivo, el archivo no cambia mientras tanto: su ficha dice **En espera** y nombra el documento que lo detiene, por ejemplo «Este archivo queda como estaba: claude · rules/estilo.md cambió o salió de Agentes sin tu aprobación. Apruébalo, o vuelve a ponerlo y sácalo tú.».

Un documento que puso en Agentes un agente, y no tú, tampoco se escribe: su ficha dice «Lo puso aquí un agente, así que ningún agente lo lee hasta que lo muevas tú.». Muévelo tú a otra subcarpeta de Agentes que hayas creado tú (cambiarlo de orden no basta), y se aplica. Si un agente creó o renombró la carpeta en que está, pasa lo mismo con todo lo que hay dentro: muévelo a una carpeta tuya.

### Escrito por un Tisty anterior

Una versión anterior de Tisty escribía los archivos sin anotar de qué documentos venía cada uno. La primera vez que la versión nueva los mira, compara: si el archivo dice lo mismo que sus documentos, lo adopta sin decir nada. Si dice otra cosa, la ficha queda en **Escrito por un Tisty anterior**, con el aviso «Un Tisty anterior escribió este archivo sin anotar de qué documentos venía, así que queda como está hasta que elijas». **Ver el archivo** te muestra qué tiene, y **Sobrescribir con Tisty** lo escribe desde tus documentos, con su respaldo antes.

## Lo que nunca se lee

- **Los ajustes de tus agentes**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` y `keybindings.json`. Para buscarlos, Tisty solo mira donde van las reglas, las skills y la memoria, nunca la carpeta principal de un agente, salvo el `settings.json` y `plugins/installed_plugins.json` de Claude Code, solo para saber si el plugin de Tisty ya trae sus skills; y dentro de una skill un archivo con esos nombres sí entra, porque es de la skill. Esto es de la carpeta Agentes: conectar un agente y ponerle sus hooks lee y escribe algunos de ellos, como cuenta [Privacidad](Privacidad#los-agentes-que-conectas).
- **Las credenciales**, por su nombre y sin abrirlas: `.credentials.json`, `credentials.json`, `auth.json`, `.env` y sus variantes, `.netrc`, y las llaves `.pem`, `.key`, `.p12`, `.pfx`, `id_rsa` o `id_ed25519`.
- **Un archivo que parece llevar un secreto**, como un token o una llave privada, aunque su nombre sea inocente. La revisión se hace en este equipo y busca formas conocidas, así que puede no reconocerlas todas: deja los secretos fuera de las reglas de tus agentes.
- **Las conversaciones de Claude Code**: de su carpeta `projects` solo se mira la carpeta `memory` de cada proyecto.
- **Los enlaces simbólicos** y los archivos de más de 512 kB.

Y un documento de Agentes solo se escribe donde van las reglas, las skills y la memoria de cada agente, nunca en sus ajustes ni en sus credenciales.

**La memoria del agente** viene sin marcar la primera vez, y una vez en Tisty es solo para leer: el agente la sigue escribiendo, y cuando cambia, la ficha dice **Cambiado por fuera** y **Traer el cambio** la pone al día en Tisty. No tiene **Editar**, **Sobrescribir con Tisty** ni **Recuperar**, y en tus otros equipos [solo se lee](#la-memoria-es-del-equipo-que-la-trajo).

Qué guarda Tisty de todo esto, y qué viaja, está en [Privacidad](Privacidad#tus-agentes-en-la-carpeta-agentes).

## Con tu agente, por MCP

Un agente que dejas entrar por [[MCP|Agentes-y-MCP]] **lee** tus documentos de Agentes como cualquier otro, sincronice o no, y su guía le explica qué son. **No los escribe: te propone el cambio, y solo tú lo apruebas.** Tampoco puede traer, sincronizar, dejar de sincronizar, ocultar ni mostrar Agentes: ninguna herramienta MCP lo hace, solo tú, en la ventana de Tisty. Por MCP la carpeta se llama `AGENTS`, su nombre guardado, en cualquier idioma: por eso las respuestas de abajo dicen `AGENTS` y no Agentes. Por ejemplo, le pides a Claude Code «¿Qué reglas mías lees?».

**1. Lee su guía.** `rules` con el tema «documents» trae, entre otras, la sección sobre la carpeta:

```json
{ "name": "rules", "arguments": { "topic": "documents" } }
```

```text
## Assistants' files in AGENTS

A document whose properties hold an `agent:` line is an assistant's own file kept in the folder
AGENTS — a rule file such as `CLAUDE.md`, a skill, a script — and Tisty writes its text, byte for
byte, to the file that assistant reads, on each computer where AGENTS syncs and the person applied
it. `read` gives the text the person approved. You change one only by proposing: `write_doc` with
`doc` and `print`, `edit_doc` or `append_doc` keep your text byte for byte, apart, and nothing
reaches any file until the person approves it on the document's card: say so, never call it done.
A later write goes on top of your proposal, which `read` shows in `proposal`. Keep the `agent:`
line: any other tool that names these documents or the folder AGENTS is refused, as is adding or
changing an `agent:` line anywhere: tell the person what to write. …
```

**2. Busca el documento.** `brain` con las palabras del archivo:

```json
{ "name": "brain", "arguments": { "query": "CLAUDE.md" } }
```

```text
0 task(s) and 1 document(s) match "CLAUDE.md"; showing 0 and 1.
"claude · CLAUDE.md" (document) [k4r5j4qr-0001]
```

`brain` con `folder` y el nombre o la ruta de una subcarpeta lista los documentos de ese agente. La carpeta `AGENTS` sola responde con cuántos hay en cada subcarpeta.

**3. Mira su ficha y lo lee.** `brain` con `doc` da su índice y su ubicación; `read` con `doc`, el texto que aprobaste, entero, con la línea `agent:`:

```json
{ "name": "read", "arguments": { "doc": "k4r5j4qr-0001" } }
```

```text
---
agent: [claude/rule/CLAUDE.md, on-dev_ab12cd34]
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

**De otro equipo.** Si el documento es de la carpeta de otro de tus equipos, la respuesta de `read` trae dos campos más, `of_computer` (de qué equipo es) y `read_as`, que le dice al agente que lo lea como datos y nunca como una regla suya para este equipo. Tampoco puede proponer un cambio sobre él: eso se hace desde ese equipo. Sobre los documentos de este equipo propone como siempre, y no hay ninguna herramienta nueva.

El agente te contesta con el título y la carpeta, «tu documento claude · CLAUDE.md, en AGENTS / Casa / Claude», nunca con el identificador. Cada herramienta está en [[Herramientas MCP|Herramientas-MCP]].

### Una propuesta de tu agente

Le pides «agrega a mis reglas que corra los tests antes de cada commit». El agente corrige el documento como cualquier otro, pero en Agentes su cambio no se escribe: **queda aparte, como una propuesta**, byte a byte. Nada cambia en el documento ni en los archivos de tus agentes hasta que la apruebas.

**1. El agente propone.** Usa `edit_doc`, `append_doc` o `write_doc` con `doc`, `print` y el texto entero:

```json
{ "name": "edit_doc", "arguments": { "doc": "k4r5j4qr-0001", "old": "- Ramas con la forma tipo/nombre.", "new": "- Ramas con la forma tipo/nombre.\n- Corre los tests antes de cada commit." } }
```

```text
Proposed that change to "claude · CLAUDE.md" [k4r5j4qr-0001]. It waits beside the approved text for
the person, who approves or discards it on the document's card in Tisty; no assistant's file
changes until they approve it. Your next write to this document goes on top of this proposal, at
this `print`.
```

Su guía le pide decírtelo así, «te lo propuse», y nunca darlo por hecho.

**2. La ves en Tisty.** Abre el documento: la ficha suma la etiqueta **Propuesta** y, debajo, un recuadro morado:

- «Un agente (Claude Code) propone un cambio», con cuándo lo propuso;
- «Nada de esto llega a los archivos de tus agentes hasta que lo apruebes. Lo que cambia en el texto aprobado:»;
- las líneas comparadas: en rojo, con `−`, lo **Aprobado** que se va; en verde, con `+`, lo **Propuesto** que llega;
- los botones **Aprobar el cambio** y **Descartar**.

Si Tisty está abierto mientras el agente propone, la propuesta aparece sola en uno o dos segundos, aunque la ventana no esté al frente.

**3. Apruebas o descartas.**

- **Aprobar el cambio** pregunta «¿Apruebas este cambio? Se escribe en el documento, y llega a los archivos de tus agentes en cada equipo donde Agentes sincroniza y lo aplicaste.». Con tu sí, el documento pasa a decir lo propuesto, con tu aprobación, y en cada equipo donde Agentes sincroniza y ese documento se aplica, Tisty lo escribe en `~/.claude/CLAUDE.md` como cualquier cambio tuyo, con su respaldo antes. Lo que decía antes queda en su historial.
- **Descartar** pregunta «¿Descartas esta propuesta? El documento queda como lo aprobaste.». La propuesta se va y nada más cambia.

Cada botón actúa solo sobre lo que la ficha te mostró. Si el agente volvió a escribir mientras mirabas, Tisty no elige por ti: «La propuesta cambió desde que la ficha la mostró. Revísala de nuevo antes de elegir.». Mientras editas el documento, **Aprobar el cambio** no aparece: pulsa **Listo** primero.

**Lo que conviene saber:**

- **Una propuesta por documento.** Si el agente vuelve a escribir, su cambio va encima de su propia propuesta, y `read` se la entrega aparte del texto aprobado, en `proposal`. Si escribe de vuelta el texto aprobado, la retira.
- **Lo aprobado sigue llegando a tus agentes** mientras una propuesta espera: una propuesta en una regla no detiene nada más. El cuadro de datos de la ficha lo dice: «Sí · un cambio propuesto por un agente espera tu aprobación».
- **Si el texto aprobado cambió después**, la ficha avisa «Se propuso sobre una versión anterior, así que aprobarla puede deshacer un cambio hecho después.». Mira bien las líneas antes de aprobar.
- **Si el texto del documento [espera tu aprobación](#por-aprobar)**, primero apruebas su texto con **Aprobar este texto** y después la propuesta: «El texto de este documento aún no tiene tu aprobación. Apruébalo primero y después la propuesta.».
- **Los `#palabras` de un documento de agente no son etiquetas de Tisty.** Una regla que dice «usa #rust» no aparece bajo esa etiqueta.

### Lo que no puede proponer

**La línea `agent:` no cambia**, porque decide en qué archivo se escribe el documento:

```json
{ "name": "edit_doc", "arguments": { "doc": "k4r5j4qr-0001", "old": "claude/rule/CLAUDE.md", "new": "codex/rule/AGENTS.md" } }
```

```text
"claude · CLAUDE.md" [k4r5j4qr-0001] would get another `agent:` line, which only the person
changes, so nothing was proposed. Tell the person what you would change and why.
```

**Cualquier otra herramienta** que nombre un documento de Agentes o la carpeta `AGENTS` se rechaza: mover, archivar, marcar o resumir un documento, hacerlo página de otro, unirlo a otro idioma, adjuntarle un archivo o volver a una versión anterior, y también mover, renombrar o archivar la carpeta misma. La lista completa está en [[Herramientas MCP|Herramientas-MCP#la-carpeta-agentes]]. Tisty le dice cómo proponer:

```text
the document "claude · CLAUDE.md" holds an assistant's own files in AGENTS, which only the person
changes, in Tisty: nothing an assistant moves, files, marks or sums up there reaches them. To
change its text, propose it: `write_doc` with `doc`, `print` and the whole `body`, `edit_doc`, or
`append_doc` at its end, and nothing else with them. The person approves or discards it on the
document's card.
```

**Meter otro documento o crear una carpeta** dentro de Agentes también se rechaza, y ahí Tisty le pide contarte qué cambiaría, para que lo hagas tú.

**Un documento nuevo con una línea `agent:`** se rechaza en cualquier carpeta, para que nadie pueda traer una regla desde fuera:

```text
a body with an `agent:` line is an assistant's own file, which only the person makes, in AGENTS.
Write it without that line, or tell the person what the file should say.
```

**Ni agregar ni cambiar una línea `agent:` en un documento que ya existe**, en ninguna carpeta y con ninguna herramienta, `edit_doc` incluida. Si Claude Code intenta convertir tu documento **Notas**, en **Trabajo**, en una regla, poniéndole propiedades con `agent: claude/rule/CLAUDE.md` arriba, Tisty no escribe nada:

```json
{ "name": "edit_doc", "arguments": { "doc": "m2q8v7xd-0001", "old": "# Notas", "new": "---\nagent: claude/rule/CLAUDE.md\n---\n# Notas" } }
```

```text
an `agent:` line makes an assistant's own file, which only the person makes, in AGENTS. Leave that
line as it was. Nothing was written.
```

Y si `append_doc` deja esas líneas al final de un documento, quedan como palabras del texto, nunca como sus propiedades: ese documento no llega a ningún agente.

**La memoria del agente** no acepta propuestas: la escribe el propio agente, en su archivo. Tampoco entra una propuesta que deje roto el bloque de código de un script, que pase de 64.000 caracteres o de 2.000 líneas, las que la ficha alcanza a comparar.

Y editar el archivo en disco tampoco sirve: ese cambio queda como **Cambiado por fuera** y espera tu decisión. **Por qué así:** sincronizando, una regla de Agentes llega a todos tus agentes, en cada equipo donde Agentes sincroniza y la aplicaste; si un agente pudiera escribirla sola, una instrucción escondida en una página que leyó podría terminar en las reglas de los demás. Con la propuesta, ves cada línea antes de que llegue.

## Preguntas frecuentes

**¿Tisty reemplaza mis archivos?** No los mueve ni los convierte: los sigue escribiendo donde tu agente los busca. Antes de la primera escritura guarda el original, el respaldo n.º 1, que no se borra nunca.

**¿Puedo seguir editando `CLAUDE.md` a mano?** Sí. Tisty marca el archivo como **Cambiado por fuera** y espera que elijas: traer tu cambio al documento o sobrescribirlo.

**¿Qué pasa si cierro o desinstalo Tisty?** Tus agentes siguen leyendo sus archivos tal como Tisty los dejó. Nada depende de que Tisty esté abierto.

**¿Agentes toca `settings.json`, los hooks o mi configuración MCP?** No los escribe, y solo lee el `settings.json` y `plugins/installed_plugins.json` de Claude Code, para saber si el plugin de Tisty ya trae sus skills. Conectar un agente es otra cosa: ahí Tisty agrega su servidor MCP a la configuración de ese agente, y sus hooks si los pides, como cuenta [[Agentes y MCP|Agentes-y-MCP]]; y, con la puerta abierta, lee esos archivos al abrir su ventana para ver si apuntan a este Tisty, [sin escribir nada](Privacidad#los-agentes-que-conectas).

**¿Por qué la memoria del agente viene sin marcar?** Porque la escribe el agente, puede hablar de tus proyectos, y una vez en Tisty viaja con tu sincronización y tus respaldos como cualquier documento. La traes solo si quieres leerla en Tisty.

**¿Qué es la línea `@tisty-rules.md` en mi `GEMINI.md`?** Es la única línea que Tisty agrega a ese archivo, para que Gemini lea tus reglas, que van en `tisty-rules.md`. El resto de `GEMINI.md` lo escribe Gemini. Si la borras, Tisty no la vuelve a poner: Gemini queda [en pausa](#gemini-en-pausa) hasta que pulsas **Volver a ponerla**.

**¿Agentes ve los agentes que uso dentro de WSL?** No. En Windows, Tisty mira tu carpeta de usuario de Windows, y un agente que corre dentro de WSL guarda sus archivos en tu carpeta de Linux.

**¿Y las reglas de cada repositorio?** Quedan donde están. Agentes maneja los archivos de tu usuario, no el `AGENTS.md` o el `CLAUDE.md` de cada proyecto.

**¿Puedo escribir una regla nueva desde cero?** Sí: botón derecho sobre la carpeta de un agente de tu equipo, como **Claude**, y **Documento nuevo**. El documento ya empieza con su línea `agent:`, y tú solo escribes la regla. En **Global** no: no es la carpeta de un solo agente, así que el documento nuevo empieza sin línea `agent:`. Si Agentes sincroniza, Tisty la escribe apenas escribes algo. Mira [Un documento nuevo ya es una regla](#un-documento-nuevo-ya-es-una-regla).

**¿Cuántos respaldos guarda?** El original, el respaldo n.º 1, que nunca se borra, y además los últimos 20 de cada archivo, en este equipo.

**¿Por qué tengo una carpeta Cursor vacía en Agentes?** Tisty encontró Cursor en este equipo, y cada agente que encuentra tiene su subcarpeta, aunque esté vacía. Si no la quieres, bórrala: Tisty no la vuelve a crear. Mira [Cómo queda la carpeta](#cómo-queda-la-carpeta).

**Instalé un agente nuevo. ¿Cómo lo sumo?** Botón derecho sobre **Agentes** y **Buscar de nuevo**: Tisty le crea su subcarpeta y te ofrece sus archivos nuevos ya marcados. Mira [Buscar de nuevo](#buscar-de-nuevo).

**¿Puedo pedirle a mi agente que cambie mis reglas?** Sí, y te lo propone: su cambio queda aparte, en la ficha del documento, hasta que pulsas **Aprobar el cambio** o **Descartar**. Nada llega a tus agentes antes. Mira [Una propuesta de tu agente](#una-propuesta-de-tu-agente).

**Uso Tisty en dos equipos. ¿Mis reglas llegan solas al otro?** Llegan como documentos, con tu sincronización, en la carpeta de tu otro equipo, y ahí se leen pero no se escriben en los agentes del otro. Si quieres la misma regla en los dos, muévela a la raíz con **Mover a › Agentes** (o a la carpeta suelta del agente, como **Codex**), y en el otro equipo pulsa **Sincronizar** y después **Aplicar aquí** en el documento, la primera vez. Después se pone al día sola. Mira [En varios equipos](#en-varios-equipos).

**¿Puedo tener una regla solo en un equipo?** Sí: déjala en la carpeta de ese equipo, y nadie más la escribe. Una de la raíz que no quieres en uno de ellos, ahí pulsa **No en este equipo** en su ficha.

**¿Por qué me pregunta «¿Cuál manda aquí?»?** Porque en este equipo dos documentos, el de tu carpeta y uno de la raíz, van al mismo archivo o a la misma skill. Elige con uno de los dos botones **Usar**, que nombran cada documento; mientras no lo hagas, no cambia nada. Mira [Si dos documentos van al mismo archivo](#si-dos-documentos-van-al-mismo-archivo).

**¿Puedo editar lo de mi otro equipo?** No desde este: lo lees, lo mueves, lo copias como texto y lo sacas. La ficha dice «Es de otro de tus equipos: en este solo se lee.». Mira [Documentos de otros equipos](#documentos-de-otros-equipos).

**Borré una regla. ¿Qué pasa con su archivo?** Al eliminarla, Tisty te pregunta en ese mismo aviso «¿Quitar también de Claude, Codex y Copilot?», con los agentes que la tienen: **Quitar también** borra sus archivos y guarda un respaldo de cada uno, y **Solo de Tisty** los deja donde están, listados en Configuración por si después cambias de idea. Mira [Eliminar una regla o una skill](#eliminar-una-regla-o-una-skill). Si la sacaste de otra forma, por ejemplo moviéndola fuera de Agentes, Tisty pregunta «¿Quitar también el archivo?»: mira [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo).

**La primera vez dejé un archivo sin marcar. ¿Lo puedo traer ahora?** Sí, con **Traer otro archivo**: desde el botón derecho sobre una subcarpeta de Agentes, como Claude; con Agentes abierta, desde el botón **+** de la Biblioteca o la paleta de comandos; o desde Configuración › Agentes › Carpeta Agentes. Mira [Traer otro archivo](#traer-otro-archivo).

**¿Por qué tengo Agentes en la barra si nunca la usé?** Está ahí, en gris, para que sepas que existe. Así no lee nada: solo mira si existe la carpeta de cada agente, como `~/.claude`, y la lista de nombres y tamaños aparece recién cuando haces clic en ella. Si no la quieres ver, botón derecho y **Ocultar de la barra**; vuelve con **Volver a mostrar**, en Configuración › Aspecto y ventana › Carpetas ocultas.

**¿Cuál es la diferencia entre Traer y Sincronizar?** **Traer** copia los archivos a Tisty y no escribe nada en ellos; **Sincronizar** hace que lo que cambias en Tisty llegue a esos archivos. Mira [Traer o sincronizar](#traer-o-sincronizar).

**Traje mis reglas y las cambié en Tisty, pero mi agente no lo ve.** Agentes está traída sin sincronizar: su robot está en gris y, al abrirla, arriba dice «No se sincroniza con tus agentes · Sincronizar». Pulsa **Sincronizar**.

**¿Dejar de sincronizar borra algo?** No. Ningún archivo de tus agentes se toca y ningún documento se borra: tus agentes siguen leyendo lo que Tisty escribió la última vez, y tus documentos quedan en Agentes.

**¿Mi agente puede proponerme un cambio con Agentes oculta o sin sincronizar?** Sí: la propuesta queda en la ficha de su documento, y la apruebas o la descartas ahí. Aprobada, cambia el documento; llega a los archivos de tus agentes cuando Agentes sincroniza.

**¿Por qué la barra no me dice si algo espera?** Porque Agentes se ve como cualquier carpeta: solo su color dice si sincroniza. Lo que espera tu decisión está en la ficha de cada documento y en la tarjeta **Agentes** de Configuración › Agentes › Carpeta Agentes, y sincronizando, Tisty te pregunta por los archivos pendientes apenas entras en ella. Mira [Cómo queda la carpeta](#cómo-queda-la-carpeta).

**¿Por qué Agentes no tiene documentos?** La creaste con **Traer y sincronizar** sin que Tisty encontrara archivos, o sus documentos salieron mientras Tisty estaba cerrado, por ejemplo al sincronizar con otro equipo. Escribe una regla con **Documento nuevo** en la carpeta de un agente de tu equipo, o trae un archivo con **Buscar de nuevo** o **Traer otro archivo**.

## Si algo no cuadra

**Un agente no aparece en la lista.** Tisty solo ve un agente cuya carpeta existe: `~/.claude`, `~/.codex`, `~/.copilot`, `~/.gemini` o `~/.cursor`. Si la moviste con una variable como `CODEX_HOME`, defínela como variable de tu usuario y vuelve a abrir Tisty. Si nunca usaste ese agente en este equipo, ábrelo una vez para que cree su carpeta.

**Un archivo aparece atenuado.** El motivo está a su lado: mira [lo que dice la lista](#lo-que-dice-la-lista-junto-a-un-archivo).

**Mi cambio en Tisty no llega al archivo.** Mira el estado en la ficha del documento:

- **En pausa**: reanuda ese agente, o la carpeta, en la ficha o en Configuración › Agentes › Carpeta Agentes.
- **Sin permiso en este equipo**: este equipo todavía no te dejó leer los archivos de tus agentes. Pulsa **Sincronizar** y elige en el aviso de la primera vez.
- **La ficha dice «La carpeta Agentes no se sincroniza con tus agentes.»**: Agentes está traída sin sincronizar, con su robot gris. Pulsa **Sincronizar**.
- **Cambiado por fuera** o **Cambiado en los dos lados**: Tisty espera tu decisión antes de escribir.
- **Nada que escribir**, **El archivo es un enlace** o **El bloque de código está roto**: la tabla de [cada estado](#cada-estado) dice qué hacer.
- **«Ningún agente de este equipo lee este documento todavía.»**: revisa la línea `agent:`. El nivel debe ser un agente que esté en este equipo, y el nombre, un archivo que ese agente lea.
- **«En este equipo otro documento escribe este archivo.»** o **«¿Cuál manda aquí?»**: dos documentos van al mismo archivo. Elige con los botones **Usar**, que nombran cada documento, o deja uno. Mira [Si dos documentos van al mismo archivo](#si-dos-documentos-van-al-mismo-archivo).
- **«Es de otro de tus equipos»**: ese documento se escribe en ese equipo, no en este.
- **«Está en la carpeta de un equipo y su texto dice otro»**: pulsa **Según su carpeta**.
- **En espera**: el texto del documento espera tu aprobación. La ficha lo nombra; [apruébalo](#por-aprobar).
- **Escrito por un Tisty anterior**: mira el archivo y, si te quedas con el documento, **Sobrescribir con Tisty**.
- **Un aviso ámbar sobre otro equipo o tu aprobación**: el documento todavía no se aplica en este equipo. Mira [En varios equipos](#en-varios-equipos).

**Mi regla no llega a mi otro equipo.** Una regla de la carpeta de un equipo se escribe solo en ese equipo. Si la quieres en los dos, muévela a la raíz. En el otro equipo, revisa que Agentes tenga su robot coral (si está gris, pulsa **Sincronizar**) y abre el documento: si dice «Este documento viene de otro equipo», pulsa **Aplicar aquí**; si dice «Elegiste no aplicarlo en este equipo.», también. Si la ficha dice «Ningún agente de este equipo lee este documento todavía.», ese equipo no tiene el agente de la regla.

**Una ficha dice «Su texto aún no tiene tu aprobación» y yo no cambié nada.** El texto de un documento cambió sin pasar por la ventana de Tisty, por ejemplo con un paquete importado o una edición directa en la carpeta de datos. El aviso te dice quién lo escribió y qué cambió desde lo último que aprobaste: léelo, y si está bien, **Aprobar este texto**. Mira [Por aprobar](#por-aprobar).

**Escribí una regla y su archivo quedó en Cambiado en los dos lados.** Ese archivo ya existía y no lo trajiste, así que Tisty no sabe qué parte es tuya. No lo sobrescribas: en **Configuración › Agentes › Carpeta Agentes**, pulsa **Revisar** y, en **Existe en los dos lados**, **Traer a Agentes**. El archivo entra como un documento con lo que dice, y el original queda como respaldo n.º 1. Mira [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste).

**Me pregunta «¿Quitar también el archivo?» y no borré nada.** También cuenta como sacar una regla cambiar su línea `agent:` por la de otro archivo, o volver a una versión anterior que no tenía ficha. Si no quieres tocar nada, **Dejarlo** lo deja donde está.

**Agentes no está en la barra.** La ocultaste, en este equipo: está en **Configuración › Aspecto y ventana › Carpetas ocultas**, y **Volver a mostrar** la devuelve. Si la vaciaste con Tisty abierto, no se oculta: vuelve sola a la barra, en gris, y un clic en ella trae otra vez lo que encuentre.

**El robot de Agentes está gris.** No sincroniza en este equipo. Si el nombre también está gris, todavía no traes nada: haz clic en ella. Si el nombre se ve normal, está traída: pulsa **Sincronizar**. Mira [Tres estados en la barra](#tres-estados-en-la-barra).

**Ocultar me pide dejar de sincronizar.** Agentes sincroniza, y nunca queda oculta y sincronizando, ni Agentes ni sus subcarpetas, como **Global**. **Dejar de sincronizar y ocultar** hace las dos cosas; tus archivos de agente no se tocan. Si prefieres que siga sincronizando, **Cancelar**.

**Una subcarpeta de Agentes no tiene Cambiar nombre ni Diagrama nuevo.** Es a propósito: dentro de Agentes, cada carpeta es de un agente, y Tisty sabe a quién le escribe por su nombre. Su botón derecho ofrece solo **Documento nuevo**, **Traer otro archivo**, la pausa de su agente, **Ocultar de la barra** y **Eliminar**; la carpeta de otro equipo, solo **Ocultar de la barra**. Mira [El botón derecho de una subcarpeta](#el-botón-derecho-de-una-subcarpeta).

**El botón derecho de Agentes solo ofrece sincronizar, buscar de nuevo u ocultar.** Es a propósito. **Buscar de nuevo** trae lo nuevo de tus agentes; para elegir tú cada archivo, usa **Traer otro archivo** en el botón derecho de una subcarpeta, como Claude, o en Configuración › Agentes › Carpeta Agentes. Y no se mueve ni cambia de nombre porque por dentro se guarda como `AGENTS`, y con otro nombre o en otro lugar Tisty y tus agentes dejarían de encontrarla. Mira [Agentes se queda arriba](#agentes-se-queda-arriba).

**Quise mover o renombrar Agentes y no me dejó.** Tisty te dijo «Agentes se queda arriba, con su nombre y su robot, para seguir cuidando las reglas de tus agentes. Su color y su lugar entre tus carpetas sí los cambias». Es a propósito: mira [Agentes se queda arriba](#agentes-se-queda-arriba).

**Toda la carpeta dejó de escribir.** Revisa que Agentes tenga su robot coral: si está gris, no sincroniza en este equipo, y **Sincronizar** la pone otra vez al día. Si un agente no recibe nada, mira su interruptor en Configuración › Agentes › Carpeta Agentes.

**Codex no lee mis reglas.** Si tienes `~/.codex/AGENTS.override.md`, Codex lee solo ese archivo, y un documento con `agent: codex/rule/AGENTS.md` se escribe en `AGENTS.md`, que Codex no lee mientras exista el otro. Si creaste el override después de la primera vez, [tráelo a Agentes](#el-agentsoverridemd-de-codex).

**Gemini no lee mis reglas.** Si su ficha dice **En pausa**, falta en `~/.gemini/GEMINI.md` la línea `@tisty-rules.md` que Tisty había puesto: pulsa **Volver a ponerla**, en la ficha o en Configuración › Agentes › Carpeta Agentes. Tisty no la vuelve a poner por su cuenta, porque borrarla es tu decisión. Mira [Gemini en pausa](#gemini-en-pausa).

**No aparece Traer el cambio.** El documento es de Global, o lo estás editando: pulsa **Listo** y vuelve a mirar.

**«El archivo cambió mientras Tisty lo escribía, así que no lo tocó.»** Otro programa escribió el archivo al mismo tiempo. Tisty no pisa nada: vuelve a mirar la ficha y elige.

**Recuperé un respaldo y dice Cambiado por fuera.** Es lo esperado: [un respaldo recuperado no cambia tu documento](#respaldos). Elige **Traer el cambio** para quedarte con él.

## Ver también

- [Privacidad](Privacidad#tus-agentes-en-la-carpeta-agentes): qué lee Agentes, cuándo, y qué se queda en este equipo.
- [[Agentes y MCP|Agentes-y-MCP]]: conectar un agente a Tisty.
- [Versiones anteriores](Documentos#versiones-anteriores): el historial de un documento.
- [Carpetas](Documentos#carpetas): filtrar, archivar y desarchivar.
- [Propiedades](https://markdown.tisty.app/es/propiedades), en la guía de Markdown de Tisty: la línea `agent:` y sus marcas.
