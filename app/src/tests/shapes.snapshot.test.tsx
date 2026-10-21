import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import Glyph from "../ui/Glyph";
import Repeated from "../ui/Repeated";
import Tags from "../ui/Tags";

describe("the markup of small parts of the window", () => {
  it("draws the repeated lists with the button to join them", () => {
    const { asFragment } = render(
      <Repeated
        seen={[
          { name: "Trabajo", lists: 2, tasks: 14 },
          { name: "Familia", lists: 3, tasks: 0 },
        ]}
        held={false}
        join={() => {}}
        className="join"
      />,
    );
    expect(asFragment()).toMatchSnapshot();
  });

  it("draws the repeated lists as a held button", () => {
    const { asFragment } = render(
      <Repeated
        seen={[{ name: "Casa", lists: 2, tasks: 1 }]}
        held
        join={() => {}}
        className="join"
      />,
    );
    expect(asFragment()).toMatchSnapshot();
  });

  it("draws the tag picker with the chosen ones marked", () => {
    const { asFragment } = render(
      <Tags
        tags={[
          { tag: "work", tasks: 4, docs: 0 },
          { tag: "home", tasks: 2, docs: 3 },
        ]}
        chosen={["home"]}
        onToggle={() => {}}
      />,
    );
    expect(asFragment()).toMatchSnapshot();
  });

  it("draws a glyph from the table and a plain mark", () => {
    const { asFragment } = render(
      <>
        <Glyph name="home" />
        <Glyph name="🦫" className="h-6 w-6" />
      </>,
    );
    expect(asFragment()).toMatchSnapshot();
  });
});
