const WAITS = 10_000;
export const ASIDE_MARGIN = 80;

export interface Settling {
  at: () => HTMLElement | null;
  want: number | (() => number | null);
  done: () => void;
  put: (down: number) => void;
  watch?: () => HTMLElement | null;
}

export const settles = ({ at, want, done, put, watch }: Settling) => {
  let frame = 0;
  let mine = -1;
  let over = false;
  let seeing: ResizeObserver | null = null;
  let timer = 0;
  let writing = false;
  let tallness = -1;
  let typing: HTMLElement | null = null;

  const sheet = at();

  function nudged() {
    if (writing) return;
    const room = at();
    if (!room || mine < 0) return;
    if (room.scrollHeight !== tallness) return;
    if (room.scrollTop !== mine) stop();
  }

  function stop() {
    if (over) return;
    over = true;
    seeing?.disconnect();
    typing?.removeEventListener("beforeinput", stop);
    sheet?.removeEventListener("scroll", nudged);
    window.clearTimeout(timer);
    cancelAnimationFrame(frame);
    done();
  }

  const reach = () => {
    const room = at();
    if (!room) return stop();
    const asked = typeof want === "function" ? want() : want;
    if (asked === null) return;
    writing = true;
    room.scrollTop = asked;
    mine = room.scrollTop;
    writing = false;
    put(mine);
    tallness = room.scrollHeight;
    if (mine >= asked) stop();
  };

  frame = requestAnimationFrame(reach);
  sheet?.addEventListener("scroll", nudged);
  timer = window.setTimeout(stop, WAITS);

  const grows = watch?.();
  if (grows) {
    // A body still rendering keeps growing, and each growth reaches again: once a key is
    // pressed the view is theirs, not the one they left last time.
    typing = grows;
    grows.addEventListener("beforeinput", stop);
    if (typeof ResizeObserver !== "undefined") {
      seeing = new ResizeObserver(() => {
        if (!over) reach();
      });
      seeing.observe(grows);
    }
  }

  return stop;
};
