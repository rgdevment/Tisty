import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { laidIn, withCard, withDeclared } from "../front";
import { t } from "../locales";
import { turnsIn } from "../ui/printing";
import StampFields from "../ui/StampFields";
import { sheeted } from "../ui/sheeting";
import { cardOn } from "../ui/stamped";

describe("the card a document carries", () => {
  it("is the document's own choice, written down only when it is on", () => {
    expect(laidIn("# Plan\n").carded).toBe(false);
    const on = withCard("# Plan\n", true);
    expect(on).toBe("---\ncard: on\n---\n# Plan\n");
    expect(laidIn(on).carded).toBe(true);
    expect(withCard(on, false)).toBe("# Plan\n");
  });

  it("writes the author and version as plain lines, quoting what YAML would misread", () => {
    expect(withDeclared("# Plan\n", "author", "  Mario Hidalgo ")).toBe(
      "---\nauthor: Mario Hidalgo\n---\n# Plan\n",
    );
    expect(withDeclared("# Plan\n", "version", "1.2: borrador")).toBe(
      '---\nversion: "1.2: borrador"\n---\n# Plan\n',
    );
    expect(withDeclared('---\nauthor: "A"\n---\n# Plan\n', "author", "")).toBe("# Plan\n");
  });

  it("asks for the author and the version only once the card is on, offering who signs", async () => {
    const onCard = vi.fn();
    const onDeclare = vi.fn();
    const off = render(
      <StampFields carded={false} author="" version="" onCard={onCard} onDeclare={onDeclare} />,
    );
    expect(screen.queryByRole("textbox")).toBeNull();
    await userEvent.click(screen.getByRole("switch", { name: t("cardShown") }));
    expect(onCard).toHaveBeenCalledWith(true);
    off.unmount();

    render(
      <StampFields
        carded
        author=""
        version="1.0"
        signer="Mario Hidalgo"
        onCard={onCard}
        onDeclare={onDeclare}
      />,
    );
    const [author, version] = screen.getAllByRole("textbox") as HTMLInputElement[];
    expect(author.placeholder).toBe("Mario Hidalgo");
    await userEvent.type(author, "Equipo Norte");
    fireEvent.blur(author);
    expect(onDeclare).toHaveBeenCalledWith("author", "Equipo Norte");
    fireEvent.blur(version);
    expect(onDeclare).toHaveBeenCalledTimes(1);
  });

  it("sits under the title on screen, where the sheets count it, and tells the PDF where they turn", () => {
    document.body.innerHTML =
      '<article class="page"><h1 data-at="0-6">Plan</h1><p data-at="7-20">uno</p><p data-at="21-30">dos</p></article>';
    const sheet = document.querySelector<HTMLElement>(".page") as HTMLElement;
    cardOn(document, [{ label: "Autor", said: "Ana" }]);
    expect(sheet.children[1].className).toBe("stamp");
    expect(sheet.children[1].hasAttribute("inert")).toBe(true);
    cardOn(document, [{ label: "Autor", said: "Eva" }]);
    expect(sheet.querySelectorAll(".stamp")).toHaveLength(1);

    const sizes = [
      [10, 30],
      [40, 60],
      [100, 30],
      [130, 30],
    ];
    Array.from(sheet.children).forEach((one, at) => {
      Object.defineProperty(one, "offsetTop", { value: sizes[at][0] });
      Object.defineProperty(one, "offsetHeight", { value: sizes[at][1] });
    });
    Object.defineProperty(sheet, "offsetTop", { value: 0 });
    sheeted(document, { wide: 794, tall: 160, x: 10, y: 10 });

    expect(turnsIn(document)).toEqual(["21-30"]);
  });
});
