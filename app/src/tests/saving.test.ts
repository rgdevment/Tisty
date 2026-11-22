import { describe, expect, it, vi } from "vitest";
import { letAsideGo } from "../keptAside";
import {
  allUnanswered,
  busy,
  cut,
  holds,
  kept,
  landedWords,
  leftUnsaved,
  PATIENCE,
  queued,
  resentThen,
  type SaveTurn,
  settled,
  stalledOn,
  unansweredWords,
  unsettled,
} from "../saving";

const never = <T>() => new Promise<T>(() => {});

const told = (seen: string[], name: string, work: Promise<unknown>) =>
  work.then(
    (said) => seen.push(`${name} ${String(said)}`),
    (why: { code?: string }) => seen.push(`${name} ${why?.code}`),
  );

describe("the saves of one document", () => {
  it("go one after another while each answers", async () => {
    const seen: string[] = [];
    let first = (_: string) => {};
    void told(
      seen,
      "first",
      kept("orden", "uno", () => new Promise<string>((done) => (first = done))),
    );
    void told(
      seen,
      "second",
      kept("orden", "dos", async () => "dos"),
    );
    await vi.advanceTimersByTimeAsync(PATIENCE / 2);
    expect(seen).toEqual([]);

    first("uno");
    await vi.advanceTimersByTimeAsync(0);
    expect(seen).toEqual(["first uno", "second dos"]);
  });

  it("let the next one go once one has not answered for the whole patience, and say it stopped", async () => {
    const seen: string[] = [];
    void told(seen, "stuck", kept("colgado", "uno", never));
    void told(
      seen,
      "next",
      kept("colgado", "dos", async () => "dos"),
    );

    await vi.advanceTimersByTimeAsync(PATIENCE - 1);
    expect(seen).toEqual([]);
    await vi.advanceTimersByTimeAsync(1);

    expect(seen).toEqual(["stuck saveStuck", "next dos"]);
  });

  it("each carry a newer turn of the same page than the one before", async () => {
    const turns: SaveTurn[] = [];
    await kept("turnos", "uno", async (turn) => turns.push(turn));
    await kept("otro", "dos", async (turn) => turns.push(turn));
    await kept("turnos", "tres", async (turn) => turns.push(turn));

    expect(turns.map((one) => one.hand)).toEqual([turns[0].hand, turns[0].hand, turns[0].hand]);
    expect(turns[1].n).toBeGreaterThan(turns[0].n);
    expect(turns[2].n).toBeGreaterThan(turns[1].n);
  });

  it("keep the words of one that did not answer until a later one answers", async () => {
    let turn: SaveTurn | undefined;
    void kept("quedan", "leche y pan", (sent) => {
      turn = sent;
      return never();
    }).catch(() => {});
    await vi.advanceTimersByTimeAsync(PATIENCE);

    expect(unansweredWords("quedan")).toBe("leche y pan");
    expect(landedWords("quedan", turn?.n)).toBe("leche y pan");
    expect(landedWords("quedan", (turn?.n ?? 0) + 1)).toBeUndefined();

    await kept("quedan", "leche y pan", async () => "guardado");
    expect(unansweredWords("quedan")).toBeUndefined();
  });

  it("keep the words of one that was refused, without saying it stopped", async () => {
    await kept("rechazado", "leche y pan", () => Promise.reject({ code: "documentMoved" })).catch(
      () => {},
    );

    expect(unansweredWords("rechazado")).toBe("leche y pan");
    expect(stalledOn("rechazado")).toBe(false);
    cut("rechazado");
  });

  it("keep the first and the newest that stopped, either of which may land, never each one between", async () => {
    const turns: SaveTurn[] = [];
    for (const words of ["uno", "dos", "tres"]) {
      void kept("varios", words, (turn) => {
        turns.push(turn);
        return never();
      }).catch(() => {});
      await vi.advanceTimersByTimeAsync(PATIENCE);
    }
    expect(landedWords("varios", turns[1].n)).toBeUndefined();
    expect(unansweredWords("varios")).toBe("tres");

    await kept("varios", "cuatro", () => Promise.reject({ code: "documentMoved" })).catch(() => {});
    await kept("varios", "cinco", () => Promise.reject({ code: "documentMoved" })).catch(() => {});

    expect(landedWords("varios", turns[0].n)).toBe("uno");
    expect(landedWords("varios", turns[2].n)).toBe("tres");
    expect(unansweredWords("varios")).toBe("cinco");
    expect(allUnanswered()).toContainEqual(["varios", "cinco"]);
    cut("varios");
  });

  it("let go of the words of one refused for good, yet still weave from one that stopped before it", async () => {
    let first: SaveTurn | undefined;
    void kept("para siempre", "uno", (turn) => {
      first = turn;
      return never();
    }).catch(() => {});
    await vi.advanceTimersByTimeAsync(PATIENCE);
    for (const code of ["documentTooLong", "documentLocked", "noSuchDoc"]) {
      await kept("para siempre", "dos", () => Promise.reject({ code })).catch(() => {});

      expect(unansweredWords("para siempre"), code).toBeUndefined();
    }
    expect(allUnanswered().map(([id]) => id)).not.toContain("para siempre");
    expect(unsettled()).toBe(false);
    expect(landedWords("para siempre", first?.n)).toBe("uno");
    cut("para siempre");
    letAsideGo("para siempre");
  });

  it("remember one that broke as one that may have landed, without saying it stopped", async () => {
    let broke: SaveTurn | undefined;
    await kept("roto", "leche y pan", (turn) => {
      broke = turn;
      return Promise.reject({ code: "internal" });
    }).catch(() => {});

    expect(landedWords("roto", broke?.n)).toBe("leche y pan");
    expect(unansweredWords("roto")).toBe("leche y pan");
    expect(stalledOn("roto")).toBe(false);
    cut("roto");
  });

  it("keep the words of one refused while its document is away, since it may come back", async () => {
    await kept("lejos", "leche y pan", () => Promise.reject({ code: "documentAway" })).catch(
      () => {},
    );

    expect(unansweredWords("lejos")).toBe("leche y pan");
    expect(stalledOn("lejos")).toBe(false);
    cut("lejos");
  });

  it("leave nothing behind for one set aside because a newer one went first", async () => {
    await kept("apartado", "leche", () => Promise.reject({ code: "saveLate" })).catch(() => {});

    expect(unansweredWords("apartado")).toBeUndefined();
  });
});

