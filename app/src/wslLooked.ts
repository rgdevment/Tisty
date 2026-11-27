const LOOKED = "tisty.wsl.looked";

export const looked = (): string[] => {
  try {
    return JSON.parse(window.localStorage.getItem(LOOKED) ?? "[]");
  } catch {
    return [];
  }
};

export const remember = (all: string[]) => {
  try {
    window.localStorage.setItem(LOOKED, JSON.stringify(all));
  } catch {
    // A private window keeps nothing, and the distro is asked again next time.
  }
};
