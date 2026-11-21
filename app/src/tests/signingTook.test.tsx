import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Signed } from "../core";
import { useSigning } from "../ui/Signing";

const said = (over: Partial<Signed> = {}): Signed => ({
  alias: "Mario",
  name: "Mario Hidalgo",
  before: [],
  mine: 2,
  aliasImitates: false,
  nameImitates: false,
  ...over,
});

describe("what the backend says the person signs with", () => {
  it("lands in the field even when another update of the settings is already queued", () => {
    const { result } = renderHook(() => useSigning());

    act(() => {
      result.current.setMine(3);
      result.current.took(said());
    });

    expect(result.current.alias).toBe("Mario");
    expect(result.current.name).toBe("Mario Hidalgo");
    expect(result.current.signedAlias).toBe("Mario");
  });

  it("leaves a name still being typed to the person", () => {
    const { result } = renderHook(() => useSigning());
    act(() => result.current.took(said()));

    act(() => result.current.setAlias("Mar"));
    act(() => result.current.took(said({ alias: "Marisol" })));

    expect(result.current.alias).toBe("Mar");
    expect(result.current.signedAlias).toBe("Marisol");
  });

  it("is not known until the backend has answered", () => {
    const { result } = renderHook(() => useSigning());
    expect(result.current.signedAlias).toBeUndefined();

    act(() => result.current.took(said({ alias: null })));

    expect(result.current.signedAlias).toBeNull();
  });
});
