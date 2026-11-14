import { useEffect, useRef, useState } from "react";

const RESTS_AFTER_MS = 900;
const SHORTEST = 32;

type Thumbing = {
  readonly frame: () => Window | null;
  readonly ready: boolean;
  readonly turned: number;
};

type Seen = { readonly top: number; readonly tall: number };

const placed = (sheet: Element): Seen | null => {
  const { scrollTop, scrollHeight, clientHeight } = sheet;
  if (scrollHeight <= clientHeight + 1) return null;
  const tall = Math.max(SHORTEST, (clientHeight / scrollHeight) * clientHeight);
  return { top: (scrollTop / (scrollHeight - clientHeight)) * (clientHeight - tall), tall };
};

// The frame's own scrollbar takes a column beside the sheet and cuts its page breaks short, so a thumb is laid over its edge.
export function FrameThumb({ frame, ready, turned }: Thumbing) {
  const [seen, setSeen] = useState<Seen | null>(null);
  const [awake, setAwake] = useState(false);
  const resting = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const held = useRef<{ y: number; at: number } | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: each drawing may bring the frame anew, and its height with it.
  useEffect(() => {
    const inner = frame();
    const sheet = inner?.document.scrollingElement;
    if (!inner || !sheet) return;
    const measured = () => setSeen(placed(sheet));
    const rolled = () => {
      measured();
      setAwake(true);
      clearTimeout(resting.current);
      resting.current = setTimeout(() => {
        if (!held.current) setAwake(false);
      }, RESTS_AFTER_MS);
    };
    measured();
    inner.addEventListener("scroll", rolled, { passive: true });
    inner.addEventListener("resize", measured);
    return () => {
      clearTimeout(resting.current);
      inner.removeEventListener("scroll", rolled);
      inner.removeEventListener("resize", measured);
    };
  }, [ready, turned]);

  if (!seen) return null;
  const sheet = () => frame()?.document.scrollingElement ?? null;
  return (
    <div
      aria-hidden="true"
      onPointerEnter={() => setAwake(true)}
      onPointerLeave={() => {
        if (!held.current) setAwake(false);
      }}
      // The strip lies over the frame's edge, so a wheel turned on it still rolls the page.
      onWheel={(event) => sheet()?.scrollBy({ top: event.deltaY })}
      className="absolute inset-y-0 right-0 z-10 w-3"
    >
      <div
        style={{ top: seen.top, height: seen.tall }}
        onPointerDown={(event) => {
          const at = sheet()?.scrollTop ?? 0;
          held.current = { y: event.clientY, at };
          event.currentTarget.setPointerCapture(event.pointerId);
          event.preventDefault();
        }}
        onPointerMove={(event) => {
          const one = sheet();
          if (!held.current || !one) return;
          const room = one.clientHeight - seen.tall;
          const by = room > 0 ? (one.scrollHeight - one.clientHeight) / room : 0;
          one.scrollTop = held.current.at + (event.clientY - held.current.y) * by;
        }}
        onPointerUp={() => {
          held.current = null;
        }}
        className={`absolute right-[3px] w-[6px] cursor-default rounded-full bg-line transition-opacity duration-200 hover:bg-faint ${
          awake ? "opacity-100" : "opacity-0"
        }`}
      />
    </div>
  );
}
