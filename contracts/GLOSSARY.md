# Glossary

One word, one thing. The YAML binds; this says the same in a line.

| Word | Palabra | What it is |
| --- | --- | --- |
| Document | Documento | The parent. What a person writes and keeps: words, what they contain, and its pages. |
| Page | Página | A document like any other that says which document it belongs to. One level and no more: a page has no pages. |
| Sheet | Hoja | How a document or a page is laid on paper: its size and its orientation. It holds nothing. |
| Paper | Papel | The size of a sheet: A4, Letter, Folio or Tabloid. |
| Orientation | Orientación | Which way a sheet stands: portrait or landscape. |
| Page break | Salto de página | A line of three dashes, an `hr` among the blocks of the HTML. Despite its name, it cuts a sheet, not a page. |
| Element | Elemento | What the words of a document or a page contain: text, table, code, diagram, formula, attachment, picture or widget. An icon is a mark inside the text. |
| Diagram | Diagrama | A drawing inside a document. It is no document, and it travels drawn, as SVG. |
| Attachment | Adjunto | A file a document names. A picture travels with the words and is drawn; any other file travels apart and is a card, and a sound or a video can then play. |
| Widget | Widget | A fixed view inside a document, made only of HTML and CSS. It runs nothing, names nothing outside itself, and every reader draws it the same. |
| Properties | Propiedades | The block between two lines of three dashes at the start of a document. |
| Tag | Etiqueta | A word a document or a task is kept under, in the one form Tisty keeps it. |
| Language | Idioma | Each language of a document is a document of its own, shown as one. |
| Signature | Firma | Who answers for an envelope, and through it for the document. |
| Envelope | Sobre | How a document travels, as one file the server stores unopened. Per language, one for its words, pages and pictures, and a second one for its other files only when it has any. |
| Link | Link | How a shared document is reached: the web one reads, and tisty:// hands it over to a Tisty, the only place to edit. |
| Root key | Llave raíz | Who a person is, from their 24 words. It never changes: another root is another person. |
| Machine key | Llave de máquina | What signs on one machine, under a certificate from the root. It can be replaced. |
| Revocation | Revocación | The root saying it no longer answers for a machine key, since a moment. It deletes nothing and stops nothing. |
| Verdict | Veredicto | What a reader says of a signature: holds, retired, doubtful, refused or unreadable. |
| Cited | Citado | Another document shared along with this one, in its own envelopes, with its key. |
| Sync | Sincronizar | One button, and nothing by itself: it asks whether another version arrived, brings it apart and sends what changed. |
| Owner | Dueño | The root that owns what is sealed. An editor signs a version; the owner never changes. |
| Permit | Permiso | The public key of an edit token: whoever holds the token may sign a version under their own key. |
| Writer | Quien escribe | Whoever turns words into a document ready to travel: Tisty, or a fork of it. |
| Reader | Quien lee | Whoever shows a document: Tisty or Folio. |

## Words of the project

These are not in the YAML and bind nothing: they say what each thing is called.

| Word | Palabra | What it is |
| --- | --- | --- |
| Welcome guide | Guía de bienvenida | What Tisty writes in a person's store and opens the first time it runs, from `app/src-tauri/resources/guide/`. The only thing called a guide. |
| Help | Ayuda | How Tisty works, from `docs/help/`. On the web it goes alone, at `www.tisty.app/docs`, and in Spanish at `www.tisty.app/es/ayuda`. |
| Markdown language | Lenguaje Markdown | How one writes in Tisty, from `crates/tisty-markdown/reference/`. On the web it goes apart, at `markdown.tisty.app`. In the app it goes together with the Help, and works offline from the copy it ships. |
| README | README | The cover of the repository. |
| Architecture | Arquitectura | How Tisty is built, for whoever builds it: the document «Arquitectura de Tisty», kept outside this repository. |
