import { createSignal, flush, onCleanup, Show } from "solid-js";
import Bar from "./Bar";
import Finder from "./Finder";
import {
  editAsked,
  editCopy,
  type Facts,
  facts,
  type Opened,
  opened,
  type Refused,
  shown,
  source,
  themed,
  theRest,
} from "./host";
import { keyHeard } from "./keys";
import { arrivingAt, canGoBack, goBack, linkPressed } from "./links";
import MoreMenu from "./MoreMenu";
import Panels from "./Panels";
import Problem from "./Problem";
import { bringFar, bringNearPictures, farPictures } from "./pictures";
import { blockAt, remember, remembered } from "./place";
import { dress, isDark, laidIn, printed, redraw, settled, turnTheme } from "./sheet";
import { type Words, wordsFor } from "./words";

const ZOOMS = [0.8, 0.9, 1, 1.1, 1.25, 1.4, 1.6];
const PLAIN = ZOOMS.indexOf(1);
const BAR_EDGE = 90;
const REMEMBER_AFTER = 300;

const leftWords = (words: Words, missed: number, notes: number): string =>
  [
    missed > 0 ? words.say("importMissed", String(missed)) : "",
    notes > 0 ? words.say("importNotes", String(notes)) : "",
  ]
    .filter(Boolean)
    .join(" ");

const saidOf = (words: Words, problem: unknown): string => {
  const refused = problem as Refused | null;
  if (refused?.code === "documentTooBig") return words.say("editTooBig", refused.name);
  return refused?.code ? words.say("failed") : String(problem);
};

const fieldsIn = (page: HTMLElement): (readonly [string, string])[] =>
  Array.from(page.querySelectorAll(".front > div")).map(
    (row) =>
      [
        row.querySelector("dt")?.textContent ?? "",
        row.querySelector("dd")?.textContent ?? "",
      ] as const,
  );

