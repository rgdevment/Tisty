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
  capture: "Capture a task",
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
  capture: "Capturar una tarea",
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
