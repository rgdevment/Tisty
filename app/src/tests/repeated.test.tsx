import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import Repeated from "../ui/Repeated";

describe("the repeated lists in Maintenance", () => {
  it("names each repeated list, how many there are and what they hold, before joining", () => {
    render(
      <Repeated
        seen={[
          { name: "Trabajo", lists: 2, tasks: 14 },
          { name: "Familia", lists: 3, tasks: 0 },
        ]}
        held={false}
        join={() => {}}
        className=""
      />,
    );

    expect(screen.getByText("Trabajo")).toBeTruthy();
    expect(screen.getByText(fill("repeatedRow", "2", "14"))).toBeTruthy();
    expect(screen.getByText(fill("repeatedRow", "3", "0"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("repeatedDo") })).toBeTruthy();
  });

  it("says nothing repeats and offers no button when every name is single", () => {
    render(<Repeated seen={[]} held={false} join={() => {}} className="" />);

    expect(screen.getByText(t("repeatedNone"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("repeatedDo") })).toBeNull();
  });

  it("joins on a click", async () => {
    const join = vi.fn();
    render(
      <Repeated
        seen={[{ name: "Personal", lists: 2, tasks: 3 }]}
        held={false}
        join={join}
        className=""
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: t("repeatedDo") }));

    expect(join).toHaveBeenCalledTimes(1);
  });

  it("waits while another card is working", () => {
    render(
      <Repeated
        seen={[{ name: "Personal", lists: 2, tasks: 3 }]}
        held
        join={() => {}}
        className=""
      />,
    );

    const button = screen.getByRole("button", { name: t("repeatedDo") });
    expect((button as HTMLButtonElement).disabled).toBe(true);
  });
});
