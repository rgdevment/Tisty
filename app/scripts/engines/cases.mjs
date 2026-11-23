
export const WRITING = [
  "# Título del documento", "",
  "Primer párrafo con algunas palabras y **negrita** en medio.", "",
  "## Sección", "",
  "- uno", "- dos", "- tres", "",
  "- [ ] tarea pendiente", "- [x] tarea hecha", "",
  "1. primero", "2. segundo", "",
  "> una cita", "",
  "Último párrafo.", "",
].join("\n");

const LINE_WALKS_ONLY_ON_A_MAC = "Control+A and Control+E walk to a line's edges only on a Mac";
const COMMAND_WALKS_ONLY_ON_A_MAC = "Cmd with a side arrow walks to a line's edge only on a Mac";
const PAGE_REACH_ONLY_ON_A_MAC = "Cmd, Shift and Down choose to the page's end only on a Mac";

export const PIECES = [
  "# Objetos", "",
  "Antes de la imagen.", "",
  "![Rina](rina.png)", "",
  "Texto con un [enlace](https://example.com) en medio.", "",
  "```js", "const a = 1;", "```", "",
  "Tras el código.", "",
  "```mermaid", "flowchart LR", "  A --> B", "```", "",
  "Tras el diagrama.", "",
  "```widget", '<div class="grid"><div class="stat"><b>42</b></div></div>', "```", "",
  "Tras el widget.", "",
  "| a | b |", "|---|---|", "| 1 | 2 |", "",
  "Tras la tabla.", "",
  "---", "",
  "Tras la línea.", "",
  "![Doc](tisty:doc/abcdefgh-0001)", "",
  "Tras la tarjeta.", "",
  "Una imagen ![mini](mini.png) entre palabras.", "",
  "Fin.", "",
].join("\n");

export const LONG = `# Largo\n\n${Array.from({ length: 120 }, (_, n) => `Párrafo número ${n} con algo de texto para rellenar.`).join("\n\n")}\n`;

const NO_LINE_DELETE_ON_WINDOWS = "Windows has no key that deletes back to the start of a line";

const MISSING_PICTURE = PIECES.replace("](mini.png)", "](missing.png)");

const FORMULA_IN_WORDS = "# Fórmula\n\nAntes $x^2$ después\n";

const drawnFirst = (frame) => frame.parentFrame().waitForFunction(() => window.engines.drawn());

const pressedOnceDrawn = async (frame, press, key, many) => {
  for (let time = 0; time < many; time++) {
    await drawnFirst(frame);
    await press(key);
  }
};

const FOLD_WITH_TWO =
  "# Marcas\n\nAntes y más\n\n:::details Plegable\n\nContenido\n\nOtro\n\n:::\n\nDespués\n";

const ITEMS_UNDER_ONE = "# Listas\n\n- uno\n    - sub a\n    - sub b\n- dos\n";

const FOLD_AT_THE_END =
  "# Marcas\n\nAntes y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::\n";

const caretInWords = async (frame, words, offset) => {
  await drawnFirst(frame);
  await frame.evaluate(
    ([said, at]) => {
      const walk = document.createTreeWalker(document.querySelector(".page"), NodeFilter.SHOW_TEXT);
      for (let node = walk.nextNode(); node; node = walk.nextNode()) {
        if (node.data !== said) continue;
        getSelection().collapse(node, at);
        return;
      }
      throw new Error(`no "${said}" alone in a text on the page`);
    },
    [words, offset],
  );
};

