# Sincronizar y respaldar

Tisty trabaja siempre en este equipo. Para alcanzar tu trabajo desde otro, dile dónde dejar las copias, y del resto se encarga solo.

No hay nada por medio: ni cuenta, ni servidor, ni servicio nuestro. Si no sincronizas nada, Tisty nunca abre una conexión.

## Elegir la carpeta

Tisty te ofrece **Google Drive, OneDrive, iCloud y Dropbox** (los que encuentre instalados, con la carpeta ya resuelta) o **cualquier otra carpeta** que alcancen los dos equipos: un NAS, Syncthing, un disco externo que conectas los viernes.

> [!IMPORTANT]
> Quien sube y baja esa carpeta es **el programa de tu proveedor**, no Tisty. Si no está en marcha, tu trabajo espera en la carpeta hasta que lo esté. Sincronizar te da redundancia, no vuelta atrás en el tiempo: si borras una tarea, el borrado también viaja.

En Configuración › **Sincronizar** está **Sincronizar ahora**, que dice a qué hora corrió y si llegó algo o quedó escrito en la carpeta.

Google Drive, desde la 1.31 OneDrive y desde la 1.32 un servidor WebDAV tuyo también se pueden conectar directo, sin ningún programa suyo en el equipo: mira [Sincronizar con Google Drive directamente](#sincronizar-con-google-drive-directamente), [Sincronizar con OneDrive directamente](#sincronizar-con-onedrive-directamente) y [Sincronizar con tu propio servidor WebDAV](#sincronizar-con-tu-propio-servidor-webdav). Lo que Tisty guarda en cualquiera de esos tres también se puede cifrar: mira [Cifrar tu nube](#cifrar-tu-nube).

Cuando un equipo deja una carpeta, escribe en ella una nota corta, `README - Tisty.txt`, que dice adónde se fueron sus copias y cómo volver. La quita cuando vuelve a elegir esa carpeta.

## Un lugar que ya guarda un Tisty

**Un equipo nuevo se une sin preguntar.** Si este equipo todavía no guarda nada tuyo —solo la guía y las listas iniciales que Tisty pone la primera vez que se abre—, simplemente toma lo que ya guarda la carpeta o tu Google Drive. Lo que tenía queda antes en un respaldo dentro de la carpeta privada de Tisty.

Muestra **«Trayendo tu Tisty desde …»**, con el nombre de la carpeta o de Google Drive, una barra en movimiento y los conteos a medida que llegan las cosas: la historia, después los documentos, después los adjuntos. Mientras más guarda el lugar, más tarda esta primera vuelta, y tus tareas llegan primero. **Seguir trabajando mientras llega** te deja entrar a Tisty mientras la ronda sigue: el pie de la barra lateral sigue contando lo que llega y, si la ronda no pudo terminar, ahí dice por qué. Si cierras Tisty, sigue donde quedó la próxima vez.

Una ronda fallida suelta tampoco te detiene: entras y las rondas siguientes lo siguen intentando. Solo cuando sigue fallando se detiene a decir qué se interpone, con **Entrar de todos modos**.

Si guarda algo que hiciste tú —una tarea, un documento, una lista tuya, un cambio en la guía—, Tisty te pregunta, nombrando el lugar: **Fusionar las dos**, **Conservar esta máquina** o **Quedarme con lo de la carpeta** (**Quedarme con lo de Google Drive**). Elijas lo que elijas, antes se escribe un respaldo, y si hay una sincronización en curso tu elección espera a que termine.

Si tu nube está cifrada, antes hay que autorizar al equipo nuevo: mira [Tus otros equipos](#tus-otros-equipos).

## Confirmar tus equipos

**Cada equipo firma lo que escribe.** Uno que ya conocías se reconoce solo cuando se actualiza. Uno nuevo (recién instalado o reinstalado) espera a que confirmes que es tuyo:

1. Aparece un aviso en la barra lateral con el nombre de ese equipo, y **Confirmar…** te muestra su código de veinte números.
2. En ese equipo, abre Configuración › **Sincronizar**, en **Tus equipos**. **Mostrar en grande** deja el código fácil de leer desde lejos.
3. Si los códigos coinciden, **Sí, es el mismo** lo deja pasar y trae lo que escribió. Si no, **No coincide** lo deja fuera.

Se hace una vez en cada uno de tus otros equipos. Hasta entonces no entra nada de lo que ese equipo escribe, y desde entonces lo que no responda a su clave se rechaza, y Tisty te lo dice.

**Cambiar nombre**, en ese mismo lugar, elige cómo ven este equipo los demás; vacío, vuelve al nombre del sistema.

## Cuando dos equipos escriben el mismo documento

Tisty une los cambios **bloque a bloque**: editas la introducción en un equipo, alguien edita el párrafo final en el otro, y los dos cambios llegan sin preguntarte nada. Solo un choque real se vuelve pregunta: la ventana te muestra las dos versiones una junto a otra y te quedas con **Esta**, **La otra** o **Las dos**.

## Adjuntos grandes

Por defecto cada equipo lleva todos los adjuntos, así que cualquiera abre cualquier cosa sin red. Para archivos de más de **25 MB**, Configuración ofrece otras dos formas:

- guardar solo lo que adjuntó este equipo y traer el resto cuando lo abres;
- dejarlos en la carpeta compartida y en ningún otro lugar.

Una copia local solo se suelta después de comprobar que la de la carpeta compartida tiene el mismo hash. Si la carpeta no está disponible, Tisty lo dice claramente.

## Respaldos

**Respalda a mano, sincronices o no**: una carpeta compartida no es un respaldo. En **Configuración › Tus datos**, **Crear respaldo…** escribe un zip con toda tu historia, que guardas donde quieras, y desde ahí mismo se restaura.

## Sincronizar con Google Drive directamente

Desde la 1.30, Tisty puede sincronizar con tu propio Google Drive sin el programa de escritorio de Google. Aquí es Tisty quien habla con Drive, así que, a diferencia de una carpeta, sabe cuándo algo anda mal y lo dice en la tarjeta de Google Drive, en **Configuración › Sincronizar**. Diga lo que diga la tarjeta, lo que escribes está a salvo en este equipo y sube en cuanto desaparece la causa.

### Conectarlo

**Configuración › Sincronizar › Tu servicio de nube › Google Drive › Conectar.** Tu navegador se abre en la página de Google; entra en tu cuenta y permite que Tisty guarde ahí sus copias. Tisty pide solo un espacio oculto propio en tu Drive, así que no puede ver nada más ahí, y sus copias no aparecen entre tus archivos. Después, **Empezar a subir**: la primera vez sube todo lo que tienes, y si cierras Tisty o se corta la red, sigue donde quedó. Haz lo mismo en tu otro equipo, con la misma cuenta de Google.

En un equipo que todavía no guarda nada tuyo, si tu Drive ya tiene tu Tisty, la tarjeta dice **«Tu Google Drive ya tiene tu Tisty»** y el botón dice **Sincronizar**: no hay nada que subir primero, y Tisty trae a este equipo lo que hay ahí. Un equipo que ya guarda trabajo propio primero te pregunta (mira [Un lugar que ya guarda un Tisty](#un-lugar-que-ya-guarda-un-tisty)).

Una ronda de sincronización que pasa del minuto dice en el botón **Sincronizar ahora** cuánto lleva, **Sincronizando desde hace 3 min…**, para que una primera vuelta larga nunca parezca congelada.

Solo se puede conectar desde la ventana. Los asistentes nunca tocan la red, y una copia de Tisty compilada sin las credenciales de Google del proyecto no ofrece la nube.

Antes de que entres en tu cuenta, y antes de que empiece un movimiento a Drive, Tisty revisa que Google responda: una petición a `www.googleapis.com` sin nada tuyo, que se abandona a los 10 segundos. Si un proxy, un filtro de tu trabajo o un antivirus se interpone, si revisa las conexiones seguras, o si Google tiene problemas, Tisty te lo dice ahí en rojo, con palabras simples y con **Revisar de nuevo**; si Google solo tardó en responder, te lo dice en gris, y que puedes conectarte igual. Nunca te impide conectar, nunca rodea un proxy y nunca corre en una ronda de sincronización.

### Lo que viaja y lo que se queda

Suben tu historia, tus documentos y su configuración. Los adjuntos se quedan en el equipo donde se agregaron; en otro, cada uno baja la primera vez que lo abres ahí. **Liberar espacio**, en Configuración › Mantenimiento, suelta los adjuntos que no abres hace tiempo; la próxima vez que abras uno, vuelve a bajar. Los adjuntos tienen un tope de 650 MB, en una tarea o en un documento, igual que con una carpeta, y nada de más de 750 MB se trae desde la nube.

### Lo que dice la tarjeta

- **«Google ya no deja entrar a Tisty»**: se quitó el permiso, desde tu cuenta de Google o con **Quitar el acceso de Tisty en todos tus equipos**, o venció. Pulsa **Reconectar** y vuelve a entrar. Quitar el acceso en un equipo lo quita en todos, así que cada uno te lo va a pedir.
- **«Esta copia es de otra cuenta»**: entraste con una cuenta de Google distinta de la que esta copia ya usa para sincronizar. Tisty se detiene antes que mezclar dos cuentas. Pulsa **Cambiar cuenta** y entra con la que usaste antes. Si de verdad quieres sincronizar con otra cuenta, múdate a ella (más abajo).
- **«Tu Google Drive está lleno»**: tu Drive no tiene más espacio. Tisty manda primero tus tareas; el resto, incluido lo que mandan tus otros equipos, espera hasta que liberes espacio en Drive o consigas más. **Ver espacio** abre la página de Google para eso.
- **«Tu nube llegó a su límite de hoy»**: cada equipo tiene su propia cuota diaria, el equivalente a 2.000.000 de peticiones y 50 GB descargados, para que ninguna copia pueda gastarse la cuota que Google le da al proyecto. Es por equipo, no por persona. Cuando se agota, la sincronización sigue sola al día siguiente. **Uso de hoy en la nube**, en **Liberar espacio** de Configuración › Mantenimiento, muestra cuánto queda.
- **«Google pidió esperar un poco»**: Google está frenando a todos por un momento. Tisty espera lo que Google pide y sigue solo. No hay nada que hacer.
- **«La nube está en pausa por ahora»**: el proyecto puede pausar la sincronización en todas las copias por un tiempo, si su cuota compartida con Google corre riesgo; el aviso va firmado, así que nadie más puede mandarlo. Se levanta solo. **Apagar**, en la tarjeta, hace lo mismo solo en este equipo, hasta que lo vuelvas a encender.
- **«Google tiene problemas»**: el problema está del lado de Google. Tisty vuelve a intentarlo solo; no hay nada que hacer.
- **«No se pudo llegar a tu nube»**: Tisty no pudo hablar con Google. Si este equipo no tiene conexión, solo dice eso, y todo sube en cuanto vuelve. Si otra cosa se sigue interponiendo, debajo dice qué, en rojo: mira más abajo.

### Cuando algo se interpone con Google

Una ronda fallida es un tropiezo, y Tisty no dice nada. Cuando las rondas siguen sin llegar a Google —una segunda seguida, o dos minutos después de que falló la primera—, la tarjeta dice qué se interpone, en rojo, y las mismas palabras aparecen después de **Sincronizar ahora**, al pie de la barra lateral y en una mudanza que quedó en espera. Lo que se interpone puede cambiar de una ronda a otra; sigue siendo una misma racha de fallos, y la tarjeta nombra lo último. Una conexión que se corta a medio traspaso no cuenta como bloqueo: solo se reintenta. En todos los casos lo que escribes queda a salvo en este equipo y Tisty lo sigue intentando solo.

- **«Algo está bloqueando la conexión con Google Drive desde hace un rato: puede ser un proxy, un filtro de tu empresa o un antivirus.»**: algo entre este equipo y Google rechaza a Tisty. En una red de trabajo o de estudio, pide a quien la administra que deje a Tisty llegar a `accounts.google.com`, `oauth2.googleapis.com` y `www.googleapis.com`. En tu propio equipo, revisa si tu antivirus o tu firewall bloquean a Tisty, y permítelo ahí.
- **«Un filtro de tu red o un antivirus está revisando las conexiones seguras y Tisty no puede confiar en él.»**: algo abre las conexiones seguras para mirar dentro, y Tisty solo manda tus copias por una conexión que puede comprobar que es de Google. En una red de trabajo, pide que dejen pasar a Tisty, es decir, que lo dejen fuera de esa revisión. En tu propio equipo, busca en tu antivirus la opción que revisa las conexiones seguras (HTTPS) y deja a Tisty fuera de ella.
- **«Google Drive está tardando en responder desde esta red.»**: la red está lenta o saturada. No hay nada que hacer; si sigue así, probar otra red te dice si el problema es esta.

Tisty nunca rodea un proxy ni un filtro, y tú tampoco deberías: no apagues por tu cuenta la protección de tu empresa, pide que dejen pasar a Tisty. Las mismas causas aparecen en rojo, con **Revisar de nuevo**, cuando conectas Google Drive o te mudas a él.

### Dejar Google Drive

**Mudarme a otro lugar…** lleva tus copias a una carpeta o de vuelta solo a este equipo. Tisty primero trae a casa todo lo que vivía solo en Drive, y después les dice a tus otros equipos adónde te fuiste, para que cada uno te siga cuando sincronice. No se borra nada en Drive a menos que lo pidas; **Borrar para siempre mis copias en Google Drive** las borra después, y como el espacio de Tisty en Drive no tiene papelera, no se pueden recuperar. Para detener un equipo sin mudar los demás, usa **Desconectar este equipo**: las copias se quedan en Drive.

### ¿Dónde están mis copias en Drive?

En un espacio oculto de tu Drive que es de Tisty: no aparecen entre tus archivos, y solo Tisty las abre. Para sacarlas, múdate a una carpeta desde Tisty. Desconectar las conserva, y si vuelves más adelante a la misma cuenta, las retoma, también después de *Desconectar de Drive* en **Google Drive › Configuración › Administrar apps**, que solo retira el acceso de Tisty. Solo **«Eliminar datos ocultos de la app»** ahí las borra, igual que **Borrar para siempre mis copias en Google Drive** en Tisty.

## Sincronizar con OneDrive directamente

Desde la 1.31, Tisty puede sincronizar de la misma forma con tu propio OneDrive personal, sin el programa de escritorio de Microsoft. Funciona como Google Drive, más arriba: la misma primera ronda, la misma tarjeta en **Configuración › Sincronizar**, la misma revisión del camino antes de que inicies sesión y la misma mudanza para dejarlo.

- **Conectarlo:** **Configuración › Sincronizar › Tu servicio de nube › OneDrive › Conectar**. Tu navegador se abre en la página de Microsoft; entra en tu cuenta y permite que Tisty guarde ahí sus copias. Tisty nunca ve tu contraseña.
- **Dónde quedan las copias:** en *Aplicaciones › Tisty* de tu OneDrive. A diferencia del espacio oculto de Google, es una carpeta que puedes ver. Déjasela a Tisty: lo que cambies ahí a mano se lee como un cambio de otro equipo. Borrarla manda tus copias a la papelera de reciclaje de OneDrive, desde donde se pueden restaurar.
- **Las subidas largas** siguen donde se cortaron, y una escritura que falla a medias se vuelve a intentar.
- **Soltarlo:** **Desconectar este equipo** olvida el acceso solo en ese equipo. **Quitar el acceso de Tisty en todos tus equipos** lo olvida aquí y abre tu cuenta de Microsoft en el navegador, donde quitas a Tisty de todos tus equipos a la vez. Tus copias quedan en tu OneDrive.

Antes de que inicies sesión, Tisty revisa que Microsoft responda, con una petición a `graph.microsoft.com` que no lleva nada tuyo. Igual que Drive, solo se puede conectar desde la ventana.

## Sincronizar con tu propio servidor WebDAV

Desde la 1.32, Tisty puede guardar sus copias en un servidor tuyo (Nextcloud, ownCloud, Synology y otros) hablando con él por WebDAV, sin ningún programa por medio y sin iniciar sesión en el navegador. Funciona como Google Drive, más arriba: la misma primera ronda, la misma tarjeta en **Configuración › Sincronizar** y la misma mudanza para dejarlo.

- **Conectarlo:** **Configuración › Sincronizar › Tu servicio de nube › WebDAV**. Escribe la **Dirección del servidor**, tu **Usuario** y una **Contraseña**, y pulsa **Conectar**. Antes de guardar nada, Tisty revisa el servidor: entra con lo que escribiste y comprueba que el servidor no deje que dos equipos escriban uno encima del otro; si lo deja, no se conecta nada. Solo se aceptan direcciones `https://`, porque la contraseña viaja en cada petición; `http://` a secas se rechaza, salvo hacia este mismo equipo.
- **La contraseña** queda en este equipo, guardada por el sistema: en el llavero en macOS, en un archivo sellado para tu usuario en Windows. Nunca llega al registro, a un respaldo ni a un paquete. Mejor una contraseña de aplicación de tu servidor que la principal, porque ahí la puedes quitar cuando quieras. Si el servidor vuelve a pedirla, Tisty solo te pide la contraseña.
- **Dónde quedan las copias:** en una carpeta *Tisty* dentro de la carpeta a la que apunta tu dirección; Tisty no toca nada fuera de ella. Ahí la puedes ver. Déjasela a Tisty: lo que cambies ahí a mano se lee como un cambio de otro equipo.
- **Tu servidor pone el límite y el ritmo:** lo que tenga libre es tu techo, y Tisty busca cambios cada dos minutos. Los adjuntos también quedan siempre en este equipo, así que no se ofrece **Liberar espacio**.
- **Soltarlo:** **Desconectar este equipo** olvida la contraseña solo en ese equipo y conserva tus copias, aquí y en el servidor. Para quitar el acceso en todos tus equipos, borra la contraseña de aplicación, o cambia la contraseña, en tu servidor. **Borrar para siempre mis copias en WebDAV** las borra del servidor, que las manda a su papelera si tiene una.

## Cifrar tu nube

Desde la 1.36, lo que Tisty guarda en tu Google Drive, tu OneDrive o tu servidor WebDAV se puede cifrar con una llave que sale de una frase de 24 palabras que solo tú conoces. Quien vea esos archivos, tu proveedor incluido, ya no puede leer tus tareas ni tus documentos. Lo que tienes en tus equipos no cambia, y una carpeta que otro programa mantiene al día nunca se cifra.

### Activarlo

En **Configuración › Seguridad**, pulsa **Cifrar mi nube…**. Antes de empezar, Tisty revisa tres cosas:

1. **Que tu frase esté anotada.** Si todavía no tienes una, **Crear mi frase** la crea en este equipo. **Guardar como PDF** te da una hoja para guardarla fuera de Tisty, y tres de sus palabras, escritas de vuelta, confirman que la anotaste.
2. **Que haya espacio en la nube.** Mientras cifra, tu nube ocupa el doble, hasta que se va la copia antigua.
3. **Que cada uno de tus equipos tenga un Tisty que conozca la nube cifrada**, la 1.36 o una posterior. Tisty nombra el equipo que se haya quedado atrás: actualízalo, o quítalo de tus equipos si ya no lo usas.

Después, **Cifrar mi nube** te muestra el avance. Puedes seguir trabajando, o cerrar Tisty y que siga donde quedó. Hasta el último paso, cancelar no pierde nada: lo de antes queda intacto.

Cuando dice **«Tu nube está cifrada»**, la copia antigua sin cifrar sigue en la nube hasta que pulses **Borrar la copia sin cifrar**, que borra solo los archivos cuya copia cifrada ya está ahí. Un archivo demasiado grande para que el proveedor lo acepte cifrado queda fuera y sigue como estaba.

### Tus otros equipos

**Uno que ya sincronizaba contigo recibe la llave solo.** Espera un momento y la toma de otro de tus equipos que tenga Tisty abierto, y mientras tanto no escribe nada sin cifrar.

**Un equipo nuevo se autoriza desde uno que ya lee la nube cifrada:**

1. En el equipo nuevo, elige **Autorizar desde otro equipo**. Te muestra veinte dígitos.
2. En el otro, abre **Configuración › Seguridad › Autorizar un equipo nuevo** y escríbelos.
3. De vuelta en el equipo nuevo, pulsa **Ya los aceptó**. Mientras la llave va en camino lo ves en curso; cuando llega, el equipo nuevo trae tu nube cifrada, y la barra de abajo y Configuración › Seguridad dicen cuánto lleva.

Si no escribes nada, no se entrega nada. O, en el equipo nuevo, **Escribir mi frase** recibe tus 24 palabras: ahí solo queda la llave de la nube, nunca la frase.

Si luego el equipo nuevo se queda con el historial de tu nube, no te pregunta por tus otros equipos, y el que recibió sus dígitos no vuelve a pedirlos; cada uno de tus otros equipos te pide confirmar su código una vez, como con cualquier equipo nuevo. Elígelo dentro de la semana en que escribiste los dígitos: pasado ese plazo, el que los recibió te lo pregunta una vez más.

### Lo que no oculta

Tu proveedor guarda un tiempo versiones anteriores sin cifrar, en su historial y su papelera. Los nombres y tamaños de los archivos, cuándo cambian y cuántos equipos tienes siguen a la vista.

### Guarda bien la frase

La frase se guarda solo en el equipo donde la creaste, y **Ver la frase**, en Configuración › Seguridad, te la vuelve a mostrar ahí. Nunca pasa por las manos de nadie, así que nadie te la puede devolver: sin ella o sin uno de tus equipos, nadie puede abrir tu nube, ni siquiera nosotros.

### Volver atrás

**Descifrar mi nube**, en el mismo lugar, vuelve a copiar todo sin cifrar y borra la copia cifrada. Tus otros equipos se dan cuenta y siguen sincronizando sin que hagas nada en cada uno. Tu frase sigue siendo tuya.

## Mudarte a otro lugar

Cambiar dónde sincroniza Tisty es una mudanza, no un interruptor: todo lo que ya tienes tiene que viajar. **Configuración › Sincronizar › Mudarme a otro lugar…** lleva tus copias entre solo este equipo, una carpeta, Google Drive, OneDrive y tu servidor WebDAV.

- Antes de empezar, la ventana te dice lo que va a pasar: qué baja, qué sube, si a tu Drive le cabe, y cuáles de tus otros equipos te van a seguir. Marca **Entiendo que la mudanza tarda y que no se puede dejar a medias.** y pulsa **Mover mis copias**.
- Va en dos pasos. Primero llega a este equipo todo lo que solo vive en el lugar que dejas, **Trayendo desde tu nube** o **Trayendo desde la carpeta**, porque después de la mudanza Tisty ya no lo lee. Después todo va al nuevo lugar, **Subiendo a Google Drive** o **Copiando a la carpeta**, con su barra y lo que queda, **faltan 12 archivos**. Cuando hacen falta los dos, la ventana muestra **Paso 1 de 2** y **Paso 2 de 2**.
- Las pausas se dicen tal cual: **Google pidió una pausa; sigue a las 14:05**, **Sin conexión a internet**, o cuántos segundos faltan para reintentar. Si se detiene, **Seguir** la retoma donde quedó. Nada cambia de lugar hasta que todo está ahí.
- **Ocultar y seguir trabajando** deja el avance abajo en la barra lateral. Deja Tisty abierto hasta que termine.
- Al terminar, tus otros equipos se enteran de adónde te fuiste, y cada uno te sigue cuando sincroniza. Los que nunca confirmaron este no lo hacen: abre Tisty en ellos y elige tú el nuevo lugar.

Si tienes el programa de escritorio de Google, elegir su carpeta es más rápido y no gasta nada de tu cupo diario.

## Después de una actualización

Cada evento lleva una versión, y un equipo solo lee lo que entiende. Cuando un equipo se actualiza a una versión que escribe algo nuevo, el otro deja de sincronizar **en ambos sentidos** hasta que también se actualice. No se pierde nada y nada llega a medio entender: actualiza el que quedó atrás y los dos siguen donde estaban.

Desde la 1.34, un equipo que sigue con Tisty 1.30 a 1.33 y llega a tu nube o a tu carpeta se detiene y te pide actualizarlo en vez de escribir encima de lo que dejó uno más nuevo. Mientras no lo hagas, ese equipo no sincroniza, pero no pierdes nada: lo que tiene se queda donde estaba y se pone al día en cuanto lo actualices. Una copia de una tienda puede tardar un poco en recibir la versión nueva.

## Cambié algo en un equipo y no aparece en el otro

Con una carpeta, Tisty escribe y lee en ella, y **quien la sube es el programa de tu proveedor**: Google Drive, OneDrive, iCloud, Dropbox, Syncthing. Si ese programa no está corriendo, Tisty sigue guardando tu trabajo sin problema y no tiene cómo saber que nadie lo está recogiendo. Si en cambio conectaste Google Drive, OneDrive o un servidor WebDAV directo, Tisty te dice qué pasa en su propia tarjeta: mira [Lo que dice la tarjeta](#lo-que-dice-la-tarjeta).

Revisa en este orden; los tres primeros cubren casi todos los casos.

1. **¿Está corriendo el programa de tu nube?** Busca su icono en la barra de menús, arriba a la derecha (macOS), o junto al reloj, incluidos los iconos ocultos (Windows). Si su inicio automático está apagado, basta un reinicio para que no arranque. Ábrelo y dale unos minutos.
2. **¿Está en pausa o con la sesión cerrada?** Un cliente abierto no siempre está sincronizando: revisa en su propio panel que no diga en pausa, que la cuenta siga con la sesión iniciada y que no te hayas quedado sin espacio.
3. **¿Esa carpeta está entre las que sincroniza?** Muchos clientes te dejan elegir carpetas. Si la que le diste a Tisty quedó fuera de esa selección, todo funciona menos lo único que importa.
4. **¿El otro equipo está encendido?** Tiene que abrir Tisty para recoger los cambios. Mientras está apagado, tu trabajo espera en la carpeta.
5. **¿El otro equipo te está esperando?** En **Configuración › Sincronizar › Tus equipos** aparece como **Esperando**: confírmalo (arriba), revisando que la clave sea la misma carácter por carácter. Lo que escribió entretanto se queda en la carpeta y llega entero.
6. **Revísalo con tus propios ojos.** Configuración › Sincronizar muestra dos fechas: **mi última ronda** (cuándo Tisty leyó y escribió en la carpeta; *no* significa que algo se haya subido) y **lo último que llegó de otro equipo**. Si la segunda lleva días quieta y tus equipos están encendidos, el problema está ahí. **Abrir la carpeta** te la muestra: busca ahí tus archivos recientes y revisa con tu proveedor, desde su web por ejemplo, si también están allá arriba.

Si todo eso está bien, **Configuración › Sincronizar › Volver a enviar todo** copia de nuevo a la carpeta todo lo de este equipo sin preguntar si ya está ahí, que es justo lo que necesitas cuando un cliente de nube se saltó un archivo.

### ¿Por qué Tisty no me avisa solo?

Porque no puede saberlo. Con una carpeta, Tisty no habla con ninguna nube: no tiene una cuenta tuya, ni credenciales, ni idea de qué proveedor (si es que hay alguno) está detrás de la carpeta. Tampoco le pregunta al programa de tu proveedor, así que te muestra los datos que sí conoce en vez de un veredicto que no puede respaldar.

### ¿Se pierde algo mientras tanto?

No. Tu trabajo está completo en el equipo donde lo hiciste, y el registro solo añade: nada se sobrescribe ni se descarta. En cuanto la carpeta vuelve a moverse, los dos equipos se ponen al día solos, aunque los dos hayan estado trabajando a la vez. Si los dos escribieron la misma parte de un documento, tú eliges cuál se queda (mira [más arriba](#cuando-dos-equipos-escriben-el-mismo-documento)).
