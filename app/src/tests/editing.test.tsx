import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";

const dialog = vi.hoisted(() => ({ sure: true, asked: 0 }));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => {
    dialog.asked += 1;
    return Promise.resolve(dialog.sure);
  },
}));

const written: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: "unset",
  order: "a0",
  description: "the one for accounting",
  steps: [{ id: "01S", text: "collect the figures", done: false, order: "a0" }],
  log: [
    {
      id: "01E",
      at: "2026-08-10 09:00:00",
      tz: "America/Santiago",
      body: "spoke to accounting",
    },
  ],
  volume: { steps: 1, steps_done: 0, journal: 1, described: true },
};

function open(task: Task = written) {
  const on = {
    patch: vi.fn(),
    step: vi.fn(),
    mark: vi.fn(),
    dropStep: vi.fn(),
    moveStep: vi.fn(),
    log: vi.fn(),
    discard: vi.fn(),
    reopen: vi.fn(),
  };
  render(
    <Detail
      task={task}
      lists={[]}
      known={[]}
      expanded={false}
      onExpand={() => {}}
      onCollapse={() => {}}
      onPatch={on.patch}
      onStep={on.step}
      onMark={on.mark}
      onDropStep={on.dropStep}
      onLog={on.log}
      onComplete={() => {}}
      onDiscard={on.discard}
      onReopen={on.reopen}
      onStillOpen={() => {}}
      onErase={() => {}}
      onFold={() => {}}
      onReadAs={() => {}}
      onOpenToAgents={() => {}}
      onClose={() => {}}
    />,
  );
  return on;
}

const box = (name: string) =>
  screen.getByRole("textbox", {
    name: new RegExp(name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
  }) as HTMLTextAreaElement | HTMLInputElement;

const into = async (user: ReturnType<typeof userEvent.setup>, name: string) => {
  await user.click(screen.getByLabelText(name));
  return box(name);
};

describe("escape discards, blur keeps", () => {
  it("puts the title back and writes nothing", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(box("Title"));
    await user.type(box("Title"), "something else{Escape}");

    expect(on.patch).not.toHaveBeenCalled();
    expect(box("Title").value).toBe("write the report");
  });

  it("puts the description back and writes nothing", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(await into(user, "Description"));
    await user.type(box("Description"), "never mind{Escape}");

    expect(on.patch).not.toHaveBeenCalled();
    expect(screen.getByLabelText("Description").textContent).toContain("the one for accounting");
  });

  it("puts a step back and writes nothing", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(box("collect the figures"));
    await user.type(box("collect the figures"), "never mind{Escape}");

    expect(on.step).not.toHaveBeenCalled();
    expect(box("collect the figures").value).toBe("collect the figures");
  });

  it("puts a journal entry back and writes nothing", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(await into(user, "spoke to accounting"));
    await user.type(box("spoke to accounting"), "never mind{Escape}");

    expect(on.log).not.toHaveBeenCalled();
    expect(screen.getByLabelText("spoke to accounting").textContent).toContain(
      "spoke to accounting",
    );
  });

  it("keeps the edit when the field is simply left", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(await into(user, "Description"));
    await user.type(box("Description"), "the one for the bank");
    await user.tab();

    expect(on.patch).toHaveBeenCalledWith({ description: "the one for the bank" });
  });
});

describe("what an edit is allowed to do", () => {
  it("refuses to empty the title, because a task without one cannot be found", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(box("Title"));
    await user.tab();

    expect(on.patch).not.toHaveBeenCalled();
    expect(box("Title").value).toBe("write the report");
  });

  it("refuses to empty a step", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.clear(box("collect the figures"));
    await user.tab();

    expect(on.step).not.toHaveBeenCalled();
    expect(box("collect the figures").value).toBe("collect the figures");
  });

  it("says nothing when the text comes back unchanged", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.click(box("Title"));
    await user.tab();

    expect(on.patch).not.toHaveBeenCalled();
  });

  it("adds a step on Enter and empties the box", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.type(box("Add a step"), "call accounting{Enter}");

    expect(on.step).toHaveBeenCalledWith("call accounting");
    expect(box("Add a step").value).toBe("");
  });

  it("keeps a journal entry on Enter, which is how one is sent", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.type(await into(user, "Journal"), "they answered{Enter}");

    expect(on.log).toHaveBeenCalledWith("they answered");
    expect(screen.getByLabelText("Journal").textContent).toContain("What happened");
  });

  it("keeps a journal entry of several lines, written with Shift", async () => {
    const user = userEvent.setup();
    const on = open();

    await user.type(
      await into(user, "Journal"),
      "they answered{Shift>}{Enter}{/Shift}and then some",
    );
    expect(on.log).not.toHaveBeenCalled();

    await user.tab();
    expect(on.log).toHaveBeenCalledWith(["they answered", "and then some"].join("\n"));
  });
});

const erasers = () =>
  screen
    .getAllByRole("button", { name: /^Remove / })
    .filter((one) => !one.getAttribute("aria-label")?.includes("collect the figures"));

describe("taking a journal entry out", () => {
  it("asks for that entry by its id", async () => {
    const user = userEvent.setup();
    const on = open();

    dialog.sure = true;
    expect(erasers()).toHaveLength(1);
    await user.click(erasers()[0]);

    await vi.waitFor(() => expect(on.log).toHaveBeenCalledWith("", "01E"));
  });

  it("asks first, and leaves the entry when the answer is no", async () => {
    const user = userEvent.setup();
    const on = open();
    dialog.sure = false;
    dialog.asked = 0;

    await user.click(erasers()[0]);

    await vi.waitFor(() => expect(dialog.asked).toBe(1));
    expect(on.log).not.toHaveBeenCalled();
  });

  it("does not draw an entry left empty, which is how one is taken out", () => {
    open({
      ...written,
      log: [
        ...(written.log ?? []),
        { id: "02E", at: "2026-08-11 09:00:00", tz: "America/Santiago", body: "  " },
      ],
    });

    expect(erasers()).toHaveLength(1);
  });

  it("does not draw it on a settled task either", () => {
    open({
      ...written,
      status: "done",
      log: [{ id: "02E", at: "2026-08-11 09:00:00", tz: "America/Santiago", body: "" }],
    });

    expect(screen.queryByText("Journal")).toBeNull();
  });
});

describe("a settled task", () => {
  it("offers to reopen instead of to discard", () => {
    open({ ...written, status: "done" });
    expect(screen.getByRole("button", { name: /Reopen/ })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Not doing it/ })).toBeNull();
  });
});

describe("the entry an agent's word rests on", () => {
  it("offers no way to take it out while the word stands", () => {
    open({
      ...written,
      resolved: { at: "2026-08-10T09:00:00Z", by: "dev_agent", entry: "01E" },
    } as Task);

    expect(erasers()).toHaveLength(0);
  });
});
