#!/usr/bin/env node
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const TONGUES = ["es", "en"];
const SPAN_DAYS = 730;

const flags = new Map();
const loose = [];
for (const one of process.argv.slice(2)) {
  const hit = /^--([a-z][a-z-]*)(?:=(.*))?$/.exec(one);
  if (hit) flags.set(hit[1], hit[2] ?? "");
  else loose.push(one);
}
const tongue = (flags.get("lang") || loose.find((one) => TONGUES.includes(one)) || "es").toLowerCase();
const profile = loose.find((one) => !TONGUES.includes(one)) || "showcase";

if (!TONGUES.includes(tongue)) {
  console.error(`unknown language: ${tongue} (${TONGUES.join(" · ")})`);
  process.exit(1);
}
if (!/^[A-Za-z0-9_-]{1,48}$/.test(profile)) {
  console.error(`unusable profile name: ${profile}`);
  process.exit(1);
}

const md = (...lines) => lines.join("\n");

const FOLDERS = [
  { key: "work", icon: "💼", color: "blue" },
  { key: "minutes", icon: "📝", color: "indigo", inside: "work" },
  { key: "projects", icon: "🚀", color: "purple", inside: "work" },
  { key: "infra", icon: "🗄️", color: "teal", inside: "projects" },
  { key: "design", icon: "📐", color: "pink", inside: "work" },
  { key: "home", icon: "🏠", color: "amber" },
  { key: "kitchen", icon: "🍞", color: "orange", inside: "home" },
  { key: "rina", icon: "🐕", color: "brown", inside: "home" },
  { key: "money", icon: "💰", color: "green" },
  { key: "learning", icon: "📚", color: "indigo" },
  { key: "rust", icon: "🦀", color: "red", inside: "learning" },
  { key: "music", icon: "🎸", color: "purple", inside: "learning" },
  { key: "travel", icon: "🧭", color: "teal" },
  { key: "lisbon", icon: "🏛️", color: "amber", inside: "travel" },
  { key: "health", icon: "🩺", color: "red" },
];

const PAPERS = [
  { key: "minutes", folder: "minutes", ago: 706 },
  { key: "minutesOne", pageOf: "minutes", ago: 700 },
  { key: "minutesTwo", pageOf: "minutes", ago: 520 },
  { key: "minutesThree", pageOf: "minutes", ago: 240 },
  { key: "server", folder: "infra", ago: 648 },
  { key: "serverBackup", pageOf: "server", ago: 640 },
  { key: "serverAlerts", pageOf: "server", ago: 410 },
  { key: "deploy", folder: "infra", ago: 505 },
  { key: "runbook", folder: "infra", ago: 118 },
  { key: "roadmap", folder: "projects", ago: 470 },
  { key: "decisions", folder: "projects", ago: 132 },
  { key: "worknotes", folder: "work", ago: 196 },
  { key: "houseManual", folder: "home", ago: 452 },
  { key: "sourdough", folder: "kitchen", ago: 604 },
  { key: "preserves", folder: "kitchen", ago: 210 },
  { key: "rina", folder: "rina", ago: 662 },
  { key: "rinaFirst", pageOf: "rina", ago: 655 },
  { key: "rinaSecond", pageOf: "rina", ago: 290 },
  { key: "budget", folder: "money", ago: 560 },
  { key: "taxes", folder: "money", ago: 148 },
  { key: "rustNotes", folder: "rust", ago: 396 },
  { key: "rustErrors", folder: "rust", ago: 164 },
  { key: "guitar", folder: "music", ago: 322 },
  { key: "scales", folder: "music", ago: 96 },
  { key: "lisbon", folder: "lisbon", ago: 246 },
  { key: "lisbonFood", folder: "lisbon", ago: 238 },
  { key: "checkups", folder: "health", ago: 176 },
];

const ROUTINES = [
  { key: "plants", every: 7, turns: 20, skipAt: -1 },
  { key: "backups", every: 7, turns: 24, skipAt: 9 },
  { key: "rent", every: 30, turns: 24, skipAt: -1 },
  { key: "carService", every: 180, turns: 4, skipAt: -1 },
  { key: "domain", every: 360, turns: 2, skipAt: -1 },
];

