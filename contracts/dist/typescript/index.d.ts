export declare const VERSION: "1.0.0-draft.4";

export type EnvelopeKind = "text" | "media";

export type PartName = "first" | "rest" | "file";

export type SyncEvery = "off" | "every5" | "every10";

export type SheetsWord = "on";

export type SaidPriority = "do" | "decide" | "delegate" | "minor";

export type Verdict = "holds" | "revoked" | "refused";

export type Paper = "a4" | "letter" | "folio" | "tabloid";

export type Orientation = "portrait" | "landscape";

export type Status = "open" | "done" | "dropped";

export type Priority = "do" | "decide" | "delegate" | "unset" | "minor";

/** A file a document names. It travels in the document's media envelope. A picture is drawn when it arrives; any other file shows as a card and is asked for the first time it is opened. */
export interface Attachment {
  at: number;
  name: string;
  kind?: string;
  size?: number;
}

/** A small isolated view inside a document, which can run its own script. Tisty runs it. Whoever does not implement this contract never runs it, and shows a card that says it is seen in Tisty. */
export interface Widget {
  title?: string;
  body: string;
}

/** A document is the parent: what a person writes and keeps. It holds its words, with all they contain, in one language, and it can hold pages. */
export interface Document {
  id: string;
  title?: string;
  tags?: Tag[];
  lang?: string;
  sheet?: Sheet;
  owner?: string;
  signed_as?: string;
  made?: string;
  wrote?: string;
  by?: string;
  born_by?: string;
  edited_by?: string;
  body: string;
  html?: string;
  pages?: Page[];
  media?: string;
  attachments?: Attachment[];
  cited?: Cited[];
  properties?: Property[];
  languages?: Translation[];
}

export interface DocumentReader {
  show(document: Document): void;
  setPaper(paper: Paper): void;
  setSheets(on: boolean): void;
  setOrientation(orientation: Orientation): void;
  index(): string[];
  details(): Document;
  verdict(): Verdict;
  goToPage(at: number): void;
  setLanguage(lang: string): void;
  openInTisty(): void;
  copyLink(): string;
}

export interface DocumentWriter {
  draw(body: string): string;
}

/** How a document travels. One envelope per language holds its words and all its pages, with references ready, so reading them takes no other request. A second one holds its pictures and files, and exists only when it has any. These fields are the manifest, written in clear, and all the server reads of an envelope. */
export interface Envelope {
  contract: string;
  kind: EnvelopeKind;
  id: string;
  version: number;
  from?: number;
  sealed: boolean;
  head?: string;
  commitment?: string;
  parts: EnvelopePart[];
}

export interface EnvelopeWriter {
  seal(document: Document): Envelope;
}

export interface EnvelopeReader {
  open(envelope: Envelope, key: string): Document;
}

/** One part of an envelope, as the manifest tells it. The server holds its limits to these sizes. */
export interface EnvelopePart {
  name: PartName;
  compressed: boolean;
  sealed_size: number;
  open_size: number;
}

export interface SyncWriter {
  sync(): void;
  setSync(every: SyncEvery): void;
}

/** The properties at the start of a document, between two lines of three dashes. The first line is ---, the words start right under it, and --- or ... closes them. Past their limits they stay as written and none is read. A value that says nothing Tisty knows stays written too and does nothing, so no document fails for its properties: the writer reads them and gives the document what they mean. This holds what they mean, once read. */
export interface FrontMatter {
  title?: string;
  tags?: string[];
  paper?: Paper;
  sheets?: SheetsWord;
  orientation?: Orientation;
  lang?: string;
  translationKey?: string;
  task?: string;
  scheduled?: string;
  due?: string;
  remind?: string[];
  list?: string;
  priority?: SaidPriority;
  repeat?: string;
}

/** One property as it was written, including those Tisty does not know, which it keeps and shows. */
export interface Property {
  key: string;
  values: string[];
}

/** A page is a document like any other, with one difference: it says which document it belongs to. There is one level and no more, so a page never has pages of its own. It travels inside its document's envelopes, so what they hold is said once. */
export interface Page {
  id: string;
  title?: string;
  tags?: Tag[];
  lang?: string;
  sheet?: Sheet;
  owner?: string;
  signed_as?: string;
  made?: string;
  wrote?: string;
  by?: string;
  born_by?: string;
  edited_by?: string;
  body: string;
  html?: string;
  properties?: Property[];
  languages?: Translation[];
  page_of: string;
  at: number;
}

/** Where the same document is in another language. */
export interface Translation {
  lang: string;
  document: string;
  title?: string;
  key?: string;
}

/** A word a document or a task is kept under, in the one form Tisty keeps it. Capitals and accents come off and a space or an underscore turns into a dash, so #Camión and #camion are one tag. In a sentence, #word needs two characters and a letter, and it does not count glued to the text before it, inside code or in [[#2]]. */
export interface Tag {
  name: string;
}

/** Who answers for an envelope, and through it for the document. The root key is who a person is; the machine key signs, under a certificate from the root. It travels in clear, so the server checks it without opening anything. */
export interface Signature {
  root: string;
  machine: string;
  since: string;
  certificate: string;
  signed: string;
}

/** Another document shared along with this one. It keeps its own envelopes, because it is not part of this document. */
export interface Cited {
  document: string;
  key: string;
  title?: string;
}

/** How a document or a page is laid on paper: its size and its orientation. A sheet holds nothing of its own, and measures on screen what it measures on paper. A line of three dashes, the page break, cuts a sheet and never a page. */
export interface Sheet {
  paper?: Paper;
  on?: boolean;
  orientation?: Orientation;
}

/** A task is something to do. It ends, and is kept whole. Only whoever needs tasks implements this. */
export interface Task {
  id: string;
  title: string;
  status: Status;
  priority?: Priority;
  description?: string;
  date?: string;
  deadline?: string;
  list?: string;
  tags?: Tag[];
  repeat?: string;
  part_of?: string;
  after?: string;
  source?: string;
  steps?: Step[];
  notes?: Note[];
  reminders?: Reminder[];
}

/** One line of a task's checklist. */
export interface Step {
  text: string;
  done: boolean;
}

/** Something recorded on a task, with when. */
export interface Note {
  at: string;
  body: string;
}

/** When a task rings. */
export interface Reminder {
  at: string;
}

export type ContractName = "text" | "table" | "code" | "diagram" | "formula" | "attachment" | "widget" | "document" | "envelope" | "envelope_part" | "sync" | "front_matter" | "property" | "page" | "translation" | "tag" | "signature" | "cited" | "sheet" | "style" | "task" | "step" | "note" | "reminder";
export declare const CONTRACTS: Record<ContractName, unknown>;
export declare const ALLOWED: Record<string, { attributes: string[]; classes: string[]; class_starts: string[] }>;
/** What is wrong with a value under this contract: nothing when it holds. */
export declare function check(contract: ContractName, value: unknown): string[];
/** What in the HTML of a document is outside the contracts. It is no cleaner: it says whether a document speaks this language. */
export declare function checkHtml(html: string): string[];