export default function App(props: { readonly asked: string | null; readonly close: () => void }) {
  const words = wordsFor(props.asked);
  const [one, setOne] = createSignal<Opened | null>(null);
  const [about, setAbout] = createSignal<Facts | null>(null);
  const [whole, setWhole] = createSignal(false);
  const [written, setWritten] = createSignal<string | null>(null);
  const [zoom, setZoom] = createSignal(PLAIN);
  const [finding, setFinding] = createSignal(false);
  const [more, setMore] = createSignal(false);
  const [side, setSide] = createSignal(false);
  const [note, setNote] = createSignal<string | null>(null);
  const [fields, setFields] = createSignal<readonly (readonly [string, string])[]>([]);
  const [far, setFar] = createSignal<readonly HTMLImageElement[]>([]);
  let page!: HTMLElement;

  let everyPiece = () => {};
  const laid = new Promise<void>((done) => {
    everyPiece = done;
  });

  const fail = (problem: unknown) => setNote(saidOf(words, problem));

  const look = async () => {
    const now = await opened();
    if (now.kind === "drawn") {
      page.innerHTML = now.first;
      setFields(fieldsIn(page));
    }
    document.title = now.name;
    setOne(now);
    flush();
    await shown();
    if (now.kind === "missed" || now.first.trim() === "") return;
    void facts().then(setAbout, () => {});
    await laidIn(page, now.more ? await theRest() : []);
    setWhole(true);
    everyPiece();
    bringNearPictures(page);
    setFar(farPictures(page));
    const anchor = arrivingAt();
    const block = remembered(now.path);
    if (anchor) document.getElementById(anchor)?.scrollIntoView({ block: "start" });
    else if (block !== undefined) page.children[block]?.scrollIntoView({ block: "start" });
    void settled(page).catch(fail);
    await dress(words.sayToSheet, words.locale);
  };

  void look().catch(async (error_) => {
    fail(error_);
    await shown();
  });

  const edit = () =>
    editCopy()
      .then((left) => {
        if (!left) return setNote(words.say("editAfterWelcome"));
        const said = leftWords(words, left.missed, left.notes);
        if (said) setNote(said);
        else props.close();
      })
      .catch(fail);

  const stopListening = editAsked(edit);
  onCleanup(() => void stopListening.then((stop) => stop()));

  void themed(isDark()).catch(() => {});

  const turn = () => {
    turnTheme();
    void themed(isDark()).catch(() => {});
    if (whole()) void redraw().catch(fail);
  };

  const toSource = () => {
    if (written() !== null) return setWritten(null);
    void source().then(setWritten, fail);
  };

  const toPaper = () => void printed().catch(fail);

  const scale = (step: number) => {
    const at = step === 0 ? PLAIN : Math.max(0, Math.min(ZOOMS.length - 1, zoom() + step));
    setZoom(at);
    page.style.setProperty("zoom", String(ZOOMS[at]));
  };

  const go = (id: string) => {
    setWritten(null);
    flush();
    void laid.then(() => document.getElementById(id)?.scrollIntoView({ block: "start" }));
  };

  const stepHead = (by: number) => {
    const now = one();
    const tops = (now?.kind === "drawn" ? now.heads : []).map((head) =>
      document.getElementById(head.id),
    );
    const reached = tops.findLastIndex(
      (top) => (top?.getBoundingClientRect().top ?? 0) <= BAR_EDGE + 8,
    );
    tops[Math.max(0, reached + by)]?.scrollIntoView({ block: "start" });
  };

  const leave = () => {
    if (more()) return setMore(false);
    if (finding()) return setFinding(false);
    if (written() !== null) return setWritten(null);
    setSide(false);
  };

  const keyed = keyHeard({
    find: () => setFinding(true),
    side: () => setSide((was) => !was),
    theme: turn,
    zoom: scale,
    head: stepHead,
    back: () => void goBack().catch(fail),
    source: toSource,
    print: toPaper,
    close: props.close,
    escape: leave,
  });
  window.addEventListener("keydown", keyed);
  onCleanup(() => window.removeEventListener("keydown", keyed));

  let waiting: ReturnType<typeof setTimeout> | undefined;
  const scrolled = () => {
    clearTimeout(waiting);
    waiting = setTimeout(() => {
      const now = one();
      if (now && whole() && written() === null) remember(now.path, blockAt(page, BAR_EDGE));
    }, REMEMBER_AFTER);
  };
  window.addEventListener("scroll", scrolled, { passive: true });
  onCleanup(() => {
    clearTimeout(waiting);
    window.removeEventListener("scroll", scrolled);
  });

  const drawn = () => {
    const now = one();
    return now?.kind === "drawn" ? now : null;
  };
  const empty = () => drawn()?.first.trim() === "";

  return (
    <>
      <Bar
        words={words}
        name={one()?.name ?? ""}
        place={one()?.place ?? ""}
        onFind={() => setFinding(true)}
        onTheme={turn}
        onMore={() => setMore((was) => !was)}
        onSide={() => setSide((was) => !was)}
        onBack={canGoBack() ? () => void goBack().catch(fail) : undefined}
      />
      <Show when={more()}>
        <MoreMenu
          words={words}
          path={one()?.path ?? ""}
          onClose={() => setMore(false)}
          onFind={() => setFinding(true)}
          onZoom={scale}
          onPrint={toPaper}
          fail={fail}
        />
      </Show>
      <Show when={finding() && whole()}>
        <Finder words={words} within={page} onClose={() => setFinding(false)} />
      </Show>
      <Show when={note()}>
        <p class="note" role="alert">
          {note()}
        </p>
      </Show>
      <main class="desk">
        <div class="sheet-column">
          <Show when={far().length > 0}>
            <button
              class="far-pictures"
              type="button"
              onClick={() => {
                void bringFar(far()).then((failed) => {
                  if (failed > 0) setNote(words.say("farFailed", String(failed)));
                });
                setFar([]);
              }}
            >
              {words.say("farPictures", String(far().length))}
            </button>
          </Show>
          <article
            class="page"
            ref={(sheet) => {
              page = sheet;
              sheet.addEventListener("click", (e) => linkPressed(e, one()?.path ?? "", fail));
            }}
            hidden={drawn() === null || empty() || written() !== null}
          />
          <Show when={written() !== null}>
            <pre class="written">{written()}</pre>
          </Show>
          <Show when={one()?.kind === "missed" || empty()}>
            <Problem
              words={words}
              opened={one() as Opened}
              onRetry={() => location.reload()}
              onEdit={edit}
              fail={fail}
            />
          </Show>
        </div>
        <Show when={drawn() !== null && !empty()}>
          <Panels
            words={words}
            open={side()}
            heads={drawn()?.heads ?? []}
            place={one()?.place ?? ""}
            about={about()}
            fields={fields()}
            sourced={written() !== null}
            onEdit={edit}
            onSource={toSource}
            onGo={go}
          />
        </Show>
      </main>
    </>
  );
}
