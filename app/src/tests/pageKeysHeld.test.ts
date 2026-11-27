import { afterEach, describe, expect, it } from "vitest";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const TITLE = '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1>';
const ANTES = '<p data-at="5-11"><span data-src="5-10">Antes</span></p>';

const TWO = "# T\n\nHola mundo bonito\n\nSegundo párrafo\n";
const LIST = "# T\n\nAntes\n\n- uno\n- dos\n- tres\n";
const STEPS = "# T\n\n- [ ] a\n- [ ] b\n- [x] c\n";
const TABLE = "# T\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
const SECTIONS = "Antes\n\n## Pasos\n\n- uno\n- dos\n";
const PICTURE = "# T\n\nAntes\n\n![Rina](rina.png)\n\nTercero\nFin\n";

const DRAWINGS: Record<string, string> = {
  "# T\n\nHola mundo bonitoX\n\nOtro párrafo\n": `${TITLE}<p data-at="5-24"><span data-src="5-23">Hola mundo bonitoX</span></p><p data-at="25-39"><span data-src="25-38">Otro párrafo</span></p>`,
  "Antes\n\n## Pasos\n\n- uno\n- dos\n":
    '<p data-at="0-6"><span data-src="0-5">Antes</span></p><h2 id="pasos" data-at="7-16"><span data-src="10-15">Pasos</span></h2><ul data-at="17-29"><li data-at="17-23"><span data-src="19-22">uno</span></li><li data-at="23-29"><span data-src="25-28">dos</span></li></ul>',
  "Antes\n\n## Pasos\n\nuno\n- dos\n":
    '<p data-at="0-6"><span data-src="0-5">Antes</span></p><h2 id="pasos" data-at="7-16"><span data-src="10-15">Pasos</span></h2><p data-at="17-21"><span data-src="17-20">uno</span></p><ul data-at="21-27"><li data-at="21-27"><span data-src="23-26">dos</span></li></ul>',
  "# T\n\n- [ ] a\n\nb\n- [x] c\n": `${TITLE}<ul data-at="5-14"><li class="task" data-at="5-14"><span class="check" data-done="false"></span><span data-src="11-12">a</span></li></ul><p data-at="14-16"><span data-src="14-15">b</span></p><ul data-at="16-24"><li class="task" data-at="16-24"><span class="check" data-done="true"></span><span data-src="22-23">c</span></li></ul>`,
  "# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n": `${TITLE}<p data-at="5-24"><span data-src="5-23">Hola mundo bonitoX</span></p><p data-at="25-42"><span data-src="25-41">Segundo párrafo</span></p>`,
  "# T\n\nAntes\n\nuno\n- dos\n- tres\n": `${TITLE}${ANTES}<p data-at="12-16"><span data-src="12-15">uno</span></p><ul data-at="16-29"><li data-at="16-22"><span data-src="18-21">dos</span></li><li data-at="22-29"><span data-src="24-28">tres</span></li></ul>`,
  "# T\n\n- [ ] a\n- [ ] b\n- [x] c\n": `${TITLE}<ul data-at="5-29"><li class="task" data-at="5-13"><span class="check" data-done="false"></span><span data-src="11-12">a</span></li><li class="task" data-at="13-21"><span class="check" data-done="false"></span><span data-src="19-20">b</span></li><li class="task" data-at="21-29"><span class="check" data-done="true"></span><span data-src="27-28">c</span></li></ul>`,
  "# T\n\n- [ ] a\n- b\n- [x] c\n": `${TITLE}<ul data-at="5-25"><li class="task" data-at="5-13"><span class="check" data-done="false"></span><span data-src="11-12">a</span></li><li data-at="13-17"><span data-src="15-16">b</span></li><li class="task" data-at="17-25"><span class="check" data-done="true"></span><span data-src="23-24">c</span></li></ul>`,
  "# T\n\nAntes\n\n| a | b |\n|---|---|\n| 1 | 2 |\n": `${TITLE}${ANTES}<div class="table" data-at="12-42"><table><colgroup><col><col></colgroup><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table></div>`,
  "# T\n\nAntesX\n\n| a | b |\n|---|---|\n| 1 | 2 |\n": `${TITLE}<p data-at="5-12"><span data-src="5-11">AntesX</span></p><div class="table" data-at="13-43"><table><colgroup><col><col></colgroup><thead><tr><th scope="col">a</th><th scope="col">b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table></div>`,
  "# T\n\nHola mundo bonitoXy\n\nSegundo párrafo\n": `${TITLE}<p data-at="5-25"><span data-src="5-24">Hola mundo bonitoXy</span></p><p data-at="26-43"><span data-src="26-42">Segundo párrafo</span></p>`,
  "# T\n\nHola mundo bonitoX@\n\nSegundo párrafo\n": `${TITLE}<p data-at="5-25"><span data-src="5-24">Hola mundo bonitoX@</span></p><p data-at="26-43"><span data-src="26-42">Segundo párrafo</span></p>`,
  "# T\n\nHola mundo bonito\n\negundo párrafo\n": `${TITLE}<p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p><p data-at="24-40"><span data-src="24-39">egundo párrafo</span></p>`,
  "# T\n\nHola mundo \n\nSegundo párrafo\n": `${TITLE}<p data-at="5-17"><span data-src="5-15">Hola mundo</span></p><p data-at="18-35"><span data-src="18-34">Segundo párrafo</span></p>`,
  "# T\n\nAntes\n\nuno\n\n- dos\n- tres\n": `${TITLE}${ANTES}<p data-at="12-16"><span data-src="12-15">uno</span></p><ul data-at="17-30"><li data-at="17-23"><span data-src="19-22">dos</span></li><li data-at="23-30"><span data-src="25-29">tres</span></li></ul>`,
  "Antes\n\n## Pasos\n\nuno\n\n- dos\n":
    '<p data-at="0-6"><span data-src="0-5">Antes</span></p><h2 id="pasos" data-at="7-16"><span data-src="10-15">Pasos</span></h2><p data-at="17-21"><span data-src="17-20">uno</span></p><ul data-at="22-28"><li data-at="22-28"><span data-src="24-27">dos</span></li></ul>',
  "# T\n\n- [ ] a\n\nb\n\n- [x] c\n": `${TITLE}<ul data-at="5-13"><li class="task" data-at="5-13"><span class="check" data-done="false"></span><span data-src="11-12">a</span></li></ul><p data-at="14-16"><span data-src="14-15">b</span></p><ul data-at="17-25"><li class="task" data-at="17-25"><span class="check" data-done="true"></span><span data-src="23-24">c</span></li></ul>`,
  [TWO]: `${TITLE}<p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p><p data-at="24-41"><span data-src="24-40">Segundo párrafo</span></p>`,
  [LIST]: `${TITLE}${ANTES}<ul data-at="12-31"><li data-at="12-18"><span data-src="14-17">uno</span></li><li data-at="18-24"><span data-src="20-23">dos</span></li><li data-at="24-31"><span data-src="26-30">tres</span></li></ul>`,
  [PICTURE]: `${TITLE}${ANTES}<p data-at="12-30"><img src="rina.png" alt="Rina" loading="lazy" data-src="12-29" contenteditable="false"></p><p data-at="31-43"><span data-src="31-38">Tercero</span><br data-src="38-39" contenteditable="false"><span data-src="39-42">Fin</span></p>`,
};

