import { useState } from "react";
import { diaryStick, diaryWrite, type Today } from "../diary";
import { fill, t } from "../locales";
import { IDEA_AT_MOST } from "../notes";
import Glyph from "./Glyph";

interface Props {
  onBefore: () => Promise<unknown>;
  onWritten: (today: Today) => void;
  onError: (problem: unknown) => void;
}

export default function DiaryComposer({ onBefore, onWritten, onError }: Props) {
  const [text, setText] = useState("");
  const [noting, setNoting] = useState(false);
  const [keeping, setKeeping] = useState(false);

  const keep = () => {
    const said = text.trim();
    if (!said || keeping) return;
    setKeeping(true);
    onBefore()
      .catch(() => {})
      .then(() => (noting ? diaryStick(said) : diaryWrite(said)))
      .then((today) => {
        setText("");
        setNoting(false);
        onWritten(today);
      })
      .catch(onError)
      .finally(() => setKeeping(false));
  };

  const keys = (e: React.KeyboardEvent) => {
    if (e.key === "Escape" && noting) setNoting(false);
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey || noting)) {
      e.preventDefault();
      keep();
    }
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        keep();
      }}
      className="flex items-end gap-2 rounded-[10px] border border-line bg-bg px-3 py-2 focus-within:border-accent"
    >
      {noting ? (
        <input
          autoFocus
          value={text}
          maxLength={IDEA_AT_MOST}
          placeholder={fill("diaryNote", String(IDEA_AT_MOST))}
          aria-label={t("diaryStick")}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={keys}
          className="min-w-0 flex-1 bg-transparent py-1 text-[13px] text-ink outline-none placeholder:text-faint"
        />
      ) : (
        <textarea
          rows={1}
          value={text}
          placeholder={t("diaryWriting")}
          aria-label={t("diaryWriting")}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={keys}
          className="max-h-60 min-w-0 flex-1 resize-none bg-transparent py-1 text-[13px] text-ink outline-none [field-sizing:content] placeholder:text-faint"
        />
      )}
      <button
        type="button"
        aria-pressed={noting}
        aria-label={t("diaryStick")}
        title={t("diaryStick")}
        onClick={() => setNoting((was) => !was)}
        className="grid size-7 shrink-0 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink aria-pressed:bg-accent-soft aria-pressed:text-accent"
      >
        <Glyph name="sticker" className="h-[15px] w-[15px]" />
      </button>
      <button
        type="submit"
        aria-label={t("diaryKeep")}
        title={t("diaryKeep")}
        className="grid size-7 shrink-0 place-items-center rounded-full bg-accent text-white disabled:opacity-40"
        disabled={!text.trim() || keeping}
      >
        <Glyph name="send" className="h-[14px] w-[14px]" />
      </button>
    </form>
  );
}