export const CASES = [
  {
    name: "newPageTitleTyped",
    doc: "",
    run: async ({ typed }) => { await typed("Hola"); },
    was: "",
    now: "# Hola",
  },
  {
    name: "newPageTitleWithCaretBesideIt",
    doc: "",
    run: async ({ typed, frame }) => {
      await frame.evaluate(() => getSelection().collapse(document.querySelector(".page"), 1));
      await typed("Hola");
    },
    was: "",
    now: "# Hola",
  },
  {
    name: "typeThenBackspace",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("algunas"); await typed(" xyz"); await press("Backspace", 2); },
    was: "",
    now: "x ",
  },
  {
    name: "typeThenDelete",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("algunas"); await typed(" xyz"); await press("ArrowLeft", 2); await press("Delete"); },
    was: "",
    now: "xz ",
  },
  {
    name: "typeThenWordBack",
    doc: WRITING,
    run: async ({ clickAt, typed, press, wordBack, page, frame }) => { await clickAt("algunas"); await typed(" uno dos"); await wordBack(); },
    was: "",
    now: "uno  ",
  },
  {
    name: "enterTypeBackspace",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Sección"); await press("Enter"); await typed("abc"); await press("Backspace"); await typed("Z"); },
    was: "",
    now: "abZ\n\n",
  },
  {
    name: "spaceEndPara",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("en medio."); await typed(" Otra frase aquí"); },
    was: "",
    now: " Otra frase aquí",
  },
  {
    name: "spaceEndListItem",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("dos"); await typed(" más cosas"); },
    was: "",
    now: " más cosas",
  },
  {
    name: "spaceEndHeading",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Sección"); await typed(" dos"); },
    was: "",
    now: " dos",
  },
  {
    name: "typeThenArrowType",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("algunas"); await typed("XY"); await press("ArrowLeft"); await typed("Z"); },
    was: "",
    now: "XZY",
  },
  {
    name: "backspaceToEmptyThenType",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Último párrafo."); await press("Backspace", 15); await typed("Nuevo"); },
    was: "Último párrafo.",
    now: "Nuevo",
  },
  {
    name: "emoji",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("cita"); await typed(" 👍🏽"); await press("Backspace"); },
    was: "",
    now: " ",
  },
  {
    name: "typeMid",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("algunas"); await typed(" nuevas"); },
    was: "",
    now: "nuevas ",
  },
  {
    name: "typeEndDoc",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Último párrafo."); await typed(" Fin"); },
    was: "",
    now: " Fin",
  },
  {
    name: "enterMid",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("con algunas"); await press("Enter"); await typed("X"); },
    was: "",
    now: "\n\nX",
  },
  {
    name: "enterEndHeading",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Sección"); await press("Enter"); await typed("bajo"); },
    was: "",
    now: "bajo\n\n",
  },
  {
    name: "listEnter",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("dos"); await press("Enter"); await typed("dos y medio"); },
    was: "",
    now: "dos y medio\n- ",
  },
  {
    name: "listEnterEmptyExits",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("tres"); await press("Enter"); await press("Enter"); await typed("fuera"); },
    was: "",
    now: "fuera\n\n",
  },
  {
    name: "taskEnter",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("pendiente"); await press("Enter"); await typed("otra"); },
    was: "",
    now: " ] otra\n- [",
  },
  {
    name: "orderedEnter",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("segundo"); await press("Enter"); await typed("tercero"); },
    was: "",
    now: "3. tercero\n",
  },
  {
    name: "quoteType",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("cita"); await typed(" larga"); },
    was: "",
    now: " larga",
  },
  {
    name: "backspaceMid",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("palabras"); await press("Backspace", 3); },
    was: "ras",
    now: "",
  },
  {
    name: "backspaceJoin",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Último", true); await press("Backspace"); },
    was: "\n\n",
    now: "",
  },
  {
    name: "deleteForwardJoin",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("cita"); await press("Delete"); },
    was: "\n\n",
    now: "",
  },
  {
    name: "backspaceListStart",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("dos", true); await press("Backspace"); },
    was: "- dos",
    now: "\ndos\n",
  },
  {
    name: "wordBackspace",
    doc: WRITING,
    run: async ({ clickAt, typed, press, wordBack, page, frame }) => { await clickAt("palabras"); await wordBack(); },
    was: "palabras",
    now: "",
  },
  {
    name: "lineBackspace",
    macOnly: NO_LINE_DELETE_ON_WINDOWS,
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("palabras"); await press("Meta+Backspace"); },
    was: "Primer párrafo con algunas palabras",
    now: "",
  },
  {
    name: "selectOver",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("algunas"); await press("Shift+ArrowLeft", 7); await typed("varias"); },
    was: "algun",
    now: "vari",
  },
  {
    name: "selectAcrossDelete",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("negrita"); await press("Shift+ArrowDown", 2); await press("Backspace"); },
    was: " en medio.\n\n## Sección\n\n- uno",
    now: "",
  },
  {
    name: "selectAllType",
    doc: WRITING,
    run: async ({ clickAt, typed, press, command, page, frame }) => { await clickAt("Primer"); await command("a"); await typed("Todo"); },
    was: "ítulo del documento\n\nPrimer párrafo con algunas palabras y **negrita** en medio.\n\n## Sección\n\n- uno\n- dos\n- tres\n\n- [ ] tarea pendiente\n- [x] tarea hecha\n\n1. primero\n2. segundo\n\n> una cita\n\nÚltimo párrafo.",
    now: "odo",
  },
  {
    name: "arrowsThenType",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("uno"); await press("ArrowDown"); await press("End"); await typed("!"); },
    was: "",
    now: "!",
  },
  {
    name: "tabNest",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("dos"); await press("Tab"); },
    was: "",
    now: "  ",
  },
  {
    name: "shiftTab",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("dos"); await press("Tab"); await press("Shift+Tab"); },
    was: "",
    now: "",
  },
  {
    name: "boldKeys",
    doc: WRITING,
    run: async ({ clickAt, typed, press, command, page, frame }) => { await clickAt("párrafo"); await command("b"); await typed("fuerte"); await command("b"); await typed(" normal"); },
    was: "",
    now: "**fuerte** normal",
  },
  {
    name: "undoRedo",
    doc: WRITING,
    run: async ({ clickAt, typed, press, command, page, frame }) => { await clickAt("algunas"); await typed(" uno"); await page.waitForTimeout(1100); await typed(" dos"); await command("z"); await command("Shift+z"); },
    was: "",
    now: "uno dos ",
  },
  {
    name: "insideBold",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("negr"); await typed("XX"); },
    was: "",
    now: "XX",
  },
  {
    name: "typeAfterBoldEnd",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("negrita"); await typed("ZZ"); },
    was: "",
    now: "ZZ",
  },
  {
    name: "hashTag",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Último párrafo."); await typed(" #etiqueta "); },
    was: "",
    now: " #etiqueta ",
  },
  {
    name: "emDashAccent",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("cita"); await typed(" ñandú café"); },
    was: "",
    now: " ñandú café",
  },
  {
    name: "checkboxClick",
    doc: WRITING,
    run: async ({ clickAt, typed, press, page, frame }) => { const off = await frame.frameElement().then((e) => e.boundingBox()); const b = await frame.evaluate(() => { const c = document.querySelector(".page .check, .page input[type=checkbox]"); const r = c?.getBoundingClientRect(); return r && { x: r.left + r.width/2, y: r.top + r.height/2 }; }); if (b) await page.mouse.click(off.x + b.x, off.y + b.y); await page.waitForTimeout(300); },
    was: "",
    now: "",
  },
  {
    name: "backStartAfterPicture",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Texto", true); await press("Backspace"); },
    was: "![Rina](rina.png)",
    now: "",
  },
  {
    name: "deleteEndBeforePicture",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Antes de la imagen."); await press("Delete"); },
    was: "![Rina](rina.png)",
    now: "",
  },
  {
    name: "backAfterLinkText",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("enlace"); await press("Backspace"); },
    was: "e",
    now: "",
  },
  {
    name: "backInsideLink",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("enl"); await press("Backspace"); },
    was: "l",
    now: "",
  },
  {
    name: "typeAfterLink",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("enlace"); await typed("X"); },
    was: "",
    now: "X",
  },
  {
    name: "backStartAfterCode",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras el código", true); await press("Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "wordBackStartAfterCode",
    doc: PIECES,
    run: async ({ clickAt, typed, press, wordBack, page, frame }) => { await clickAt("Tras el código", true); await wordBack(); },
    was: "",
    now: "",
  },
  {
    name: "lineBackStartAfterCode",
    macOnly: NO_LINE_DELETE_ON_WINDOWS,
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras el código", true); await press("Meta+Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "backStartAfterDiagram",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras el diagrama", true); await press("Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "lineBackStartAfterDiagram",
    macOnly: NO_LINE_DELETE_ON_WINDOWS,
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras el diagrama", true); await press("Meta+Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "backStartAfterWidget",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras el widget", true); await press("Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "backStartAfterTable",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras la tabla", true); await press("Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "lineBackStartAfterTable",
    macOnly: NO_LINE_DELETE_ON_WINDOWS,
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras la tabla", true); await press("Meta+Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "wordBackStartAfterTable",
    doc: PIECES,
    run: async ({ clickAt, typed, press, wordBack, page, frame }) => { await clickAt("Tras la tabla", true); await wordBack(); },
    was: "",
    now: "",
  },
  {
    name: "backStartAfterRule",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras la línea", true); await press("Backspace"); },
    was: "---",
    now: "",
  },
  {
    name: "backStartAfterCard",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras la tarjeta", true); await press("Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "lineBackStartAfterCard",
    macOnly: NO_LINE_DELETE_ON_WINDOWS,
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras la tarjeta", true); await press("Meta+Backspace"); },
    was: "",
    now: "",
  },
  {
    name: "backAfterInlinePicture",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt(" entre palabras", true); await press("Backspace"); },
    was: "![mini](mini.png)",
    now: "",
  },
  {
    name: "deleteBeforeInlinePicture",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Una imagen"); await press("ArrowRight"); await press("Delete"); },
    was: "![mini](mini.png)",
    now: "",
  },
  {
    name: "selectAcrossCodeDelete",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("medio."); await press("Shift+ArrowDown", 2); await press("Backspace"); },
    was: "js\nconst a = 1;\n```\n\nTras el códig",
    now: "",
  },
  {
    name: "enterAfterCodeThenType",
    doc: PIECES,
    run: async ({ clickAt, typed, press, page, frame }) => { await clickAt("Tras el código", true); await press("Enter"); await typed("N"); },
    was: "",
    now: "N\n\n",
  },
  {
    name: "headingFromLine",
    doc: WRITING,
    run: async ({ clickAt, typed }) => {
      await clickAt("Último", true);
      await typed("## ");
    },
    was: "",
    now: "## ",
  },
  {
    name: "chooseAllLongType",
    doc: LONG,
    run: async ({ clickAt, typed, command }) => {
      await clickAt("número 60 con");
      await command("a");
      await typed("todo");
    },
    was: LONG.slice(2, LONG.length - 1),
    now: "todo",
  },
  {
    name: "firstItemBackspace",
    doc: "- [ ] uno\n\nFin.\n",
    run: async ({ clickAt, press }) => {
      await clickAt("uno", true);
      await press("Backspace");
    },
    was: "[ ]",
    now: "",
  },
  {
    name: "backspaceInTable",
    doc: PIECES,
    run: async ({ press, page, frame }) => {
      const cell = await frame.evaluate(() => {
        const first = document.querySelector(".page td");
        first.scrollIntoView({ block: "center" });
        const box = first.getBoundingClientRect();
        return { x: box.left + 2, y: box.top + box.height / 2 };
      });
      const sheet = await (await frame.frameElement()).boundingBox();
      await page.mouse.click(sheet.x + cell.x, sheet.y + cell.y);
      await page.waitForTimeout(150);
      await press("Backspace");
    },
    was: "",
    now: "",
  },
  {
    name: "controlEPastTagAtLineEnd",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Etiquetas\n\nAntes del final #etiqueta\n\nSiguiente línea.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("Antes", true);
      await busyTyped(["Control+e"], "X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes del final #etiqueta",
    now: "YAntes del final #etiquetaX",
  },
  {
    name: "controlEPastCodeAtLineEnd",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Código\n\nAntes del final `cod`\n\nSiguiente línea.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("Antes", true);
      await busyTyped(["Control+e"], "X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes del final `cod`",
    now: "YAntes del final `cod`X",
  },
  {
    name: "controlEPastFormulaAtLineEnd",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Fórmula\n\nAntes del final $x^2$\n\nSiguiente línea.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("Antes", true);
      await busyTyped(["Control+e"], "X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes del final $x^2$",
    now: "YAntes del final $x^2$X",
  },
  {
    name: "controlABeforeTagAtLineStart",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Etiquetas\n\n#etiqueta al inicio\n\nSiguiente línea.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("al inicio");
      await busyTyped(["Control+a"], "X");
      await clickAt("al inicio");
      await typed("Y");
    },
    was: "#etiqueta al inicio",
    now: "X#etiqueta al inicioY",
  },
  {
    name: "footnoteMarkKeepsTheCaret",
    doc: "# Notas\n\nTexto con nota[^1] y más palabras.\n\n[^1]: La nota al pie.\n",
    run: async ({ clickAt, typed, page, frame }) => {
      await clickAt("palabras");
      const mark = await frame.evaluate(() => {
        const box = document.querySelector(".page sup a").getBoundingClientRect();
        return { x: box.left + box.width / 2, y: box.top + box.height / 2 };
      });
      const sheet = await (await frame.frameElement()).boundingBox();
      await page.mouse.click(sheet.x + mark.x, sheet.y + mark.y);
      await page.waitForTimeout(150);
      await typed("X");
      await clickAt("La nota", true);
      await typed("Y");
    },
    was: ".\n\n[^1]: ",
    now: "X.\n\n[^1]: Y",
  },
  {
    name: "deleteSpaceBeforeInlinePicture",
    doc: PIECES,
    run: async ({ clickAt, press }) => { await clickAt("Una imagen"); await press("Delete"); },
    was: " ",
    now: "",
  },
  {
    name: "deleteBeforeMissingPicture",
    doc: MISSING_PICTURE,
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Una");
      await caretInWords(frame, "Una imagen ", "Una imagen ".length);
      await press("Delete");
    },
    was: "![mini](missing.png)",
    now: "",
  },
  {
    name: "backAfterMissingPicture",
    doc: MISSING_PICTURE,
    run: async ({ clickAt, press }) => {
      await clickAt(" entre palabras", true);
      await press("Backspace");
    },
    was: "![mini](missing.png)",
    now: "",
  },
  {
    name: "commandRightPastTagAtLineEnd",
    macOnly: COMMAND_WALKS_ONLY_ON_A_MAC,
    doc: "# Etiquetas\n\nAntes del final #etiqueta\n\nSiguiente línea.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("Antes", true);
      await busyTyped(["Meta+ArrowRight"], "X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes del final #etiqueta",
    now: "YAntes del final #etiquetaX",
  },
  {
    name: "commandLeftKeepsAStepOpenedByATag",
    macOnly: COMMAND_WALKS_ONLY_ON_A_MAC,
    doc: "# Pasos\n\n- [ ] uno\n- [ ] #etiqueta dos\n\nFin.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("dos");
      await busyTyped(["Meta+ArrowLeft", "Meta+ArrowRight"], "X");
      await clickAt("uno", true);
      await typed("Y");
    },
    was: "uno\n- [ ] #etiqueta dos",
    now: "Yuno\n- [ ] #etiqueta dosX",
  },
  {
    name: "controlEStopsAtTheLineBreakAfterATag",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("uno", true);
      await busyTyped(["Control+e"], "X");
      await clickAt("uno", true);
      await typed("Y");
    },
    was: "uno #etiqueta",
    now: "Yuno #etiquetaX",
  },
  {
    name: "controlAStartsTheLineAfterABreak",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Saltos\n\nuno\\\n#etiqueta tres\n\nFin.\n",
    run: async ({ clickAt, typed, busyTyped }) => {
      await clickAt("tres");
      await busyTyped(["Control+a"], "X");
      await clickAt("tres");
      await typed("Y");
    },
    was: "#etiqueta tres",
    now: "X#etiqueta tresY",
  },
  {
    name: "commandShiftEndChoosesToTheLastWords",
    doc: LONG,
    run: async ({ clickAt, typed, command }) => {
      await clickAt("Largo");
      await command("Shift+End");
      await typed("X");
    },
    was: LONG.slice(7, LONG.length - 1),
    now: "X",
  },
  {
    name: "commandShiftEndCutsThroughATableAtTheEnd",
    doc: "# Tabla\n\nAntes.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, command, cut }) => {
      await clickAt("Antes");
      await command("Shift+End");
      await cut();
    },
    was: ".\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    now: "",
  },
  {
    name: "leftTypesBeforeATagThatOpensAStep",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] #etiqueta dos\n\nFin.\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 0);
      await press("ArrowLeft");
      await typed("X");
      await caretInWords(frame, "uno", 0);
      await typed("Y");
    },
    was: "uno\n- [ ] ",
    now: "Yuno\n- [ ] X",
  },
  {
    name: "rightFromTheStepAboveTypesBeforeATagThatOpensAStep",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] #etiqueta dos\n\nFin.\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno", 3);
      await press("ArrowRight");
      await typed("X");
      await caretInWords(frame, "uno", 0);
      await typed("Y");
    },
    was: "uno\n- [ ] ",
    now: "Yuno\n- [ ] X",
  },
  {
    name: "backspaceAfterATagThatOpensAStepTakesTheTag",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] #etiqueta dos\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 0);
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "leftTypesBeforeCodeThatOpensAStep",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] `cod` dos\n\nFin.\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 0);
      await press("ArrowLeft");
      await typed("X");
      await caretInWords(frame, "uno", 0);
      await typed("Y");
    },
    was: "uno\n- [ ] ",
    now: "Yuno\n- [ ] X",
  },
  {
    name: "backspaceAfterCodeThatOpensAStepTakesTheCode",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] `cod` dos\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 0);
      await press("Backspace");
    },
    was: "`cod`",
    now: "",
  },
  {
    name: "commandLeftStandsBeforeATagThatOpensAStep",
    macOnly: COMMAND_WALKS_ONLY_ON_A_MAC,
    doc: "# Pasos\n\n- [ ] uno\n- [ ] #etiqueta dos\n\nFin.\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 2);
      await press("Meta+ArrowLeft");
      await typed("X");
      await caretInWords(frame, "uno", 0);
      await typed("Y");
    },
    was: "uno\n- [ ] ",
    now: "Yuno\n- [ ] X",
  },
  {
    name: "controlAStandsBeforeATagThatOpensAStep",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Pasos\n\n- [ ] uno\n- [ ] #etiqueta dos\n\nFin.\n",
    run: async ({ clickAt, typed, busyTyped, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 2);
      await busyTyped(["Control+a"], "X");
      await caretInWords(frame, "uno", 0);
      await typed("Y");
    },
    was: "uno\n- [ ] ",
    now: "Yuno\n- [ ] X",
  },
  {
    name: "backspaceOnWordsChosenInsideATagTakesTheTag",
    doc: "# Etiquetas\n\nHola #etiqueta mundo\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("mundo");
      await frame.evaluate(() => {
        const tag = document.querySelector(".page .tag")?.firstChild;
        if (!tag) throw new Error("no tag on the page");
        getSelection().setBaseAndExtent(tag, 1, tag, 5);
      });
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "deletingAParagraphsFirstWordLeavesTheCaretOnItsWords",
    doc: "# Palabras\n\nAntes.\n\nHola mundo\n\nFin.\n",
    run: async ({ clickAt, typed, press, wordBack, page, frame }) => {
      await clickAt("mundo");
      await caretInWords(frame, "Hola mundo", 4);
      await wordBack();
      await page.waitForTimeout(400);
      await press("ArrowRight");
      await typed("X");
    },
    was: "Hola m",
    now: " mX",
  },
  {
    name: "deleteSpaceBeforeMissingPicture",
    doc: MISSING_PICTURE,
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Una");
      await caretInWords(frame, "Una imagen ", "Una imagen ".length - 1);
      await press("Delete");
    },
    was: " ",
    now: "",
  },
  {
    name: "wordBackTakesAStopAloneAfterBold",
    doc: "# Marcas\n\n我们**公园**。\n",
    run: async ({ clickAt, wordBack, frame }) => {
      await clickAt("我们");
      await caretInWords(frame, "。", 1);
      await wordBack();
    },
    was: "。",
    now: "",
  },
  {
    name: "wordForwardTakesAStopAloneBeforeBold",
    doc: "# Marcas\n\n。**公园**我们\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("我们");
      await caretInWords(frame, "。", 0);
      await press((await macIn(frame)) ? "Alt+Delete" : "Control+Delete");
    },
    was: "。",
    now: "",
  },
  {
    name: "deleteChosenAcrossBoldEnd",
    doc: "# Marcas\n\nAntes **negrita** después\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await chosenIn(frame, ["negrita", 3], [" después", 3]);
      await press("Backspace");
    },
    was: "rita** de",
    now: "**",
  },
  {
    name: "typeOverChosenAcrossBoldEnd",
    doc: "# Marcas\n\nAntes **negrita** después\n",
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("Antes");
      await chosenIn(frame, ["negrita", 3], [" después", 3]);
      await typed("X");
    },
    was: "rita** de",
    now: "X**",
  },
  {
    name: "lineBackInsideBold",
    macOnly: NO_LINE_DELETE_ON_WINDOWS,
    doc: "# Marcas\n\nAntes **negrita** después\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "negrita", 3);
      await press("Meta+Backspace");
    },
    was: "Antes **neg",
    now: "**",
  },
  {
    name: "backBetweenTagAndFootnote",
    doc: "# Marcas\n\nTexto #tag[^1] fin.\n\n[^1]: Nota.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Texto");
      await caretAfterPiece(frame, ".page p .tag");
      await press("Backspace");
    },
    was: "#tag",
    now: "",
  },
  {
    name: "backBetweenPictureAndTagInLaterItem",
    doc: "# Marcas\n\n- uno\n- ![p](mini.png)#tag dos\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretAfterPiece(frame, ".page li img");
      await press("Backspace");
    },
    was: "![p](mini.png)",
    now: "",
  },
  {
    name: "deleteBeforeTaskBoxTakesTheBox",
    doc: "# Marcas\n\n- [ ] uno\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await frame.evaluate(() => getSelection().collapse(document.querySelector(".page li"), 0));
      await press("Delete");
    },
    was: "[ ]",
    now: "",
  },
  {
    name: "backAfterAnUnderscoreLeftOpen",
    doc: "# Marcas\n\n*a _b* c_\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Marcas");
      await caretAtEndOf(frame, ".page p");
      await press("Backspace");
    },
    was: "_",
    now: "",
  },
  {
    name: "boldWaitsPastABackspaceHeldAfterCode",
    doc: "# Marcas\n\n```js\nx\n```\n\nHola\n",
    run: async ({ clickAt, press, typed, command, frame }) => {
      await clickAt("Hola");
      await caretInWords(frame, "Hola", 0);
      await command("b");
      await press("Backspace");
      await typed("X");
    },
    was: "",
    now: "**X**",
  },
  {
    name: "leftCrossesFormulaInWords",
    doc: FORMULA_IN_WORDS,
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("después");
      await caretInWords(frame, " después", 0);
      await press("ArrowLeft");
      await typed("X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes ",
    now: "YAntes X",
  },
  {
    name: "rightCrossesFormulaInWords",
    doc: FORMULA_IN_WORDS,
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes ", 6);
      await press("ArrowRight");
      await typed("X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes $x^2$",
    now: "YAntes $x^2$X",
  },
  {
    name: "shiftLeftChoosesFormulaWhole",
    doc: FORMULA_IN_WORDS,
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("después");
      await caretInWords(frame, " después", 0);
      await press("Shift+ArrowLeft");
      await typed("X");
    },
    was: "$x^2$",
    now: "X",
  },
  {
    name: "shiftRightChoosesFormulaWhole",
    doc: FORMULA_IN_WORDS,
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes ", 6);
      await press("Shift+ArrowRight");
      await typed("X");
    },
    was: "$x^2$",
    now: "X",
  },
  {
    name: "leftBeforeFormulaAtLineStart",
    doc: "# Fórmula\n\n$x^2$ al inicio\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("inicio");
      await caretInWords(frame, " al inicio", 0);
      await press("ArrowLeft");
      await typed("X");
      await clickAt("inicio");
      await typed("Y");
    },
    was: "$x^2$ al inicio",
    now: "X$x^2$ al inicioY",
  },
  {
    name: "rightPastFormulaAtLineEnd",
    doc: "# Fórmula\n\nal final $x^2$\n\nOtra línea.\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("final");
      await caretInWords(frame, "al final ", 9);
      await press("ArrowRight");
      await typed("X");
      await clickAt("al final", true);
      await typed("Y");
    },
    was: "al final $x^2$",
    now: "Yal final $x^2$X",
  },
  {
    name: "leftStopsBetweenTwoFormulas",
    doc: "# Fórmula\n\nAntes $x^2$$y^2$ después\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("después");
      await caretInWords(frame, " después", 0);
      await press("ArrowLeft");
      await typed("X");
      await clickAt("Antes", true);
      await typed("Y");
    },
    was: "Antes $x^2$",
    now: "YAntes $x^2$X",
  },
  {
    name: "leftCrossesFormulaInListItem",
    doc: "# Fórmula\n\n- uno $x^2$ dos\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 0);
      await press("ArrowLeft");
      await typed("X");
      await clickAt("dos");
      await typed("Y");
    },
    was: "$x^2$ dos",
    now: "X$x^2$ dosY",
  },
  {
    name: "leftCrossesFormulaRightAfterTaskBox",
    doc: "# Fórmula\n\n- [ ] $x^2$ dos\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 0);
      await press("ArrowLeft");
      await typed("X");
      await clickAt("dos");
      await typed("Y");
    },
    was: "$x^2$ dos",
    now: "X$x^2$ dosY",
  },
  {
    name: "shiftLeftChoosesDisplayFormulaAmongWords",
    doc: "# Fórmula\n\nAntes $$x^2$$ después\n",
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("después");
      await caretInWords(frame, " después", 1);
      await press("Shift+ArrowLeft");
      await typed("X");
    },
    was: "$$x^2$$",
    now: "X",
  },
  {
    name: "deleteAtCodeBlockEndKeepsItsFence",
    doc: "# Marcas\n\nAntes\n\n```js\nconst a = 1;\nconst b = 2;\n```\n\nDespués\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await caretBeforeCodeEnd(frame);
      await press("Delete");
    },
    was: "",
    now: "",
  },
  {
    name: "titleTypedAgainAfterItsLettersGo",
    doc: "",
    run: async ({ typed, press }) => {
      await typed("Hola");
      await press("Backspace", 4);
      await typed("Otra");
    },
    was: "",
    now: "# Otra",
  },
  {
    name: "titleTypedAgainAfterAllIsChosenAndDeleted",
    doc: "# Título\n\nHola mundo\n",
    run: async ({ clickAt, command, press, typed }) => {
      await clickAt("Hola");
      await command("a");
      await press("Backspace");
      await typed("Nuevo");
    },
    was: "Título\n\nHola mundo\n",
    now: "Nuevo",
  },
  {
    name: "backAtHeadingStartTurnsItIntoWords",
    doc: "# Marcas\n\nAntes\n\n## Sección\n\nTexto bajo\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Sección", 0);
      await press("Backspace");
    },
    was: "## ",
    now: "",
  },
  {
    name: "backAtQuoteStartTurnsItsFirstLineIntoWords",
    doc: "# Marcas\n\nAntes\n\n> una cita\n> segunda línea\n\nDespués\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "una cita", 0);
      await press("Backspace");
    },
    was: "> una cita",
    now: "una cita\n",
  },
  {
    name: "shiftEnterBreaksTheLineInAParagraph",
    doc: "# Marcas\n\nHola mundo bonito\n\nSegundo\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("Segundo");
      await caretInWords(frame, "Hola mundo bonito", 4);
      await press("Shift+Enter");
      await typed("X");
    },
    was: "",
    now: "\\\nX",
  },
  {
    name: "shiftEnterBreaksTheLineInAListItem",
    doc: "# Marcas\n\n- uno\n- dos\n- tres\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "dos", 3);
      await press("Shift+Enter");
      await typed("más");
    },
    was: "",
    now: "\\\n  más",
  },
  {
    name: "wordBackAtParagraphStartOnlyJoinsIt",
    doc: "# Marcas\n\nHola mundo bonito\n\nSegundo párrafo\n",
    run: async ({ clickAt, wordBack, frame }) => {
      await clickAt("Hola");
      await caretInWords(frame, "Segundo párrafo", 0);
      await wordBack();
    },
    was: "\n\n",
    now: "",
  },
  {
    name: "wordForwardAtParagraphEndOnlyJoinsIt",
    doc: "# Marcas\n\nHola mundo bonito\n\nSegundo párrafo\n",
    run: async ({ clickAt, wordForward, frame }) => {
      await clickAt("Hola");
      await caretInWords(frame, "Hola mundo bonito", 17);
      await wordForward();
    },
    was: "\n\n",
    now: "",
  },
  {
    name: "wordForwardAtItemEndOnlyJoinsTheItemUnderIt",
    doc: "# Marcas\n\n- Item\n  - Sub\n",
    run: async ({ clickAt, wordForward, frame }) => {
      await clickAt("Marcas");
      await caretInWords(frame, "Item", 4);
      await wordForward();
    },
    was: "\n  - ",
    now: "",
  },
  {
    name: "twoWordsBackAtParagraphEndLeaveOneSpace",
    doc: "# Marcas\n\nHola mundo bonito\n\nSegundo\n",
    run: async ({ clickAt, wordBack, page, frame }) => {
      await clickAt("Segundo");
      await caretInWords(frame, "Hola mundo bonito", 17);
      await wordBack();
      await page.waitForFunction(() => window.engines.drawn());
      await wordBack();
    },
    was: "mundo bonito",
    now: "",
  },
  {
    name: "typeOverAChoicePastATableAtTheEnd",
    doc: "# Marcas\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, command, typed, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes y más", 6);
      await command("Shift+End");
      await typed("X");
    },
    was: "y más\n\n| a | b |\n|---|---|\n| 1 | 2 |",
    now: "X",
  },
  {
    name: "backOverAChoicePastATableAtTheEnd",
    doc: "# Marcas\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, command, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes y más", 6);
      await command("Shift+End");
      await press("Backspace");
    },
    was: "y más\n\n| a | b |\n|---|---|\n| 1 | 2 |",
    now: "",
  },
  {
    name: "boldThenItalicOnTheWordsStillChosen",
    doc: "# Marcas\n\nHola mundo bonito\n",
    run: async ({ clickAt, command, frame }) => {
      await clickAt("Hola");
      await chosenIn(frame, ["Hola mundo bonito", 5], ["Hola mundo bonito", 10]);
      await command("b");
      await command("i");
    },
    was: "mundo",
    now: "***mundo***",
  },
  {
    name: "boldTwiceOnTheWordsStillChosenTakesItOff",
    doc: "# Marcas\n\nHola mundo bonito\n",
    run: async ({ clickAt, command, frame }) => {
      await clickAt("Hola");
      await chosenIn(frame, ["Hola mundo bonito", 5], ["Hola mundo bonito", 10]);
      await command("b");
      await command("b");
    },
    was: "",
    now: "",
  },
  {
    name: "typeOverAChoiceEndingRightAfterATableMidPage",
    doc: "# Marcas\n\nAntes y\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nDespués\n",
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("Antes");
      await chosenPastBlock(frame, "Antes y", 6, ".page > .table");
      await typed("X");
    },
    was: "y\n\n| a | b |\n|---|---|\n| 1 | 2 |",
    now: "X",
  },
  {
    name: "typeOverAChoiceEndingRightAfterAFoldMidPage",
    doc: "# Marcas\n\nAntes y\n\n:::details Plegable\n\nDentro\n\n:::\n\nDespués\n",
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("Antes");
      await chosenPastBlock(frame, "Antes y", 6, ".page > details");
      await typed("X");
    },
    was: "y\n\n:::details Plegable\n\nDentro\n\n:::",
    now: "X",
  },
  {
    name: "rightStopsAfterATagAtLineEnd",
    doc: "# Etiquetas\n\nAntes del final #etiqueta\n\nSiguiente línea.\n",
    run: async ({ clickAt, busyTyped, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes del final ", 16);
      await busyTyped(["ArrowRight", "Backspace"], "");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "rightStopsAfterCodeAtLineEnd",
    doc: "# Código\n\nAntes del final `cod`\n\nSiguiente línea.\n",
    run: async ({ clickAt, busyTyped, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes del final ", 16);
      await busyTyped(["ArrowRight", "Backspace"], "");
    },
    was: "`cod`",
    now: "",
  },
  {
    name: "rightIntoAStepStandsAfterItsBox",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] dos tres\n\nFin.\n",
    run: async ({ clickAt, busyTyped, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 3);
      await busyTyped(["ArrowRight", "Delete"], "");
    },
    was: "d",
    now: "",
  },
  {
    name: "leftOutOfAStepLeavesItsBoxBehind",
    doc: "# Pasos\n\n- [ ] uno\n- [ ] dos tres\n\nFin.\n",
    run: async ({ clickAt, typed, busyTyped, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "dos tres", 0);
      await busyTyped(["ArrowLeft"], "X");
      await caretInWords(frame, "Pasos", 0);
      await typed("Y");
    },
    was: "Pasos\n\n- [ ] uno",
    now: "YPasos\n\n- [ ] unoX",
  },
  {
    name: "homeAndEndWalkToTheLineEdgesPastTags",
    doc: "# Líneas\n\n#uno del final y #dos\n\nOtra.\n",
    run: async ({ clickAt, busyTyped, frame }) => {
      await clickAt("final");
      await caretInWords(frame, " del final y ", 4);
      await busyTyped(["End"], "E");
      await busyTyped(["Home"], "H");
    },
    was: "#uno del final y #dos",
    now: "H#uno del final y #dosE",
  },
  {
    name: "shiftEndChoosesToTheLineEnd",
    doc: "# Líneas\n\nAntes del final y #etiqueta\n\nOtra.\n",
    run: async ({ clickAt, busyTyped, frame }) => {
      await clickAt("final");
      await caretInWords(frame, "Antes del final y ", 10);
      await busyTyped(["Shift+End"], "E");
    },
    was: "final y #etiqueta",
    now: "E",
  },
  {
    name: "commandADeletesAPageEndingInATable",
    doc: "# Tabla\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, command, press }) => {
      await clickAt("Antes");
      await command("a");
      await press("Backspace");
    },
    was: "# Tabla\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    now: "",
  },
  {
    name: "commandACutsAPageOpeningWithATable",
    doc: "| a | b |\n|---|---|\n| 1 | 2 |\n\nDespués y más\n",
    run: async ({ clickAt, command, cut }) => {
      await clickAt("Después");
      await command("a");
      await cut();
    },
    was: "| a | b |\n|---|---|\n| 1 | 2 |\n\nDespués y más",
    now: "",
  },
  {
    name: "leftAtTheFirstStepsStartStaysAfterItsBox",
    doc: "- [ ] uno dos\n\nFin.\n",
    run: async ({ clickAt, busyTyped, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno dos", 0);
      await busyTyped(["ArrowLeft", "Delete"], "");
    },
    was: "u",
    now: "",
  },
  {
    name: "commandAThenLeftOnAPageOpeningWithStepsStaysAfterTheBox",
    doc: "- [ ] uno dos\n\nFin.\n",
    run: async ({ clickAt, command, busyTyped }) => {
      await clickAt("Fin");
      await command("a");
      await busyTyped(["ArrowLeft", "Delete"], "");
    },
    was: "u",
    now: "",
  },
  {
    name: "commandAThenTypingReplacesAPageEndingInATable",
    doc: "# Tabla\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, command, typed }) => {
      await clickAt("Antes");
      await command("a");
      await typed("Z");
    },
    was: "Tabla\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |",
    now: "Z",
  },
  {
    name: "keysAheadOfTheDrawingAtAParagraphStartTakeTheirOwnLettersThenJoinIt",
    doc: "# Adelante\n\nHola mundo bonito\n\nSegundo párrafo\n\nTercero\n",
    run: async ({ clickAt, typed, press, drawingsHeld, frame }) => {
      await clickAt("Segundo");
      await caretInWords(frame, "Segundo párrafo", 0);
      await drawingsHeld(async () => {
        await typed("abc");
        await press("Backspace", 4);
      });
    },
    was: "\n\n",
    now: "",
  },
  {
    name: "keysAheadOfTheDrawingInAnItemTakeTheirOwnLetterNotTheBullet",
    doc: "# Adelante\n\nAntes\n\n- uno\n- dos\n- tres\n",
    run: async ({ clickAt, typed, press, drawingsHeld, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "dos", 0);
      await drawingsHeld(async () => {
        await typed("abc");
        await press("Backspace");
      });
    },
    was: "",
    now: "ab",
  },
  {
    name: "keysAheadOfAnOlderDrawingLeaveAPictureAboveInPlace",
    doc: "# Adelante\n\nAntes\n\n![Rina](rina.png)\n\nTercero\n",
    run: async ({ clickAt, press, drawingsHeld, oneDrawingLet }) => {
      await clickAt("Tercero");
      await drawingsHeld(async () => {
        await press("Backspace", 2);
        await oneDrawingLet();
        await press("Backspace", 3);
      });
    },
    was: "rcero",
    now: "",
  },
  {
    name: "keysAheadOfAnOlderDrawingLeaveAPageBreakAboveInPlace",
    doc: "# Adelante\n\nAntes\n\n---\n\nTercero\n",
    run: async ({ clickAt, press, drawingsHeld, oneDrawingLet }) => {
      await clickAt("Tercero");
      await drawingsHeld(async () => {
        await press("Backspace", 2);
        await oneDrawingLet();
        await press("Backspace", 3);
      });
    },
    was: "rcero",
    now: "",
  },
  {
    name: "keysAheadOfAnOlderDrawingUnderATableAreNotLost",
    doc: "# Adelante\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nTercero\n",
    run: async ({ clickAt, press, drawingsHeld, oneDrawingLet }) => {
      await clickAt("Tercero");
      await drawingsHeld(async () => {
        await press("Backspace", 2);
        await oneDrawingLet();
        await press("Backspace", 3);
      });
    },
    was: "rcero",
    now: "",
  },
  {
    name: "keysAheadOfTheDrawingAtAParagraphEndLeaveATableBelowWhole",
    doc: "# Adelante\n\nPara\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, typed, press, drawingsHeld }) => {
      await clickAt("Para");
      await drawingsHeld(async () => {
        await typed("X");
        await press("Delete");
      });
    },
    was: "",
    now: "X",
  },
  {
    name: "backspaceAfterDeleteAtAParagraphStartJoinsItWhole",
    doc: "# Adelante\n\nHola mundo bonito\n\nSegundo párrafo\n\nTercero\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Segundo");
      await caretInWords(frame, "Segundo párrafo", 0);
      await press("Delete");
      await press("Backspace");
    },
    was: "\n\nS",
    now: "",
  },
  {
    name: "deleteAfterTypingAtAParagraphEndJoinsTheNextWhole",
    doc: "# Adelante\n\nHola mundo bonito\n\nSegundo párrafo\n\nTercero\n",
    run: async ({ clickAt, typed, press }) => {
      await clickAt("bonito");
      await typed("X");
      await press("Delete");
    },
    was: "\n\n",
    now: "X",
  },
  {
    name: "enterThenBackspaceGivesTheParagraphBackWhole",
    doc: "# Adelante\n\nHola mundo bonito\n\nSegundo párrafo\n\nTercero\n",
    run: async ({ clickAt, press }) => {
      await clickAt("Segundo");
      await press("Enter");
      await press("Backspace");
    },
    was: "",
    now: "",
  },
  {
    name: "backspaceTwiceAtAListsFirstItemTakesItIntoTheParagraphAbove",
    doc: "# Adelante\n\nAntes\n\n- uno\n- dos\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await pressedOnceDrawn(frame, press, "Backspace", 2);
    },
    was: "\n\n- uno",
    now: "uno\n",
  },
  {
    name: "backspaceThriceAtAStepTakesItsWordsIntoTheStepAbove",
    doc: "# Adelante\n\n- [ ] a\n- [ ] b\n- [x] c\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("b");
      await caretInWords(frame, "b", 0);
      await pressedOnceDrawn(frame, press, "Backspace", 3);
    },
    was: "\n- [ ] ",
    now: "",
  },
  {
    name: "rightPastATagBeforeALineBreakReachesTheNextLine",
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight", 2);
      await press("Backspace");
    },
    was: "\\\n",
    now: "",
  },
  {
    name: "leftPastATagBeforeALineBreakReachesItsStart",
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "dos tres", 0);
      await press("ArrowLeft", 2);
      await press("Delete");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "rightStopsAfterATagBeforeANestedList",
    doc: "# Listas\n\n- uno #etiqueta\n  - dentro\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dentro");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight");
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "leftFromANestedListStopsAfterTheTag",
    doc: "# Listas\n\n- uno #etiqueta\n  - dentro\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dentro");
      await caretInWords(frame, "dentro", 0);
      await press("ArrowLeft");
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "leftFromANestedStepStopsAfterTheTag",
    doc: "# Pasos\n\n- [ ] uno #etiqueta\n  - [ ] dentro\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dentro");
      await caretInWords(frame, "dentro", 0);
      await press("ArrowLeft");
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "twoBackspacesAfterTypingPastALineEndTagKeepTheTag",
    doc: "# Etiquetas\n\nAntes del final #etiqueta\n\nSiguiente línea.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes del final ", 16);
      await press("ArrowRight");
      await typed(" y");
      await pressedOnceDrawn(frame, press, "Backspace", 2);
    },
    was: "",
    now: "",
  },
  {
    name: "backspacePastTheLetterAfterATagBeforeALineBreakTakesTheTag",
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dos");
      await drawnFirst(frame);
      await frame.evaluate(() => {
        const letter = document.querySelector(".page .tag")?.nextSibling;
        if (!letter?.parentNode) throw new Error("no letter after the tag");
        getSelection().collapse(letter.parentNode, [...letter.parentNode.childNodes].indexOf(letter) + 1);
      });
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "backspaceOnAnIndentedLineAfterATagTakesTheBreak",
    doc: "# Saltos\n\n- dos #etiqueta\\\n  más\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("más");
      await caretInWords(frame, "dos ", 4);
      await press("ArrowRight", 2);
      await press("Backspace");
    },
    was: "\\\n",
    now: "",
  },
  {
    name: "deleteAfterCodeThatEndsAnItemJoinsItsNestedItem",
    doc: "# Listas\n\n- tres `c`\n  - dentro\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dentro");
      await caretInWords(frame, "tres ", 5);
      await press("ArrowRight");
      await press("Delete");
    },
    was: "\n  - ",
    now: "",
  },
  {
    name: "shiftRightAfterATagBeforeALineBreakChoosesTheBreak",
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight");
      await press("Shift+ArrowRight");
      await typed("X");
    },
    was: "\\\n",
    now: "X",
  },
  {
    name: "rightAfterATagGoesIntoAnEmptyNestedStep",
    doc: "# Pasos\n\n- uno #etiqueta\n  - [ ] \n  - [ ] dos\n\nFin.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight", 2);
      await typed("X");
      await caretInWords(frame, "Pasos", 0);
      await typed("Y");
    },
    was: "Pasos\n\n- uno #etiqueta\n  - [ ] ",
    now: "YPasos\n\n- uno #etiqueta\n  - [ ] X",
  },
  {
    name: "backspaceAfterTwoFormulasSplitByABreakLeavesThemWhole",
    doc: "# Fórmulas\n\n$x$\\\n$y$\n\nDespués\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Después");
      await caretInWords(frame, "Después", 0);
      await press("Backspace");
    },
    was: "",
    now: "",
  },
  {
    name: "rightOnAChoiceBeforeATagThatEndsABrokenLineEndsAtTheChoice",
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("dos");
      await drawnFirst(frame);
      await chosenIn(frame, ["uno ", 0], ["uno ", 4]);
      await press("ArrowRight");
      await typed("X");
      await caretInWords(frame, "Saltos", 0);
      await typed("Y");
    },
    was: "Saltos\n\nuno ",
    now: "YSaltos\n\nuno X",
  },
  {
    name: "controlFWalksPastATagBeforeALineBreak",
    macOnly: LINE_WALKS_ONLY_ON_A_MAC,
    doc: "# Saltos\n\nuno #etiqueta\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno ", 4);
      await press("Control+f", 2);
      await typed("X");
      await caretInWords(frame, "Saltos", 0);
      await typed("Y");
    },
    was: "Saltos\n\nuno #etiqueta\\\n",
    now: "YSaltos\n\nuno #etiqueta\\\nX",
  },
  {
    name: "rightAfterATagGoesIntoAnEmptyStepWithSubSteps",
    doc: "# Pasos\n\n- uno #etiqueta\n  - [ ] \n    - [ ] hijo\n\nFin.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("hijo");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight", 2);
      await typed("X");
      await caretInWords(frame, "Pasos", 0);
      await typed("Y");
    },
    was: "Pasos\n\n- uno #etiqueta\n  - [ ] ",
    now: "YPasos\n\n- uno #etiqueta\n  - [ ] X",
  },
  {
    name: "rightStopsAfterATagBeforeANestedCodeBlock",
    doc: "# Listas\n\n- uno #etiqueta\n  ```\n  x\n  ```\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Fin");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight");
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "leftFromANestedQuoteStopsAfterTheTag",
    doc: "# Listas\n\n- uno #etiqueta\n  > cita\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("cita");
      await caretInWords(frame, "cita", 0);
      await press("ArrowLeft");
      await press("Backspace");
    },
    was: "#etiqueta",
    now: "",
  },
  {
    name: "deleteAfterATagBeforeANestedCodeBlockKeepsItsFence",
    doc: "# Listas\n\n- uno #etiqueta\n  ```\n  x\n  ```\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Fin");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight");
      await press("Delete");
    },
    was: "",
    now: "",
  },
  {
    name: "deleteAfterATagBeforeANestedQuoteKeepsIt",
    doc: "# Listas\n\n- uno #etiqueta\n  > cita\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("cita");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight");
      await press("Delete");
    },
    was: "",
    now: "",
  },
  {
    name: "rightIntoBoldWordsAfterALineBreakMovesALetter",
    doc: "# B\n\nuno #etiqueta\\\n**bold #otra**\\\nc\n\nFin.\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("bold");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight", 3);
      await typed("X");
      await caretInWords(frame, "B", 0);
      await typed("Y");
    },
    was: "B\n\nuno #etiqueta\\\n**b",
    now: "YB\n\nuno #etiqueta\\\n**bX",
  },
  {
    name: "aCaretInTheLetterAfterBoldStandsOutsideTheBold",
    doc: "# B\n\nuno **#etiqueta**\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("dos");
      await drawnFirst(frame);
      await frame.evaluate(() => {
        const letter = document.querySelector(".page strong")?.nextSibling;
        if (!letter) throw new Error("no letter after the bold");
        getSelection().collapse(letter, 1);
      });
      await typed("X");
      await caretInWords(frame, "B", 0);
      await typed("Y");
    },
    was: "B\n\nuno **#etiqueta**",
    now: "YB\n\nuno **#etiqueta**X",
  },
  {
    name: "rightPastABoldTagBeforeALineBreakReachesTheNextLine",
    doc: "# Saltos\n\nuno **#etiqueta**\\\ndos tres\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight", 2);
      await press("Backspace");
    },
    was: "\\\n",
    now: "",
  },
  {
    name: "commandShiftHomeChoosesUpToATableThatOpensThePage",
    doc: "| a | b |\n|---|---|\n| 1 | 2 |\n\nDespués y más\n",
    run: async ({ clickAt, command, press, frame }) => {
      await clickAt("Después");
      await caretInWords(frame, "Después y más", 3);
      await command("Shift+Home");
      await press("Backspace");
    },
    was: "| a | b |\n|---|---|\n| 1 | 2 |\n\nDes",
    now: "",
  },
  {
    name: "commandShiftDownChoosesPastATableThatEndsThePage",
    macOnly: PAGE_REACH_ONLY_ON_A_MAC,
    doc: "# Tabla\n\nAntes y más\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes y más", 3);
      await press("Meta+Shift+ArrowDown");
      await press("Backspace");
    },
    was: "es y más\n\n| a | b |\n|---|---|\n| 1 | 2 |",
    now: "",
  },
  {
    name: "deleteOnAChoiceFromAfterATagIntoANestedItemTakesIt",
    doc: "# Listas\n\n- uno #etiqueta\n  - dentro\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("dentro");
      await caretInWords(frame, "uno ", 4);
      await press("ArrowRight");
      await press("Shift+ArrowRight");
      await press("Delete");
    },
    was: "\n  - ",
    now: "",
  },
  {
    name: "typeOverAChoiceToTheEndOfAClosedFold",
    doc: FOLD_AT_THE_END,
    run: async ({ clickAt, command, typed, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Antes y más", 6);
      await command("Shift+End");
      await typed("X");
    },
    was: "y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::",
    now: "X",
  },
  {
    name: "backOverAChoiceToTheEndOfAnOpenFold",
    doc: `${FOLD_AT_THE_END}\nDespués\n`,
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await foldsOpened(frame);
      await chosenToTheEndOf(frame, ["Antes y más", 6], ".page details .table");
      await press("Backspace");
    },
    was: "y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::",
    now: "",
  },
  {
    name: "backspaceAtAFirstItemLeavesItsWordsApartFromTheList",
    doc: "# Marcas\n\nAntes\n\n- uno\n- dos\n- tres\n\nDespués\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await press("Backspace");
    },
    was: "- uno",
    now: "uno\n",
  },
  {
    name: "twoBackspacesAtATaskLeaveItsWordsApartFromTheTasks",
    doc: "# Marcas\n\n- [ ] a\n- [ ] b\n- [x] c\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("b");
      await caretInWords(frame, "b", 0);
      await press("Backspace", 2);
    },
    was: "- [ ] b",
    now: "\nb\n",
  },
  {
    name: "shiftEnterInAHeadingStartsAParagraph",
    doc: "# Marcas\n\nAntes\n\n## Sección\n\nTexto bajo\n",
    run: async ({ clickAt, press, typed, frame }) => {
      await clickAt("Antes");
      await caretInWords(frame, "Sección", 3);
      await press("Shift+Enter");
      await typed("X");
    },
    was: "",
    now: "\n\nX",
  },
  {
    name: "boldOverAChoiceIntoAFoldLeavesItsOpeningLine",
    doc: FOLD_WITH_TWO,
    run: async ({ clickAt, command, frame }) => {
      await clickAt("Antes");
      await foldsOpened(frame);
      await drawnFirst(frame);
      await chosenIn(frame, ["Antes y más", 6], ["Contenido", 5]);
      await command("b");
    },
    was: "y más\n\n:::details Plegable\n\nConte",
    now: "**y más**\n\n:::details Plegable\n\n**Conte**",
  },
  {
    name: "typeOverAChoiceIntoAnOpenFoldUndoesIt",
    doc: `${FOLD_AT_THE_END}\nDespués\n`,
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("Antes");
      await foldsOpened(frame);
      await drawnFirst(frame);
      await chosenIn(frame, ["Antes y más", 6], ["Contenido", 5]);
      await typed("X");
    },
    was: "y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::",
    now: "Xnido\n\n| a | b |\n|---|---|\n| 1 | 2 |",
  },
  {
    name: "cutAChoiceIntoAnOpenFoldUndoesIt",
    doc: `${FOLD_AT_THE_END}\nDespués\n`,
    run: async ({ clickAt, cut, frame }) => {
      await clickAt("Antes");
      await foldsOpened(frame);
      await drawnFirst(frame);
      await chosenIn(frame, ["Antes y más", 6], ["Contenido", 5]);
      await cut();
    },
    was: "y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::",
    now: "nido\n\n| a | b |\n|---|---|\n| 1 | 2 |",
  },
  {
    name: "typeOverAChoiceIntoAClosedFoldsNameUndoesIt",
    doc: `${FOLD_AT_THE_END}\nDespués\n`,
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("Antes");
      await drawnFirst(frame);
      await chosenIn(frame, ["Antes y más", 6], ["Plegable", 4]);
      await typed("X");
    },
    was: "y más\n\n:::details Plegable\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n:::",
    now: "X\n\nContenido\n\n| a | b |\n|---|---|\n| 1 | 2 |",
  },
  {
    name: "backOverAChoiceOutOfAnOpenFoldKeepsItsClosingLine",
    doc: FOLD_WITH_TWO,
    run: async ({ clickAt, press, frame }) => {
      await clickAt("Antes");
      await foldsOpened(frame);
      await drawnFirst(frame);
      await chosenIn(frame, ["Otro", 2], ["Después", 3]);
      await press("Backspace");
    },
    was: "ro\n\n:::\n\nDes",
    now: "\n\n:::\n\n",
  },
  {
    name: "typeOverWordsChosenInAFoldsNameWritesInTheName",
    doc: FOLD_WITH_TWO,
    run: async ({ clickAt, typed, frame }) => {
      await clickAt("Antes");
      await foldsOpened(frame);
      await drawnFirst(frame);
      await chosenIn(frame, ["Plegable", 2], ["Plegable", 4]);
      await typed("X");
    },
    was: "eg",
    now: "X",
  },
  {
    name: "backspaceAtAnItemRaisesTheItemsUnderIt",
    doc: ITEMS_UNDER_ONE,
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await press("Backspace");
    },
    was: "- uno\n    - sub a\n    ",
    now: "uno\n\n- sub a\n",
  },
  {
    name: "backspaceAtANumberedItemRaisesItsItemsAndTheirsALevel",
    doc: "# Listas\n\n1. cero\n2. uno\n    1. sub a\n        1. hondo\n    2. sub b\n3. dos\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await press("Backspace");
    },
    was: "2. uno\n    1. sub a\n        1. hondo\n    ",
    now: "\nuno\n\n1. sub a\n    1. hondo\n",
  },
  {
    name: "twoBackspacesAtAStepRaiseTheStepsUnderIt",
    doc: "# Pasos\n\n- [ ] uno\n    - [ ] sub a\n      más de a\n- [x] dos\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await pressedOnceDrawn(frame, press, "Backspace", 2);
    },
    was: "- [ ] uno\n    - [ ] sub a\n    ",
    now: "uno\n\n- [ ] sub a\n",
  },
  {
    name: "backspaceAtAnItemRaisesTheItemsUnderItWithWindowsLineEnds",
    doc: ITEMS_UNDER_ONE.replaceAll("\n", "\r\n"),
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await press("Backspace");
    },
    was: "- uno\r\n    - sub a\r\n    ",
    now: "uno\r\n\r\n- sub a\r\n",
  },
  {
    name: "backspaceAtAnItemHoldingAFoldsOpeningLineKeepsItsClosingLine",
    doc: "# Pliegues\n\n- uno\n\n  :::details Plegable\n\n  dentro\n\n:::\n\nFin.\n",
    run: async ({ clickAt, press, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await press("Backspace");
    },
    was: "- uno\n\n  :::details Plegable\n\n  ",
    now: "uno\n\n:::details Plegable\n\n",
  },
  {
    name: "undoAfterItemsRoseGivesTheListBackWhole",
    doc: ITEMS_UNDER_ONE,
    run: async ({ clickAt, press, command, page, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await press("Backspace");
      await drawnFirst(frame);
      await command("z");
      await drawnFirst(frame);
      const back = await page.evaluate(() => window.engines.text());
      if (back !== ITEMS_UNDER_ONE) throw new Error(`undo left ${JSON.stringify(back)}`);
      await command("Shift+z");
    },
    was: "- uno\n    - sub a\n    ",
    now: "uno\n\n- sub a\n",
  },
  {
    name: "deleteAheadOfItsDrawingAtAParagraphEndJoinsTheNextOnceDrawn",
    doc: "# Adelante\n\nHola mundo bonito\n\nSegundo párrafo\n\nTercero\n",
    run: async ({ clickAt, typed, press, drawingsHeld }) => {
      await clickAt("bonito");
      await drawingsHeld(async () => {
        await typed("X");
        await press("Delete");
      });
    },
    was: "\n\n",
    now: "X",
  },
  {
    name: "backspacesAheadOfTheirDrawingAtAListsFirstItemTakeItIntoTheParagraphAbove",
    doc: "# Adelante\n\nAntes\n\n- uno\n- dos\n",
    run: async ({ clickAt, press, drawingsHeld, frame }) => {
      await clickAt("uno");
      await caretInWords(frame, "uno", 0);
      await drawingsHeld(() => press("Backspace", 2));
    },
    was: "\n\n- ",
    now: "",
  },
  {
    name: "backspacesAheadOfTheirDrawingsAtAStepTakeItsWordsIntoTheStepAbove",
    doc: "# Adelante\n\n- [ ] a\n- [ ] b\n- [x] c\n",
    run: async ({ clickAt, press, drawingsHeld, frame }) => {
      await clickAt("b");
      await caretInWords(frame, "b", 0);
      await drawingsHeld(() => press("Backspace", 3));
    },
    was: "\n- [ ] ",
    now: "",
  },
];

const macIn = (frame) => frame.evaluate(() => navigator.userAgent.includes("Macintosh"));

const chosenIn = (frame, from, to) =>
  frame.evaluate(
    ([start, end]) => {
      const textOf = (said) => {
        const walk = document.createTreeWalker(document.querySelector(".page"), NodeFilter.SHOW_TEXT);
        for (let node = walk.nextNode(); node; node = walk.nextNode()) if (node.data === said) return node;
        throw new Error(`no "${said}" alone in a text on the page`);
      };
      const range = document.createRange();
      range.setStart(textOf(start[0]), start[1]);
      range.setEnd(textOf(end[0]), end[1]);
      getSelection().removeAllRanges();
      getSelection().addRange(range);
    },
    [from, to],
  );

const caretAfterPiece = (frame, selector) =>
  frame.evaluate((said) => {
    const piece = document.querySelector(said);
    if (!piece) throw new Error(`no ${said} on the page`);
    const holder = piece.parentNode;
    getSelection().collapse(holder, [...holder.childNodes].indexOf(piece) + 1);
  }, selector);

const caretAtEndOf = (frame, selector) =>
  frame.evaluate((said) => {
    const holder = document.querySelector(said);
    if (!holder) throw new Error(`no ${said} on the page`);
    const walk = document.createTreeWalker(holder, NodeFilter.SHOW_TEXT);
    let last = null;
    for (let node = walk.nextNode(); node; node = walk.nextNode()) last = node;
    if (!last) throw new Error(`no words in ${said}`);
    getSelection().collapse(last, last.length);
  }, selector);

const caretBeforeCodeEnd = async (frame) => {
  await frame.parentFrame().waitForFunction(() => window.engines.drawn());
  await frame.evaluate(() => {
    const walk = document.createTreeWalker(document.querySelector(".page pre code"), NodeFilter.SHOW_TEXT);
    let last = null;
    for (let node = walk.nextNode(); node; node = walk.nextNode()) last = node;
    if (!last?.data.endsWith("\n")) throw new Error("no code that ends with its line feed");
    getSelection().collapse(last, last.length - 1);
  });
};

const chosenPastBlock = async (frame, words, offset, block) => {
  await caretInWords(frame, words, offset);
  await frame.evaluate((said) => {
    const sheet = document.querySelector(".page");
    const one = document.querySelector(said);
    if (!one) throw new Error(`no ${said} on the page`);
    getSelection().extend(sheet, [...sheet.childNodes].indexOf(one) + 1);
  }, block);
};

const foldsOpened = (frame) =>
  frame.evaluate(() => {
    for (const fold of document.querySelectorAll(".page details")) fold.open = true;
  });

const chosenToTheEndOf = async (frame, [words, offset], selector) => {
  await caretInWords(frame, words, offset);
  await frame.evaluate((holder) => {
    const end = document.querySelector(holder);
    if (!end) throw new Error(`no ${holder} on the page`);
    getSelection().extend(end, end.childNodes.length);
  }, selector);
};