const ES = {
  lists: ["Casa", "Trabajo", "Salud", "Finanzas", "Estudio"],
  folders: {
    work: "Trabajo",
    minutes: "Actas",
    projects: "Proyectos",
    infra: "Infraestructura",
    design: "Diseño",
    home: "Casa",
    kitchen: "Cocina",
    rina: "Rina",
    money: "Finanzas",
    learning: "Aprendizaje",
    rust: "Rust",
    music: "Música",
    travel: "Viajes",
    lisbon: "Lisboa",
    health: "Salud",
  },
  papers: {
    minutes: md(
      "# Actas del equipo",
      "",
      "Cada acta ocupa su propia página. Aquí queda lo que no cambia de una reunión a otra: quién convoca, cómo se escribe y qué se hace con lo que queda abierto.",
      "",
      "## Cómo funciona",
      "",
      "- La reunión dura media hora y empieza puntual.",
      "- Quien convoca escribe el acta el mismo día, no al día siguiente.",
      "- Un acuerdo sin dueño y sin fecha vuelve a la lista de abajo.",
      "",
      "| Papel | Quién | Suplente |",
      "| :--- | :--- | ---: |",
      "| Convoca | Ana | Bruno |",
      "| Escribe | rota | — |",
      "| Cierra acuerdos | Marta | Ana |",
      "",
      "> [!NOTE]",
      "> Un acuerdo sin dueño no es un acuerdo: es una idea con buena intención.",
      "",
      "## Lo que sigue abierto",
      "",
      "- [x] Migrar el histórico de facturación",
      "- [x] Renovar el contrato de soporte",
      "- [ ] Sustituir la impresora del segundo piso",
      "- [ ] Decidir quién lleva la relación con el proveedor nuevo",
      ""
    ),
    minutesOne: md(
      "# Acta: cambio de tarifas",
      "",
      "Estuvimos Ana, Bruno y yo. Media hora, sin pantalla compartida.",
      "",
      "## Lo que quedó decidido",
      "",
      "- El cambio de tarifas entra **el primer día del trimestre**, no antes.",
      "- Bruno prepara el correo a los clientes con dos semanas de aviso.",
      "- Ana revisa los contratos que vencen antes del cambio.",
      "",
      "## Lo que quedó en el aire",
      "",
      "| Tema | Quién | Cuándo |",
      "| :--- | :--- | ---: |",
      "| Migrar el histórico | sin dueño | por definir |",
      "| Sustituir la impresora | Ana | cuando llegue el presupuesto |",
      "",
      "> No se toca el sistema viejo hasta que la migración esté probada.",
      "",
      "La fecha que no se mueve es ==la del primer día del trimestre==; la que sigue en el aire es <mark data-pen=\"blue\">la impresora</mark>.",
      ""
    ),
    minutesTwo: md(
      "# Acta: revisión de mitad de año",
      "",
      "Reunión larga, la única del año que se pasa de la media hora. Vinieron los cuatro.",
      "",
      "## Números que miramos",
      "",
      "| Indicador | Meta | Real |",
      "| :--- | ---: | ---: |",
      "| Incidencias abiertas | 12 | 9 |",
      "| Tiempo medio de respuesta | 4 h | 5,2 h |",
      "| Clientes que renovaron | 85 % | 88 % |",
      "",
      "El tiempo de respuesta es el único que no cumple, y sabemos por qué: las dos semanas de vacaciones no se cubrieron.",
      "",
      "## Acuerdos",
      "",
      "1. Cubrir el turno de vacaciones con una rotación escrita, no de palabra.",
      "2. Bajar el objetivo de incidencias abiertas a diez.",
      "3. Dejar de contar los tickets duplicados como incidencias.",
      "",
      "> [!IMPORTANT]",
      "> La rotación se escribe antes de que alguien pida vacaciones, no después.",
      ""
    ),
    minutesThree: md(
      "# Acta: cierre del proyecto de migración",
      "",
      "Última reunión sobre la migración. Se cerró en dos meses menos de lo previsto porque el histórico resultó más limpio de lo que temíamos.",
      "",
      "## Qué se migró",
      "",
      "- Facturación completa, desde el primer registro.",
      "- Contactos, sin los duplicados que arrastrábamos.",
      "- Adjuntos, salvo los que ya no apuntaban a nada.",
      "",
      "## Qué aprendimos",
      "",
      "- Medir antes de estimar habría ahorrado la mitad de las reuniones.",
      "- La ventana de corte del fin de semana fue innecesaria: bastaba con una noche.",
      "",
      "> [!TIP]",
      "> El sistema viejo queda de solo lectura seis meses más. Después se apaga y se guarda una copia fría.",
      ""
    ),
    server: md(
      "# El servidor de casa",
      "",
      "Lo que hay que recordar cuando algo deja de responder. Nada de esto es *urgente* hasta que lo es, y entonces no hay tiempo de averiguarlo.",
      "",
      "## Qué corre ahí",
      "",
      "| Servicio | Puerto | Arranca solo | Depende de |",
      "| :--- | :---: | :---: | ---: |",
      "| respaldo | 8384 | sí | disco externo |",
      "| biblioteca | 8096 | sí | respaldo |",
      "| dns | 53 | sí | — |",
      "| panel | 9090 | no | — |",
      "",
      "La **biblioteca** depende del respaldo porque lee de la misma unidad. Si el disco no monta, los dos caen juntos y el orden de arranque importa.",
      "",
      "```mermaid title=\"Arranque y dependencias\"",
      "flowchart TD",
      "  D[Disco externo] --> R[Respaldo]",
      "  R --> B[Biblioteca]",
      "  N[Red] --> S[DNS]",
      "  N --> P[Panel]",
      "  R -.-> A[Aviso al telefono]",
      "```",
      "",
      "## Cuando el disco se llena",
      "",
      "```bash title=\"limpiar.sh\"",
      "docker system prune -a --volumes",
      "journalctl --vacuum-time=7d",
      "find /var/lib/media/cache -type f -mtime +30 -delete",
      "```",
      "",
      "Antes de podar conviene saber cuánto crece al día. Es una resta y una división, pero escrita se recuerda mejor:",
      "",
      "```math title=\"Crecimiento diario en GiB\"",
      "g = \\frac{b_{hoy} - b_{ayer}}{1024^3}",
      "```",
      "",
      "Con `g` por encima de dos hay algo escribiendo de más, y casi siempre son los registros de un contenedor que se reinicia en bucle.",
      "",
      "> [!CAUTION]",
      "> Nunca podar sin comprobar antes que la última copia terminó bien. Una poda tarda un minuto; rehacer una copia de dos terabytes tarda una noche.",
      "",
      "## Repaso trimestral",
      "",
      "- [x] Comprobar el espacio libre de cada volumen",
      "- [x] Restaurar un archivo al azar desde la copia",
      "- [ ] Revisar los certificados que vencen",
      "- [ ] Actualizar el sistema base",
      "",
      "La documentación de la herramienta de sincronización está en [rsync.samba.org](https://rsync.samba.org), y el manual del disco lo tengo en papel en el cajón de abajo.",
      ""
    ),
    serverBackup: md(
      "# Respaldos y restauración",
      "",
      "Tres copias, dos medios, una fuera de casa. Lo demás es opinión.",
      "",
      "| Copia | Dónde | Cada cuánto | Retención |",
      "| :--- | :--- | :---: | ---: |",
      "| caliente | disco externo | diaria | 30 días |",
      "| fría | disco que vive en la oficina | mensual | 12 meses |",
      "| remota | almacenamiento cifrado | semanal | 90 días |",
      "",
      "```bash title=\"respaldo.sh\"",
      "set -euo pipefail",
      "rsync -a --delete --link-dest=../ultimo ~/datos/ /mnt/copias/$(date +%F)/",
      "ln -sfn /mnt/copias/$(date +%F) /mnt/copias/ultimo",
      "```",
      "",
      "> [!WARNING]",
      "> Una copia que nunca se ha restaurado no es una copia. Una vez al trimestre saco un archivo cualquiera y compruebo que se abre.",
      ""
    ),
    serverAlerts: md(
      "# Cuando algo deja de responder",
      "",
      "El orden importa: casi siempre es la capa de abajo la que falló, y mirar primero la de arriba cuesta media hora.",
      "",
      "1. ¿Hay corriente y hay red? El cable del disco ya falló dos veces.",
      "2. ¿El disco está montado? Si no, nada de lo que corre encima va a arrancar.",
      "3. ¿Queda espacio? Un volumen lleno se comporta como un servicio caído.",
      "4. Recién entonces, los registros del servicio.",
      "",
      "```bash",
      "systemctl --failed",
      "df -h /mnt/datos",
      "journalctl -u biblioteca -n 200 --no-pager",
      "```",
      "",
      "> [!TIP]",
      "> Si el aviso llegó al teléfono pero el servicio responde, el problema es el aviso, no el servicio. Eso también se anota.",
      ""
    ),
    deploy: md(
      "# Despliegue, paso a paso",
      "",
      "Se despliega a media mañana, nunca a última hora. Si algo sale mal quiero el día por delante, no la noche.",
      "",
      "1. Ejecutar la batería de pruebas completa en local.",
      "2. Etiquetar la versión y esperar a que el servidor de integración termine.",
      "3. Poner el aviso de mantenimiento, si el cambio toca la base de datos.",
      "4. Desplegar, mirar los registros dos minutos y quitar el aviso.",
      "",
      "```bash title=\"desplegar.sh\"",
      "cargo test --workspace",
      "git tag -a \"v$1\" -m \"version $1\"",
      "ssh servidor 'cd /srv/app && ./actualizar.sh'",
      "```",
      "",
      "> [!IMPORTANT]",
      "> La vuelta atrás tiene que estar probada antes del despliegue, no pensada durante.",
      ""
    ),
    runbook: md(
      "# Qué mirar cuando algo falla",
      "",
      "Un guion corto para no improvisar con el pulso alto.",
      "",
      "| Síntoma | Primera sospecha | Comprobación |",
      "| :--- | :--- | :--- |",
      "| Lentitud general | disco lleno | `df -h` |",
      "| Errores intermitentes | reinicio en bucle | registros del servicio |",
      "| Nada responde | red o corriente | ping al equipo |",
      "",
      "Lo que aprendí a la mala: **anotar la hora exacta** en que empezó. Sin esa hora, los registros son un pajar.",
      ""
    ),
    roadmap: md(
      "# Plan del proyecto",
      "",
      "Tres bloques, en este orden. Ninguno empieza sin que el anterior esté cerrado de verdad.",
      "",
      "## Bloque uno: dejar de perder datos",
      "",
      "- Copias automáticas y probadas.",
      "- Un registro de cambios que se pueda leer.",
      "",
      "## Bloque dos: que otra persona pueda operarlo",
      "",
      "- Documentar el despliegue y la vuelta atrás.",
      "- Sacar las contraseñas de los archivos de configuración.",
      "",
      "## Bloque tres: que sea agradable de usar",
      "",
      "- Pantalla de estado con lo que importa y nada más.",
      "- Avisos que no se ignoren por costumbre.",
      "",
      "> [!NOTE]",
      "> El tercer bloque es el que más ganas dan de empezar y el que peor envejece si los otros dos no están.",
      ""
    ),
    decisions: md(
      "# Decisiones que ya no se discuten",
      "",
      "Cada una con su fecha y su motivo. Si el motivo deja de valer, la decisión se revisa; mientras tanto, no se vuelve a abrir.",
      "",
      "## Guardar el histórico como registro de eventos",
      "",
      "Se eligió porque el estado se puede reconstruir y porque un error se corrige añadiendo, no borrando. El costo es que el archivo crece siempre.",
      "",
      "## Nada de servidores propios",
      "",
      "Todo corre en el equipo de quien lo usa. Sincronizar es cosa de la carpeta compartida que ya exista, no nuestra.",
      "",
      "## Una sola base de datos, en archivo",
      "",
      "| Opción | Por qué no |",
      "| :--- | :--- |",
      "| Base de datos en red | obliga a un servidor |",
      "| Un archivo por entidad | se rompe al sincronizar |",
      "",
      "> [!CAUTION]",
      "> Cambiar cualquiera de estas tres obliga a migrar datos de gente real. No se toca sin un plan de vuelta atrás escrito.",
      ""
    ),
    worknotes: md(
      "# Notas sueltas de trabajo",
      "",
      "Lo que no merece un documento propio pero se olvida en dos semanas.",
      "",
      "- El proveedor de correo corta los envíos masivos a partir de quinientos por hora.",
      "- El contrato de soporte cubre incidencias, no cambios: pedir cambios por el otro canal.",
      "- La sala grande hay que reservarla con antelación; la chica no.",
      "",
      "Frase que vale para casi todo aquí: *si no está escrito, hay que preguntarlo dos veces*.",
      ""
    ),
    houseManual: md(
      "# El manual de la casa",
      "",
      "Para quien se quede cuidando la casa, y para mí mismo cuando no me acuerde.",
      "",
      "| Cosa | Dónde | Detalle |",
      "| :--- | :--- | :--- |",
      "| Llave de paso del agua | bajo el lavaplatos | gira a la derecha |",
      "| Tablero eléctrico | pasillo | el interruptor grande es el general |",
      "| Filtros del extractor | cajón alto | se lavan cada dos meses |",
      "",
      "## Basura",
      "",
      "- Orgánica: martes y viernes por la noche.",
      "- Reciclaje: solo los viernes.",
      "",
      "> [!TIP]",
      "> El portero tiene una copia de la llave. Si la puerta se traba, es cuestión de levantarla un poco mientras giras.",
      ""
    ),
    sourdough: md(
      "# Pan de masa madre",
      "",
      "La masa lleva viva desde el primer intento. Refrescarla la noche antes.",
      "",
      "## Ingredientes",
      "",
      "| Qué | Cuánto |",
      "| :--- | ---: |",
      "| Harina de fuerza | 500 g |",
      "| Agua templada | 350 g |",
      "| Masa madre activa | 100 g |",
      "| Sal | 10 g |",
      "",
      "## Cómo va",
      "",
      "1. Mezclar harina y agua, y dejarlo reposar cuarenta minutos.",
      "2. Añadir la masa madre y la sal.",
      "3. Tres pliegues, uno cada media hora.",
      "4. Levar en frío toda la noche.",
      "5. Horno a 250 °C con vapor los primeros veinte minutos.",
      "",
      "La última vez salió apretada por meterla al horno demasiado pronto. Lo único que merece quedar escrito: <mark data-pen=\"green\">fermentar en frío</mark>, nunca sobre la mesa de la cocina.",
      ""
    ),
    preserves: md(
      "# Conservas de temporada",
      "",
      "Lo que se guarda y cuándo conviene comprarlo barato.",
      "",
      "- Tomate: al final del verano, cuando sobra y baja de precio.",
      "- Duraznos en almíbar: dos semanas antes que el tomate.",
      "- Pimientos asados: siguen a los tomates.",
      "",
      "> [!WARNING]",
      "> Los frascos hierven veinte minutos después de cerrados, y se guardan solo si la tapa quedó hundida. Si suena al apretarla, va al refrigerador y se come esa semana.",
      ""
    ),
    rina: md(
      "# Rina, año a año",
      "",
      "La vida de la perra contada por años, un capítulo por página. Aquí queda lo que no cambia: quién la atiende, qué come y qué le pasa cuando se queda sola.",
      "",
      "| Dato | Cuál |",
      "| :--- | :--- |",
      "| Veterinaria | Clínica del Sur |",
      "| Alimento | seco, dos tazas al día |",
      "| Alergias | ninguna conocida |",
      "",
      "> [!NOTE]",
      "> Si se queda sola más de seis horas, ladra. No es ansiedad: es aburrimiento, y se arregla con un paseo largo antes.",
      ""
    ),
    rinaFirst: md(
      "# El primer año",
      "",
      "Llegó pesando poco más de tres kilos y con un miedo cerval a las escaleras.",
      "",
      "- [x] Vacunas al día",
      "- [x] Perder el miedo a las escaleras",
      "- [x] Aprender a quedarse sola una hora",
      "- [ ] Dejar de perseguir motos",
      "",
      "Lo de las escaleras se resolvió en tres semanas, subiendo un escalón por día con premio. Lo de las motos sigue igual.",
      ""
    ),
    rinaSecond: md(
      "# El segundo año",
      "",
      "Ya pesa lo que va a pesar y duerme donde quiere.",
      "",
      "| Control | Cuándo | Resultado |",
      "| :--- | :--- | ---: |",
      "| Vacuna anual | otoño | sin novedad |",
      "| Limpieza dental | invierno | sarro leve |",
      "| Antiparasitario | cada tres meses | al día |",
      "",
      "Lo que cambió este año: aprendió que la mochila significa paseo, y ahora se sienta al lado en cuanto la ve.",
      ""
    ),
    budget: md(
      "# Presupuesto del año",
      "",
      "Una hoja para saber si el año cierra o no. No es contabilidad: es una **regla de tres honesta** que se revisa cada trimestre.",
      "",
      "## Cómo se reparte",
      "",
      "| Categoría | Objetivo | Real | Diferencia |",
      "| :--- | ---: | ---: | ---: |",
      "| Vivienda | 30 % | 32 % | +2 |",
      "| Alimentación | 18 % | 17 % | −1 |",
      "| Transporte | 10 % | 8 % | −2 |",
      "| Salud | 8 % | 9 % | +1 |",
      "| Ahorro | 20 % | 18 % | −2 |",
      "| Todo lo demás | 14 % | 16 % | +2 |",
      "",
      "La vivienda se pasa siempre, y no hay mucho que hacer con eso mientras el contrato dure. Lo que sí se puede mover es *todo lo demás*, que es donde se esconden las suscripciones olvidadas.",
      "",
      "```mermaid title=\"A dónde va cada peso\"",
      "pie showData",
      "  \"Vivienda\" : 32",
      "  \"Alimentacion\" : 17",
      "  \"Transporte\" : 8",
      "  \"Salud\" : 9",
      "  \"Ahorro\" : 18",
      "  \"Otros\" : 16",
      "```",
      "",
      "## Cuánto rinde el ahorro",
      "",
      "Con un aporte mensual constante y una tasa mensual pequeña, el saldo al cabo de `n` meses no es el aporte por los meses: es bastante más, y esa diferencia es la razón de no dejarlo en la cuenta corriente.",
      "",
      "```math title=\"Ahorro con aportes mensuales\"",
      "S_n = A \\cdot \\frac{(1 + i)^n - 1}{i}",
      "```",
      "",
      "Con `A = 200` y `i = 0,004` mensual, a cinco años el saldo pasa de catorce mil, contra los doce mil que suman los aportes.",
      "",
      "```python title=\"ahorro.py\"",
      "def saldo(aporte, tasa, meses):",
      "    return aporte * ((1 + tasa) ** meses - 1) / tasa",
      "",
      "print(round(saldo(200, 0.004, 60), 2))",
      "```",
      "",
      "> [!IMPORTANT]",
      "> El ahorro se aparta el mismo día que entra el sueldo. Lo que queda al final del mes nunca es lo que sobra: es lo que no se apartó.",
      "",
      "## Revisión trimestral",
      "",
      "- [x] Comparar objetivo y real por categoría",
      "- [x] Dar de baja lo que no se usó en tres meses",
      "- [ ] Renegociar el plan de internet",
      "- [ ] Revisar el seguro antes de que se renueve solo",
      "",
      "Las tasas de referencia las miro en [el sitio del banco central](https://www.bcentral.cl), no en las que anuncia cada banco.",
      ""
    ),
    taxes: md(
      "# Impuestos: lo que hay que juntar",
      "",
      "Se junta durante el año y se ordena en una tarde, no al revés.",
      "",
      "| Documento | De dónde sale | Cuándo llega |",
      "| :--- | :--- | ---: |",
      "| Certificado de sueldos | empleador | comienzo del año |",
      "| Intereses del crédito | banco | comienzo del año |",
      "| Boletas emitidas | portal | todo el año |",
      "| Gastos de salud | clínica | a pedido |",
      "",
      "> [!TIP]",
      "> Todo lo que llega por correo se guarda el mismo día en la carpeta del año. Buscarlo después cuesta tres veces más.",
      ""
    ),
    rustNotes: md(
      "# Rust: notas de lectura",
      "",
      "Lo que fui entendiendo del libro, escrito con mis palabras. Si algo aquí está mal, es que lo entendí mal, no que el libro lo cuente así.",
      "",
      "## Préstamos, en una frase",
      "",
      "Puedes tener **muchas lecturas** o **una escritura**, nunca las dos cosas a la vez. Casi todos los errores del compilador que me costaron una tarde salen de olvidar esa frase.",
      "",
      "```rust title=\"src/paseo.rs\"",
      "#[derive(Debug, Clone, Copy, PartialEq)]",
      "enum Momento {",
      "    Manana,",
      "    Tarde,",
      "}",
      "",
      "fn toca_salir(hora: u8, ya_salio: bool) -> Option<Momento> {",
      "    match (hora, ya_salio) {",
      "        (7..=9, false) => Some(Momento::Manana),",
      "        (20..=22, false) => Some(Momento::Tarde),",
      "        _ => None,",
      "    }",
      "}",
      "```",
      "",
      "El `match` sobre una tupla es lo que más echo de menos en otros lenguajes: el compilador avisa si dejas un caso fuera.",
      "",
      "## Cómo viaja un valor",
      "",
      "```mermaid title=\"Movimiento y prestamo\"",
      "stateDiagram-v2",
      "  [*] --> Propio",
      "  Propio --> Movido: se pasa por valor",
      "  Propio --> Prestado: se pasa por referencia",
      "  Prestado --> Propio: termina el prestamo",
      "  Movido --> [*]",
      "```",
      "",
      "## Por qué el compilador es tan lento",
      "",
      "No es solo el tamaño del código: es cuántas veces se instancia cada función genérica. Con `t` tipos distintos y `f` funciones genéricas, el trabajo crece como el producto, no como la suma:",
      "",
      "```math title=\"Instancias generadas\"",
      "N = \\sum_{k=1}^{f} t_k \\qquad \\text{con } t_k \\ge 1",
      "```",
      "",
      "De ahí la recomendación de que las funciones genéricas grandes llamen a una función interna no genérica: se instancia el envoltorio, no el cuerpo.",
      "",
      "> [!NOTE]",
      "> El libro está en [doc.rust-lang.org/book](https://doc.rust-lang.org/book/) y se lee gratis. La versión en papel sirve para subrayar, que es como se me queda.",
      "",
      "## Pendiente de entender",
      "",
      "- [x] Préstamos y tiempos de vida básicos",
      "- [x] `Result` y el operador de propagación",
      "- [ ] Tiempos de vida en estructuras",
      "- [ ] Cuándo hace falta `Pin`",
      ""
    ),
    rustErrors: md(
      "# Errores que ya me costaron una tarde",
      "",
      "Cada uno con lo que significaba de verdad, no con lo que dice el mensaje.",
      "",
      "| Mensaje | Lo que pasaba |",
      "| :--- | :--- |",
      "| `cannot borrow as mutable` | había una lectura viva más abajo |",
      "| `does not live long enough` | devolvía una referencia a algo local |",
      "| `the trait is not implemented` | faltaba un `use`, no una implementación |",
      "",
      "```diff",
      "- let primero = lista.first();",
      "- lista.push(nuevo);",
      "+ let primero = lista.first().copied();",
      "+ lista.push(nuevo);",
      "```",
      "",
      "El `copied()` corta el préstamo porque copia el valor en vez de quedarse mirando la lista. Media tarde para tres letras.",
      ""
    ),
    guitar: md(
      "# Volver a la guitarra",
      "",
      "Veinte minutos al día valen más que dos horas el domingo. Eso ya lo probé al revés.",
      "",
      "## Rutina corta",
      "",
      "1. Cinco minutos de cuerdas al aire, sin prisa.",
      "2. Diez de la escala que toque esa semana.",
      "3. Cinco de una canción, aunque salga mal.",
      "",
      "> [!TIP]",
      "> El metrónomo lento es aburrido y es lo único que funciona. Subir dos golpes por semana, no más.",
      ""
    ),
    scales: md(
      "# Escalas y digitaciones",
      "",
      "Lo mínimo para no perderme en el mástil.",
      "",
      "| Escala | Empieza en | Se usa para |",
      "| :--- | :---: | :--- |",
      "| Pentatónica menor | quinto traste | casi todo |",
      "| Mayor | tercer traste | melodías |",
      "| Blues | quinto traste | el sonido de siempre |",
      "",
      "La pentatónica es la misma forma repetida cinco veces a lo largo del mástil. Aprender las cinco posiciones cuesta un mes; después el mástil deja de ser un misterio.",
      ""
    ),
    lisbon: md(
      "# Lisboa en cinco días",
      "",
      "Vuelo por la mañana, vuelta el sábado por la tarde. Se camina mucho, así que zapatos cómodos y nada de agenda apretada.",
      "",
      "## Antes de salir",
      "",
      "- [x] Reservar el alojamiento en Alfama",
      "- [x] Avisar en el trabajo",
      "- [ ] Renovar el pasaporte",
      "- [ ] Cambiar algo de efectivo",
      "",
      "## Gastos previstos",
      "",
      "| Concepto | Estimado |",
      "| :--- | ---: |",
      "| Pasajes | 180 |",
      "| Alojamiento, cuatro noches | 320 |",
      "| Comidas | 200 |",
      "",
      "<mark data-pen=\"pink\">El pasaporte es lo que no puede esperar</mark>: seis semanas si va por la vía lenta.",
      ""
    ),
    lisbonFood: md(
      "# Dónde comer en Lisboa",
      "",
      "Anotado de lo que me recomendaron, sin probar todavía.",
      "",
      "- Una tasca cerca del mirador de Graça, de las que no tienen carta.",
      "- El mercado de la ribera, temprano y entre semana.",
      "- Pastelería de barrio, la de la esquina antes de bajar a la plaza.",
      "",
      "> [!NOTE]",
      "> Lo caro está en la calle principal y lo bueno a dos cuadras. Vale para casi cualquier ciudad, pero aquí se nota más.",
      ""
    ),
    checkups: md(
      "# Controles y vacunas",
      "",
      "Un lugar donde mirar antes de pedir hora, para no repetir exámenes.",
      "",
      "| Control | Cada cuánto | Último |",
      "| :--- | :---: | ---: |",
      "| Sangre completa | anual | al día |",
      "| Dentista | semestral | atrasado |",
      "| Vista | bianual | al día |",
      "",
      "> [!IMPORTANT]",
      "> Los resultados se guardan aquí el mismo día que llegan. El sobre de papel se pierde siempre.",
      ""
    ),
  },
  routines: {
    plants: { t: "regar las plantas del balcón", c: "cada domingo", g: ["casa", "jardin"], l: 0 },
    backups: { t: "revisar que los respaldos terminaron bien", c: "cada viernes", g: ["servidor"], l: 1 },
    rent: { t: "pagar el arriendo", c: "cada mes", g: ["finanzas"], l: 3 },
    carService: { t: "mantención del auto", c: "cada 6 meses", g: ["auto"], l: 0 },
    domain: { t: "renovar el dominio y el certificado", c: "cada 12 meses", g: ["servidor"], l: 1 },
  },
  open: [
    { t: "revisar el presupuesto del trimestre", p: "do", l: 1, g: ["trabajo", "finanzas"], d: 0 },
    { t: "enviar la factura pendiente al cliente", p: "do", l: 3, g: ["finanzas"], d: 0 },
    { t: "confirmar la reunión con el proveedor", p: "do", l: 1, g: ["trabajo"], d: 0 },
    { t: "pedir cita con el dentista", p: "do", l: 2, g: ["salud"], d: -2 },
    { t: "renovar el seguro del auto", p: "do", l: 3, g: ["auto", "finanzas"], d: -1 },
    { t: "responder el correo del seguro médico", p: "do", l: 2, g: ["salud"], d: 1 },
    { t: "cerrar el informe de incidencias", p: "do", l: 1, g: ["trabajo"], d: 1, dl: 3 },
    { t: "pagar la cuenta de la luz", p: "do", l: 3, g: ["finanzas", "casa"], d: 2 },
    { t: "llevar el auto a la revisión técnica", p: "do", l: 0, g: ["auto"], d: 3 },
    {
      t: "preparar la presentación del comité",
      p: "do",
      l: 1,
      g: ["trabajo"],
      d: 2,
      dl: 4,
      b: "Veinte minutos, con la parte de números al final. La sala grande hay que reservarla.",
      s: ["reunir las cifras del trimestre", "escribir el guion", "reservar la sala", "ensayar una vez en voz alta"],
      done: 2,
      n: [
        "Marta pasó las cifras corregidas: la caída de julio era un duplicado.",
        "Preguntan por el plan del año que viene, conviene dejar una lámina lista.",
      ],
    },
    { t: "comprar los pasajes del viaje", p: "do", l: 0, g: ["viaje"], d: 4 },
    { t: "firmar el contrato de arriendo", p: "do", l: 0, g: ["casa"], d: 5 },
    { t: "actualizar el respaldo del servidor", p: "do", l: 1, g: ["servidor"], d: 0 },
    { t: "retirar las recetas de la farmacia", p: "do", l: 2, g: ["salud"], d: -3 },
    { t: "revisar el estado de cuenta bancario", p: "do", l: 3, g: ["finanzas"], d: 6 },
    { t: "avisar en el trabajo de las vacaciones", p: "do", l: 1, g: ["trabajo"], d: 7 },
    {
      t: "planificar el viaje de fin de año",
      p: "decide",
      l: 0,
      g: ["viaje", "familia"],
      d: 12,
      b: "Dos semanas, saliendo antes de que suban los pasajes.",
      s: ["comparar precios de pasajes", "elegir alojamiento", "pedir los días en el trabajo"],
      done: 1,
      n: ["Los pasajes suben fuerte a partir de la última semana del mes."],
    },
    { t: "estudiar el temario de la certificación", p: "decide", l: 4, g: ["estudio"], d: 10 },
    { t: "rediseñar la página personal", p: "decide", l: 4, g: ["estudio", "diseño"], d: 20 },
    { t: "leer el libro de arquitectura de software", p: "decide", l: 4, g: ["libros", "estudio"] },
    { t: "ordenar el armario del pasillo", p: "decide", l: 0, g: ["casa"], d: 15 },
    { t: "comparar planes de internet", p: "decide", l: 3, g: ["finanzas", "casa"], d: 9 },
    { t: "armar el plan de ahorro", p: "decide", l: 3, g: ["finanzas"], d: 18 },
    { t: "buscar un curso de fotografía", p: "decide", l: 4, g: ["estudio"] },
    {
      t: "definir la arquitectura del módulo nuevo",
      p: "decide",
      l: 1,
      g: ["trabajo", "codigo"],
      d: 14,
      b: "Hay que decidir si sigue el mismo registro de eventos o si conviene una tabla aparte.",
      s: ["escribir las dos opciones", "medir el costo de migrar", "presentarlo al equipo"],
      done: 1,
      n: ["Bruno prefiere la tabla aparte por lo simple; el costo está en mantener dos formas de leer."],
    },
    { t: "escribir la documentación del despliegue", p: "decide", l: 1, g: ["trabajo", "servidor"], d: 11 },
    { t: "revisar la política de respaldos", p: "decide", l: 1, g: ["servidor"], d: 16 },
    { t: "plantar el huerto del balcón", p: "decide", l: 0, g: ["casa", "jardin"], d: 21 },
    { t: "preparar el examen de inglés", p: "decide", l: 4, g: ["estudio"], d: 25 },
    { t: "elegir el regalo de aniversario", p: "decide", l: 0, g: ["regalos", "familia"], d: 13 },
    { t: "ordenar las fotos del año anterior", p: "decide", l: 0, g: ["fotos"] },
    { t: "pedir presupuesto al pintor", p: "delegate", l: 0, g: ["casa"], d: 5 },
    { t: "encargar la torta de cumpleaños", p: "delegate", l: 0, g: ["familia"], d: 8 },
    { t: "delegar el informe de gastos", p: "delegate", l: 1, g: ["trabajo", "finanzas"], d: 3 },
    { t: "coordinar el retiro del sofá viejo", p: "delegate", l: 0, g: ["casa"], d: 6 },
    { t: "pedir el certificado al contador", p: "delegate", l: 3, g: ["finanzas"], d: 9 },
    { t: "avisar al portero del cambio de horario", p: "delegate", l: 0, g: ["casa"], d: 2 },
    { t: "encargar los materiales del taller", p: "delegate", l: 1, g: ["trabajo"], d: 7 },
    { t: "ordenar los marcadores del navegador", p: "minor", g: ["equipo"] },
    { t: "limpiar la carpeta de descargas", p: "minor", g: ["equipo"] },
    { t: "dar de baja las suscripciones sin uso", p: "minor", l: 3, g: ["finanzas"], d: 17 },
    { t: "cambiar el fondo de pantalla", p: "minor", g: ["equipo"] },
    { t: "probar el juego que quedó pendiente", p: "minor", g: ["ocio"] },
    { t: "revisar la lista de películas pendientes", p: "minor", g: ["ocio"], d: 22 },
    { t: "buscar un profesor de guitarra", l: 4, g: ["musica"] },
    { t: "anotar las ideas para el podcast", g: ["ideas"] },
    { t: "revisar el itinerario del viaje", l: 0, g: ["viaje"], d: 19 },
    { t: "probar la receta de pan integral", l: 0, g: ["cocina"], d: 8 },
    { t: "actualizar el currículum", l: 1, g: ["trabajo"] },
    { t: "escribir a la tía Marta", g: ["familia"], d: 4 },
    { t: "medir el pasillo para la estantería", l: 0, g: ["casa"], d: 6 },
    { t: "revisar las notas de la última consulta", l: 2, g: ["salud"], d: 3 },
  ],
  history: [
    {
      t: "cambiar la cerradura de la puerta",
      p: "do",
      l: 0,
      g: ["casa"],
      b: "La llave costaba girar desde el invierno y terminó por trabarse del todo.",
      s: ["pedir presupuesto", "comprar la cerradura", "cambiarla", "hacer copias"],
      done: 4,
      n: [
        "El cerrajero cobró la mitad que la ferretería por el mismo modelo.",
        "Quedaron tres copias: una para el portero.",
      ],
    },
    { t: "pagar la patente del auto", p: "do", l: 3, g: ["auto", "finanzas"] },
    {
      t: "migrar el histórico de facturación",
      p: "do",
      l: 1,
      g: ["trabajo", "codigo"],
      b: "Doce años de registros, con formatos distintos según la época.",
      s: [
        "exportar del sistema viejo",
        "escribir el convertidor",
        "revisar una muestra",
        "cargar todo",
        "apagar el sistema viejo",
      ],
      done: 5,
      n: [
        "Los registros anteriores al cambio de sistema traían la fecha al revés.",
        "La carga completa tardó cuatro horas, no la noche entera que temíamos.",
      ],
    },
    { t: "renovar el permiso de circulación", p: "do", l: 3, g: ["auto", "finanzas"] },
    { t: "declarar los impuestos del año", p: "do", l: 3, g: ["finanzas"] },
    {
      t: "arreglar la filtración del baño",
      p: "do",
      l: 0,
      g: ["casa"],
      b: "Mancha en el cielo del piso de abajo. El vecino avisó de buena manera.",
      s: ["cortar el agua", "llamar al gasfíter", "revisar el cielo de abajo"],
      done: 3,
      n: ["Era la unión del flexible, no la cañería. Media hora de trabajo."],
    },
    { t: "llevar el auto a la mantención", p: "do", l: 0, g: ["auto"] },
    { t: "renovar el pasaporte", p: "do", l: 0, g: ["viaje"] },
    { t: "contratar el seguro del hogar", p: "do", l: 3, g: ["casa", "finanzas"] },
    {
      t: "preparar la presentación del cierre de año",
      p: "do",
      l: 1,
      g: ["trabajo"],
      b: "Media hora, con espacio para preguntas al final.",
      s: ["juntar los números", "escribir el guion", "ensayar"],
      done: 3,
      n: ["Salió mejor de lo esperado; la parte de números fue la que menos preguntas levantó."],
    },
    { t: "cambiar las ampolletas del pasillo", p: "minor", l: 0, g: ["casa"] },
    { t: "hacer la limpieza dental", p: "do", l: 2, g: ["salud"] },
    { t: "ponerse la vacuna de la influenza", p: "do", l: 2, g: ["salud"] },
    { t: "renovar los lentes", p: "decide", l: 2, g: ["salud"] },
    {
      t: "ordenar el escritorio y los cables",
      p: "minor",
      l: 0,
      g: ["casa", "equipo"],
      b: "Todo entra en una bandeja bajo la mesa y se ve mucho mejor.",
      s: ["comprar la bandeja", "etiquetar los cables", "esconder la regleta"],
      done: 3,
    },
    { t: "revisar el contrato de soporte", p: "decide", l: 1, g: ["trabajo"] },
    {
      t: "escribir la documentación del proyecto",
      p: "decide",
      l: 1,
      g: ["trabajo", "codigo"],
      b: "Faltaba lo básico: cómo se levanta el entorno y cómo se despliega.",
      s: ["describir el entorno local", "documentar el despliegue", "explicar la vuelta atrás"],
      done: 3,
      n: ["La parte de la vuelta atrás obligó a probarla, y ahí salió que faltaba un paso."],
    },
    { t: "actualizar el sistema del servidor", p: "do", l: 1, g: ["servidor"] },
    { t: "renovar el certificado del dominio", p: "do", l: 1, g: ["servidor"] },
    { t: "cambiar el disco del respaldo", p: "do", l: 1, g: ["servidor"] },
    {
      t: "preparar la mudanza de la oficina",
      p: "do",
      l: 1,
      g: ["trabajo"],
      b: "Presupuesto aceptado. El montacargas se reserva con dos días de aviso.",
      s: ["pedir cajas", "avisar al edificio", "contratar el traslado", "desmontar los escritorios"],
      done: 4,
      n: [
        "El traslado cobró menos con embalaje incluido que sin él.",
        "El montacargas del edificio nuevo es más angosto: los escritorios grandes van por la escalera.",
      ],
    },
    { t: "comprar el regalo de cumpleaños", l: 0, g: ["regalos", "familia"] },
    { t: "organizar el almuerzo familiar", l: 0, g: ["familia"] },
    { t: "devolver el libro a la biblioteca", p: "delegate", g: ["libros"] },
    { t: "terminar el curso de bases de datos", p: "decide", l: 4, g: ["estudio"] },
    {
      t: "aprender a hacer pan de masa madre",
      p: "decide",
      l: 0,
      g: ["cocina"],
      b: "Cuatro intentos hasta que salió con miga abierta.",
      s: [
        "conseguir masa madre",
        "hacer el primer intento",
        "corregir la hidratación",
        "repetir con fermentado en frío",
      ],
      done: 4,
      n: [
        "El primero salió apretado por hornear demasiado pronto.",
        "Con fermentado en frío toda la noche cambia por completo.",
      ],
    },
    { t: "cambiar las cuerdas de la guitarra", p: "minor", l: 4, g: ["musica"] },
    { t: "pintar la pared del living", p: "decide", l: 0, g: ["casa"] },
    { t: "revisar la póliza del seguro médico", p: "decide", l: 2, g: ["salud", "finanzas"] },
    { t: "cambiar de plan de teléfono", p: "decide", l: 3, g: ["finanzas"] },
    { t: "hacer el inventario de la bodega", p: "minor", l: 0, g: ["casa"] },
    {
      t: "resolver la caída del sitio",
      p: "do",
      l: 1,
      g: ["servidor", "trabajo"],
      b: "Dos horas abajo. Empezó a media tarde y nadie recibió el aviso.",
      s: ["encontrar la causa", "levantar el servicio", "arreglar el aviso", "escribir qué pasó"],
      done: 4,
      n: [
        "El disco de registros estaba lleno; el servicio no arrancaba y ningún mensaje lo explicaba.",
        "El aviso no salió porque el propio sistema de avisos vive en el mismo equipo. Eso hay que separarlo.",
      ],
    },
    { t: "renovar la suscripción del correo", p: "minor", l: 3, g: ["finanzas"] },
    { t: "sacar el certificado de residencia", p: "delegate", g: ["tramites"] },
    { t: "actualizar los datos en el banco", p: "delegate", l: 3, g: ["finanzas", "tramites"] },
    { t: "llevar la ropa a la tintorería", p: "delegate", l: 0, g: ["casa"] },
    { t: "coordinar la visita del técnico", p: "delegate", l: 0, g: ["casa"] },
    { t: "pedir hora al kinesiólogo", p: "do", l: 2, g: ["salud"] },
    { t: "comprar zapatillas nuevas", l: 0, g: ["compras", "deporte"] },
    { t: "inscribirse en la carrera del parque", p: "decide", g: ["deporte"] },
    {
      t: "armar la estantería del pasillo",
      p: "decide",
      l: 0,
      g: ["casa"],
      b: "Cuatro tablas y dos horas. Lo difícil fue nivelarla contra la pared torcida.",
      s: ["medir el hueco", "comprar los materiales", "cortar las tablas", "montarla"],
      done: 4,
    },
    { t: "revisar las suscripciones del año", p: "minor", l: 3, g: ["finanzas"] },
    { t: "hacer copia de las fotos del viaje", p: "minor", g: ["fotos"] },
    { t: "cambiar el filtro del agua", p: "minor", l: 0, g: ["casa"] },
    { t: "renovar la licencia de conducir", p: "do", l: 0, g: ["auto", "tramites"] },
    { t: "vender la bicicleta vieja", p: "minor", g: ["compras"] },
    { t: "arreglar el enchufe de la cocina", p: "do", l: 0, g: ["casa"] },
    {
      t: "cerrar el trimestre con el contador",
      p: "do",
      l: 3,
      g: ["finanzas", "trabajo"],
      b: "Todo lo del trimestre en una carpeta, ordenado por fecha.",
      s: ["juntar las boletas", "cuadrar los gastos", "enviar la carpeta"],
      done: 3,
      n: ["Faltaban dos boletas de un proveedor; las reenviaron el mismo día."],
    },
    { t: "limpiar los filtros del extractor", p: "minor", l: 0, g: ["casa", "cocina"] },
  ],
  dropped: [
    { t: "aprender a tocar el piano", p: "minor", l: 4, g: ["musica", "estudio"] },
    { t: "cambiar el auto por uno más pequeño", p: "decide", l: 3, g: ["auto", "finanzas"] },
    { t: "escribir un blog semanal", p: "minor", g: ["ideas"] },
    { t: "mudarse de barrio", p: "decide", l: 0, g: ["casa"] },
    { t: "abrir una cuenta en otro banco", p: "minor", l: 3, g: ["finanzas"] },
    { t: "hacer un huerto en la terraza grande", p: "minor", l: 0, g: ["casa", "jardin"] },
    { t: "comprar la consola nueva", p: "minor", g: ["ocio", "compras"] },
    { t: "retomar el curso de alemán", p: "decide", l: 4, g: ["estudio"] },
    { t: "cambiar el refrigerador", p: "decide", l: 0, g: ["casa"] },
    { t: "organizar el viaje al sur en invierno", p: "decide", l: 0, g: ["viaje"] },
  ],
};

