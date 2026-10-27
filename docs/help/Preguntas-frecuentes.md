# Preguntas frecuentes

## Lo básico

**¿Tisty es gratis?**
Sí, y lo va a seguir siendo. AGPL-3.0, sin versión de pago, sin funciones guardadas para después, sin prueba que caduque. Los [términos comerciales](https://github.com/rgdevment/Tisty/blob/main/docs/COMMERCIAL.md) existen solo para organizaciones que no pueden cumplir la AGPL.

**¿Necesita cuenta o conexión?**
Ninguna de las dos. Tisty hace una sola petición que no pediste (ver si hay una versión más nueva) y no envía nada para hacerla. Desconecta el equipo de la red y todo lo demás funciona igual. Mira [[Privacidad]].

**¿Dónde se guardan mis datos?**
En una carpeta de tu propio disco. **Configuración › Tus datos** muestra la ruta, y *Mostrar carpeta* la abre. Mira [Instalación](Instalación#dónde-viven-tus-datos).

**¿Qué pasa con mis notas si Tisty desaparece?**
Se quedan donde están: Markdown y texto plano, que se leen con `cat`, se buscan con `grep` y se abren con cualquier editor.

**¿Puedo exportar mis tareas?**
No hay un botón de exportar porque no hay nada que convertir: ya son archivos abiertos. Los documentos son Markdown en `docs/`. Las tareas viven en `store/`, una carpeta por equipo, como un historial que solo crece, con un evento JSON por línea, así que cualquier herramienta las lee. Por ejemplo, cada tarea que se creó alguna vez:

```sh
cat store/*/*.tisty | jq -c 'select(.op == "task.add") | {id, title: .d.title}'
```

Eso es el historial, no el estado de hoy: un evento posterior renombra, termina o mueve una tarea. Para ver las tareas como están ahora, pídeselas a tu asistente por [[MCP|Asistentes-y-MCP]], o **Configuración › Tus datos** escribe un zip con todo. Si necesitas un formato concreto para llevarte Tisty a otro lado, dilo en un issue.

**¿Mis datos están cifrados?**
En tu disco no, por decisión: la protección son los permisos de tu sistema operativo, y los archivos se pueden leer con herramientas que ya tienes. En una nube con la que Tisty habla directo, sí pueden estarlo: mira [Cifrar tu nube](Sincronizar-y-respaldar#cifrar-tu-nube). [PRIVACY.md](https://github.com/rgdevment/Tisty/blob/main/PRIVACY.md) explica por qué.

**¿Puedo usarlo con mi equipo de trabajo?**
No. Tisty es para una persona por diseño: sin asignaciones, sin permisos, sin tableros.

**¿Hay app móvil? ¿Linux?**
No hay app móvil: Tisty es un programa de escritorio para Windows y macOS. Tus documentos son Markdown en una carpeta, así que un teléfono que llegue a esa carpeta puede leerlos y editarlos con cualquier app. Tampoco hay versión para Linux; un asistente que corre en Linux dentro de Windows, con WSL, llega a Tisty igual desde Configuración › Asistentes.

**¿Funciona con un lector de pantalla?**
Que se sepa, no. Toda la ventana funciona con el teclado y ese camino está probado, pero nunca se ha probado con un lector de pantalla real. Si usas uno, un [reporte](https://github.com/rgdevment/Tisty/issues) que diga dónde falla sería de lo más útil.

## IA y asistentes

**¿Tisty usa IA?**
No, y no la va a usar. El lenguaje natural son reglas en tu equipo. Si usas un asistente propio, Tisty deja una [[puerta MCP|Asistentes-y-MCP]]: tú la abres y la puedes cerrar.

**Un asistente dice que una de sus tareas está hecha. ¿La cerró?**
No. Nada de lo que hace un asistente cierra una tarea. La marca, con su cuenta en la bitácora, y la tarea espera en **Por confirmar** hasta que decidas. Mira [[Qué puede hacer un asistente|Qué-puede-hacer-un-asistente]].

**¿Un asistente puede escribir encima de un documento que estoy editando?**
Solo encima de la versión que leyó. Para reemplazar un documento entero tiene que devolver la huella del texto que leyó; si escribiste entretanto, no se escribe nada y tiene que volver a leer. Lo que reemplazó queda guardado, y la ventana te ofrece volver atrás, solo cuando esa copia está exactamente una escritura atrás, que es la única vez en que puede hacer lo que dice. Si intentas guardar encima de un cambio que no habías visto, te pregunta cuál se queda.

## Documentos

**¿Puedo traer mis notas desde otra app?**
Los documentos sí: *Importar un documento* toma `.md`, `.markdown`, `.txt`, `.html` o de Word (`.docx`), de a uno, y lo pasa a Markdown: de Word a Markdown, de HTML a Markdown, de Obsidian a Markdown. Pegar desde Word, Google Docs o una página web también lo convierte. Un `.tistyx` de otro Tisty se desempaca entero. Las tareas hay que escribirlas. Mira [Documentos](Documentos#importar).

**¿Puedo leer un archivo Markdown sin traerlo a Tisty?**
Sí: elige Tisty en **Abrir con**, o sigue un enlace `tisty://open?path=…`. Se abre en solo lectura en una ventana propia, y **Editar en Tisty** lo trae cuando quieras. Mira [Documentos](Documentos#abrir-un-archivo-markdown-de-afuera).

**Un documento se abrió solo para leer y yo no lo pedí.**
Está archivado, o lo está una página suya (o su documento). La barra del pie dice cuál es. Un documento con etiquetas HTML no queda solo para leer: se abre, se edita y se guarda entero. Mira [El editor Markdown](Editor-Markdown#cuando-un-documento-se-abre-solo-para-leer).

**¿Los diagramas y las fórmulas necesitan internet?**
No. Los dos se dibujan con código que va dentro de Tisty: no se descarga nada, no se carga ninguna fuente de ninguna parte y funcionan con la red desconectada. Los dos siguen siendo Markdown que GitHub también dibuja. Mira [diagramas, fórmulas y widgets](https://markdown.tisty.app/es/diagramas-formulas-y-widgets) en la guía de Markdown de Tisty.

**¿Por qué sobrevive el ancho de las columnas de mi tabla, y dónde se guarda?**
En lo larga que se dibuja la raya bajo el encabezado. Mira [tablas](https://markdown.tisty.app/es/tablas) en la guía de Markdown de Tisty.

## Sincronización

**¿Puedo usarlo en dos equipos?**
Sí, a través de una carpeta que ya sincronizas o directamente con tu Google Drive, tu OneDrive o tu servidor WebDAV. Mira [[Sincronizar y respaldar|Sincronizar-y-respaldar]].

**Cambié algo en un equipo y no aparece en el otro.**
Casi siempre el programa de la nube no está corriendo, está en pausa o no sincroniza esa carpeta. La lista completa está en [Sincronizar y respaldar](Sincronizar-y-respaldar#cambié-algo-en-un-equipo-y-no-aparece-en-el-otro).

**¿Puedo sincronizar con Google Drive sin el programa de Google?**
Sí, desde la 1.30: **Configuración › Sincronizar › Tu servicio de nube › Google Drive › Conectar**. Tisty pide solo un espacio oculto propio en tu Drive, no ve nada más ahí y te dice en su tarjeta cuando algo anda mal. Qué significa cada aviso está en [Sincronizar y respaldar](Sincronizar-y-respaldar#sincronizar-con-google-drive-directamente).

**¿Dónde están mis copias en Google Drive, y cómo las borro?**
En un espacio oculto de tu Drive que solo abre Tisty. Desconectar las conserva, y también *Desconectar de Drive* en **Google Drive › Configuración › Administrar apps**, que solo retira el acceso de Tisty. Solo **«Eliminar datos ocultos de la app»** ahí, o **Borrar para siempre mis copias en Google Drive** en Tisty, las borra, y ese espacio no tiene papelera.

**¿Puedo sincronizar con OneDrive sin el programa de Microsoft?**
Sí, desde la 1.31: **Configuración › Sincronizar › Tu servicio de nube › OneDrive › Conectar**. Tisty guarda sus copias en *Aplicaciones › Tisty*, una carpeta que puedes ver en tu OneDrive; déjasela a Tisty. Mira [Sincronizar y respaldar](Sincronizar-y-respaldar#sincronizar-con-onedrive-directamente).

**¿Puedo sincronizar con mi propio servidor?**
Sí, desde la 1.32, por WebDAV: Nextcloud, ownCloud, Synology y otros. En **Configuración › Sincronizar › Tu servicio de nube › WebDAV** escribes la dirección del servidor, tu usuario y una contraseña, y Tisty revisa el servidor antes de conectarse; no se abre ningún navegador. La contraseña queda en este equipo, guardada por el sistema, y las copias van a una carpeta *Tisty* del servidor. Solo se acepta `https://`, salvo hacia este mismo equipo. Mira [Sincronizar y respaldar](Sincronizar-y-respaldar#sincronizar-con-tu-propio-servidor-webdav).

**¿Puedo cifrar mi nube?**
Sí, desde la 1.36, con Google Drive, OneDrive o WebDAV: **Configuración › Seguridad › Cifrar mi nube…** cifra lo que Tisty guarda ahí con una frase de 24 palabras que solo tú conoces, así que el proveedor ya no puede leer tus tareas ni tus documentos. Una carpeta que otro programa mantiene al día no se cifra. Tus otros equipos reciben la llave solos; uno nuevo se autoriza con veinte dígitos que escribes en un equipo que ya tiene la llave, o con tu frase. El proveedor sigue viendo los nombres y tamaños de los archivos, cuántos equipos tienes y, por un tiempo, versiones anteriores sin cifrar. Sin la frase o sin uno de tus equipos, nadie puede abrirla, ni siquiera nosotros. Mira [Cifrar tu nube](Sincronizar-y-respaldar#cifrar-tu-nube).

**Tisty dice que algo está bloqueando Google Drive.**
Un proxy, un filtro de tu trabajo o un antivirus se interpone entre este equipo y Google, o revisa las conexiones seguras de modo que Tisty no puede confiar en ellas. Mientras tanto, tu trabajo queda a salvo en este equipo. En una red de trabajo, pide a quien la administra que deje pasar a Tisty; en tu propio equipo, permite a Tisty en tu antivirus. Nunca rodees por tu cuenta la protección de tu empresa. Qué significa cada aviso está en [Sincronizar y respaldar](Sincronizar-y-respaldar#cuando-algo-se-interpone-con-google).

**Uno de mis equipos dejó de sincronizar después de una actualización.**
Actualiza el que quedó atrás. Un equipo solo lee lo que entiende, así que los dos dejan de sincronizar en ambos sentidos hasta que coinciden. Un equipo que sigue con Tisty 1.30 a 1.33 se detiene y te pide actualizarlo en vez de escribir encima de uno más nuevo. No se pierde nada, y lo que escribas entretanto se queda en ese equipo.

## Comparación

**¿En qué se diferencia de Obsidian, Notion o Todoist?**
Tisty guarda tareas y documentos en un solo almacén con un solo buscador, y terminar una tarea guarda todo lo que juntó en vez de esconderlo. El README tiene [una tabla comparativa](https://github.com/rgdevment/Tisty/blob/main/README.es.md) que también dice en qué es mejor cada uno de los otros.
