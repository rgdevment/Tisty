import type { t } from "./locales";
import type { Tab } from "./views";

type Word = Parameters<typeof t>[0];

export interface Article {
  slug: string;
  heading?: string;
}

export interface Page {
  key: Tab;
  label: Word;
  icon: string;
  lead?: Word;
  help?: { en: Article; es: Article };
}

const SYNC = { en: "sync-and-backup", es: "sincronizar-y-respaldar" };
const SETTINGS = { en: "settings", es: "configuracion" };

const PAGES: Record<Tab, Page> = {
  summary: { key: "summary", label: "tabSummary", icon: "home", lead: "summaryLead" },
  window: {
    key: "window",
    label: "tabWindow",
    icon: "monitor",
    lead: "leadWindow",
    help: {
      en: { slug: SETTINGS.en, heading: "Look and window" },
      es: { slug: SETTINGS.es, heading: "Aspecto y ventana" },
    },
  },
  notices: {
    key: "notices",
    label: "tabNotices",
    icon: "bell",
    lead: "leadNotices",
    help: { en: { slug: "spread-and-reminders" }, es: { slug: "reparto-y-recordatorios" } },
  },
  writing: {
    key: "writing",
    label: "tabWriting",
    icon: "signature",
    lead: "leadWriting",
    help: { en: { slug: "documents" }, es: { slug: "documentos" } },
  },
  sync: {
    key: "sync",
    label: "tabSync",
    icon: "cloud",
    lead: "leadSync",
    help: { en: { slug: SYNC.en }, es: { slug: SYNC.es } },
  },
  security: {
    key: "security",
    label: "tabSecurity",
    icon: "shield",
    lead: "leadSecurity",
    help: {
      en: { slug: SYNC.en, heading: "Encrypting your cloud" },
      es: { slug: SYNC.es, heading: "Cifrar tu nube" },
    },
  },
  copies: {
    key: "copies",
    label: "bandCopies",
    icon: "backup",
    lead: "leadCopies",
    help: {
      en: { slug: SYNC.en, heading: "Backups" },
      es: { slug: SYNC.es, heading: "Respaldos" },
    },
  },
  storage: {
    key: "storage",
    label: "tabStorage",
    icon: "drive",
    lead: "leadStorage",
    help: { en: { slug: "privacy" }, es: { slug: "privacidad" } },
  },
  agents: {
    key: "agents",
    label: "tabAgents",
    icon: "wand",
    lead: "leadAgents",
    help: { en: { slug: "assistants-and-mcp" }, es: { slug: "asistentes-y-mcp" } },
  },
  health: {
    key: "health",
    label: "tabUpkeep",
    icon: "health",
    lead: "leadHealth",
    help: {
      en: { slug: SETTINGS.en, heading: "Health" },
      es: { slug: SETTINGS.es, heading: "Salud" },
    },
  },
  about: { key: "about", label: "aboutScreen", icon: "info", lead: "leadAbout" },
};

export const GROUPS: { label?: Word; brand?: string; pages: Tab[] }[] = [
  { label: "groupHere", pages: ["window", "notices", "writing"] },
  { label: "tabData", pages: ["sync", "security", "copies", "storage"] },
  { label: "groupLinks", pages: ["agents"] },
  { brand: "Tisty", pages: ["health", "about"] },
];

export const pageOf = (tab: Tab): Page => PAGES[tab];

export const articleOf = (tab: Tab, spanish: boolean): Article | undefined => {
  const help = pageOf(tab).help;
  return help && (spanish ? help.es : help.en);
};

export const TABS = Object.keys(PAGES) as Tab[];