const EN = {
  lists: ["Home", "Work", "Health", "Money", "Study"],
  folders: {
    work: "Work",
    minutes: "Minutes",
    projects: "Projects",
    infra: "Infrastructure",
    design: "Design",
    home: "Home",
    kitchen: "Kitchen",
    rina: "Rina",
    money: "Money",
    learning: "Learning",
    rust: "Rust",
    music: "Music",
    travel: "Travel",
    lisbon: "Lisbon",
    health: "Health",
  },
  papers: {
    minutes: md(
      "# Team minutes",
      "",
      "Every set of minutes gets its own page. What stays here is what does not change from one meeting to the next: who calls it, how it gets written, and what happens to whatever is left open.",
      "",
      "## How it works",
      "",
      "- The meeting lasts half an hour and starts on time.",
      "- Whoever calls it writes the minutes the same day, not the morning after.",
      "- An agreement with no owner and no date goes back to the list below.",
      "",
      "| Part | Who | Stand-in |",
      "| :--- | :--- | ---: |",
      "| Calls it | Ana | Bruno |",
      "| Writes it | rotates | — |",
      "| Closes agreements | Marta | Ana |",
      "",
      "> [!NOTE]",
      "> An agreement without an owner is not an agreement. It is a good intention.",
      "",
      "## Still open",
      "",
      "- [x] Migrate the billing history",
      "- [x] Renew the support contract",
      "- [ ] Replace the printer on the second floor",
      "- [ ] Decide who talks to the new supplier",
      ""
    ),
    minutesOne: md(
      "# Minutes: the price change",
      "",
      "Ana, Bruno and me. Half an hour, no screen sharing.",
      "",
      "## Settled",
      "",
      "- The new prices start **on the first day of the quarter**, not before.",
      "- Bruno writes to customers with two weeks of notice.",
      "- Ana checks the contracts that end before the change.",
      "",
      "## Left hanging",
      "",
      "| Subject | Who | When |",
      "| :--- | :--- | ---: |",
      "| Migrate the history | nobody yet | to be decided |",
      "| Replace the printer | Ana | once the quote arrives |",
      "",
      "> Nothing touches the old system until the migration has been tested.",
      "",
      "The date that does not move is ==the first day of the quarter==. The one still up in the air is <mark data-pen=\"blue\">the printer</mark>.",
      ""
    ),
    minutesTwo: md(
      "# Minutes: mid-year review",
      "",
      "The long one, the only meeting of the year that runs past half an hour. All four of us.",
      "",
      "## Numbers we looked at",
      "",
      "| Measure | Target | Actual |",
      "| :--- | ---: | ---: |",
      "| Open incidents | 12 | 9 |",
      "| Median response time | 4 h | 5.2 h |",
      "| Customers who renewed | 85% | 88% |",
      "",
      "Response time is the only one we missed, and we know why: two weeks of holiday went uncovered.",
      "",
      "## Agreed",
      "",
      "1. Cover holidays with a written rotation, not a spoken one.",
      "2. Bring the open-incident target down to ten.",
      "3. Stop counting duplicate tickets as incidents.",
      "",
      "> [!IMPORTANT]",
      "> The rotation gets written before anyone asks for time off, not after.",
      ""
    ),
    minutesThree: md(
      "# Minutes: closing the migration",
      "",
      "Last meeting about the migration. It finished two months early because the history turned out cleaner than we feared.",
      "",
      "## What moved",
      "",
      "- Billing in full, back to the first record.",
      "- Contacts, without the duplicates we had been carrying.",
      "- Attachments, except the ones pointing at nothing.",
      "",
      "## What we learned",
      "",
      "- Measuring before estimating would have saved half the meetings.",
      "- The weekend cut-over window was never needed. One night was enough.",
      "",
      "> [!TIP]",
      "> The old system stays read-only for six more months. After that it goes off and we keep a cold copy.",
      ""
    ),
    server: md(
      "# The server at home",
      "",
      "What to remember when something stops answering. None of this is *urgent* until it is, and by then there is no time to work it out.",
      "",
      "## What runs there",
      "",
      "| Service | Port | Starts on its own | Depends on |",
      "| :--- | :---: | :---: | ---: |",
      "| backup | 8384 | yes | external disk |",
      "| library | 8096 | yes | backup |",
      "| dns | 53 | yes | — |",
      "| panel | 9090 | no | — |",
      "",
      "The **library** depends on the backup because both read the same drive. If the disk does not mount, the two go down together, and the start-up order matters.",
      "",
      "```mermaid title=\"Start-up and dependencies\"",
      "flowchart TD",
      "  D[External disk] --> R[Backup]",
      "  R --> B[Library]",
      "  N[Network] --> S[DNS]",
      "  N --> P[Panel]",
      "  R -.-> A[Alert to the phone]",
      "```",
      "",
      "## When the disk fills up",
      "",
      "```bash title=\"clean.sh\"",
      "docker system prune -a --volumes",
      "journalctl --vacuum-time=7d",
      "find /var/lib/media/cache -type f -mtime +30 -delete",
      "```",
      "",
      "Before pruning it helps to know how fast it grows. It is a subtraction and a division, but written down it sticks:",
      "",
      "```math title=\"Daily growth in GiB\"",
      "g = \\frac{b_{today} - b_{yesterday}}{1024^3}",
      "```",
      "",
      "A `g` above two means something is writing more than it should, and it is almost always the logs of a container restarting in a loop.",
      "",
      "> [!CAUTION]",
      "> Never prune without checking that the last copy finished cleanly. Pruning takes a minute. Rebuilding two terabytes takes a night.",
      "",
      "## Quarterly pass",
      "",
      "- [x] Check free space on every volume",
      "- [x] Restore one file at random from the backup",
      "- [ ] Look at the certificates about to expire",
      "- [ ] Update the base system",
      "",
      "The sync tool is documented at [rsync.samba.org](https://rsync.samba.org), and the disk manual is on paper in the bottom drawer.",
      ""
    ),
    serverBackup: md(
      "# Backups and restoring",
      "",
      "Three copies, two kinds of media, one of them out of the house. The rest is opinion.",
      "",
      "| Copy | Where | How often | Kept for |",
      "| :--- | :--- | :---: | ---: |",
      "| hot | external disk | daily | 30 days |",
      "| cold | disk that lives at the office | monthly | 12 months |",
      "| remote | encrypted storage | weekly | 90 days |",
      "",
      "```bash title=\"backup.sh\"",
      "set -euo pipefail",
      "rsync -a --delete --link-dest=../latest ~/data/ /mnt/copies/$(date +%F)/",
      "ln -sfn /mnt/copies/$(date +%F) /mnt/copies/latest",
      "```",
      "",
      "> [!WARNING]",
      "> A copy that has never been restored is not a copy. Once a quarter I pull out any file at all and check that it opens.",
      ""
    ),
    serverAlerts: md(
      "# When something stops answering",
      "",
      "The order matters. It is nearly always the layer underneath that broke, and starting at the top costs half an hour.",
      "",
      "1. Is there power, is there network? The disk cable has failed twice already.",
      "2. Is the disk mounted? If not, nothing on top of it is going to start.",
      "3. Is there space left? A full volume behaves exactly like a dead service.",
      "4. Only then, the service logs.",
      "",
      "```bash",
      "systemctl --failed",
      "df -h /mnt/data",
      "journalctl -u library -n 200 --no-pager",
      "```",
      "",
      "> [!TIP]",
      "> If the alert reached the phone but the service answers, the alert is the problem, not the service. That gets written down too.",
      ""
    ),
    deploy: md(
      "# Deploying, step by step",
      "",
      "Deploys happen mid-morning, never late in the day. If something breaks I want the day ahead of me, not the night.",
      "",
      "1. Run the whole test suite locally.",
      "2. Tag the version and wait for the build server to finish.",
      "3. Put up the maintenance notice, if the change touches the database.",
      "4. Deploy, watch the logs for two minutes, take the notice down.",
      "",
      "```bash title=\"deploy.sh\"",
      "cargo test --workspace",
      "git tag -a \"v$1\" -m \"version $1\"",
      "ssh server 'cd /srv/app && ./update.sh'",
      "```",
      "",
      "> [!IMPORTANT]",
      "> The way back has to be tested before the deploy, not thought about during it.",
      ""
    ),
    runbook: md(
      "# What to look at when it breaks",
      "",
      "A short script so I do not improvise with my pulse up.",
      "",
      "| Symptom | First suspicion | Check |",
      "| :--- | :--- | :--- |",
      "| Everything slow | disk full | `df -h` |",
      "| Errors come and go | restart loop | service logs |",
      "| Nothing answers | network or power | ping the box |",
      "",
      "Learned the hard way: **write down the exact time** it started. Without that time the logs are a haystack.",
      ""
    ),
    roadmap: md(
      "# Project plan",
      "",
      "Three blocks, in this order. None of them starts until the one before is really finished.",
      "",
      "## Block one: stop losing data",
      "",
      "- Automatic backups that have been restored at least once.",
      "- A change log a person can actually read.",
      "",
      "## Block two: someone else can run it",
      "",
      "- Write down the deploy and the way back.",
      "- Get the passwords out of the config files.",
      "",
      "## Block three: make it pleasant",
      "",
      "- A status screen with what matters and nothing else.",
      "- Alerts nobody learns to ignore.",
      "",
      "> [!NOTE]",
      "> Block three is the one I most want to start and the one that ages worst if the first two are missing.",
      ""
    ),
    decisions: md(
      "# Decisions we no longer argue about",
      "",
      "Each with its date and its reason. If the reason stops holding, the decision gets revisited. Until then it stays shut.",
      "",
      "## Keep the history as a log of events",
      "",
      "Chosen because state can be rebuilt from it and because a mistake is fixed by adding, not by deleting. The cost is that the file only ever grows.",
      "",
      "## No servers of our own",
      "",
      "Everything runs on the machine of whoever uses it. Syncing is the job of whatever shared folder already exists, not ours.",
      "",
      "## One database, in a file",
      "",
      "| Option | Why not |",
      "| :--- | :--- |",
      "| Networked database | forces a server |",
      "| One file per entity | breaks when synced |",
      "",
      "> [!CAUTION]",
      "> Changing any of these three means migrating real people's data. Nobody touches them without a written way back.",
      ""
    ),
    worknotes: md(
      "# Loose work notes",
      "",
      "Things that do not deserve a document of their own but are gone in two weeks.",
      "",
      "- The mail provider throttles bulk sends above five hundred an hour.",
      "- The support contract covers incidents, not changes: ask for changes on the other channel.",
      "- The big meeting room needs booking ahead. The small one does not.",
      "",
      "A line that covers most of this place: *if it is not written down, you will have to ask twice*.",
      ""
    ),
    houseManual: md(
      "# The house manual",
      "",
      "For whoever house-sits, and for me when I forget.",
      "",
      "| Thing | Where | Detail |",
      "| :--- | :--- | :--- |",
      "| Water shut-off | under the sink | turns clockwise |",
      "| Fuse box | hallway | the big switch is the main one |",
      "| Extractor filters | top drawer | wash every two months |",
      "",
      "## Bins",
      "",
      "- Food waste: Tuesday and Friday nights.",
      "- Recycling: Friday only.",
      "",
      "> [!TIP]",
      "> The caretaker has a spare key. If the door sticks, lift it slightly while you turn.",
      ""
    ),
    sourdough: md(
      "# Sourdough bread",
      "",
      "The starter has been alive since the first attempt. Feed it the night before.",
      "",
      "## What goes in",
      "",
      "| What | How much |",
      "| :--- | ---: |",
      "| Strong flour | 500 g |",
      "| Warm water | 350 g |",
      "| Active starter | 100 g |",
      "| Salt | 10 g |",
      "",
      "## How it goes",
      "",
      "1. Mix flour and water, leave it forty minutes.",
      "2. Add the starter and the salt.",
      "3. Three folds, half an hour apart.",
      "4. Cold proof overnight.",
      "5. Oven at 250 °C with steam for the first twenty minutes.",
      "",
      "Last time it came out tight because it went in too early. The one thing worth writing down: <mark data-pen=\"green\">cold proof</mark>, never on the counter.",
      ""
    ),
    preserves: md(
      "# Preserves by season",
      "",
      "What gets put away, and when it is cheap enough to bother.",
      "",
      "- Tomatoes: late summer, when there are too many and the price drops.",
      "- Peaches in syrup: two weeks before the tomatoes.",
      "- Roasted peppers: right after the tomatoes.",
      "",
      "> [!WARNING]",
      "> Jars boil for twenty minutes once sealed, and only get stored if the lid has sucked down. If it clicks when you press it, into the fridge and eaten that week.",
      ""
    ),
    rina: md(
      "# Rina, year by year",
      "",
      "The dog's life told one year per page. What stays here is what does not change: who looks after her, what she eats, and what happens when she is left alone.",
      "",
      "| Detail | Which |",
      "| :--- | :--- |",
      "| Vet | South Clinic |",
      "| Food | dry, two cups a day |",
      "| Allergies | none known |",
      "",
      "> [!NOTE]",
      "> Left alone more than six hours, she barks. It is not anxiety, it is boredom, and a long walk beforehand fixes it.",
      ""
    ),
    rinaFirst: md(
      "# The first year",
      "",
      "She arrived at a little over three kilos and terrified of stairs.",
      "",
      "- [x] Vaccinations up to date",
      "- [x] Get over the stairs",
      "- [x] Learn to be alone for an hour",
      "- [ ] Stop chasing motorbikes",
      "",
      "The stairs took three weeks, one step a day with a treat. The motorbikes are exactly as they were.",
      ""
    ),
    rinaSecond: md(
      "# The second year",
      "",
      "She now weighs what she is going to weigh and sleeps wherever she likes.",
      "",
      "| Check | When | Result |",
      "| :--- | :--- | ---: |",
      "| Yearly shot | autumn | nothing to report |",
      "| Teeth cleaning | winter | mild tartar |",
      "| Worming | every three months | up to date |",
      "",
      "What changed this year: she learned that the backpack means a walk, and now sits beside it the moment it comes out.",
      ""
    ),
    budget: md(
      "# The year's budget",
      "",
      "One sheet to know whether the year closes or not. This is not accounting: it is an **honest rule of thumb**, revisited every quarter.",
      "",
      "## How it splits",
      "",
      "| Category | Target | Actual | Gap |",
      "| :--- | ---: | ---: | ---: |",
      "| Housing | 30% | 32% | +2 |",
      "| Food | 18% | 17% | −1 |",
      "| Transport | 10% | 8% | −2 |",
      "| Health | 8% | 9% | +1 |",
      "| Saving | 20% | 18% | −2 |",
      "| Everything else | 14% | 16% | +2 |",
      "",
      "Housing always runs over, and there is not much to do about it while the lease lasts. What does move is *everything else*, which is where forgotten subscriptions hide.",
      "",
      "```mermaid title=\"Where the money goes\"",
      "pie showData",
      "  \"Housing\" : 32",
      "  \"Food\" : 17",
      "  \"Transport\" : 8",
      "  \"Health\" : 9",
      "  \"Saving\" : 18",
      "  \"Other\" : 16",
      "```",
      "",
      "## What the saving earns",
      "",
      "With a steady monthly deposit and a small monthly rate, the balance after `n` months is not the deposit times the months. It is quite a lot more, and that difference is the whole reason not to leave it in the current account.",
      "",
      "```math title=\"Saving with monthly deposits\"",
      "S_n = A \\cdot \\frac{(1 + i)^n - 1}{i}",
      "```",
      "",
      "With `A = 200` and `i = 0.004` a month, five years puts the balance over fourteen thousand against the twelve thousand actually paid in.",
      "",
      "```python title=\"saving.py\"",
      "def balance(deposit, rate, months):",
      "    return deposit * ((1 + rate) ** months - 1) / rate",
      "",
      "print(round(balance(200, 0.004, 60), 2))",
      "```",
      "",
      "> [!IMPORTANT]",
      "> The saving comes out the day the salary lands. What is left at the end of the month is never what was spare: it is what was never set aside.",
      "",
      "## Quarterly review",
      "",
      "- [x] Compare target and actual by category",
      "- [x] Cancel anything unused for three months",
      "- [ ] Renegotiate the internet plan",
      "- [ ] Look at the insurance before it renews itself",
      "",
      "Reference rates come from [the central bank's own site](https://www.bankofengland.co.uk), not from what each bank advertises.",
      ""
    ),
    taxes: md(
      "# Tax time: what to gather",
      "",
      "Gathered through the year and sorted in one afternoon, not the other way round.",
      "",
      "| Document | Comes from | Arrives |",
      "| :--- | :--- | ---: |",
      "| Pay certificate | employer | start of the year |",
      "| Loan interest | bank | start of the year |",
      "| Invoices issued | portal | all year |",
      "| Medical costs | clinic | on request |",
      "",
      "> [!TIP]",
      "> Anything that arrives by post goes into the year's folder the same day. Finding it later costs three times as long.",
      ""
    ),
    rustNotes: md(
      "# Rust: reading notes",
      "",
      "What I worked out from the book, in my own words. If something here is wrong, that is me misreading it, not the book saying it.",
      "",
      "## Borrowing, in one line",
      "",
      "You can have **many reads** or **one write**, never both at once. Nearly every compiler error that cost me an afternoon comes from forgetting that line.",
      "",
      "```rust title=\"src/walk.rs\"",
      "#[derive(Debug, Clone, Copy, PartialEq)]",
      "enum Slot {",
      "    Morning,",
      "    Evening,",
      "}",
      "",
      "fn time_to_go_out(hour: u8, been_out: bool) -> Option<Slot> {",
      "    match (hour, been_out) {",
      "        (7..=9, false) => Some(Slot::Morning),",
      "        (20..=22, false) => Some(Slot::Evening),",
      "        _ => None,",
      "    }",
      "}",
      "```",
      "",
      "Matching on a tuple is what I miss most elsewhere: the compiler tells you when a case is missing.",
      "",
      "## How a value travels",
      "",
      "```mermaid title=\"Moving and borrowing\"",
      "stateDiagram-v2",
      "  [*] --> Owned",
      "  Owned --> Moved: passed by value",
      "  Owned --> Borrowed: passed by reference",
      "  Borrowed --> Owned: the borrow ends",
      "  Moved --> [*]",
      "```",
      "",
      "## Why the compiler is slow",
      "",
      "It is not just how much code there is: it is how many times each generic function gets stamped out. With `t` distinct types and `f` generic functions, the work grows like the product, not the sum:",
      "",
      "```math title=\"Instances generated\"",
      "N = \\sum_{k=1}^{f} t_k \\qquad \\text{with } t_k \\ge 1",
      "```",
      "",
      "Hence the advice that a big generic function should call a non-generic inner one: the wrapper is what gets stamped out, not the body.",
      "",
      "> [!NOTE]",
      "> The book lives at [doc.rust-lang.org/book](https://doc.rust-lang.org/book/) and is free to read. The paper copy is for underlining, which is how it sticks for me.",
      "",
      "## Still to understand",
      "",
      "- [x] Borrowing and basic lifetimes",
      "- [x] `Result` and the question mark",
      "- [ ] Lifetimes on structs",
      "- [ ] When `Pin` is actually needed",
      ""
    ),
    rustErrors: md(
      "# Errors that already cost me an afternoon",
      "",
      "Each with what it actually meant, not what the message says.",
      "",
      "| Message | What was going on |",
      "| :--- | :--- |",
      "| `cannot borrow as mutable` | a read was still alive further down |",
      "| `does not live long enough` | returning a reference to a local |",
      "| `the trait is not implemented` | a missing `use`, not a missing impl |",
      "",
      "```diff",
      "- let first = list.first();",
      "- list.push(item);",
      "+ let first = list.first().copied();",
      "+ list.push(item);",
      "```",
      "",
      "`copied()` ends the borrow because it copies the value instead of keeping an eye on the list. Half an afternoon for seven letters.",
      ""
    ),
    guitar: md(
      "# Back to the guitar",
      "",
      "Twenty minutes a day beats two hours on Sunday. I have already tested that the wrong way round.",
      "",
      "## Short routine",
      "",
      "1. Five minutes on open strings, no rush.",
      "2. Ten on whichever scale is this week's.",
      "3. Five on a song, even if it comes out badly.",
      "",
      "> [!TIP]",
      "> The slow metronome is boring and it is the only thing that works. Two clicks faster a week, no more.",
      ""
    ),
    scales: md(
      "# Scales and shapes",
      "",
      "The minimum needed not to get lost on the neck.",
      "",
      "| Scale | Starts at | Good for |",
      "| :--- | :---: | :--- |",
      "| Minor pentatonic | fifth fret | nearly everything |",
      "| Major | third fret | melodies |",
      "| Blues | fifth fret | the usual sound |",
      "",
      "The pentatonic is one shape repeated five times along the neck. Learning all five positions takes a month, and after that the neck stops being a mystery.",
      ""
    ),
    lisbon: md(
      "# Five days in Lisbon",
      "",
      "Morning flight out, Saturday afternoon back. A lot of walking, so comfortable shoes and nothing tightly scheduled.",
      "",
      "## Before leaving",
      "",
      "- [x] Book the place in Alfama",
      "- [x] Tell work",
      "- [ ] Renew the passport",
      "- [ ] Get some cash",
      "",
      "## Rough costs",
      "",
      "| Item | Estimate |",
      "| :--- | ---: |",
      "| Flights | 180 |",
      "| Four nights | 320 |",
      "| Eating | 200 |",
      "",
      "<mark data-pen=\"pink\">The passport is the one that cannot wait</mark>: six weeks if it goes the slow way.",
      ""
    ),
    lisbonFood: md(
      "# Where to eat in Lisbon",
      "",
      "Written down from what people told me, none of it tried yet.",
      "",
      "- A tasca near the Graça viewpoint, one of those with no menu.",
      "- The riverside market, early and on a weekday.",
      "- A neighbourhood pastelaria, the one on the corner before the square.",
      "",
      "> [!NOTE]",
      "> The expensive places are on the main street and the good ones two blocks off it. True almost anywhere, but more obvious here.",
      ""
    ),
    checkups: md(
      "# Check-ups and shots",
      "",
      "Somewhere to look before booking, so nothing gets repeated.",
      "",
      "| Check | How often | Last one |",
      "| :--- | :---: | ---: |",
      "| Full bloods | yearly | up to date |",
      "| Dentist | twice a year | overdue |",
      "| Eyes | every two years | up to date |",
      "",
      "> [!IMPORTANT]",
      "> Results go in here the day they arrive. The paper envelope always gets lost.",
      ""
    ),
  },
  routines: {
    plants: { t: "water the balcony plants", c: "every sunday", g: ["home", "garden"], l: 0 },
    backups: { t: "check the backups finished cleanly", c: "every friday", g: ["server"], l: 1 },
    rent: { t: "pay the rent", c: "every month", g: ["money"], l: 3 },
    carService: { t: "car service", c: "every 6 months", g: ["car"], l: 0 },
    domain: { t: "renew the domain and the certificate", c: "every 12 months", g: ["server"], l: 1 },
  },
  open: [
    { t: "go over the quarter's budget", p: "do", l: 1, g: ["work", "money"], d: 0 },
    { t: "send the outstanding invoice", p: "do", l: 3, g: ["money"], d: 0 },
    { t: "confirm the meeting with the supplier", p: "do", l: 1, g: ["work"], d: 0 },
    { t: "book an appointment with the dentist", p: "do", l: 2, g: ["health"], d: -2 },
    { t: "renew the car insurance", p: "do", l: 3, g: ["car", "money"], d: -1 },
    { t: "reply to the health insurer", p: "do", l: 2, g: ["health"], d: 1 },
    { t: "close the incident report", p: "do", l: 1, g: ["work"], d: 1, dl: 3 },
    { t: "pay the electricity bill", p: "do", l: 3, g: ["money", "home"], d: 2 },
    { t: "take the car in for its inspection", p: "do", l: 0, g: ["car"], d: 3 },
    {
      t: "put the committee talk together",
      p: "do",
      l: 1,
      g: ["work"],
      d: 2,
      dl: 4,
      b: "Twenty minutes, with the numbers at the end. The big room needs booking.",
      s: ["gather the quarter's figures", "write the running order", "book the room", "run through it out loud once"],
      done: 2,
      n: [
        "Marta sent the corrected figures: the July dip was a duplicate.",
        "They will ask about next year, so keep one slide ready.",
      ],
    },
    { t: "buy the flights", p: "do", l: 0, g: ["travel"], d: 4 },
    { t: "sign the tenancy agreement", p: "do", l: 0, g: ["home"], d: 5 },
    { t: "update the server backup", p: "do", l: 1, g: ["server"], d: 0 },
    { t: "pick up the prescriptions", p: "do", l: 2, g: ["health"], d: -3 },
    { t: "go through the bank statement", p: "do", l: 3, g: ["money"], d: 6 },
    { t: "tell work about the holiday dates", p: "do", l: 1, g: ["work"], d: 7 },
    {
      t: "plan the trip at the end of the year",
      p: "decide",
      l: 0,
      g: ["travel", "family"],
      d: 12,
      b: "Two weeks, leaving before the fares climb.",
      s: ["compare flight prices", "pick somewhere to stay", "ask for the days off"],
      done: 1,
      n: ["Fares jump sharply from the last week of the month."],
    },
    { t: "work through the certification syllabus", p: "decide", l: 4, g: ["study"], d: 10 },
    { t: "redesign the personal site", p: "decide", l: 4, g: ["study", "design"], d: 20 },
    { t: "read the software architecture book", p: "decide", l: 4, g: ["books", "study"] },
    { t: "clear out the hallway cupboard", p: "decide", l: 0, g: ["home"], d: 15 },
    { t: "compare internet plans", p: "decide", l: 3, g: ["money", "home"], d: 9 },
    { t: "set up the savings plan", p: "decide", l: 3, g: ["money"], d: 18 },
    { t: "find a photography course", p: "decide", l: 4, g: ["study"] },
    {
      t: "settle the shape of the new module",
      p: "decide",
      l: 1,
      g: ["work", "code"],
      d: 14,
      b: "We have to decide whether it keeps the same event log or gets a table of its own.",
      s: ["write up both options", "measure what migrating costs", "take it to the team"],
      done: 1,
      n: ["Bruno prefers the separate table for how simple it is; the cost is keeping two ways to read."],
    },
    { t: "write up the deployment", p: "decide", l: 1, g: ["work", "server"], d: 11 },
    { t: "review the backup policy", p: "decide", l: 1, g: ["server"], d: 16 },
    { t: "plant the balcony boxes", p: "decide", l: 0, g: ["home", "garden"], d: 21 },
    { t: "prepare for the language exam", p: "decide", l: 4, g: ["study"], d: 25 },
    { t: "choose the anniversary present", p: "decide", l: 0, g: ["gifts", "family"], d: 13 },
    { t: "sort last year's photos", p: "decide", l: 0, g: ["photos"] },
    { t: "get a quote from the painter", p: "delegate", l: 0, g: ["home"], d: 5 },
    { t: "order the birthday cake", p: "delegate", l: 0, g: ["family"], d: 8 },
    { t: "hand over the expenses report", p: "delegate", l: 1, g: ["work", "money"], d: 3 },
    { t: "arrange collection of the old sofa", p: "delegate", l: 0, g: ["home"], d: 6 },
    { t: "ask the accountant for the certificate", p: "delegate", l: 3, g: ["money"], d: 9 },
    { t: "tell the caretaker about the new hours", p: "delegate", l: 0, g: ["home"], d: 2 },
    { t: "order the workshop materials", p: "delegate", l: 1, g: ["work"], d: 7 },
    { t: "tidy the browser bookmarks", p: "minor", g: ["computer"] },
    { t: "empty the downloads folder", p: "minor", g: ["computer"] },
    { t: "cancel the unused subscriptions", p: "minor", l: 3, g: ["money"], d: 17 },
    { t: "change the desktop wallpaper", p: "minor", g: ["computer"] },
    { t: "try the game still sitting there", p: "minor", g: ["fun"] },
    { t: "go through the film watchlist", p: "minor", g: ["fun"], d: 22 },
    { t: "find a guitar teacher", l: 4, g: ["music"] },
    { t: "write down the podcast ideas", g: ["ideas"] },
    { t: "go over the trip itinerary", l: 0, g: ["travel"], d: 19 },
    { t: "try the wholemeal loaf recipe", l: 0, g: ["cooking"], d: 8 },
    { t: "bring the CV up to date", l: 1, g: ["work"] },
    { t: "write to aunt Marta", g: ["family"], d: 4 },
    { t: "measure the hallway for the shelf", l: 0, g: ["home"], d: 6 },
    { t: "read back the notes from the last appointment", l: 2, g: ["health"], d: 3 },
  ],
  history: [
    {
      t: "change the front door lock",
      p: "do",
      l: 0,
      g: ["home"],
      b: "The key had been stiff since the winter and finally jammed for good.",
      s: ["get a quote", "buy the lock", "fit it", "cut spare keys"],
      done: 4,
      n: [
        "The locksmith charged half what the hardware shop wanted for the same model.",
        "Three spares cut, one of them for the caretaker.",
      ],
    },
    { t: "pay the road tax", p: "do", l: 3, g: ["car", "money"] },
    {
      t: "migrate the billing history",
      p: "do",
      l: 1,
      g: ["work", "code"],
      b: "Twelve years of records, in different shapes depending on the era.",
      s: [
        "export from the old system",
        "write the converter",
        "check a sample",
        "load everything",
        "switch the old system off",
      ],
      done: 5,
      n: [
        "Records from before the system change had the date the wrong way round.",
        "The full load took four hours, not the whole night we feared.",
      ],
    },
    { t: "renew the vehicle licence", p: "do", l: 3, g: ["car", "money"] },
    { t: "file the year's tax return", p: "do", l: 3, g: ["money"] },
    {
      t: "fix the bathroom leak",
      p: "do",
      l: 0,
      g: ["home"],
      b: "A stain on the ceiling downstairs. The neighbour was decent about it.",
      s: ["turn the water off", "call the plumber", "check the ceiling downstairs"],
      done: 3,
      n: ["It was the flexible hose joint, not the pipe. Half an hour of work."],
    },
    { t: "take the car in for a service", p: "do", l: 0, g: ["car"] },
    { t: "renew the passport", p: "do", l: 0, g: ["travel"] },
    { t: "take out home insurance", p: "do", l: 3, g: ["home", "money"] },
    {
      t: "put the year-end talk together",
      p: "do",
      l: 1,
      g: ["work"],
      b: "Half an hour, with room for questions at the end.",
      s: ["gather the numbers", "write the running order", "rehearse"],
      done: 3,
      n: ["It went better than expected; the numbers drew the fewest questions."],
    },
    { t: "replace the hallway bulbs", p: "minor", l: 0, g: ["home"] },
    { t: "get the teeth cleaned", p: "do", l: 2, g: ["health"] },
    { t: "have the flu jab", p: "do", l: 2, g: ["health"] },
    { t: "get new glasses", p: "decide", l: 2, g: ["health"] },
    {
      t: "tidy the desk and the cables",
      p: "minor",
      l: 0,
      g: ["home", "computer"],
      b: "It all fits in one tray under the desk and looks far better.",
      s: ["buy the tray", "label the cables", "hide the power strip"],
      done: 3,
    },
    { t: "review the support contract", p: "decide", l: 1, g: ["work"] },
    {
      t: "write the project documentation",
      p: "decide",
      l: 1,
      g: ["work", "code"],
      b: "The basics were missing: how to bring the environment up and how to deploy.",
      s: ["describe the local setup", "document the deploy", "explain the way back"],
      done: 3,
      n: ["Writing the way back forced us to test it, and that is where a missing step turned up."],
    },
    { t: "update the server's operating system", p: "do", l: 1, g: ["server"] },
    { t: "renew the domain certificate", p: "do", l: 1, g: ["server"] },
    { t: "swap the backup disk", p: "do", l: 1, g: ["server"] },
    {
      t: "get the office move ready",
      p: "do",
      l: 1,
      g: ["work"],
      b: "Quote accepted. The goods lift needs booking two days ahead.",
      s: ["order boxes", "tell the building", "book the movers", "take the desks apart"],
      done: 4,
      n: [
        "The movers charged less with packing included than without it.",
        "The lift in the new building is smaller: the big desks go up the stairs.",
      ],
    },
    { t: "buy the birthday present", l: 0, g: ["gifts", "family"] },
    { t: "arrange the family lunch", l: 0, g: ["family"] },
    { t: "take the book back to the library", p: "delegate", g: ["books"] },
    { t: "finish the databases course", p: "decide", l: 4, g: ["study"] },
    {
      t: "learn to bake sourdough",
      p: "decide",
      l: 0,
      g: ["cooking"],
      b: "Four attempts before the crumb opened up.",
      s: ["get hold of a starter", "first attempt", "fix the hydration", "try again with a cold proof"],
      done: 4,
      n: [
        "The first one was tight because it went in the oven too early.",
        "A cold proof overnight changes it completely.",
      ],
    },
    { t: "restring the guitar", p: "minor", l: 4, g: ["music"] },
    { t: "paint the living room wall", p: "decide", l: 0, g: ["home"] },
    { t: "read through the health cover", p: "decide", l: 2, g: ["health", "money"] },
    { t: "switch phone plans", p: "decide", l: 3, g: ["money"] },
    { t: "take stock of the storage room", p: "minor", l: 0, g: ["home"] },
    {
      t: "deal with the site going down",
      p: "do",
      l: 1,
      g: ["server", "work"],
      b: "Two hours down. It started mid-afternoon and nobody got the alert.",
      s: ["find the cause", "bring the service back", "fix the alert", "write down what happened"],
      done: 4,
      n: [
        "The log disk was full; the service would not start and said nothing about why.",
        "The alert never fired because the alerting lives on the same box. That has to be split.",
      ],
    },
    { t: "renew the mail subscription", p: "minor", l: 3, g: ["money"] },
    { t: "get the proof of address", p: "delegate", g: ["paperwork"] },
    { t: "update the details at the bank", p: "delegate", l: 3, g: ["money", "paperwork"] },
    { t: "take the coats to the cleaners", p: "delegate", l: 0, g: ["home"] },
    { t: "arrange the engineer's visit", p: "delegate", l: 0, g: ["home"] },
    { t: "book the physiotherapist", p: "do", l: 2, g: ["health"] },
    { t: "buy new running shoes", l: 0, g: ["shopping", "sport"] },
    { t: "sign up for the park run", p: "decide", g: ["sport"] },
    {
      t: "build the hallway shelf",
      p: "decide",
      l: 0,
      g: ["home"],
      b: "Four boards and two hours. The hard part was levelling it against a crooked wall.",
      s: ["measure the gap", "buy the materials", "cut the boards", "put it up"],
      done: 4,
    },
    { t: "audit the year's subscriptions", p: "minor", l: 3, g: ["money"] },
    { t: "back up the trip photos", p: "minor", g: ["photos"] },
    { t: "change the water filter", p: "minor", l: 0, g: ["home"] },
    { t: "renew the driving licence", p: "do", l: 0, g: ["car", "paperwork"] },
    { t: "sell the old bicycle", p: "minor", g: ["shopping"] },
    { t: "fix the kitchen socket", p: "do", l: 0, g: ["home"] },
    {
      t: "close the quarter with the accountant",
      p: "do",
      l: 3,
      g: ["money", "work"],
      b: "Everything for the quarter in one folder, sorted by date.",
      s: ["gather the receipts", "reconcile the spend", "send the folder"],
      done: 3,
      n: ["Two receipts from one supplier were missing; they resent them the same day."],
    },
    { t: "clean the extractor filters", p: "minor", l: 0, g: ["home", "cooking"] },
  ],
  dropped: [
    { t: "learn the piano", p: "minor", l: 4, g: ["music", "study"] },
    { t: "swap the car for a smaller one", p: "decide", l: 3, g: ["car", "money"] },
    { t: "write a weekly blog", p: "minor", g: ["ideas"] },
    { t: "move to another neighbourhood", p: "decide", l: 0, g: ["home"] },
    { t: "open an account at another bank", p: "minor", l: 3, g: ["money"] },
    { t: "put a vegetable patch on the big terrace", p: "minor", l: 0, g: ["home", "garden"] },
    { t: "buy the new console", p: "minor", g: ["fun", "shopping"] },
    { t: "go back to the German course", p: "decide", l: 4, g: ["study"] },
    { t: "replace the fridge", p: "decide", l: 0, g: ["home"] },
    { t: "plan a winter trip south", p: "decide", l: 0, g: ["travel"] },
  ],
};

