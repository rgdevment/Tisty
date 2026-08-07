const en = {
  inbox: "Inbox",
  today: "Today",
  upcoming: "Upcoming",
  tags: "Tags",
  archive: "Archive",
  search: "Search",
  lists: "Lists",
  nothingSelected: "Nothing selected",
  nothingOpen: "Nothing here",
  noTagsYet: "No tags yet",
  addTask: "Add a task",
  addToInbox: "Add to the inbox",
  addForToday: "Add for today",
  addToList: "Add to {name}",
  addWithTag: "Add with {name}",
  searchEverywhere: "Search everywhere, archive included",
  searchArchive: "Search the archive",
  scopeEither: "All",
  scopeOpen: "Open",
  scopeArchived: "Archived",
  untitled: "A title is required",
  noSuchList: "No list matches {name}",
  ambiguousList: "Several lists match {name}",
  badTag: "{name} is not a valid tag",
  notATaskId: "That is not a task",
  notAListId: "That is not a list",
  internal: "{name}",
  steps: "Steps",
  journal: "Journal",
  description: "Description",
  expand: "Full screen",
  collapse: "Back to three columns",
  urgent: "Urgent",
  high: "High",
  medium: "Medium",
  minimise: "Minimise",
  maximise: "Maximise",
  close: "Close",
  fieldDate: "date",
  fieldDeadline: "deadline",
  fieldList: "list",
  fieldTag: "tag",
  fieldPriority: "priority",
  hintDates: "dates as you say them",
  hintPick: "to pick",
};

/// Missing a key in another catalogue is a compile error, not a runtime «⟨?⟩».
type Catalog = typeof en;

const es: Catalog = {
  inbox: "Bandeja de entrada",
  today: "Hoy",
  upcoming: "Próximo",
  tags: "Etiquetas",
  archive: "Archivo",
  search: "Buscador",
  lists: "Listas",
  nothingSelected: "Nada seleccionado",
  nothingOpen: "Nada por aquí",
  noTagsYet: "Ninguna etiqueta todavía",
  addTask: "Añadir una tarea",
  addToInbox: "Añadir a la bandeja",
  addForToday: "Añadir para hoy",
  addToList: "Añadir a {name}",
  addWithTag: "Añadir con {name}",
  searchEverywhere: "Buscar en todo, incluido el archivo",
  searchArchive: "Buscar en el archivo",
  scopeEither: "Todas",
  scopeOpen: "Abiertas",
  scopeArchived: "Archivadas",
  untitled: "Hace falta un título",
  noSuchList: "No encontré ninguna lista que coincida con {name}",
  ambiguousList: "Varias listas coinciden con {name}",
  badTag: "{name} no es una etiqueta válida",
  notATaskId: "Eso no es una tarea",
  notAListId: "Eso no es una lista",
  internal: "{name}",
  steps: "Pasos",
  journal: "Bitácora",
  description: "Descripción",
  expand: "Pantalla completa",
  collapse: "Volver a tres columnas",
  urgent: "Urgente",
  high: "Alta",
  medium: "Media",
  minimise: "Minimizar",
  maximise: "Maximizar",
  close: "Cerrar",
  fieldDate: "fecha",
  fieldDeadline: "límite",
  fieldList: "lista",
  fieldTag: "etiqueta",
  fieldPriority: "prioridad",
  hintDates: "fechas tal como se dicen",
  hintPick: "para elegir",
};

const catalogs: Record<string, Catalog> = { en, es };

let code = navigator.language;
let spoken = en;

const known = (raw: string) => catalogs[raw.split(/[-_.]/)[0].toLowerCase()];

export function adopt(configured?: string) {
  code = configured ?? navigator.language;
  spoken = known(code) ?? en;
}

adopt();

export const locale = (): string => code;
export const t = (key: keyof Catalog): string => spoken[key];

export const fill = (key: keyof Catalog, name: string): string =>
  spoken[key].replace("{name}", name);