let writing: PageWriting | null = null;
let landing: string | null = null;
let ours = true;
let changedElsewhere: (next: string) => void = () => {};

afterEach(() => {
  writing?.stop();
  writing = null;
  landing = null;
  ours = true;
  document.body.innerHTML = "";
  document.documentElement.classList.remove("untitled");
});

const drawingOf = (now: string) => {
  const drawn = DRAWINGS[now];
  if (drawn === undefined) throw new Error(`no drawing for ${JSON.stringify(now)}`);
  return drawn;
};

const opened = (body: string) => {
  let text = body;
  changedElsewhere = (next) => {
    text = next;
  };
  document.body.innerHTML = `<article class="page">${drawingOf(body)}</article>`;
  writing = writtenOn(document, {
    text: () => text,
    write: (next) => {
      text = next;
    },
    shown: () => {},
    redraw: (now) => {
      landing = now;
    },
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translator: { fromHtml: async () => "", fromText: async () => null },
    ask: () => {},
    blocks: () => {},
    acts: () => {},
    ours: () => ours,
  });
  return () => text;
};

const settled = () => new Promise((done) => setTimeout(done, 0));

const laidSilently = async () => {
  await settled();
  const now = landing;
  landing = null;
  const sheet = document.querySelector(".page");
  if (now === null || !sheet) throw new Error("no drawing was asked");
  sheet.innerHTML = drawingOf(now);
};

