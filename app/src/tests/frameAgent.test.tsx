import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Reaching } from "../ui/dressing";
import Frame from "../ui/Frame";

const framed = (agent?: boolean) =>
  render(
    <Frame
      html=""
      turn={0}
      label="page"
      tall={false}
      hidden={false}
      agent={agent}
      reach={() => ({}) as Reaching}
      onReady={() => {}}
    />,
  );

describe("the frame an agent's file is drawn in", () => {
  it("marks its page as an agent's, so the page drops its paper and title", () => {
    const { container } = framed(true);
    const frame = container.querySelector("iframe") as HTMLIFrameElement;
    expect(frame.contentDocument?.documentElement.classList.contains("agent")).toBe(true);
  });

  it("leaves any other document's page unmarked", () => {
    const { container } = framed();
    const frame = container.querySelector("iframe") as HTMLIFrameElement;
    expect(frame.contentDocument?.documentElement.classList.contains("agent")).toBe(false);
  });
});
