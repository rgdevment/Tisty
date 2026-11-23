import { SLASHED, type Slashed } from "tisty-editor/edit";
import { afterEach, describe, expect, it } from "vitest";
import { t } from "../locales";
import type { Seat } from "../ui/pageWriter";
import { type PageWriting, writtenOn } from "../ui/pageWriting";

const TWO = "# T\n\nHola mundo bonito\n\nSegundo párrafo\n";
const PLACED =
  '<h1 id="t" data-at="0-4"><span data-src="2-3">T</span></h1><p data-at="5-23"><span data-src="5-22">Hola mundo bonito</span></p><p data-at="24-41"><span data-src="24-40">Segundo párrafo</span></p>';
const FILE = SLASHED.find((one) => one.key === "file") as Slashed;

let writing: PageWriting | null = null;

afterEach(() => {
  writing?.stop();
  writing = null;
  document.body.innerHTML = "";
});

const settled = () => new Promise((done) => setTimeout(done, 0));

const held = () => {
  let answer: (said: string) => void = () => {};
  const asked = new Promise<string>((done) => {
    answer = done;
  });
  return { asked, answer };
};

const onPage = () => {
  const here = { text: TWO, seat: { current: null } as Seat | null, lost: [] as string[] };
  const translating = held();
  const attaching = held();
  let put: ((text: string) => boolean) | null = null;
  document.body.innerHTML = `<article class="page">${PLACED}</article>`;
  writing = writtenOn(document, {
    text: () => here.text,
    write: (next) => {
      here.text = next;
    },
    shown: () => {},
    redraw: () => {},
    near: () => Promise.reject(new Error("no core here")),
    dressed: () => {},
    translated: () => translating.asked,
    pasted: () => attaching.asked,
    ask: (_kind, _at, given) => {
      put = given;
    },
    blocks: () => {},
    acts: () => {},
    seat: () => here.seat,
    lost: (notice) => here.lost.push(notice),
  });
  const words = document.querySelector('[data-src="5-22"]')?.firstChild as Text;
  document.getSelection()?.collapse(words, words.length);
  return { here, translating, attaching, asked: () => put };
};

const wordsPasted = () => {
  const event = new InputEvent("beforeinput", {
    inputType: "insertFromPaste",
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "getTargetRanges", { value: () => [] });
  Object.defineProperty(event, "dataTransfer", {
    value: { getData: (kind: string) => (kind === "text/html" ? "<b>y</b>" : "y") },
  });
  document.querySelector(".page")?.dispatchEvent(event);
};

const filePasted = () => {
  const event = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", {
    value: { files: [new File(["x"], "x.png")], getData: () => "" },
  });
  document.querySelector(".page")?.dispatchEvent(event);
};

describe("words pasted that come back late", () => {
  it("are written while the page still shows the document they were pasted on", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    translating.answer("**y**");
    await settled();
    expect(here.text).toBe("# T\n\nHola mundo bonito**y**\n\nSegundo párrafo\n");
    expect(here.lost).toEqual([]);
  });

  it("write nothing once the page shows another document, and say so", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    here.seat = { current: () => true };
    here.text = "# Otro\n\nNada que ver\n";
    translating.answer("**y**");
    await settled();
    expect(here.text).toBe("# Otro\n\nNada que ver\n");
    expect(here.lost).toEqual([t("pastedLost")]);
  });

  const shownBy = (seat: Seat, shownNow: string) => {
    const laid: string[] = [];
    seat.current = (next) => {
      const now = next(shownNow);
      if (now !== null) laid.push(now);
      return now !== null;
    };
    return laid;
  };

  it("are handed to whoever shows their document now once this page stopped writing", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    writing?.stop();
    const laid = shownBy(here.seat as Seat, TWO);
    translating.answer("**y**");
    await settled();
    expect(laid).toEqual(["# T\n\nHola mundo bonito**y**\n\nSegundo párrafo\n"]);
    expect(here.text).toBe(TWO);
    expect(here.lost).toEqual([]);
  });

  it("follow their words when words were written above them meanwhile", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    writing?.stop();
    const laid = shownBy(here.seat as Seat, `Arriba\n\n${TWO}`);
    translating.answer("**y**");
    await settled();
    expect(laid).toEqual([`Arriba\n\n# T\n\nHola mundo bonito**y**\n\nSegundo párrafo\n`]);
    expect(here.lost).toEqual([]);
  });

  it("are not laid where the words they were pasted among are gone, and say so", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    writing?.stop();
    const laid = shownBy(here.seat as Seat, TWO.replace("bonito\n\nSegundo", "Segundo"));
    translating.answer("**y**");
    await settled();
    expect(laid).toEqual([]);
    expect(here.lost).toEqual(["What you pasted came in, but the place it was for is gone"]);
  });

  it("follow their words on the same page when keys were typed before them meanwhile", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    here.text = TWO.replace("Hola", "Hola, hola");
    translating.answer("**y**");
    await settled();
    expect(here.text).toBe("# T\n\nHola, hola mundo bonito**y**\n\nSegundo párrafo\n");
    expect(here.lost).toEqual([]);
  });

  it("say so when nobody shows their document any more", async () => {
    const { here, translating } = onPage();
    wordsPasted();
    writing?.stop();
    here.seat = null;
    translating.answer("**y**");
    await settled();
    expect(here.text).toBe(TWO);
    expect(here.lost).toEqual([t("pastedLost")]);
  });
});

describe("a file pasted that comes back late", () => {
  it("is laid under its block while the page still shows its document", async () => {
    const { here, attaching } = onPage();
    filePasted();
    attaching.answer("![x](x.png)");
    await settled();
    expect(here.text).toBe("# T\n\nHola mundo bonito\n\n![x](x.png)\n\nSegundo párrafo\n");
  });

  it("writes nothing once the page shows another document, and says so", async () => {
    const { here, attaching } = onPage();
    filePasted();
    here.seat = { current: () => true };
    here.text = "# Otro\n\nNada que ver\n";
    attaching.answer("![x](x.png)");
    await settled();
    expect(here.text).toBe("# Otro\n\nNada que ver\n");
    expect(here.lost).toEqual([t("attachmentLost")]);
  });
});

describe("what the slash menu asks for and comes back late", () => {
  it("is laid while the page still shows its document", () => {
    const { here, asked } = onPage();
    writing?.slashed(FILE);
    expect(asked()?.("![x](x.png)")).toBe(true);
    expect(here.text).toBe("# T\n\nHola mundo bonito\n\n![x](x.png)\n\nSegundo párrafo\n");
  });

  it("is refused once the page shows another document, so the asker tells it was lost", () => {
    const { here, asked } = onPage();
    writing?.slashed(FILE);
    here.seat = { current: () => true };
    here.text = "# Otro\n\nNada que ver\n";
    expect(asked()?.("![x](x.png)")).toBe(false);
    expect(here.text).toBe("# Otro\n\nNada que ver\n");
  });
});
