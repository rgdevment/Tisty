// Each word once, English then Spanish, so the two languages cannot drift apart.
const words = {
  agentNoConsent: [
    "This computer has not allowed Tisty to read or write agent files",
    "Este equipo no le dio permiso a Tisty para leer o escribir archivos de agentes",
  ],
  agentOff: [
    "Sync is paused for this assistant",
    "La sincronización de este asistente está en pausa",
  ],
  agentDocChanged: [
    "This document changed since its card was drawn. Read it again before approving it.",
    "Este documento cambió desde que se mostró su ficha. Léelo de nuevo antes de aprobarlo.",
  ],
  agentHeld: [
    "A document in this file changed without your approval, so the file is left as it is",
    "Un documento de este archivo cambió sin tu aprobación, así que el archivo queda como está",
  ],
  agentOwn: [
    "The agent writes this file itself, so Tisty only reads it",
    "Este archivo lo escribe el agente, así que Tisty solo lo lee",
  ],
  agentEmpty: [
    "There is nothing to write, and Tisty never empties an agent's file",
    "No hay nada que escribir, y Tisty nunca vacía el archivo de un agente",
  ],
  agentLinked: [
    "The file is a link, and writing would replace the link",
    "El archivo es un enlace, y escribirlo reemplazaría el enlace",
  ],
  agentChanged: [
    "The file changed while Tisty was writing, so it was left alone",
    "El archivo cambió mientras Tisty lo escribía, así que no lo tocó",
  ],
  agentUnattributed: [
    "The change touches more than one document, so it cannot be brought into this one",
    "El cambio toca más de un documento, así que no se puede traer a este",
  ],
  agentNoBackup: ["That backup is gone", "Ese respaldo ya no existe"],
  agentTooBig: [
    "The file is larger than a document may hold",
    "El archivo es más grande de lo que cabe en un documento",
  ],
  agentSecret: [
    "The file reads like it holds a credential ({name}), so Tisty did not bring it in",
    "El archivo parece guardar una credencial ({name}), así que Tisty no lo trajo",
  ],
  agentNotText: [
    "The file is not plain text, so it cannot be brought into a document",
    "El archivo no es texto, así que no se puede traer a un documento",
  ],
  agentBrokenFence: [
    "The code block that holds the file is no longer whole. Mend it in the document first",
    "El bloque de código que guarda el archivo ya no está entero. Arréglalo primero en el documento",
  ],
  agentsFirstTitle: ["Your agents, in one place", "Tus agentes, en un solo lugar"],
  agentsFirstLead: [
    "Tisty looks for the rules, skills and memory of the agents you have and brings them here as documents. From then on, what you change in AGENTS reaches your agents by itself.",
    "Tisty busca las reglas, skills y memoria de los agentes que tienes y las trae aquí como documentos. Desde ese momento, lo que cambies en AGENTS llega solo a tus agentes.",
  ],
  agentsFirstPrivate: [
    "It stays in your Tisty: on your computers and wherever you sync. Nobody else sees it.",
    "Queda en tu Tisty: en tus equipos y donde tú sincronices. Nadie más lo ve.",
  ],
  agentsFirstSyncs: ["It syncs by itself with {name}.", "Se sincroniza solo con {name}."],
  agentsFirstBackup: [
    "Before changing an agent's file, Tisty keeps a backup on this computer.",
    "Antes de cambiar un archivo de un agente, Tisty guarda un respaldo en este equipo.",
  ],
  agentsFirstPick: [
    "This is what I found. Choose what to bring:",
    "Encontré esto. Elige qué traer:",
  ],
  agentsFirstNone: [
    "I found no agent files on this computer. You can still allow it: AGENTS fills up as you write your rules.",
    "No encontré archivos de agentes en este equipo. Igual puedes autorizarlo: AGENTS se llena a medida que escribes tus reglas.",
  ],
  agentsFirstCount: ["{name} · {other} files", "{name} · {other} archivos"],
  agentsFirstCountOne: ["{name} · 1 file", "{name} · 1 archivo"],
  agentsFirstHeld: ["already in AGENTS", "ya está en AGENTS"],
  agentsFirstAllHeld: ["{name} · already in AGENTS", "{name} · ya está en AGENTS"],
  agentsFirstAllHere: [
    "Everything I found is already in AGENTS. Allow this computer to keep it in sync:",
    "Todo lo que encontré ya está en AGENTS. Autoriza este equipo para mantenerlo al día:",
  ],
  agentsBroughtMany: [
    "{name} documents came into AGENTS, and Tisty keeps them in sync from now on.",
    "Entraron {name} documentos a AGENTS, y desde ahora Tisty los mantiene al día.",
  ],
  agentsBroughtOne: [
    "1 document came into AGENTS, and Tisty keeps it in sync from now on.",
    "Entró 1 documento a AGENTS, y desde ahora Tisty lo mantiene al día.",
  ],
  agentsBroughtNone: [
    "Nothing new came in. This computer now keeps AGENTS in sync.",
    "No entró nada nuevo. Desde ahora este equipo mantiene AGENTS al día.",
  ],
  agentsSkip_settings: ["settings, left out", "ajustes, no entra"],
  agentsSkip_credentials: ["credentials, left out", "credenciales, no entra"],
  agentsSkip_tisty: ["Tisty's own", "propio de Tisty"],
  agentsSkip_linked: ["a link, left out", "es un enlace, no entra"],
  agentsSkip_toobig: ["too big", "demasiado grande"],
  agentsSkip_crowded: ["too many files", "demasiados archivos"],
  agentsKind_rule: ["Rule", "Regla"],
  agentsKind_skill: ["Skill", "Skill"],
  agentsKind_memory: ["Agent memory", "Memoria del agente"],
  agentsNotNow: ["Not now", "Ahora no"],
  agentsBringSync: ["Bring and sync", "Traer y sincronizar"],
  agentsAllow: ["Allow and sync", "Autorizar y sincronizar"],
  agentsBringing: ["Bringing…", "Trayendo…"],
  agentsRefused: ["These did not come in:", "Esto no entró:"],
  agentsRefused_noConsent: [
    "This computer has not allowed it yet.",
    "Este equipo todavía no lo autorizó.",
  ],
  agentsRefused_secret: [
    "It looks like it holds a credential, such as a key or a token, so it stays out.",
    "Parece llevar una credencial, como una clave o un token, así que no entra.",
  ],
  agentsRefused_tooBig: [
    "It is larger than a document can hold.",
    "Es más grande de lo que cabe en un documento.",
  ],
  agentsRefused_notText: ["It is not text.", "No es texto."],
  agentsRefused_linked: ["It is a link, so it stays out.", "Es un enlace, así que no entra."],
  agentsRefused_name: [
    "Its name cannot be kept as an agent document.",
    "Su nombre no se puede guardar como documento de agente.",
  ],
  agentsRefused_other: ["It could not be read: {name}", "No se pudo leer: {name}"],
  agentsBothTitle: ["It exists on both sides", "Existe en los dos lados"],
  agentsBothLead: [
    "{name} already has a file that differs from Tisty's. Choose which one to keep. The other one goes to the backups.",
    "{name} ya tiene un archivo distinto del de Tisty. Elige cuál conservar. Lo otro queda en respaldos.",
  ],
  agentsBothComposed: [
    "This file is made of several documents, so from here you can only keep Tisty's or leave it as it is.",
    "Este archivo se arma con varios documentos: desde aquí solo puedes conservar el de Tisty o dejarlo como está.",
  ],
  agentsInTisty: ["In Tisty", "En Tisty"],
  agentsInFile: ["In the file", "En el archivo"],
  agentsKeepTisty: ["Keep Tisty's", "Conservar el de Tisty"],
  agentsKeepFile: ["Keep the file", "Conservar el archivo"],
  agentsLater: ["Decide later", "Decidir después"],
  agentsDone: ["Done", "Listo"],
  agentsSettingsTitle: ["AGENTS", "AGENTS"],
  agentsSettingsLine: [
    "Your agents' rules, skills and memory as Tisty documents. Tisty writes the file each agent reads.",
    "Las reglas, skills y memoria de tus agentes como documentos de Tisty. Tisty escribe el archivo que lee cada agente.",
  ],
  agentsSettingsCreate: ["Create the AGENTS folder", "Crear la carpeta AGENTS"],
  agentsSettingsAsk: [
    "This computer has not allowed it yet.",
    "Este equipo todavía no lo autorizó.",
  ],
  agentsSettingsAllow: ["Allow on this computer", "Autorizar en este equipo"],
  agentsSettingsAll: ["Sync the AGENTS folder", "Sincronizar la carpeta AGENTS"],
  agentsSettingsOne: ["Sync {name}", "Sincronizar {name}"],
  agentsSettingsNone: ["No agents found on this computer.", "No encontré agentes en este equipo."],
  agentsSettingsBothOne: [
    "1 file says something different in Tisty and in the agent.",
    "1 archivo dice algo distinto en Tisty y en el agente.",
  ],
  agentsSettingsBoth: [
    "{name} files say something different in Tisty and in the agent.",
    "{name} archivos dicen algo distinto en Tisty y en el agente.",
  ],
  agentsSettingsReview: ["Review", "Revisar"],
  agentsSettingsOff: [
    "Off: its files stay as they are, and nothing in Tisty is deleted.",
    "Apagado: sus archivos quedan como están y no se borra nada en Tisty.",
  ],
  agentCard: ["Agent document", "Documento de agente"],
  agentRule: ["Rule", "Regla"],
  agentSkill: ["Skill", "Skill"],
  agentMemory: ["Agent memory", "Memoria del agente"],
  agentGlobal: ["Global", "Global"],
  agentSynced: ["In sync", "Al día"],
  agentPending: ["About to be written", "Por escribir"],
  agentOutside: ["Changed outside", "Cambiado por fuera"],
  agentBoth: ["Changed on both sides", "Cambiado en los dos lados"],
  agentPaused: ["Paused", "En pausa"],
  agentNotAllowed: ["Not allowed on this computer", "Sin permiso en este equipo"],
  agentNothing: ["Nothing to write", "Nada que escribir"],
  agentLinkedFile: ["The file is a link", "El archivo es un enlace"],
  agentBroken: ["The code block is broken", "El bloque de código está roto"],
  agentUnreadable: ["Cannot be read", "No se puede leer"],
  agentHeldPill: ["On hold", "En espera"],
  agentHeldSaid: [
    "This file stays as it was: {name} changed or left AGENTS without your approval. Approve it, or put it back and take it out yourself.",
    "Este archivo queda como estaba: {name} cambió o salió de AGENTS sin tu aprobación. Apruébalo, o vuelve a ponerlo y sácalo tú.",
  ],
  agentUnproven: ["Written by an earlier Tisty", "Escrito por un Tisty anterior"],
  agentUnprovenSaid: [
    "An earlier Tisty wrote this file without noting which documents it came from, so it is left as it is until you choose",
    "Un Tisty anterior escribió este archivo sin anotar de qué documentos venía, así que queda como está hasta que elijas",
  ],
  agentHereWaiting: [
    "This document comes from another computer. Apply it to the agents on this one?",
    "Este documento viene de otro equipo. ¿Lo aplicas a los agentes de este?",
  ],
  agentHereDeclined: [
    "You chose not to apply it on this computer.",
    "Elegiste no aplicarlo en este equipo.",
  ],
  agentHereNotAgreed: [
    "Its text does not have your approval yet, so it reaches no agent until you approve it.",
    "Su texto aún no tiene tu aprobación, así que no llega a ningún agente hasta que lo apruebes.",
  ],
  agentHerePlacedByAgent: [
    "An agent put it here, so no agent reads it until you move it yourself.",
    "Lo puso aquí un agente, así que ningún agente lo lee hasta que lo muevas tú.",
  ],
  agentHereOtherMemory: [
    "Another computer's agent memory: on this one it is only read.",
    "Memoria del agente de otro equipo: en este solo se lee.",
  ],
  agentApplyHere: ["Apply here", "Aplicar aquí"],
  agentNotHere: ["Not on this computer", "No en este equipo"],
  agentAgree: ["Approve this text", "Aprobar este texto"],
  agentAgreeSure: [
    "Approve what this document says now? Read it first: it is written to your agents' files.",
    "¿Apruebas lo que dice ahora este documento? Léelo antes: se escribe en los archivos de tus agentes.",
  ],
  agentMemoryOwn: [
    "The agent writes this file itself: Tisty shows it and brings it in, and never writes over it.",
    "Este archivo lo escribe el agente: Tisty lo muestra y lo trae, y nunca lo sobrescribe.",
  ],
  agentNowhere: [
    "No assistant on this computer reads this document yet.",
    "Ningún asistente de este equipo lee este documento todavía.",
  ],
  agentTwin: [
    "Another document in AGENTS already writes this file, so this one is left out.",
    "Otro documento de AGENTS ya escribe este archivo, así que este queda fuera.",
  ],
  agentWrittenTo: ["{name} reads it at", "{name} lo lee en"],
  agentAsCode: ["Written as is", "Se escribe tal cual"],
  agentAsMarkdown: ["Written as Markdown", "Se escribe como Markdown"],
  agentComposed: ["together with {name} other documents", "junto con otros {name} documentos"],
  agentComposedOne: ["together with one other document", "junto con otro documento"],
  agentFolderOff: [
    "Sync is off for the whole AGENTS folder. Turn it back on in Settings › Assistants.",
    "La sincronización está apagada para toda la carpeta AGENTS. Vuelve a encenderla en Ajustes › Asistentes.",
  ],
  agentChangedAt: ["file changed {name}", "el archivo cambió {name}"],
  agentBackups: ["{name} backups", "{name} respaldos"],
  agentBackupsOne: ["1 backup", "1 respaldo"],
  agentBackupsAll: ["Backups", "Respaldos"],
  agentBackupsOf: [
    "What {name} held before Tisty wrote over it. The last 20 are kept.",
    "Lo que tenía {name} antes de que Tisty lo sobrescribiera. Se guardan los últimos 20.",
  ],
  agentBackupNamed: ["Backup #{name} · {other}", "Respaldo n.º {name} · {other}"],
  agentBackupTitle: ["Backup #{name}", "Respaldo n.º {name}"],
  agentPause: ["Pause {name}", "Pausar {name}"],
  agentResume: ["Resume {name}", "Reanudar {name}"],
  agentPauseSync: ["Pause sync with {name}", "Pausar la sincronización con {name}"],
  agentResumeSync: ["Resume sync with {name}", "Reanudar la sincronización con {name}"],
  agentOutsideSaid: ["The file changed outside Tisty", "El archivo cambió por fuera de Tisty"],
  agentBothSaid: [
    "The file and this document both changed",
    "Cambiaron el archivo y este documento",
  ],
  agentOutsideWhere: [
    "{name} no longer says what this document says. Choose which to keep: whatever you leave goes to the backups.",
    "{name} ya no dice lo mismo que este documento. Elige cuál conservar: lo que dejes queda en los respaldos.",
  ],
  agentBring: ["Bring the change in", "Traer el cambio"],
  agentOverwrite: ["Overwrite with Tisty", "Sobrescribir con Tisty"],
  agentKeepFile: ["Keep the file's version", "Quedarse con el archivo"],
  agentKeepFileSure: [
    "Make this document read as {name} does now? What it says now stays in its history.",
    "¿Dejar este documento como está ahora {name}? Lo que dice ahora queda en su historial.",
  ],
  agentSeeFile: ["See the file", "Ver el archivo"],
  agentSee: ["See", "Ver"],
  agentRestore: ["Restore", "Recuperar"],
  agentEdit: ["Edit", "Editar"],
  agentDoneEditing: ["Done", "Listo"],
  agentFileNow: ["The file as it is now", "El archivo como está ahora"],
  agentOverwriteSure: [
    "Write this document over {name}? What the file holds now is kept as a backup.",
    "¿Escribir este documento sobre {name}? Lo que tiene ahora el archivo queda en un respaldo.",
  ],
  agentRestoreSure: [
    "Put this backup back in {name}? What the file holds now is kept as a backup.",
    "¿Devolver este respaldo a {name}? Lo que tiene ahora el archivo queda en un respaldo.",
  ],
  agentsFolderEmpty: ["empty", "vacía"],
  agentsFolderSynced: ["in sync", "al día"],
  agentsFolderChanges: ["{name} changed", "{name} con cambios"],
  agentsFolderWaiting: ["{name} waiting", "{name} en espera"],
  agentsFolderUnagreed: ["{name} to approve", "{name} por aprobar"],
  agentsPaused: ["off", "apagada"],
  agentNoCard: [
    "This document is not an agent document",
    "Este documento no es un documento de agente",
  ],
  agentNoPlace: [
    "No assistant reads this document at that path any more",
    "Ningún asistente lee ya este documento en esa ruta",
  ],
  agentBringSame: [
    "This document already says what the file says; only the way the file ends differs. Overwrite it with Tisty to make them agree.",
    "Este documento ya dice lo mismo que el archivo; solo cambia cómo termina el archivo. Sobrescríbelo con Tisty para que coincidan.",
  ],
} as const satisfies Record<string, readonly [string, string]>;

type Key = keyof typeof words;

const side = (n: 0 | 1) =>
  Object.fromEntries(Object.entries(words).map(([key, said]) => [key, said[n]])) as Record<
    Key,
    string
  >;

export const agentsEn = side(0);
export const agentsEs = side(1);