const landed = async () => {
  await laidSilently();
  writing?.drawn();
  await settled();
};

const words = (said: string) => {
  const walk = document.createTreeWalker(
    document.querySelector(".page") as Node,
    NodeFilter.SHOW_TEXT,
  );
  for (let node = walk.nextNode(); node; node = walk.nextNode())
    if ((node as Text).data.includes(said)) return node as Text;
  throw new Error(`no ${said} on the page`);
};

const caretAt = (said: string, end = false) => {
  const node = words(said);
  const offset = node.data.indexOf(said) + (end ? said.length : 0);
  document.getSelection()?.collapse(node, offset);
};

const sheet = () => document.querySelector(".page") as Element;

const asked = (inputType: string, data: string | null = null) => {
  const event = new InputEvent("beforeinput", { inputType, data, bubbles: true, cancelable: true });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  sheet().dispatchEvent(event);
};

const pressed = (key: string, held: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...held });
  sheet().dispatchEvent(event);
  return event.defaultPrevented;
};

const backspace = () => {
  if (!pressed("Backspace")) asked("deleteContentBackward");
};

const deleted = () => {
  if (!pressed("Delete")) asked("deleteContentForward");
};

const typed = (said: string) => {
  for (const letter of said) {
    pressed(letter);
    asked("insertText", letter);
  }
};

describe("a delete the page cannot place yet, and nothing after it", () => {
  it("joins the paragraph below once the page shows the letter typed before it", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    expect(text()).toBe("# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n");
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoXSegundo párrafo\n");
  });

  it("takes a list's first item into the paragraph above on the second Backspace", async () => {
    const text = opened(LIST);
    caretAt("uno");
    backspace();
    backspace();
    expect(text()).toBe("# T\n\nAntes\n\nuno\n\n- dos\n- tres\n");
    await landed();
    expect(text()).toBe("# T\n\nAntesuno\n\n- dos\n- tres\n");
  });

  it("takes a step's box, its bullet and then joins it to the step above, one drawing each", async () => {
    const text = opened(STEPS);
    caretAt("b");
    backspace();
    backspace();
    backspace();
    expect(text()).toBe("# T\n\n- [ ] a\n- b\n- [x] c\n");
    await landed();
    await landed();
    expect(text()).toBe("# T\n\n- [ ] ab\n- [x] c\n");
  });
});

describe("a held delete writes nothing while the page cannot show its place", () => {
  it("waits past a drawing that still shows the words before, and acts once the page shows them", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    await settled();
    writing?.drawn();
    expect(text()).toBe("# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n");
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoXSegundo párrafo\n");
  });

  it("keeps a list's first line as it is until the page draws it as a paragraph", async () => {
    const text = opened(LIST);
    caretAt("uno");
    backspace();
    backspace();
    await settled();
    writing?.drawn();
    expect(text()).toBe("# T\n\nAntes\n\nuno\n\n- dos\n- tres\n");
  });

  it("keeps the picture above an emptied line when the page still shows the line's words", async () => {
    const text = opened(PICTURE);
    caretAt("Tercero", true);
    for (let left = 7; left > 0; left--) backspace();
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n\n\nFin\n");
    backspace();
    await settled();
    writing?.drawn();
    expect(text()).toBe("# T\n\nAntes\n\n![Rina](rina.png)\n\n\nFin\n");
  });

  it("writes nothing once the sheet shows another document", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    ours = false;
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n");
  });

  it("writes nothing once the page is closed, though its drawing lands after", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    writing?.stop();
    sheet().innerHTML = drawingOf("# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n");
    writing?.drawn();
    await settled();
    expect(text()).toBe("# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n");
  });
});