const BOOK = tongue === "es" ? ES : EN;

function binary() {
  if (process.env.TISTY_BIN) return process.env.TISTY_BIN;
  const exe = process.platform === "win32" ? "tisty.exe" : "tisty";
  for (const build of ["debug", "release"]) {
    const at = path.join(ROOT, "target", build, exe);
    if (fs.existsSync(at)) return at;
  }
  throw new Error(`no tisty binary under ${path.join(ROOT, "target")}: build it first`);
}

function roots() {
  const home = os.homedir();
  if (process.platform === "win32") {
    const local = process.env.LOCALAPPDATA || path.join(home, "AppData", "Local");
    const under = path.join(local, "tisty");
    return { data: path.join(under, "data"), config: path.join(under, "config"), cache: path.join(under, "cache") };
  }
  if (process.platform === "darwin") {
    const under = path.join(home, "Library", "Application Support", "tisty");
    return { data: under, config: path.join(under, "config"), cache: path.join(home, "Library", "Caches", "tisty") };
  }
  return {
    data: path.join(home, ".local", "share", "tisty"),
    config: path.join(home, ".config", "tisty"),
    cache: path.join(home, ".cache", "tisty"),
  };
}

const BIN = binary();
const WHERE = roots();
const HOUSE = {
  data: path.join(process.env.TISTY_DATA || WHERE.data, "sandboxes", profile),
  config: path.join(process.env.TISTY_CONFIG || WHERE.config, "sandboxes", profile),
  cache: path.join(process.env.TISTY_CACHE || WHERE.cache, "sandboxes", profile),
};
const STORE = path.join(HOUSE.data, "store");
const ENV = { ...process.env, TISTY_PROFILE: profile };

