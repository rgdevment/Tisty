# Agentes: las reglas, skills y memoria de tus agentes

Uso más de un agente de IA, y cada uno lee sus reglas de un archivo distinto, en una carpeta distinta: `CLAUDE.md`, `AGENTS.md`, `copilot-instructions.md`, `GEMINI.md`. Cambiar una regla en todos era abrir cuatro archivos, y tarde o temprano uno quedaba atrás. La carpeta **Agentes** junta esos archivos en Tisty, como documentos, para que los leas y los ordenes. Y si quieres, Tisty escribe por debajo el archivo que lee cada agente.

> [!TIP]
> **En corto:** la carpeta **Agentes** guarda como documentos las reglas, las skills y la memoria de Claude Code, Codex, GitHub Copilot CLI, Gemini CLI y Cursor. Hay dos palabras, y cada una significa una sola cosa. **Traer** copia los archivos de tus agentes a Tisty, para leerlos y ordenarlos, y no escribe nada en ellos. **Sincronizar** hace que, desde ese momento, lo que cambias en Tisty llegue a esos archivos, y que Tisty te avise si alguien los cambia por fuera. Sincronizando, lo que escribes en **Global** llega a todos y lo que escribes en la carpeta de un agente, solo a él; antes de sobrescribir un archivo Tisty guarda lo que tenía, y un cambio hecho por fuera nunca se pisa. Y un agente no escribe tus reglas: te propone el cambio, y solo tú lo apruebas.

![La primera vez de Agentes en español: el aviso Tus agentes, en un solo lugar, con la X para cerrar arriba a la derecha, la línea Tisty busca las reglas, skills y memoria de tus agentes y las trae aquí como documentos, para leerlas y ordenarlas. Traer solo las copia; Traer y sincronizar además hace que lo que cambies en Agentes llegue solo a tus agentes., sus tres líneas con candado, flechas y copias (Queda en tu Tisty…, Si sincronizas, llega solo a Claude, Codex, Copilot, Gemini., Antes de cambiar un archivo de un agente, Tisty guarda un respaldo en este equipo.), y debajo Encontré esto. Elige qué traer:, la línea Gemini: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta., y tres filas plegadas, Claude · 2 reglas · 29 skills · 49 memorias, Codex · 1 regla · 4 skills y Copilot · 1 regla; abajo a la derecha, solo dos botones, Traer y Traer y sincronizar, este destacado](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-01-primera-vez.png)

## Qué es y cuándo te sirve