describe("a held delete keeps the page's rules once it acts", () => {
  it("takes a table below whole, as a piece drawn whole", async () => {
    const text = opened(TABLE);
    caretAt("Antes", true);
    typed("X");
    deleted();
    await landed();
    expect(text()).toBe("# T\n\nAntesX");
  });

  it("never joins a task page's section heading to the step freed below it", async () => {
    const text = opened(SECTIONS);
    document.documentElement.classList.add("untitled");
    caretAt("uno");
    backspace();
    backspace();
    expect(text()).toBe("Antes\n\n## Pasos\n\nuno\n\n- dos\n");
    await landed();
    expect(text()).toBe("Antes\n\n## Pasos\n\nuno\n\n- dos\n");
  });
});

describe("any other input ends a held delete as the page stood before it", () => {
  it("types a letter where the caret is and drops the delete, as a key the page could not place", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    typed("y");
    expect(text()).toBe("# T\n\nHola mundo bonitoXy\n\nSegundo párrafo\n");
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoXy\n\nSegundo párrafo\n");
  });

  it("types what AltGr writes, as any letter", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    pressed("@", { ctrlKey: true, altKey: true });
    asked("insertText", "@");
    expect(text()).toBe("# T\n\nHola mundo bonitoX@\n\nSegundo párrafo\n");
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoX@\n\nSegundo párrafo\n");
  });

  it("drops the delete when the person clicks before the page draws", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    sheet().dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoX\n\nSegundo párrafo\n");
  });

  it("takes a word back at once when the page can place it, and drops the delete held before", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    pressed("Backspace", { altKey: true });
    asked("deleteWordBackward");
    expect(text()).toBe("# T\n\nHola mundo \n\nSegundo párrafo\n");
    await landed();
    expect(text()).toBe("# T\n\nHola mundo \n\nSegundo párrafo\n");
  });

  it("takes a letter back at once when the page can place it, and drops the delete held before", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    backspace();
    expect(text()).toBe(TWO);
    await landed();
    expect(text()).toBe(TWO);
  });

  it("lets the delete act before a composition reads its place, when the page already shows it", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    await laidSilently();
    sheet().dispatchEvent(new CompositionEvent("compositionstart", { bubbles: true, data: "" }));
    sheet().dispatchEvent(new CompositionEvent("compositionend", { bubbles: true, data: "é" }));
    expect(text()).toBe("# T\n\nHola mundo bonitoXéSegundo párrafo\n");
  });
});

describe("a word delete that comes after a held delete at the same edge", () => {
  it("waits with it and acts after it, in order", async () => {
    const text = opened(LIST);
    caretAt("uno");
    backspace();
    backspace();
    pressed("Backspace", { altKey: true });
    asked("deleteWordBackward");
    expect(text()).toBe("# T\n\nAntes\n\nuno\n\n- dos\n- tres\n");
    await landed();
    expect(text()).toBe("# T\n\nuno\n\n- dos\n- tres\n");
  });
});

describe("a key held down while the page is behind", () => {
  it("holds its first press and not the repeats, so nothing more goes once it is let go", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    for (let again = 3; again > 0; again--) pressed("Delete", { repeat: true });
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoXSegundo párrafo\n");
  });
});

describe("a held delete acts only on the words and the place it was pressed on", () => {
  it("drops the delete when the words change by another way before the page draws", async () => {
    const text = opened(TWO);
    caretAt("bonito", true);
    typed("X");
    deleted();
    changedElsewhere(text().replace("Segundo", "Otro"));
    await landed();
    expect(text()).toBe("# T\n\nHola mundo bonitoX\n\nOtro párrafo\n");
  });
});