function cli(args) {
  const out = spawnSync(BIN, args, { env: ENV, encoding: "utf8", windowsHide: true });
  if (out.error) throw out.error;
  if (out.status !== 0) {
    throw new Error(`tisty ${args.join(" ")}\n${(out.stderr || "") + (out.stdout || "")}`);
  }
  return out.stdout || "";
}

class Agent {
  constructor() {
    this.child = spawn(BIN, ["mcp"], { env: ENV, stdio: ["pipe", "pipe", "pipe"], windowsHide: true });
    this.child.stderr.resume();
    this.waiting = new Map();
    this.next = 1;
    this.rest = "";
    this.child.stdout.setEncoding("utf8");
    this.child.stdout.on("data", (chunk) => this.eat(chunk));
  }

  eat(chunk) {
    this.rest += chunk;
    let cut = this.rest.indexOf("\n");
    while (cut >= 0) {
      const line = this.rest.slice(0, cut).trim();
      this.rest = this.rest.slice(cut + 1);
      if (line) {
        const said = JSON.parse(line);
        const held = this.waiting.get(said.id);
        if (held) {
          this.waiting.delete(said.id);
          held(said);
        }
      }
      cut = this.rest.indexOf("\n");
    }
  }

  send(method, params) {
    const id = this.next++;
    return new Promise((keep, drop) => {
      this.waiting.set(id, (said) => {
        if (said.error) drop(new Error(`${method}: ${said.error.message}`));
        else keep(said.result);
      });
      this.child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    });
  }