**Agentes** es una carpeta de tu Biblioteca, arriba de todo, con el icono de un robot. Está en tu barra lateral desde el primer día, en gris, hasta que traes algo. Cada documento que tiene adentro es un archivo de un agente: una regla, una skill o su memoria. Lo lees y lo editas en Tisty, con su buscador y sus versiones anteriores, pero no se ve como un documento más: se abre en su propia [ficha](#la-ficha-de-un-documento-de-agente), que te dice quién lo lee, dónde se escribe y cuántos respaldos tiene, con el texto del agente tal como lo escribiste. Si la sincronizas, Tisty escribe el archivo que ese agente lee, byte a byte.

Te sirve cuando:

- **Quieres leer y ordenar las reglas que ya tienes.** Con **Traer**, tu `CLAUDE.md`, tus skills y tus reglas de Copilot entran como documentos, con títulos y búsqueda, en vez de archivos planos en carpetas ocultas. Tus agentes siguen leyendo sus archivos como estaban.
- **Usas varios agentes y quieres las mismas reglas en todos.** Con **Sincronizar**, escribes «Responde en español» una vez en **Global**, y llega al `CLAUDE.md` de Claude Code, al `AGENTS.md` de Codex, al `copilot-instructions.md` de Copilot y a las reglas de Gemini.
- **Trabajas en más de un equipo.** Escribes la regla una vez, y cada equipo elige si la sincroniza y qué aplica a sus agentes.
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

- Jaime usa solo Claude Code y quiere leer sus reglas y sus skills con calma, sin que nada cambie en `~/.claude`. Elige **Traer**. Sus documentos quedan en **Agentes › Claude**, y Claude Code sigue leyendo su `CLAUDE.md` como siempre. Si edita uno de esos documentos, el cambio queda en Tisty y nada más.
- Yisset usa Claude Code y Codex, y quiere que los dos sepan que sus ramas van con la forma `tipo/nombre`. Elige **Traer y sincronizar**, escribe esa regla una vez en **Agentes › Global**, y Tisty la escribe al principio de su `CLAUDE.md` y de su `AGENTS.md`.

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
| Codex | `~/.agents/skills`, que comparte con Copilot y Gemini | hasta seis carpetas más abajo, como `skills/trabajo/pr/` |
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
| **propio de Tisty** | Un archivo que dejó Tisty o uno temporal, como `tisty.json` o uno que termina en `.tmp` |
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

El aviso dice «No encontré archivos de agentes en este equipo. Igual puedes sincronizar: Agentes se llena a medida que escribes tus reglas.». Debajo, cada agente que encontró sin archivos tiene su línea, como «Codex: todavía sin reglas, skills ni memoria. Recibe lo que escribas en Global o en su carpeta.». **Traer** queda desactivado, porque no hay nada que traer, y **Traer y sincronizar** sigue disponible: pulsarlo deja que este equipo sincronice y crea igual la carpeta **Agentes**, con **Global** y una subcarpeta vacía por cada agente que encontró, como **Codex** y **Gemini**. El aviso dice «No entró nada nuevo. Agentes ya sincroniza en este equipo: lo que escribas ahí llega a sus agentes.», y en la barra Agentes queda con su robot coral. Entrar en ella no vuelve a abrir el aviso.

Desde ahí tienes dos caminos:

- **Escribir tu primera regla.** Botón derecho sobre **Global** o sobre la carpeta de un agente, y **Documento nuevo**: el documento ya empieza como una regla de ese lugar, con su línea `agent:` puesta, como cuenta [Un documento nuevo ya es una regla](#un-documento-nuevo-ya-es-una-regla). Tisty escribe el archivo apenas escribes algo en él.
- **Traer lo que aparezca después.** Cuando tu agente cree sus archivos, por ejemplo después de usarlo por primera vez, tráelos con [Traer otro archivo](#traer-otro-archivo).

## Traer otro archivo

La primera vez no tiene que ser la única. Si instalas un agente nuevo, si creas un archivo de reglas que antes no existía o si dejaste algo sin marcar, lo traes después:

- **Desde la barra lateral:** botón derecho sobre una subcarpeta de Agentes, como **Agentes › Claude**, y **Traer otro archivo**. El botón derecho de Agentes misma no lo ofrece: ahí está [Buscar de nuevo](#buscar-de-nuevo), que hace lo mismo y además marca lo nuevo por ti. En el estado 1 todavía no hay nada traído, y un clic en Agentes abre la primera vez.
- **Desde el menú de la Biblioteca:** con Agentes o una de sus subcarpetas abierta, el botón **+** junto al título **Biblioteca** ofrece **Traer otro archivo**, entre las opciones para crear e importar.
- **Desde la paleta de comandos:** con Agentes o una de sus subcarpetas abierta, escribe «traer» y elige **Traer otro archivo**, en el grupo **Crear**.
- **Desde Configuración › Agentes › Carpeta Agentes:** en la tarjeta **Agentes**, el botón **Traer otro archivo**.

**Traer otro archivo no cambia si Agentes sincroniza.** Traída sin sincronizar, el archivo nuevo entra como documento y el archivo de tu agente queda como está; sincronizando, desde ese momento Tisty también lo mantiene al día.

Se abre **Traer otro archivo**, con «Estos son los archivos de agentes de este equipo que todavía no están en Agentes. Elige qué traer:» y la misma lista de la primera vez, con sus mismas reglas: una skill es una sola fila que entra entera, y lo oculto o lo que mantiene el propio agente no aparece. Con una diferencia: **no viene nada marcado**, ni siquiera las reglas. Marca lo que quieres y pulsa **Traer**: el botón no se activa hasta que marcas algo. **Ahora no** lo cierra sin traer nada. Lo que ya tiene documento dice **ya está en Agentes**, y si no queda nada por traer, el aviso dice «Todos los archivos de agentes de este equipo ya están en Agentes.».

**Un ejemplo.** Un día escribes a mano en tu editor una regla nueva para Claude Code, `~/.claude/rules/pruebas.md`:

```markdown title="~/.claude/rules/pruebas.md"
# Pruebas

- Corre los tests antes de cada commit.
```

1. Botón derecho sobre **Agentes › Claude** y **Traer otro archivo**.
2. Abre la fila de Claude: `CLAUDE.md` aparece atenuado, con **ya está en Agentes**, y `rules/pruebas.md`, sin marcar, como **Regla**.
3. Márcalo y pulsa **Traer**. Si Agentes sincroniza, el aviso dice «Entró 1 documento a Agentes, y desde ahora Tisty lo mantiene al día.»; si está traída sin sincronizar, «Entró 1 documento a Agentes. Los archivos de tus agentes quedan como están hasta que elijas Sincronizar.». **Listo** lo cierra.
4. En **Agentes › Claude** aparece **Pruebas**, con el icono de un libro. Desde ahora lo lees y lo cambias en Tisty, como cualquier regla.

Si este equipo todavía no te dejó leer los archivos de tus agentes, por ejemplo porque Agentes llegó traída desde [otro equipo](#en-varios-equipos), **Traer otro archivo** abre el aviso de la primera vez, con sus tres líneas, lo que encontró ya marcado y sus dos botones, **Traer** y **Traer y sincronizar**.

**Traer no toca nada.** Ni los archivos de tus agentes ni la cuenta que Tisty lleva de lo que escribió: un archivo que solo trajiste nunca queda anotado como escrito por Tisty, así que, si después sacas su documento de Agentes, Tisty no te ofrece quitarlo con [¿Quitar también el archivo?](#sacar-una-regla-y-su-archivo). Si más adelante eliges **Sincronizar**, un archivo que sigue tal cual lo trajiste se toma como está, y lo que se le suma, como una regla de Global, se escribe encima con su respaldo antes.

### Buscar de nuevo

**Buscar de nuevo** está en el botón derecho de Agentes misma, en el estado 2, traída sin sincronizar, y en el 3, sincronizando. Vuelve a buscar en este equipo tus agentes y sus archivos, como la primera vez: si instalaste uno nuevo, le crea su subcarpeta, y abre **Traer otro archivo** con **solo lo nuevo ya marcado**. Lo que ya está en Agentes aparece atenuado con **ya está en Agentes**, y la memoria del agente viene sin marcar, como siempre. Revisa las casillas y pulsa **Traer**.

**Un ejemplo.** Marisol tenía Agentes sincronizando con Claude y Codex. Una tarde instala Gemini CLI, le pide que recuerde el nombre de su perra, y escribe una skill nueva para Claude en `~/.claude/skills/informe/`. Botón derecho sobre **Agentes** y **Buscar de nuevo**:

1. En la barra aparece **Agentes › Gemini**, con su brújula verde, vacía.
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

**Un ejemplo.** Tienes `~/.codex/AGENTS.md` con tus reglas de Codex, y la primera vez lo dejaste sin marcar. Después escribes una regla en **Global**, que también va a ese archivo. Tisty no sabe qué parte es tuya, así que no lo pisa: te pregunta **Existe en los dos lados**, al entrar en Agentes o con **Revisar** en Configuración › Agentes › Carpeta Agentes. Como nunca trajo ese archivo, el aviso suma «Tisty nunca trajo este archivo. Si lo traes a Agentes, queda con lo que dice y con lo de Tisty agregado arriba.» y el botón **Traer a Agentes**.

Con **Traer a Agentes**, el archivo entra como un documento en **Agentes › Codex**, con sus palabras, y Tisty lo vuelve a escribir con tu regla de Global arriba. Lo que tenía antes queda en sus [respaldos](#respaldos):

```markdown title="~/.codex/AGENTS.md, después de Traer a Agentes"
# Para todos mis agentes

- Antes de borrar algo, pregunta.

# Reglas de Codex

- Usa pnpm, nunca npm.
```

**Traer otro archivo** hace lo mismo: ese `AGENTS.md` aparece en su lista, sin marcar, y al traerlo queda igual, con Global arriba y su respaldo.

### El `AGENTS.override.md` de Codex

Codex lee `~/.codex/AGENTS.override.md` en vez de `AGENTS.md` cuando existe, así que tus reglas de Global van a ese archivo mientras exista. Si lo creas después de la primera vez y tienes reglas en Global, Tisty te pregunta **Existe en los dos lados** con **Traer a Agentes**, como con cualquier archivo que nunca trajo. Sin reglas en Global no pregunta nada: tráelo con **Traer otro archivo**. Al traerlo:

- `AGENTS.override.md` queda con tu regla de Global arriba y lo que escribiste debajo, y desde ahí Codex recibe Global ahí;
- `AGENTS.md` queda solo con tus reglas de Codex, sin Global. Codex no lo lee mientras exista el override.

## Cómo queda la carpeta

Después de traer, la Biblioteca tiene una carpeta **Agentes** con una subcarpeta por agente y una **Global**. En la barra lateral, cada documento se ve con su nombre y un icono que dice qué es:

```text
Agentes
├── Global
├── Claude
│   ├── [libro]    Cómo trabajo
│   ├── [libro]    Estilo de código
│   ├── [pieza]    pr                3 archivos
│   └── [cerebro]  MEMORY.md
├── Codex
│   └── [libro]    Reglas de Codex
└── Gemini
    └── [cerebro]  GEMINI.md
```

- **El icono de cada subcarpeta**: **Global** lleva un globo azul, y cada agente un símbolo propio, que no imita su logo, en el mismo color de su etiqueta en la ficha: un globo de conversación naranja para Claude, una terminal verde azulado para Codex, unas llaves de código índigo para Copilot, una brújula verde para Gemini y un puntero café para Cursor. Los mismos iconos aparecen en la lista de la primera vez y en **Configuración › Agentes › Carpeta Agentes**. Una subcarpeta a la que le pusiste un icono tuyo lo conserva.
- **El icono de cada documento**: un libro para una **regla**, una pieza de rompecabezas para una **skill** y un cerebro para la **memoria del agente**.
- **El nombre**: el primer título del documento, como «Cómo trabajo» en un `CLAUDE.md` que empieza con `# Cómo trabajo`. Una skill toma el `name` de su `SKILL.md`. Un script, o un archivo sin títulos, se llama como el archivo: `MEMORY.md`, `extraer.py`.
- **La ruta**: pasa el puntero sobre una fila y aparece, con el agente delante: `claude · CLAUDE.md`, `claude · rules/estilo.md`.
- **Una skill es una sola fila**, con cuántos archivos tiene a la derecha, como **pr** y **3 archivos**. Un clic la abre y muestra sus archivos por su ruta dentro de la skill: `SKILL.md`, `scripts/abrir-pr.sh`. Así, una skill de 200 archivos ocupa una línea hasta que la abres. Una skill de un solo archivo se ve como un documento más.

Las subcarpetas son para que encuentres las cosas: **lo que decide adónde va un documento es su línea `agent:`**, no la carpeta en que está. Puedes mover un documento entre subcarpetas o crear las tuyas dentro de Agentes, y se sigue escribiendo en el mismo archivo.

**Una subcarpeta por cada agente de este equipo, aunque esté vacía.** Cada agente que Tisty encuentra aquí tiene su carpeta en Agentes: **Global**, **Claude**, **Codex**, **Copilot**, **Gemini** y **Cursor**, según cuáles tengas. Tisty la crea al traer, al [buscar de nuevo](#buscar-de-nuevo) o cuando aparece un agente nuevo, para que tengas dónde escribirle aunque todavía no tenga ningún archivo. Crear una carpeta no toca ningún archivo de tus agentes. Si borras una de esas subcarpetas, por ejemplo **Cursor** porque no lo usas, Tisty no la vuelve a crear. Y si dos equipos la crean a la vez antes de sincronizarse, queda una sola.

**La barra no dice nada junto a Agentes**: ninguna palabra, ni cuántos archivos esperan. Lo que espera tu decisión está en tres lugares:

| Dónde | Qué ves |
| :-- | :-- |
| **La ficha de cada documento** | Su estado, siempre, en la etiqueta de color: **Cambiado por fuera**, **Cambiado en los dos lados**, **Escrito por un Tisty anterior** o **En espera**; la etiqueta **Propuesta**, si un agente te propone un cambio; o un aviso ámbar dentro de la ficha, cuando el documento [viene de otro equipo](#en-varios-equipos) o su texto [espera tu aprobación](#por-aprobar). La tabla de [cada estado](#cada-estado) dice qué hacer con cada uno |
| **Configuración › Agentes › Carpeta Agentes**, en la tarjeta **Agentes** | Los archivos que piden tu decisión: «1 archivo dice algo distinto en Tisty y en el agente.», o «1 archivo ya no es de ninguna regla de Agentes.», cada uno con **Revisar**; y bajo **Sincronizar Gemini**, si falta [su línea](#gemini-en-pausa) |
| **Al entrar en Agentes** | Si Agentes sincroniza y un archivo existe en los dos lados, ya no es de ninguna regla o perdió la línea de Gemini, Tisty te pregunta por él apenas entras, uno por uno |

![La barra lateral con la carpeta Agentes abierta y sincronizando, con el robot en coral: dentro, Global con su globo azul y Claude con su globo de conversación naranja; en Claude, la regla Reglas, elegida, con el icono de un libro, y debajo las skills, cada una con el icono de una pieza y cuántos archivos lleva](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/help/agents/es-02-barra.png)

### El botón derecho de una subcarpeta

**Global**, **Claude**, **Codex** y cualquier carpeta dentro de Agentes tienen un menú solo de agentes. Sobre **Agentes › Claude**, mientras Agentes sincroniza, ofrece esto, y nada más:

| Opción | Qué hace |
| :-- | :-- |
| **Documento nuevo** | Crea un documento en esa carpeta que ya nace con su línea `agent:`: en **Claude**, una regla de Claude; en **Global**, una regla de Global. Mira [Un documento nuevo ya es una regla](#un-documento-nuevo-ya-es-una-regla) |
| **Traer otro archivo** | Abre la lista de los archivos de tus agentes que todavía no están en Agentes. Mira [Traer otro archivo](#traer-otro-archivo) |
| **Pausar la sincronización con Claude** | Deja de escribir los archivos de Claude en este equipo, como el botón del mismo nombre en la ficha. En pausa, dice **Reanudar la sincronización con Claude**. Mira [Pausar, dejar de sincronizar y vaciar](#pausar-dejar-de-sincronizar-y-vaciar) |
| **Ocultar de la barra** | La saca de la barra en este equipo. Si Agentes sincroniza, antes te pide dejar de sincronizar. Mira [Ocultarla de la barra](#ocultarla-de-la-barra) |
| **Eliminar** | Pregunta antes; sus documentos pasan a **Sin carpeta**, y Tisty te pregunta por los archivos que armaban, con su respaldo. Mira [Eliminar una subcarpeta](#eliminar-una-subcarpeta) |

- **Global no tiene pausa**: no es de un agente, y lo que tiene llega a todos. Para pausar uno, usa la subcarpeta de ese agente.
- **La pausa solo aparece mientras Agentes sincroniza.** Traída sin sincronizar, Tisty no escribe ningún archivo, y no hay nada que pausar. Tampoco aparece para un agente que no está instalado en este equipo, aunque su carpeta haya llegado desde otro.
- **Una carpeta dentro de una subcarpeta**, como **Agentes › Claude › Pruebas**, tiene el mismo menú que **Claude**, con la pausa de Claude.
- **No hay Diagrama nuevo, Carpeta nueva, Cambiar nombre, Mover a, Importar ni Desempaquetar.** Esas son acciones de tus carpetas; dentro de Agentes, cada carpeta es de un agente, y su nombre es el que Tisty usa para saber a quién le escribe.
- **Una subcarpeta archivada** muestra además **Desarchivar**, para que vuelva.

**Un ejemplo.** Jaime prueba a mano unas reglas nuevas de Claude esta tarde, pero quiere que Codex siga recibiendo lo de Global. Botón derecho sobre **Agentes › Claude** y **Pausar la sincronización con Claude**. Codex sigue sincronizando; Claude queda como lo dejó la última escritura. Al terminar, botón derecho otra vez y **Reanudar la sincronización con Claude**.

### La línea `agent:`

Cada documento de Agentes empieza con sus propiedades, y una de ellas dice qué archivo es:

```markdown
---
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

El valor tiene tres partes, separadas por `/`:

- **El nivel**: `global`, o un agente: `claude`, `codex`, `copilot`, `gemini` o `cursor`.
- **El tipo**: `rule`, `skill` o `memory`.
- **El nombre**: la ruta del archivo dentro de la carpeta del agente, como `CLAUDE.md`, `rules/estilo.md` o `pr/SKILL.md`.

A veces lleva marcas más, que Tisty pone solo al traer un archivo para escribirlo igual que estaba: `crlf` (líneas que terminan como en Windows), `bom` (una marca invisible al principio), `as-is` (finales de línea mezclados) y `open-end` (el archivo no termina en salto de línea). Por ejemplo, `agent: [copilot/rule/copilot-instructions.md, crlf, bom]`. La [guía de propiedades](https://markdown.tisty.app/es/propiedades) explica el resto.

La página no muestra esa línea: el documento se abre en su [ficha](#la-ficha-de-un-documento-de-agente).

## Global o un agente

**Un documento de Global llega a cada agente que Tisty encuentra en este equipo.** Uno de la carpeta de un agente llega solo a ese agente.

### Una regla en Global

Escribe en **Agentes › Global** un documento con estas propiedades:

```markdown
---
agent: global/rule/para-todos.md
title: global · para-todos.md
---
# Para todos mis agentes

- Antes de borrar algo, pregunta.
- Nunca subas cambios sin que te lo pida.
```

Esa regla no se escribe en un archivo propio: va **al principio del archivo principal de cada agente**, antes de las reglas de ese agente. Si en este equipo tienes Claude Code, Codex y Gemini, Tisty escribe:

- `~/.claude/CLAUDE.md`: la regla de Global, y después lo que dice tu documento de ese archivo, **Cómo trabajo** en el ejemplo.
- `~/.codex/AGENTS.md`: la regla de Global, y después tu documento de `AGENTS.md`, si lo tienes.
- `~/.gemini/tisty-rules.md`: la regla de Global, y después tu documento `gemini/rule/tisty-rules.md`, si lo tienes.

Así queda el `CLAUDE.md` de Claude Code:

```markdown
# Para todos mis agentes

- Antes de borrar algo, pregunta.
- Nunca subas cambios sin que te lo pida.

# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

El cuadro de datos de cada uno de esos documentos lo dice: en **Cómo trabajo**, **Lo lee** Claude y **Se escribe en** `~/.claude/CLAUDE.md`, con «junto con otro documento» debajo; en **Para todos mis agentes**, una fila por agente, **Archivo de Claude**, **Archivo de Codex** y **Archivo de Gemini**. Cursor no recibe reglas: las suyas viven en sus ajustes, no en un archivo.

> [!IMPORTANT]
> **Trae el archivo principal de cada agente.** Si `~/.codex/AGENTS.md` ya existe y nunca lo trajiste, Tisty no sabe qué parte es tuya: al escribir una regla de Global, la ficha marca ese archivo **Cambiado en los dos lados**, y **Sobrescribir con Tisty** lo reemplazaría entero por tus reglas de Global. En vez de eso, usa **Traer a Agentes**: lo trae con sus palabras y escribe Global delante, con el original como respaldo n.º 1. Lo cuenta [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste).

### Una regla solo para Claude

Claude Code también lee cada archivo de `~/.claude/rules`. Para una regla que solo Claude debe leer, escribe un documento con `agent: claude/rule/rules/pruebas.md`, en cualquier lugar de Agentes, y Tisty escribe `~/.claude/rules/pruebas.md`. Copilot hace lo mismo con su carpeta `instructions`: `agent: copilot/rule/instructions/rust.instructions.md`.

Un nombre que el agente no lee no se escribe en ninguna parte: `agent: claude/rule/notas.md` deja la ficha diciendo «Ningún agente de este equipo lee este documento todavía.».

### Un documento nuevo ya es una regla

No tienes que escribir la línea `agent:` a mano. Un documento que creas con **Documento nuevo** dentro de **Global** o de la carpeta de un agente empieza como una regla de ese lugar: sus propiedades ya traen su línea `agent:`, y tú solo escribes la regla.

| Dónde lo creas | Con qué línea empieza |
| :-- | :-- |
| **Global** | `agent: global/rule/rule.md`; el siguiente, `rule-2.md`, y así |
| **Claude** | `agent: claude/rule/CLAUDE.md` mientras ningún documento lo tenga; después, `claude/rule/rules/rule.md`, `rules/rule-2.md`… |
| **Codex** | `agent: codex/rule/AGENTS.md` mientras ningún documento lo tenga |
| **Copilot** | `agent: copilot/rule/copilot-instructions.md` mientras ningún documento lo tenga; después, `copilot/rule/instructions/rule.instructions.md`, `rule-2.instructions.md`… |
| **Gemini** | `agent: gemini/rule/tisty-rules.md` mientras ningún documento lo tenga |
| **Cursor**, Agentes misma o una carpeta tuya dentro de Agentes | Ninguna: es un documento común |

**Codex y Gemini leen un solo archivo de reglas.** Cuando un documento ya lo tiene, **Documento nuevo** en su carpeta no crea otro: abre ese mismo documento, para que sumes ahí lo nuevo. Lo que es para todos va en **Global**.

**Dos reglas Global con el mismo nombre.** Si en la casa y en el trabajo creaste una regla en **Global** antes de que los dos equipos se sincronizaran, las dos nacieron como `rule.md`. Al encontrarse, Tisty le deja el nombre a la más antigua y le da a la otra el siguiente libre, `rule-2.md`, sin tocar su texto: las dos llegan a tus agentes. En el equipo donde la otra llegó de afuera, espera su **Aplicar aquí** como cualquier documento de otro equipo. **Cursor** no tiene un archivo de reglas propio, así que un documento nuevo en su carpeta también empieza común; sus skills las traes con **Traer otro archivo**.

**Un ejemplo.** Marisol tiene Agentes sincronizando y acaba de instalar Gemini CLI. En la barra aparece **Agentes › Gemini**, con su brújula verde, vacía.

1. Botón derecho sobre **Gemini** y **Documento nuevo**. El documento se abre en su ficha, con las etiquetas **Regla** y **Gemini**, todavía sin texto.
2. Pulsa **Editar**, escribe «Responde en español.» y pulsa **Listo**.
3. Tisty escribe `~/.gemini/tisty-rules.md` con esa línea, delante de ella lo que tengas en **Global**, y agrega la línea `@tisty-rules.md` al final de `~/.gemini/GEMINI.md`, para que Gemini lo lea. La ficha pasa a **Al día**, y en la barra el documento se llama `tisty-rules.md`, porque no tiene un título; si le escribes `# Reglas de Gemini` arriba, se llama así.

Sus propiedades, que la ficha no muestra, quedaron así:

```markdown
---
agent: gemini/rule/tisty-rules.md
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
- en `~/.agents/skills/pdf/`, una sola copia que leen Codex, Copilot y Gemini.

Las propiedades propias del `SKILL.md` no ocupan el texto: su `name` es el nombre de la ficha, su `description` va debajo en una línea discreta, y el resto, como `disable-model-invocation: true`, va en filas chicas del cuadro de datos. El script se guarda en un bloque de código y se escribe tal cual, sin el bloque: la fila **Formato** del cuadro de datos dice el tipo del script, como SH · LF, y en un documento Markdown, Markdown · LF.

## Qué archivo lee cada agente

`~` es tu carpeta de usuario: `C:\Users\tú` en Windows y `/Users/tú` en macOS. Tisty solo trabaja con un agente cuya carpeta existe en este equipo.

| Agente | Su carpeta | La mueve | Reglas que escribe Tisty | Skills | Memoria, solo para leer |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Claude Code** | `~/.claude` | `CLAUDE_CONFIG_DIR` | `CLAUDE.md` y lo que va en `rules/` | `skills/` en su carpeta | `projects/*/memory/`, la memoria de cada proyecto |
| **Codex** | `~/.codex` | `CODEX_HOME` | `AGENTS.md`, y Global en `AGENTS.override.md` cuando ese archivo existe | `~/.agents/skills/` | — |
| **GitHub Copilot CLI** | `~/.copilot` | `COPILOT_HOME` | `copilot-instructions.md` y lo que va en `instructions/` | `skills/` en su carpeta; Global en `~/.agents/skills/` | — |
| **Gemini CLI** | `~/.gemini` | `GEMINI_CLI_HOME`, la carpeta que contiene `.gemini` | `tisty-rules.md`, y la línea `@tisty-rules.md` en `GEMINI.md` | `skills/` en su carpeta; Global en `~/.agents/skills/` | `GEMINI.md` |
| **Cursor** | `~/.cursor` | — | Ninguna: sus reglas viven en sus ajustes | `skills/` en su carpeta | — |

Lo que hace Tisty con cada uno:

- **Claude Code.** Trae `CLAUDE.md`, cada archivo de `rules/`, cada skill y, si la marcas, la memoria de cada proyecto, solo la carpeta `memory` y nunca las conversaciones guardadas. Escribe `CLAUDE.md` y `rules/`, y las skills.
- **Codex.** Codex lee `AGENTS.override.md` en vez de `AGENTS.md` cuando el primero existe, así que tus reglas de Global van a ese archivo mientras exista. Si lo creas después de la primera vez, [tráelo](#el-agentsoverridemd-de-codex) para que Global vaya ahí sin pisar lo tuyo. Sus skills van a `~/.agents/skills`, que `CODEX_HOME` no mueve.
- **GitHub Copilot CLI.** Trae y escribe `copilot-instructions.md` y cada archivo de `instructions/`, como `rust.instructions.md`.
- **Gemini CLI.** Gemini escribe su propio `GEMINI.md` cuando le pides que recuerde algo, así que ese archivo es suyo: Tisty lo puede traer como memoria y nunca lo sobrescribe. Tus reglas van a un archivo aparte, `~/.gemini/tisty-rules.md`, y Tisty agrega **una sola línea** al final de `GEMINI.md`, `@tisty-rules.md`, para que Gemini lo lea. No toca nada más de ese archivo, y si la línea ya está, no la repite. Si después la borras, Tisty no la vuelve a poner: Gemini queda [en pausa](#gemini-en-pausa) hasta que tú lo decides.
- **Cursor.** Solo skills, propias o de Global.

**Las variables que mueven una carpeta** se leen del entorno con que arrancó Tisty. Si usas `CLAUDE_CONFIG_DIR` para guardar Claude Code en otra parte, defínela como variable de tu usuario y vuelve a abrir Tisty, para que la vea.

## La ficha de un documento de agente

Un documento de Agentes no se ve como un documento más. Se abre **para leer**, en una sola ficha: arriba, qué es y dónde va; en el medio, el texto del agente; abajo, lo que puedes hacer. No hay columna al lado: todo lo útil está en la ficha.

![La ficha de la regla Reglas en Agentes / Claude: las etiquetas Regla, Claude y Al día; el cuadro de datos con Lo lee Claude, Se escribe en con su botón de copiar, Tamaño, Respaldos con Ver, Formato Markdown · LF, Última escritura, Original guardado y Sincroniza en este equipo Sí; debajo, el texto de la regla, y al pie Editar, Respaldos, Historial, Copiar enlace y Pausar la sincronización con Claude](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/screens/app/es-agentes-regla-light.webp)

**Arriba de todo, el camino de carpetas**: «Agentes / Claude». Un clic en una carpeta la abre.

**La cabecera:**

- **Las etiquetas**: el tipo (**Regla**, **Skill** o **Memoria del agente**), el nivel (**Global** o el agente) y el estado, con su color.
- **El nombre**, el mismo que en la barra lateral. En una skill, su `description` va debajo, en una línea discreta.

**El cuadro de datos del archivo**, el único lugar donde aparecen:

| Fila | Qué dice | Ejemplo |
| :-- | :-- | :-- |
| **Lo lee** | Qué agentes de este equipo leen el archivo | Claude, Codex |
| **Se escribe en** | La ruta completa del archivo, con un botón para copiarla. Un documento de Global tiene una por agente: **Archivo de Claude**, **Archivo de Codex**. Debajo, si el archivo se arma con otros documentos, «junto con otro documento» | `~/.claude/CLAUDE.md` |
| **Tamaño** | Cuánto pesa el archivo que lee el agente, entero, aunque se arme con varios documentos | 12,7 kB |
| **Respaldos** | Cuántos guarda este equipo, y **Ver** para abrirlos | 3 respaldos · Ver, o Ninguno todavía |
| **Formato** | Markdown, o el tipo del script; cómo terminan sus líneas, y las marcas que conserva | Markdown · LF, o PY · CRLF · BOM |
| **Última escritura** | Cuándo cambió el archivo por última vez | hoy, 19:37 |
| **Original guardado** | Si ya está el respaldo n.º 1, el archivo de antes de que Tisty lo tocara | Sí, el de antes de la primera escritura de Tisty |
| **Sincroniza en este equipo** | Si Tisty lo mantiene al día aquí | Sí; Sí, salvo Codex (en pausa); Tisty solo lo lee; No, Agentes no se sincroniza; No: espera «Aplicar aquí»; No: su texto espera tu aprobación. Con una propuesta esperando, se suma «un cambio propuesto por un agente espera tu aprobación» |
| **Compartido** | Solo si el documento tiene un enlace compartido activo: si se puede leer o editar, y **Ver** muestra los enlaces con su botón para copiarlos y las acciones de siempre, como **Dejar de compartir**. Lo cuenta [Compartir](Documentos#compartir) | Solo leer · Ver |
| **Las propiedades del archivo** | Las otras claves de su encabezado, en filas chicas de clave y valor. Si son demasiadas o muy largas para leerlas como campos, van tal como están escritas | `disable-model-invocation` true; `applyTo` `**/*.rs` |

**El tope de Codex.** Codex lee su `AGENTS.md` hasta 32 KiB y deja sin leer lo que sigue. Cuando el archivo se acerca a ese tope, la fila **Tamaño** lo dice en ámbar: «29 kB · cerca de los 32 KiB que lee Codex». Si lo pasa, lo dice en rojo: «33,8 kB · pasa los 32 KiB que lee Codex: lo demás no lo lee». Es buen momento para pasar las reglas largas a una skill.

**El texto del agente**, dentro de la misma ficha y sin hoja aparte. Es lo que escribiste, tal cual, sin secciones ni plantilla impuestas: Tisty lo dibuja con encabezados compactos, a todo el ancho de la ficha, y sin convertir su primera línea en un título. El encabezado del archivo no se repite ahí, porque ya está en el cuadro de datos. Lo que lee tu agente no cambia: el archivo sigue siendo el mismo, byte a byte.

**Los botones, al pie**, fijos abajo mientras la ficha entera se desplaza junto con el texto:

- **Editar** abre el editor de Tisty sobre el texto, con su encabezado y la barra del editor de siempre para títulos y listas, y **Listo** lo vuelve a cerrar.
- **Respaldos**, cuando el documento se escribe en un solo archivo y ese archivo tiene alguno. Abre la misma lista que **Ver** en el cuadro de datos. Un documento de **Global**, que se escribe en varios archivos, no lo tiene al pie: los respaldos de cada archivo se abren con **Ver**, en su fila.
- **Historial**: las versiones anteriores del documento, no del archivo. Lo cuenta [[Versiones anteriores|Documentos#versiones-anteriores]]. La comparación con lo **Aprobado** lee de ahí: si vacías el historial, o la versión aprobada pasa el plazo, la ficha te pide leer el texto entero antes de aprobar.
- **Historial** abre las [versiones anteriores](Documentos#versiones-anteriores) del documento.
- **Copiar enlace** copia su [enlace](Documentos#el-enlace-de-un-documento).
- **Pausar la sincronización con Claude**, cuando el documento se escribe en un solo archivo. Uno de Global tiene un botón por agente: **Pausar Claude**, **Pausar Codex**…

Un documento de memoria no tiene **Editar**: lo escribe el agente, y la ficha dice «Este archivo lo escribe el agente: Tisty lo muestra y lo trae, y nunca lo sobrescribe.».

**Un ejemplo.** Abres la regla **Horario laboral** de Codex, que va en `~/.codex/AGENTS.md` junto con tu regla de Global. Arriba ves «Agentes / Codex» y las etiquetas **Regla**, **Codex** y **Al día**. En el cuadro, «Se escribe en `~/.codex/AGENTS.md`», con «junto con otro documento» debajo, y «Tamaño 29 kB · cerca de los 32 KiB que lee Codex» en ámbar. Lees el texto, decides pasar la parte larga a una skill y pulsas **Editar** al pie.

### Cada estado

| Estado | Qué significa | Qué hacer |
| :-- | :-- | :-- |
| **Al día** | El archivo dice lo mismo que el documento | Nada |
| **Por escribir** | Cambiaste el documento y Tisty lo escribe en un instante | Nada |
| **Cambiado por fuera** | Alguien cambió el archivo fuera de Tisty, o lo borró | Elegir: [traer o sobrescribir](#un-cambio-por-fuera-traer-o-sobrescribir) |
| **Cambiado en los dos lados** | Cambiaron el archivo y el documento desde la última escritura | Elegir cuál conservar |
| **Escrito por un Tisty anterior** | Una versión anterior de Tisty escribió el archivo sin anotar de qué documentos venía, y hoy no dice lo mismo que ellos | **Ver el archivo**, y después **Sobrescribir con Tisty** si te quedas con el documento |
| **En espera** | Uno de los documentos que arman este archivo [cambió sin tu aprobación](#por-aprobar), o salió de Agentes por mano de un agente, así que el archivo queda como estaba. La ficha nombra ese documento | Aprobar ese documento, o volver a ponerlo y sacarlo tú |
| **En pausa** | Pausaste la sincronización de ese agente, o Agentes no sincroniza en este equipo; entonces la ficha suma «La carpeta Agentes no se sincroniza con tus agentes. Elige Sincronizar en la barra o en Configuración › Agentes.». En Gemini, también cuando falta en `GEMINI.md` la línea `@tisty-rules.md` que Tisty había puesto | **Reanudar** cuando quieras, o **Sincronizar** la carpeta; en Gemini, [**Volver a ponerla**](#gemini-en-pausa) |
| **Sin permiso en este equipo** | Este equipo todavía no te dejó leer los archivos de tus agentes, por ejemplo porque Agentes llegó traída desde otro equipo | **Sincronizar**, en la barra o en Configuración › Agentes › Carpeta Agentes, y elegir en el aviso de la primera vez |
| **Nada que escribir** | El documento quedó vacío, y Tisty nunca vacía un archivo de un agente | Escribir algo, o sacar el documento de Agentes |
| **El archivo es un enlace** | El archivo, o una carpeta en su camino, es un enlace simbólico; escribirlo reemplazaría el enlace | Cambiar el enlace por el archivo real |
| **El bloque de código está roto** | El bloque que guarda un script ya no está entero | Arreglar el bloque en el documento |
| **No se puede leer** | El archivo existe, pero Tisty no pudo leerlo | Revisar sus permisos |

Dos avisos más pueden aparecer bajo el cuadro de datos:

- **«Ningún agente de este equipo lee este documento todavía.»** El agente no está en este equipo, o el nombre no es un archivo que ese agente lea.
- **«Otro documento de Agentes ya escribe este archivo, así que este queda fuera.»** Dos documentos tienen la misma línea `agent:`: solo uno se escribe.

Y si el documento todavía no se aplica en este equipo, un aviso ámbar dice por qué, con el botón que lo resuelve: lo cuenta [En varios equipos](#en-varios-equipos).

## Un cambio por fuera: traer o sobrescribir

Puedes seguir editando los archivos de tus agentes con tu editor, o dejar que el agente los cambie. Tisty nunca pisa esos cambios. Esto pasa sincronizando: traída sin sincronizar, Tisty no escribe esos archivos ni te pide decidir nada, y lo que cambies ahí queda solo ahí.

**Un ejemplo.** Abres `~/.claude/CLAUDE.md` en tu editor y agregas «- Corre los tests antes de cada commit.». Si Agentes sincroniza, la próxima vez que Tisty mira (al iniciar, al volver a la ventana o al abrir el documento), la ficha de **Cómo trabajo**, el documento de ese `CLAUDE.md`, dice **Cambiado por fuera**, con un aviso:

- **Ver el archivo** muestra lo que el archivo tiene ahora.
- **Traer el cambio** escribe en el documento lo que dice el archivo. Lo que decía antes el documento queda en sus [versiones anteriores](Documentos#versiones-anteriores). Si alguien cambió el documento después de que se dibujó la ficha, Tisty no lo pisa: te pide leerlo de nuevo y volver a elegir.
- **Sobrescribir con Tisty** pregunta «¿Escribir este documento sobre `~/.claude/CLAUDE.md`? Lo que tiene ahora el archivo queda en un respaldo.» y, si dices que sí, vuelve a escribir el archivo desde el documento.

Mientras no eliges, Tisty no escribe ese archivo, aunque cambies el documento.

**Traer el cambio** solo aparece cuando el cambio se puede atribuir a este documento: el documento es de ese mismo agente, no de Global, y no lo estás editando. Si `CLAUDE.md` se arma con una regla de Global y tu documento de Claude, y el cambio cae en la parte de Global, solo queda **Sobrescribir con Tisty**: copia a mano en Global lo que quieras conservar y después sobrescribe.

**Si borraron el archivo**, la ficha dice **Cambiado por fuera** sin **Ver el archivo**: **Sobrescribir con Tisty** lo vuelve a crear.

### Cambiado en los dos lados

Si cambiaron el archivo y también el documento, la ficha dice **Cambiado en los dos lados**. Cuando el documento es el único que arma ese archivo, aparece **Quedarse con el archivo**, que pregunta «¿Dejar este documento como está ahora …? Lo que dice ahora queda en su historial.»; si no, solo **Sobrescribir con Tisty**.

Lo mismo puede pasar la primera vez que sincronizas, cuando ya tenías en Agentes un documento de un archivo que entretanto cambió. Entonces aparece **Existe en los dos lados**, con las líneas que difieren: en rojo, con `−`, lo que hay **En Tisty**; en verde, con `+`, lo que hay **En el archivo**. Eliges **Conservar el de Tisty**, **Conservar el archivo** o **Decidir después**, y lo que no conservas queda guardado: el archivo en sus [respaldos](#respaldos), el documento en sus versiones anteriores. Si el archivo se arma con varios documentos, solo puedes conservar el de Tisty o decidir después. Si Tisty nunca trajo ese archivo, el aviso suma **Traer a Agentes**, como cuenta [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste). Desde **Configuración › Agentes › Carpeta Agentes**, «1 archivo dice algo distinto en Tisty y en el agente.» y **Revisar** te traen de vuelta a ese aviso.

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

**Pausar un agente.** Mientras Agentes sincroniza, el lugar de siempre es **Configuración › Agentes › Carpeta Agentes**: en la tarjeta **Agentes**, apaga el interruptor **Sincronizar Claude**, que aparece solo mientras la carpeta sincroniza. También en la barra: botón derecho sobre **Agentes › Claude** y **Pausar la sincronización con Claude**; en pausa, el mismo menú dice **Reanudar la sincronización con Claude**. También desde la ficha de un documento, con los botones del pie: **Pausar la sincronización con Claude**, en un documento que se escribe en un solo archivo, o un botón por agente, **Pausar Claude**, **Pausar Codex**…, en uno de Global. En pausa, Tisty no abre ni escribe los archivos de ese agente: quedan como los dejó la última escritura, y tú sigues editando tus documentos. **Reanudar la sincronización con Claude** escribe lo que cambiaste, salvo un archivo que cambió por fuera mientras tanto, que queda marcado para que decidas.

**Dejar de sincronizar toda la carpeta.** Botón derecho sobre **Agentes** y **Dejar de sincronizar**, o el botón **Dejar de sincronizar** en su tarjeta de **Configuración › Agentes › Carpeta Agentes**. Pasa esto, y nada más:

- **Ningún archivo de tus agentes se toca.** Tus agentes siguen leyendo sus archivos tal como Tisty los dejó, y Tisty deja de escribirlos y de pedirte que decidas cuando cambian por fuera.
- **Ningún documento se borra.** Siguen en Agentes, y los lees y los editas como siempre; lo que cambias queda en Tisty.
- **Agentes pasa al estado 2.** Su robot vuelve al gris y, al abrirla, arriba aparece «No se sincroniza con tus agentes · Sincronizar». La ficha de cada documento dice «La carpeta Agentes no se sincroniza con tus agentes. Elige Sincronizar en la barra o en Configuración › Agentes.», y su cuadro de datos, en **Sincroniza en este equipo**, «No, Agentes no se sincroniza».

**Sincronizar otra vez.** Con Agentes en el estado 2, cualquiera de estos tres hace lo mismo: el enlace **Sincronizar** de la línea de arriba, **Sincronizar** en su botón derecho, o **Sincronizar** en su tarjeta de Configuración › Agentes › Carpeta Agentes. Si este equipo ya te había dejado leer los archivos de tus agentes, por ejemplo porque elegiste **Traer** o **Dejar de sincronizar** aquí mismo, sincroniza de una vez, sin el aviso de la primera vez. Si nunca lo hizo, como un [segundo equipo](#un-segundo-equipo-paso-a-paso), abre antes el aviso de la primera vez.

Al sincronizar, Tisty escribe lo que cambiaste en tus documentos mientras tanto, con su respaldo antes, salvo un archivo que cambió por fuera, que queda marcado para que decidas. Lo que encuentra distinto en los dos lados, o que ya no es de ninguna regla, te lo pregunta en ese mismo momento.

**La primera vez que sincronizas algo que solo habías traído**, un archivo que sigue tal cual lo trajiste se toma como está: no se pregunta nada por él. Si algo se le suma, como una regla de Global que va al principio de tu `CLAUDE.md`, Tisty lo escribe encima, con su respaldo antes. Un archivo que cambió desde que lo trajiste, en cambio, no se pisa: aparece **Existe en los dos lados** y eliges.

**Un ejemplo.** Lucas quiere probar a mano unas reglas nuevas en su `CLAUDE.md` durante una semana, sin que Tisty las toque. Elige **Dejar de sincronizar**. Edita el archivo con su editor y, en paralelo, sigue ordenando sus documentos en Agentes. Al final de la semana pulsa **Sincronizar**: la ficha de **Cómo trabajo** dice **Cambiado por fuera**, y Lucas elige **Traer el cambio** para quedarse con lo que probó, o **Sobrescribir con Tisty** para volver a lo de sus documentos.

La pausa, la sincronización y el permiso son de este equipo: dejar de sincronizar aquí no cambia nada en tus otros equipos.

### Gemini en pausa

Sincronizando, Tisty pone la línea `@tisty-rules.md` en tu `GEMINI.md` una vez. Si después la borras, o borras `GEMINI.md` entero, Tisty lo toma como tu decisión y no la vuelve a poner: Gemini queda **En pausa**, deja de recibir tus reglas, y Tisty deja de escribir `tisty-rules.md`. Solo cuenta una línea que puso Tisty: si nunca la puso ahí, por ejemplo porque todavía no tenías reglas para Gemini, que falte no deja a Gemini en pausa. Lo ves en tres lugares:

- **En la ficha** de cada regla que Gemini lee: la etiqueta **En pausa** y «Gemini no lee esta regla: falta la línea @tisty-rules.md en GEMINI.md, y Tisty no la vuelve a poner sola.», con el botón **Volver a ponerla**.
- **En Configuración › Agentes › Carpeta Agentes**, bajo **Sincronizar Gemini**: «En pausa: falta la línea @tisty-rules.md en GEMINI.md.», con **Volver a ponerla**.
- **Al entrar en Agentes**, el aviso **Falta la línea de Tisty**: «La línea @tisty-rules.md ya no está en GEMINI.md, así que Gemini no lee tus reglas. Tisty no la vuelve a poner sola: Gemini queda en pausa hasta que tú lo hagas.», con **Dejarlo en pausa** y **Volver a ponerla**.

**Volver a ponerla** agrega otra vez la línea al final de `GEMINI.md`, sin tocar lo demás, y Tisty vuelve a escribir tus reglas en `tisty-rules.md`. Mientras no lo haces, la ficha y la tarjeta lo siguen diciendo.

### Sacar una regla y su archivo

Sincronizando, Tisty deja de escribir un documento cuando lo sacas de Agentes: cuando lo borras, lo archivas o lo mueves fuera, cuando cambias su línea `agent:` por la de otro archivo, y cuando vuelves a una [versión anterior](Documentos#versiones-anteriores) que no tenía ficha, o tenía otra.

- **Si el archivo se armaba con varios documentos**, como `CLAUDE.md` con una regla de Global, Tisty lo vuelve a escribir sin esa parte, y guarda antes un respaldo.
- **Si era el único**, ninguna regla escribe ya ese archivo, pero sigue en el disco y tu agente lo sigue leyendo. Por eso, alrededor de un segundo después, Tisty te pregunta qué hacer con él.

**Un ejemplo.** Tenías la regla **Pruebas**, de `~/.claude/rules/pruebas.md`, y ya no la quieres. La borras, y se abre **¿Quitar también el archivo?**: «Ninguna regla de Agentes escribe ya este archivo de Claude, pero sigue en el disco y Claude lo sigue leyendo.», con la ruta debajo y «Si lo quitas, queda un respaldo en este equipo.».

- **Quitar el archivo** guarda un respaldo y lo borra: Claude deja de leer esa regla. Si era el último archivo de la carpeta de una skill, esa carpeta se va con él; las carpetas propias del agente, como `rules`, quedan.
- **Dejarlo** lo deja donde está, y Tisty deja de responder por él: no lo escribe ni te vuelve a preguntar. Si un día lo quieres de vuelta en Agentes, aparece en [Traer otro archivo](#traer-otro-archivo).

Si el archivo cambió después de que Tisty lo escribió, por ejemplo porque lo editaste a mano, el aviso suma «Cambió después de la última vez que Tisty lo escribió.». El respaldo guarda el archivo como está ahora, con ese cambio.

Si quitas `tisty-rules.md`, el archivo de reglas de Gemini, Tisty también saca la línea `@tisty-rules.md` de `GEMINI.md`, y nada más.

Mientras no respondes, la tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes** dice «1 archivo ya no es de ninguna regla de Agentes.», con **Revisar** para volver a la pregunta.

Tisty no pregunta, y deja el archivo como está:

- por la memoria del agente, que escribe el propio agente: la de cada proyecto de Claude Code ni el `GEMINI.md` de Gemini;
- por un archivo de un agente que pausaste, ni por ninguno mientras Agentes no sincroniza: traída sin sincronizar, sacar un documento no toca ningún archivo;
- por un archivo que solo trajiste y Tisty nunca escribió: **Traer** no lo anota como escrito por Tisty, así que no es de Tisty ofrecer quitarlo;
- por un archivo cuyo documento [espera tu aprobación](#por-aprobar);
- por un archivo que ya no existe.

En tus otros equipos pasa lo mismo: cuando la regla borrada les llega con la sincronización, cada uno te pregunta por su propio archivo la próxima vez que Tisty lo mira, y el respaldo queda en ese equipo.

### Eliminar una subcarpeta

Botón derecho sobre una subcarpeta de Agentes, como **Agentes › Claude › Pruebas**, y **Eliminar**. Tisty pregunta antes «¿Borrar «Pruebas»? Sus documentos pasan a Sin clasificar y sus subcarpetas suben un nivel. No se pierde nada, pero no se puede deshacer.». Con tu sí:

1. Sus documentos pasan a **Sin carpeta**. Como salen de Agentes, Tisty deja de escribirlos.
2. Alrededor de un segundo después, Tisty te pregunta por cada archivo que armaban, igual que en [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo): **Quitar el archivo** guarda un respaldo en este equipo y lo borra; **Dejarlo** lo deja donde está.
3. Un archivo que también arma una regla de Global, como `CLAUDE.md`, no se pregunta: Tisty lo vuelve a escribir sin esa parte, con su respaldo antes.
4. La carpeta no vuelve sola: Tisty no rehace una carpeta de Agentes que borraste.

Como al borrar una regla, no pregunta por un agente en pausa ni mientras Agentes no sincroniza: entonces Tisty no escribía ningún archivo.

**Un ejemplo.** Marisol guardaba sus reglas de pruebas en una carpeta suya, **Agentes › Claude › Pruebas**, y ya no las quiere. Botón derecho sobre **Pruebas** y **Eliminar**; un segundo después, Tisty le pregunta **¿Quitar también el archivo?** por `~/.claude/rules/pruebas.md`. Elige **Quitar el archivo**: el respaldo queda en este equipo, Claude deja de leer esas reglas y sus documentos siguen en **Sin carpeta**, por si un día los quiere de vuelta.

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

**Las subcarpetas siguen la misma regla.** **Ocultar de la barra** está siempre en el botón derecho de **Global**, **Claude** o cualquier carpeta dentro de Agentes. Mientras Agentes sincroniza, pasa por el mismo aviso, y **Dejar de sincronizar y ocultar** deja de sincronizar Agentes entera antes de ocultar esa subcarpeta: lo que tiene no debe quedar escribiendo sin que lo veas. Y si una subcarpeta oculta vuelve a sincronizar, por ejemplo con **Sincronizar** en la tarjeta, vuelve sola a la barra.

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

Si usas Tisty en más de un equipo, por ejemplo el computador de la casa y el del trabajo, tus documentos de Agentes viajan con tu sincronización como cualquier otro. Pero **cada equipo elige si sincroniza y qué aplica a sus agentes**: que la casa sincronice no hace que el trabajo sincronice, y un documento que llega de otro equipo no se escribe en ningún archivo hasta que tú lo dices, en ese equipo y desde la ventana de Tisty.

### Un segundo equipo, paso a paso

**Un ejemplo.** En el equipo de la casa elegiste **Traer y sincronizar**, trajiste tu `CLAUDE.md` y escribiste en **Agentes › Global** la regla «Para todos mis agentes». Ahí se aplicaron solas: las trajiste y las escribiste tú, en ese equipo. Ahora instalas Tisty en el equipo del trabajo y sincronizas:

1. En la barra, Agentes llega en el **estado 2**, traída sin sincronizar: el robot gris, el nombre como cualquier carpeta y sus documentos. Tus documentos ya viajaron, pero este equipo todavía no sincroniza ni leyó nada de sus agentes. Ábrela y léelos desde ya; arriba dice «No se sincroniza con tus agentes · Sincronizar».
2. Pulsa **Sincronizar**, en esa línea, en el botón derecho o en la tarjeta **Agentes** de **Configuración › Agentes › Carpeta Agentes**. Como este equipo nunca te dejó leer los archivos de sus agentes, primero se abre el aviso de la primera vez, con los archivos de este equipo. Los que ya tienen documento dicen **ya está en Agentes**, y si todo lo que encontró ya está, el aviso dice «Todo lo que encontré ya está en Agentes. Sincroniza este equipo para mantenerlo al día:».
3. Marca lo que quieras traer de este equipo, como la memoria de sus proyectos, y pulsa **Traer y sincronizar**. Agentes pasa al estado 3, con su robot coral. Si en cambio eliges **Traer**, entra lo que marcaste y este equipo sigue sin sincronizar.
4. Abre **Para todos mis agentes**. Su cuadro de datos todavía no tiene ninguna fila **Se escribe en**, y debajo, dentro de la ficha, un aviso ámbar dice «Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este?», con dos botones:
   - **Aplicar aquí** lo escribe en los archivos de los agentes de este equipo, con su respaldo antes, como siempre. Si un archivo de este equipo ya decía otra cosa, Tisty no lo pisa: la ficha queda en **Cambiado en los dos lados** y eliges.
   - **No en este equipo** lo deja fuera aquí. La ficha dice «Elegiste no aplicarlo en este equipo.», y **Aplicar aquí** sigue a mano por si cambias de idea.
5. Haz lo mismo con **Cómo trabajo**, tu `CLAUDE.md`. El cuadro de datos de cada uno dice, en **Sincroniza en este equipo**, «No: espera «Aplicar aquí»» mientras no eliges, y «Sí» después.

Lo que eliges vale para ese documento en ese equipo, y queda:

- **Un documento que ya aplicas se pone al día solo.** Si mañana cambias «Para todos mis agentes» en la casa, el trabajo lo escribe en cuanto llega, sin volver a preguntar.
- **Lo que escribes o traes en un equipo se aplica en ese equipo sin ningún clic.** Solo lo que llega de otro espera.
- **Solo espera lo que este equipo leería.** Un documento de Codex no pregunta nada en un equipo sin Codex.
- **Una vez aplicado, la ficha no ofrece deshacerlo.** Para que un agente de este equipo deje de recibirlo, pausa ese agente aquí, o deja de sincronizar Agentes en este equipo; para que ningún equipo lo reciba, sácalo de Agentes.

### Aprobar en otro equipo no es aplicar

Aprobar cambia el texto; **Aplicar aquí** decide si un equipo escribe el archivo. Son dos cosas aparte, y en un equipo donde un documento espera, solo la segunda lo escribe.

**Un ejemplo.** En el trabajo, «Para todos mis agentes» todavía espera tu **Aplicar aquí**. Mientras tanto, Claude Code propone cambiarlo, y como estás en el trabajo, lo apruebas ahí mismo:

1. **Aprobar el cambio** pregunta «¿Apruebas este cambio? Se escribe en el documento, y llega a los archivos de tus agentes en cada equipo donde Agentes sincroniza y lo aplicaste.». Con tu sí, el documento dice lo nuevo en todos tus equipos.
2. En la casa, que ya lo aplicaba, Tisty escribe el cambio en sus archivos como siempre, con su respaldo antes.
3. En el trabajo no se escribe nada todavía. La ficha cambia su aviso ámbar a «Aprobaste su texto en este equipo, pero aplicarlo aquí es aparte. ¿Lo aplicas a los agentes de este?», y siguen **Aplicar aquí** y **No en este equipo**. El cuadro de datos sigue diciendo «No: espera «Aplicar aquí»».
4. Pulsa **Aplicar aquí** cuando quieras que el trabajo lo reciba.

Pasa lo mismo con **Aprobar este texto** y con volver a una versión anterior desde el historial del documento: el texto cambia en todos tus equipos, y un equipo donde el documento esperaba sigue esperando. Lo que tú escribiste o trajiste en un equipo se aplica en ese equipo sin ningún clic, como siempre.

### Dos carpetas Agentes que se unen solas

Si dos equipos crearon Agentes antes de sincronizarse entre sí, por ejemplo porque instalaste Tisty en el trabajo sin conectar todavía la sincronización y elegiste **Traer y sincronizar** en los dos, al encontrarse aparecen **dos carpetas Agentes** en la barra por un momento.

Tisty las une solo, sin preguntar:

- **Queda la más antigua.** La misma en todos tus equipos, aunque los dos unan a la vez.
- **Las subcarpetas se juntan por nombre.** Lo que estaba en **Global** del otro equipo pasa a tu **Global**, lo de **Claude** a tu **Claude**, y así. Una subcarpeta que solo tenía el otro, como **Gemini**, pasa entera.
- **Los documentos pasan a la carpeta que queda,** con su texto, su aprobación y su historial: ninguna aprobación se pierde. La carpeta que quedó vacía se borra.
- **Ningún archivo de tus agentes cambia** por la unión, y por ninguno te pregunta «¿Quitar también el archivo?». Lo que ya se aplicaba en un equipo sigue aplicándose ahí.
- **Lo que llega del otro equipo espera** su **Aplicar aquí**, como cualquier documento de otro equipo. Si los dos trajeron su `CLAUDE.md`, ves los dos documentos en **Claude**: el tuyo se sigue escribiendo y el otro espera; elige con cuál te quedas y saca el otro.
- **Una carpeta archivada pasa entera** y sigue archivada: lo que guardaba no vuelve a escribirse.
- **Lo que llega tarde también entra.** Si en el otro equipo escribiste un documento en su **Global** justo antes de que se uniera, cuando ese documento llega Tisty lo pasa solo a tu **Global**, sin preguntar, y su archivo nunca se ofrece para quitar por esa causa.

Un caso raro que queda como está: si en la carpeta que se iría hay algo que puso un agente, Tisty no las une, porque al moverlo parecería puesto por ti. Mueve tú ese documento a una carpeta tuya y la unión se hace sola en la siguiente vuelta. Mientras haya dos carpetas Agentes sin unir, las dos quedan protegidas: tu agente solo puede proponer en cualquiera de ellas.

### La memoria es del equipo que la trajo

La memoria de un agente habla de lo que pasó en ese equipo, así que nunca se escribe en otro. En los demás equipos, su ficha dice «Memoria del agente de otro equipo: en este solo se lee.», sin botones. Cada equipo puede traer su propia memoria del mismo proyecto: aparece sin marcar en el aviso de la primera vez, aunque otro equipo ya haya traído la suya.

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

- **Los ajustes de tus agentes**: `settings.json`, `settings.local.json`, `config.toml`, `config.json`, `hooks.json`, `mcp.json`, `mcp-config.json`, `.claude.json` y `keybindings.json`. Tisty solo mira donde van las reglas, las skills y la memoria, nunca la carpeta principal de un agente, y dentro de una skill un archivo con esos nombres sí entra, porque es de la skill.
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
agent: claude/rule/CLAUDE.md
title: claude · CLAUDE.md
---
# Cómo trabajo

- Responde en español.
- Ramas con la forma tipo/nombre.
```

El agente te contesta con el título y la carpeta, «tu documento claude · CLAUDE.md, en AGENTS / Claude», nunca con el identificador. Cada herramienta está en [[Herramientas MCP|Herramientas-MCP]].

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

**¿Agentes toca `settings.json`, los hooks o mi configuración MCP?** No: Agentes no los lee ni los escribe. Conectar un agente es otra cosa: ahí Tisty agrega su servidor MCP y sus hooks a la configuración de ese agente, como cuenta [[Agentes y MCP|Agentes-y-MCP]].

**¿Por qué la memoria del agente viene sin marcar?** Porque la escribe el agente, puede hablar de tus proyectos, y una vez en Tisty viaja con tu sincronización y tus respaldos como cualquier documento. La traes solo si quieres leerla en Tisty.

**¿Qué es la línea `@tisty-rules.md` en mi `GEMINI.md`?** Es la única línea que Tisty agrega a ese archivo, para que Gemini lea tus reglas, que van en `tisty-rules.md`. El resto de `GEMINI.md` lo escribe Gemini. Si la borras, Tisty no la vuelve a poner: Gemini queda [en pausa](#gemini-en-pausa) hasta que pulsas **Volver a ponerla**.

**¿Agentes ve los agentes que uso dentro de WSL?** No. En Windows, Tisty mira tu carpeta de usuario de Windows, y un agente que corre dentro de WSL guarda sus archivos en tu carpeta de Linux.

**¿Y las reglas de cada repositorio?** Quedan donde están. Agentes maneja los archivos de tu usuario, no el `AGENTS.md` o el `CLAUDE.md` de cada proyecto.

**¿Puedo escribir una regla nueva desde cero?** Sí: botón derecho sobre **Global** o sobre la carpeta de un agente, como **Claude**, y **Documento nuevo**. El documento ya empieza con su línea `agent:`, y tú solo escribes la regla. Si Agentes sincroniza, Tisty la escribe apenas escribes algo. Mira [Un documento nuevo ya es una regla](#un-documento-nuevo-ya-es-una-regla).

**¿Cuántos respaldos guarda?** El original, el respaldo n.º 1, que nunca se borra, y además los últimos 20 de cada archivo, en este equipo.

**¿Por qué tengo una carpeta Cursor vacía en Agentes?** Tisty encontró Cursor en este equipo, y cada agente que encuentra tiene su subcarpeta, aunque esté vacía. Si no la quieres, bórrala: Tisty no la vuelve a crear. Mira [Cómo queda la carpeta](#cómo-queda-la-carpeta).

**Instalé un agente nuevo. ¿Cómo lo sumo?** Botón derecho sobre **Agentes** y **Buscar de nuevo**: Tisty le crea su subcarpeta y te ofrece sus archivos nuevos ya marcados. Mira [Buscar de nuevo](#buscar-de-nuevo).

**¿Puedo pedirle a mi agente que cambie mis reglas?** Sí, y te lo propone: su cambio queda aparte, en la ficha del documento, hasta que pulsas **Aprobar el cambio** o **Descartar**. Nada llega a tus agentes antes. Mira [Una propuesta de tu agente](#una-propuesta-de-tu-agente).

**Uso Tisty en dos equipos. ¿Mis reglas llegan solas al otro?** Llegan como documentos, con tu sincronización, y ahí Agentes aparece traída sin sincronizar. Para que lleguen a los agentes de ese equipo, pulsa **Sincronizar** allá, y después **Aplicar aquí** en cada documento, la primera vez. Después se ponen al día solas. Mira [En varios equipos](#en-varios-equipos).

**¿Puedo tener una regla solo en un equipo?** Sí: en el equipo donde no la quieres, pulsa **No en este equipo** en su ficha. Lo que elige cada equipo es solo suyo.

**Borré una regla. ¿Qué pasa con su archivo?** Tisty te pregunta «¿Quitar también el archivo?»: **Quitar el archivo** lo borra y guarda un respaldo, y **Dejarlo** lo deja donde está, sin que Tisty vuelva a escribirlo. Mira [Sacar una regla y su archivo](#sacar-una-regla-y-su-archivo).

**La primera vez dejé un archivo sin marcar. ¿Lo puedo traer ahora?** Sí, con **Traer otro archivo**: desde el botón derecho sobre una subcarpeta de Agentes, como Claude; con Agentes abierta, desde el botón **+** de la Biblioteca o la paleta de comandos; o desde Configuración › Agentes › Carpeta Agentes. Mira [Traer otro archivo](#traer-otro-archivo).

**¿Por qué tengo Agentes en la barra si nunca la usé?** Está ahí, en gris, para que sepas que existe. Así no lee nada: solo mira si existe la carpeta de cada agente, como `~/.claude`, y la lista de nombres y tamaños aparece recién cuando haces clic en ella. Si no la quieres ver, botón derecho y **Ocultar de la barra**; vuelve con **Volver a mostrar**, en Configuración › Aspecto y ventana › Carpetas ocultas.

**¿Cuál es la diferencia entre Traer y Sincronizar?** **Traer** copia los archivos a Tisty y no escribe nada en ellos; **Sincronizar** hace que lo que cambias en Tisty llegue a esos archivos. Mira [Traer o sincronizar](#traer-o-sincronizar).

**Traje mis reglas y las cambié en Tisty, pero mi agente no lo ve.** Agentes está traída sin sincronizar: su robot está en gris y, al abrirla, arriba dice «No se sincroniza con tus agentes · Sincronizar». Pulsa **Sincronizar**.

**¿Dejar de sincronizar borra algo?** No. Ningún archivo de tus agentes se toca y ningún documento se borra: tus agentes siguen leyendo lo que Tisty escribió la última vez, y tus documentos quedan en Agentes.

**¿Mi agente puede proponerme un cambio con Agentes oculta o sin sincronizar?** Sí: la propuesta queda en la ficha de su documento, y la apruebas o la descartas ahí. Aprobada, cambia el documento; llega a los archivos de tus agentes cuando Agentes sincroniza.

**¿Por qué la barra no me dice si algo espera?** Porque Agentes se ve como cualquier carpeta: solo su color dice si sincroniza. Lo que espera tu decisión está en la ficha de cada documento y en la tarjeta **Agentes** de Configuración › Agentes › Carpeta Agentes, y sincronizando, Tisty te pregunta por los archivos pendientes apenas entras en ella. Mira [Cómo queda la carpeta](#cómo-queda-la-carpeta).

**¿Por qué Agentes no tiene documentos?** La creaste con **Traer y sincronizar** sin que Tisty encontrara archivos, o sus documentos salieron mientras Tisty estaba cerrado, por ejemplo al sincronizar con otro equipo. Escribe una regla con **Documento nuevo** en **Global** o en la carpeta de un agente, o trae un archivo con **Buscar de nuevo** o **Traer otro archivo**.

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
- **«Otro documento de Agentes ya escribe este archivo»**: dos documentos tienen la misma línea `agent:`. Deja uno.
- **En espera**: otro documento del mismo archivo espera tu aprobación. La ficha lo nombra; [apruébalo](#por-aprobar).
- **Escrito por un Tisty anterior**: mira el archivo y, si te quedas con el documento, **Sobrescribir con Tisty**.
- **Un aviso ámbar sobre otro equipo o tu aprobación**: el documento todavía no se aplica en este equipo. Mira [En varios equipos](#en-varios-equipos).

**Mi regla no llega a mi otro equipo.** En ese equipo, revisa que Agentes tenga su robot coral (si está gris, pulsa **Sincronizar**) y abre el documento: si dice «Este documento viene de otro equipo», pulsa **Aplicar aquí**; si dice «Elegiste no aplicarlo en este equipo.», también. Si la ficha dice «Ningún agente de este equipo lee este documento todavía.», ese equipo no tiene el agente de la regla.

**Una ficha dice «Su texto aún no tiene tu aprobación» y yo no cambié nada.** El texto de un documento cambió sin pasar por la ventana de Tisty, por ejemplo con un paquete importado o una edición directa en la carpeta de datos. El aviso te dice quién lo escribió y qué cambió desde lo último que aprobaste: léelo, y si está bien, **Aprobar este texto**. Mira [Por aprobar](#por-aprobar).

**Escribí una regla de Global y un archivo quedó en Cambiado en los dos lados.** Ese archivo ya existía y no lo trajiste, así que Tisty no sabe qué parte es tuya. No lo sobrescribas: en **Configuración › Agentes › Carpeta Agentes**, pulsa **Revisar** y, en **Existe en los dos lados**, **Traer a Agentes**. El archivo queda con Global delante de lo tuyo, y el original, como respaldo n.º 1. Mira [Un archivo que nunca trajiste](#un-archivo-que-nunca-trajiste).

**Me pregunta «¿Quitar también el archivo?» y no borré nada.** También cuenta como sacar una regla cambiar su línea `agent:` por la de otro archivo, o volver a una versión anterior que no tenía ficha. Si no quieres tocar nada, **Dejarlo** lo deja donde está.

**Agentes no está en la barra.** La ocultaste, en este equipo: está en **Configuración › Aspecto y ventana › Carpetas ocultas**, y **Volver a mostrar** la devuelve. Si la vaciaste con Tisty abierto, no se oculta: vuelve sola a la barra, en gris, y un clic en ella trae otra vez lo que encuentre.

**El robot de Agentes está gris.** No sincroniza en este equipo. Si el nombre también está gris, todavía no traes nada: haz clic en ella. Si el nombre se ve normal, está traída: pulsa **Sincronizar**. Mira [Tres estados en la barra](#tres-estados-en-la-barra).

**Ocultar me pide dejar de sincronizar.** Agentes sincroniza, y nunca queda oculta y sincronizando, ni Agentes ni sus subcarpetas, como **Global**. **Dejar de sincronizar y ocultar** hace las dos cosas; tus archivos de agente no se tocan. Si prefieres que siga sincronizando, **Cancelar**.

**Una subcarpeta de Agentes no tiene Cambiar nombre ni Diagrama nuevo.** Es a propósito: dentro de Agentes, cada carpeta es de un agente, y Tisty sabe a quién le escribe por su nombre. Su botón derecho ofrece solo **Documento nuevo**, **Traer otro archivo**, la pausa de su agente, **Ocultar de la barra** y **Eliminar**. Mira [El botón derecho de una subcarpeta](#el-botón-derecho-de-una-subcarpeta).

**El botón derecho de Agentes solo ofrece sincronizar, buscar de nuevo u ocultar.** Es a propósito. **Buscar de nuevo** trae lo nuevo de tus agentes; para elegir tú cada archivo, usa **Traer otro archivo** en el botón derecho de una subcarpeta, como Claude, o en Configuración › Agentes › Carpeta Agentes. Y no se mueve ni cambia de nombre porque por dentro se guarda como `AGENTS`, y con otro nombre o en otro lugar Tisty y tus agentes dejarían de encontrarla. Mira [Agentes se queda arriba](#agentes-se-queda-arriba).

**Quise mover o renombrar Agentes y no me dejó.** Tisty te dijo «Agentes se queda arriba, con su nombre y su robot, para seguir cuidando las reglas de tus agentes. Su color y su lugar entre tus carpetas sí los cambias». Es a propósito: mira [Agentes se queda arriba](#agentes-se-queda-arriba).

**Toda la carpeta dejó de escribir.** Revisa que Agentes tenga su robot coral: si está gris, no sincroniza en este equipo, y **Sincronizar** la pone otra vez al día. Si un agente no recibe nada, mira su interruptor en Configuración › Agentes › Carpeta Agentes.

**Codex no lee mis reglas.** Si tienes `~/.codex/AGENTS.override.md`, Codex lee solo ese archivo. Tus reglas de Global van ahí, pero un documento con `agent: codex/rule/AGENTS.md` se escribe en `AGENTS.md`, que Codex no lee mientras exista el otro. Si creaste el override después de la primera vez, [tráelo a Agentes](#el-agentsoverridemd-de-codex).

**Gemini no lee mis reglas.** Si su ficha dice **En pausa**, falta en `~/.gemini/GEMINI.md` la línea `@tisty-rules.md` que Tisty había puesto: pulsa **Volver a ponerla**, en la ficha o en Configuración › Agentes › Carpeta Agentes. Tisty no la vuelve a poner por su cuenta, porque borrarla es tu decisión. Mira [Gemini en pausa](#gemini-en-pausa).

**No aparece Traer el cambio.** El documento es de Global, el cambio cae en la parte de otro documento del mismo archivo, o lo estás editando: pulsa **Listo** y vuelve a mirar.

**«El archivo cambió mientras Tisty lo escribía, así que no lo tocó.»** Otro programa escribió el archivo al mismo tiempo. Tisty no pisa nada: vuelve a mirar la ficha y elige.

**Recuperé un respaldo y dice Cambiado por fuera.** Es lo esperado: [un respaldo recuperado no cambia tu documento](#respaldos). Elige **Traer el cambio** para quedarte con él.

## Ver también

- [Privacidad](Privacidad#tus-agentes-en-la-carpeta-agentes): qué lee Agentes, cuándo, y qué se queda en este equipo.
- [[Agentes y MCP|Agentes-y-MCP]]: conectar un agente a Tisty.
- [Versiones anteriores](Documentos#versiones-anteriores): el historial de un documento.
- [Carpetas](Documentos#carpetas): filtrar, archivar y desarchivar.
- [Propiedades](https://markdown.tisty.app/es/propiedades), en la guía de Markdown de Tisty: la línea `agent:` y sus marcas.