describe("words a save left behind", () => {
  it("are let go at once with every save asked for before: one not begun is never sent, one on its way leaves nothing", async () => {
    const sent = new Set<string>();
    void kept("cortado", "uno", () => {
      sent.add("uno");
      return never();
    }).catch(() => {});
    void kept("cortado", "dos", async () => {
      sent.add("dos");
      return "dos";
    }).catch(() => {});
    await vi.advanceTimersByTimeAsync(0);

    cut("cortado");
    await vi.advanceTimersByTimeAsync(PATIENCE);

    expect([...sent]).toEqual(["uno"]);
    expect(unansweredWords("cortado")).toBeUndefined();
    await kept("cortado", "tres", async () => {
      sent.add("tres");
      return "tres";
    });
    expect([...sent]).toEqual(["uno", "tres"]);
  });

  it("leave nothing of a document behind once its saves land, yet one asked for before a cut never goes", async () => {
    const sent: string[] = [];
    let first = (_: string) => {};
    void kept("vaciado", "uno", () => {
      sent.push("uno");
      return new Promise<string>((done) => (first = done));
    });
    const second = kept("vaciado", "dos", async () => {
      sent.push("dos");
      return "dos";
    });
    await vi.advanceTimersByTimeAsync(0);

    cut("vaciado");
    first("uno");

    await expect(second).rejects.toEqual({ code: "saveLate" });
    await vi.advanceTimersByTimeAsync(0);
    expect(busy("vaciado")).toBeUndefined();
    await kept("vaciado", "tres", async () => {
      sent.push("tres");
      return "tres";
    });
    expect(sent).toEqual(["uno", "tres"]);
    await vi.advanceTimersByTimeAsync(0);
    expect(busy("vaciado")).toBeUndefined();
  });

  it("say a save that was let go was set aside, however it fails after", async () => {
    let fail = (_: unknown) => {};
    const going = kept(
      "soltado",
      "uno",
      () =>
        new Promise((_, refused) => {
          fail = refused;
        }),
    );
    await vi.advanceTimersByTimeAsync(0);

    cut("soltado");
    fail({ code: "documentMoved" });

    await expect(going).rejects.toEqual({ code: "saveLate" });
    expect(unansweredWords("soltado")).toBeUndefined();
  });

  it("are sent again, not only flushed, when the computer shuts down", async () => {
    const said: string[] = [];
    const off = holds(() => said.splice(said.length, 0, "flushed"), {
      resend: () => said.splice(said.length, 0, "resent"),
    });
    let ended = false;
    resentThen(async () => {
      ended = true;
    })();
    await vi.waitFor(() => expect(ended).toBe(true));
    expect(said).toEqual(["resent"]);
    off();
  });
});

describe("work that must never overlap", () => {
  it("waits its turn however long the one before takes", async () => {
    const seen: string[] = [];
    let first = (_: string) => {};
    void told(
      seen,
      "first",
      queued("ajuste", () => new Promise<string>((done) => (first = done))),
    );
    void told(
      seen,
      "second",
      queued("ajuste", async () => "dos"),
    );

    await vi.advanceTimersByTimeAsync(PATIENCE * 2);
    expect(seen).toEqual([]);

    first("uno");
    await vi.advanceTimersByTimeAsync(0);
    expect(seen).toEqual(["first uno", "second dos"]);
  });
});

describe("the pages that hold words", () => {
  it("are all asked to send them, not only the last one that said so", async () => {
    const sent: string[] = [];
    const first = holds(() => sent.push("documento"));
    const second = holds(() => sent.push("tarea"));
    second();

    await settled();

    expect(sent).toEqual(["documento"]);
    first();
  });

  it("make closing say words are left while one still holds some", async () => {
    let held = true;
    const release = holds(() => {}, { holding: () => held });

    expect(unsettled()).toBe(true);
    expect(await leftUnsaved()).toBe(true);
    held = false;
    expect(await leftUnsaved()).toBe(false);
    release();
  });
});

describe("what closing learns from the saves", () => {
  it("that nothing is left once every save answered", async () => {
    await kept("cierre", "uno", async () => "uno");

    expect(await leftUnsaved()).toBe(false);
  });

  it("that words are left when a save of a document never answered", async () => {
    void kept("cierre colgado", "uno", never).catch(() => {});

    const left = leftUnsaved();
    await vi.advanceTimersByTimeAsync(PATIENCE);

    expect(await left).toBe(true);
    cut("cierre colgado");
  });

  it("that words are left when work that must never overlap is still waiting", async () => {
    void queued("palabras colgadas", never);

    const left = leftUnsaved();
    await vi.advanceTimersByTimeAsync(PATIENCE);

    expect(await left).toBe(true);
  });
});
