import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import Composed from "../ui/Composed";

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (at: string) => at, invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn(), openPath: vi.fn() }));
vi.mock("../core", () => ({ served: vi.fn(), opened: vi.fn() }));

function tall(overflowing: boolean) {
  Object.defineProperty(HTMLElement.prototype, "scrollHeight", {
    configurable: true,
    get: () => (overflowing ? 900 : 100),
  });
  Object.defineProperty(HTMLElement.prototype, "clientHeight", {
    configurable: true,
    get: () => 100,
  });
}

afterEach(() => {
  Reflect.deleteProperty(HTMLElement.prototype, "scrollHeight");
  Reflect.deleteProperty(HTMLElement.prototype, "clientHeight");
});

const shown = (over: Partial<React.ComponentProps<typeof Composed>> = {}) =>
  render(<Composed html="<p>a long body</p>" className="prose" onWhole={() => {}} {...over} />);

describe("a body longer than the space it has", () => {
  const offer = () => screen.queryByRole("button", { name: /Read it in full/ });

  it("offers the full screen only when the body does not fit", () => {
    tall(false);
    const { unmount } = shown();
    expect(offer()).toBeNull();
    unmount();

    tall(true);
    shown();
    expect(offer()).toBeTruthy();
  });

  it("sends the reader there instead of unfolding in place", async () => {
    const user = userEvent.setup();
    tall(true);
    const onWhole = vi.fn();
    const onEnter = vi.fn();
    shown({ onWhole, onEnter });

    const one = offer();
    if (!one) throw new Error("no offer to click");
    await user.click(one);

    expect(onWhole).toHaveBeenCalled();
    expect(onEnter).not.toHaveBeenCalled();
  });

  it("never cuts where there is nowhere roomier to send anyone", () => {
    tall(true);
    shown({ onWhole: undefined });
    expect(offer()).toBeNull();
  });
});