  async hello() {
    await this.send("initialize", {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "showcase", version: "1" },
    });
    this.child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
  }

  async call(name, args) {
    const said = await this.send("tools/call", { name, arguments: args });
    if (said.isError) throw new Error(`${name}: ${(said.content || []).map((one) => one.text).join(" ")}`);
    return said.structuredContent || {};
  }

  close() {
    this.child.stdin.end();
    this.child.kill();
  }
}

function seeded(n) {
  let a = n >>> 0;
  return () => {
    a += 0x6d2b79f5;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const dice = seeded(20260903);

const NOW = new Date();
const TODAY = new Date(NOW.getFullYear(), NOW.getMonth(), NOW.getDate());
const shift = (from, days) => new Date(from.getFullYear(), from.getMonth(), from.getDate() + days);
const stamp = (day) =>
  `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, "0")}-${String(day.getDate()).padStart(2, "0")}`;
const clock = (day, hour, minute) =>
  new Date(day.getFullYear(), day.getMonth(), day.getDate(), hour, minute, 0, 0).getTime();
const back = (days) => shift(TODAY, -days);
const START = back(SPAN_DAYS);

function wipe() {
  for (const at of [HOUSE.data, HOUSE.config, HOUSE.cache]) fs.rmSync(at, { recursive: true, force: true });
}

function ledgers() {
  if (!fs.existsSync(STORE)) return [];
  const out = [];
  for (const device of fs.readdirSync(STORE).sort()) {
    const dir = path.join(STORE, device);
    if (!fs.statSync(dir).isDirectory()) continue;
    for (const file of fs.readdirSync(dir).sort()) {
      if (file.endsWith(".tisty")) out.push(path.join(dir, file));
    }
  }
  return out;
}

function written() {
  let n = 0;
  for (const at of ledgers()) {
    for (const line of fs.readFileSync(at, "utf8").split("\n")) if (line.trim()) n += 1;
  }
  return n;
}

const marks = [];
async function moment(day, run) {
  await run();
  marks.push({ day: stamp(day), upto: written() });
}

function tail(after) {
  for (const at of ledgers().reverse()) {
    const lines = fs.readFileSync(at, "utf8").split("\n").filter((one) => one.trim());
    for (let i = lines.length - 1; i >= 0; i -= 1) {
      const one = JSON.parse(lines[i]);
      if (one.op === "task.add" && one.d && one.d.after === after) return one.id;
    }
  }
  return null;
}

function phrase(entry) {
  const bits = [entry.t];
  if (entry.c) bits.push(entry.c);
  for (const tag of entry.g || []) bits.push(`#${tag}`);
  if (entry.l != null) bits.push(`@${BOOK.lists[entry.l]}`);
  return bits.join(" ");
}

function born(entry, day) {
  const args = ["add", "--json", phrase(entry)];
  if (day) args.push("--date", stamp(day));
  if (entry.p) args.push("--priority", entry.p);
  if (entry.dl != null) args.push("--deadline", stamp(shift(TODAY, entry.dl)));
  const task = JSON.parse(cli(args).trim());
  if (task.title !== entry.t) cli(["set", task.id, "--title", entry.t]);
  return task.id;
}

function flesh(id, entry) {
  if (entry.b) cli(["desc", id, entry.b]);
  for (const step of entry.s || []) cli(["step", id, "add", step]);
  for (let k = 1; k <= (entry.done || 0); k += 1) cli(["step", id, "done", String(k)]);
  for (const note of entry.n || []) cli(["log", id, note]);
}

const schedule = [];
const later = (day, run) => schedule.push({ day, run });

function shelves(agent) {
  FOLDERS.forEach((one, n) => {
    later(shift(START, 1 + n), async () => {
      const args = { name: BOOK.folders[one.key], icon: one.icon };
      if (one.color) args.color = one.color;
      if (one.inside) args.inside = BOOK.folders[one.inside];
      await agent.call("folder", args);
    });
  });
}

function papers(agent) {
  const kept = new Map();
  for (const one of PAPERS) {
    later(back(one.ago), async () => {
      const args = { body: BOOK.papers[one.key] };
      if (one.folder) args.folder = BOOK.folders[one.folder];
      if (one.pageOf) args.page_of = kept.get(one.pageOf);
      const said = await agent.call("write_doc", args);
      kept.set(one.key, said.doc);
    });
  }
}

function archive() {
  const pool = BOOK.history;
  const scrapped = BOOK.dropped;
  const seen = new Set();
  let step = 0;
  let junked = 0;

  for (let n = 0; n < 24; n += 1) {
    const first = new Date(TODAY.getFullYear(), TODAY.getMonth() - (23 - n), 1);
    const last = new Date(first.getFullYear(), first.getMonth() + 1, 0).getDate();
    const many = 5 + Math.floor(dice() * 3);
    const days = [];
    for (let k = 0; k < many; k += 1) {
      days.push(Math.min(2 + Math.floor((last - 4) * ((k + dice() * 0.7) / many)), last));
    }
    days.sort((a, b) => a - b);

    for (const at of days) {
      const day = new Date(first.getFullYear(), first.getMonth(), at);
      if (day >= TODAY) continue;
      const entry = pool[step % pool.length];
      step += 1;
      const fresh = !seen.has(entry.t);
      seen.add(entry.t);
      const due = shift(day, -(1 + Math.floor(dice() * 9)));
      later(day, () => {
        const id = born(entry, due);
        if (fresh) flesh(id, entry);
        cli(["done", id]);
      });
    }

    if (n % 12 !== 5 && dice() < 0.85) {
      const day = new Date(first.getFullYear(), first.getMonth(), Math.min(6 + Math.floor(dice() * (last - 8)), last));
      if (day < TODAY) {
        const entry = scrapped[junked % scrapped.length];
        junked += 1;
        const due = shift(day, -(3 + Math.floor(dice() * 20)));
        later(day, () => cli(["drop", born(entry, due)]));
      }
    }
  }
}

function routines() {
  for (const one of ROUTINES) {
    const said = BOOK.routines[one.key];
    const days = [];
    let ago = one.every;
    for (let k = 0; k < one.turns; k += 1) {
      days.unshift(back(ago));
      ago += one.every + (k === one.skipAt ? one.every : 0);
    }
    const held = { id: null };
    later(days[0], () => {
      held.id = born(said, days[0]);
    });
    for (let k = 0; k < one.turns; k += 1) {
      const then = days[k + 1];
      later(days[k], () => {
        if (!held.id) return;
        cli(["done", held.id]);
        held.id = tail(held.id);
        if (held.id && then) cli(["set", held.id, "--date", stamp(then)]);
      });
    }
  }
}

function ahead() {
  for (const entry of BOOK.open) {
    later(TODAY, () => {
      const id = born(entry, entry.d == null ? null : shift(TODAY, entry.d));
      flesh(id, entry);
    });
  }
}

function span(day, many) {
  const at = new Date(`${day}T00:00:00`);
  if (stamp(at) === stamp(TODAY)) {
    const shut = NOW.getTime() - 90_000;
    return [shut - 3_600_000 - Math.max(many, 1) * 45_000, shut];
  }
  return [clock(at, 8, 20), clock(at, 21, 30)];
}

function age() {
  const held = ledgers().map((at) => ({
    at,
    lines: fs
      .readFileSync(at, "utf8")
      .split("\n")
      .filter((one) => one.trim())
      .map((one) => JSON.parse(one)),
  }));
  const all = [];
  held.forEach((file, f) => file.lines.forEach((obj, i) => all.push({ f, i, obj })));
  all.sort((a, b) => {
    if (a.obj.ts !== b.obj.ts) return a.obj.ts < b.obj.ts ? -1 : 1;
    if (a.obj.by !== b.obj.by) return a.obj.by < b.obj.by ? -1 : 1;
    return (a.obj.n || 0) - (b.obj.n || 0);
  });

  const runs = [];
  let from = 0;
  for (const one of marks) {
    if (one.upto <= from) continue;
    const last = runs[runs.length - 1];
    if (last && last.day === one.day) last.to = one.upto;
    else runs.push({ day: one.day, from, to: one.upto });
    from = one.upto;
  }
  if (all.length > from) runs.push({ day: stamp(TODAY), from, to: all.length });

  let cursor = 0;
  for (const run of runs) {
    const many = run.to - run.from;
    if (many <= 0) continue;
    let [open, shut] = span(run.day, many);
    if (open <= cursor) open = cursor + 1000;
    if (shut <= open + many * 1000) shut = open + many * 1000;
    const gap = (shut - open) / (many + 1);
    for (let k = 0; k < many; k += 1) {
      const when = Math.round(open + gap * (k + 1));
      all[run.from + k].obj.ts = new Date(when).toISOString();
      delete all[run.from + k].obj.n;
      cursor = when;
    }
  }

  for (const file of held) {
    fs.writeFileSync(file.at, `${file.lines.map((one) => JSON.stringify(one)).join("\n")}\n`, "utf8");
  }
  fs.rmSync(HOUSE.cache, { recursive: true, force: true });
  return all;
}

function counted(all) {
  const tally = { folders: 0, docs: 0, pages: 0, open: 0, done: 0, dropped: 0 };
  const status = new Map();
  let first = null;
  let last = null;
  for (const { obj } of all) {
    if (!first || obj.ts < first) first = obj.ts;
    if (!last || obj.ts > last) last = obj.ts;
    if (obj.op === "folder.add") tally.folders += 1;
    else if (obj.op === "doc.add") tally[obj.d && obj.d.page_of ? "pages" : "docs"] += 1;
    else if (obj.op === "task.add") status.set(obj.id, "open");
    else if (obj.op === "task.done") status.set(obj.id, "done");
    else if (obj.op === "task.drop") status.set(obj.id, "dropped");
    else if (obj.op === "task.reopen") status.set(obj.id, "open");
    else if (obj.op === "task.delete") status.delete(obj.id);
  }
  for (const one of status.values()) tally[one] += 1;
  return { ...tally, first, last };
}

async function main() {
  wipe();
  await moment(START, () => {
    cli(["config", "set", "locale", tongue]);
    for (const name of BOOK.lists) cli(["list", "add", name]);
    cli(["agent", "--on"]);
  });

  const agent = new Agent();
  await agent.hello();

  shelves(agent);
  papers(agent);
  archive();
  routines();
  ahead();
  schedule.sort((a, b) => a.day - b.day);

  let n = 0;
  for (const one of schedule) {
    await moment(one.day, one.run);
    n += 1;
    if (n % 20 === 0) process.stdout.write(`  ${n}/${schedule.length}\r`);
  }
  agent.close();

  const tally = counted(age());

  console.log("");
  console.log(`  perfil       ${profile} (${tongue})`);
  console.log(`  almacen      ${HOUSE.data}`);
  console.log(`  carpetas     ${tally.folders}`);
  console.log(`  documentos   ${tally.docs} y ${tally.pages} paginas`);
  console.log(`  abiertas     ${tally.open}`);
  console.log(`  cerradas     ${tally.done}`);
  console.log(`  caidas       ${tally.dropped}`);
  console.log(`  fechas       ${stamp(new Date(tally.first))} a ${stamp(new Date(tally.last))}`);
  console.log("");
  console.log(`  TISTY_PROFILE=${profile}`);
}

main().catch((why) => {
  console.error(why.stack || why.message || why);
  process.exit(1);
});
