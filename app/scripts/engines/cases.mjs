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

const caretInWords = (frame, words, offset) =>
  frame.evaluate(
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
    was: "- ",
    now: "\n",
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("Antes", true);
      await press("Control+e");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("Antes", true);
      await press("Control+e");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("Antes", true);
      await press("Control+e");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("al inicio");
      await press("Control+a");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("Antes", true);
      await press("Meta+ArrowRight");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("dos");
      await press("Meta+ArrowLeft");
      await press("Meta+ArrowRight");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("uno", true);
      await press("Control+e");
      await typed("X");
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
    run: async ({ clickAt, typed, press }) => {
      await clickAt("tres");
      await press("Control+a");
      await typed("X");
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
    run: async ({ clickAt, command }) => {
      await clickAt("Antes");
      await command("Shift+End");
      await command("x");
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
    run: async ({ clickAt, typed, press, frame }) => {
      await clickAt("dos");
      await caretInWords(frame, " dos", 2);
      await press("Control+a");
      await typed("X");
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
