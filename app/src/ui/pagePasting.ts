import { t } from "../locales";
import type { Begun } from "./pageBegun";
import { blockPlaced, markedLanding, pastedAround } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { applied, type Span } from "./pageEdits";
import { laidOver } from "./pageLaidOver";
import { blockAsked, type Laid, wordsAsked } from "./pageLate";
import type { Laying } from "./pageLaying";
import { closing, opening } from "./pageLists";
import type { Mend } from "./pageMends";
import { onSheet } from "./pageNodes";
import { foldedInto, type Kept, type Writer } from "./pageWriter";

export const pastingOn = (writer: Writer, caret: Caret, laying: Laying, begun: Begun) => {
  const { page, hands } = writer;
  const { late } = laying;

  // A line holding only its mark keeps it for words of one line, and gives it up to lines or a block of their own.
  const pastedOn = (
    now: string,
    at: Span,
    said: string,
    coded: boolean,
    mend: Mend | null,
  ): Laid => {
    const marked =
      coded || mend || at.from !== at.to ? null : markedLanding(now, at.from, at.to, said);
    if (marked?.kept) return laidOver(now, at, said, false);
    const from = marked?.from ?? at.from;
    return laidOver(now, { from, to: at.to }, pastedAround(now, from, at.to, said), coded, mend);
  };

  const pasted = (one: Span, data: DataTransfer | null, coded: boolean, mend: Mend | null) => {
    const html = data?.getData("text/html") || null;
    const plain = data?.getData("text/plain") ?? "";
    const asked = wordsAsked(hands.text(), one);
    const landed = (said: string) => {
      if (!late(asked, (now, at) => pastedOn(now, at, said, coded, mend)))
        hands.lost?.(t("pastedLost"));
    };
    hands
      .translated(html, plain)
      .then(landed)
      .catch(() => landed(plain));
  };

  // A line begun and not yet written is written first, as a first key writes it, so a key typed meanwhile finds it there.
  const pastedOnBegun = (line: Kept, data: DataTransfer | null) => {
    const html = data?.getData("text/html") || null;
    const plain = data?.getData("text/plain") ?? "";
    if (!plain.trim() && !html?.trim()) return;
    const was = writer.turns.undone[writer.turns.undone.length - 1];
    begun.firstKey(line, "");
    const last = writer.turns.undone[writer.turns.undone.length - 1];
    const opening = last !== was ? last : undefined;
    const opened = hands.text();
    const at = writer.kept?.lead === undefined ? (writer.kept?.at ?? line.at) : line.at;
    const asked = wordsAsked(opened, { from: at, to: at });
    const folded = () => opening && foldedInto(writer.turns, opening, hands.text());
    // A paste that brings nothing leaves no empty opening behind, as long as nothing was written since.
    const takenBack = () => {
      const turns = writer.turns.undone;
      if (!opening || hands.text() !== opened || turns[turns.length - 1] !== opening) return;
      turns.pop();
      const end = opening.from + opening.added.length;
      const edit = { from: opening.from, to: end, insert: opening.removed };
      writer.kept = { ...line, node: page.body, offset: -1 };
      const next = applied(opened, edit);
      hands.write(next);
      laying.laidOut(opened, next, edit);
    };
    const landed = (said: string) => {
      if (!said.trim()) return takenBack();
      if (!late(asked, (now, here) => pastedOn(now, here, said, false, null), folded)) {
        takenBack();
        hands.lost?.(t("pastedLost"));
      }
    };
    hands
      .translated(html, plain)
      .then(landed)
      .catch(() => landed(plain));
  };

  const pastedFiles = (event: ClipboardEvent) => {
    const files = Array.from(event.clipboardData?.files ?? []);
    const attach = hands.pasted;
    if (
      !onSheet(page, event) ||
      !files.length ||
      !attach ||
      event.clipboardData?.getData("text/plain")
    )
      return;
    event.preventDefault();
    const span = caret.caretSpan();
    if (!span) return;
    const end = Math.max(span.to, caret.blockEnd(span.to));
    const asked = blockAsked(hands.text(), span.from, end);
    attach(files)
      .then((said) => {
        if (!said) return;
        const laid = (now: string, at: Span) => blockPlaced(now, at.from, at.to, said, said.length);
        const placed = late(asked, laid, ({ edit }) => {
          // The words typed next go under the file just laid, never into its line.
          const after = hands.text();
          const under = edit.from + edit.insert.replace(/\s+$/, "").length;
          writer.kept = {
            node: page.body,
            offset: -1,
            at: under,
            lead: opening(after, under, ""),
            tail: closing(after, under),
          };
        });
        if (!placed) hands.lost?.(t("attachmentLost"));
      })
      .catch(() => {});
  };

  return { pasted, pastedOnBegun, pastedFiles };
};
