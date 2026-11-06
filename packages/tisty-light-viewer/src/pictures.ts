import { pictureFar, pictureNear } from "./host";

const NEAR_THE_VIEW = "800px";

const said = (one: HTMLImageElement): string => one.getAttribute("src") ?? "";
const onTheWeb = (src: string): boolean => /^https?:/i.test(src);
const inTisty = (src: string): boolean => /^(tisty|data|blob):/i.test(src);

const whenNear = (pictures: HTMLImageElement[], bring: (one: HTMLImageElement) => void): void => {
  if (!("IntersectionObserver" in window)) {
    for (const one of pictures) bring(one);
    return;
  }
  const watching = new IntersectionObserver(
    (seen) => {
      for (const one of seen) {
        if (!one.isIntersecting) continue;
        watching.unobserve(one.target);
        bring(one.target as HTMLImageElement);
      }
    },
    { rootMargin: NEAR_THE_VIEW },
  );
  for (const one of pictures) watching.observe(one);
};

export const bringNearPictures = (page: HTMLElement): void => {
  const near = Array.from(page.querySelectorAll("img")).filter((one) => {
    const src = said(one);
    return src !== "" && !onTheWeb(src) && !inTisty(src);
  });
  whenNear(near, (one) => {
    void pictureNear(said(one)).then(
      (data) => {
        one.src = data;
      },
      () => {},
    );
  });
};

export const farPictures = (page: HTMLElement): HTMLImageElement[] =>
  Array.from(page.querySelectorAll("img")).filter((one) => onTheWeb(said(one)));

export const bringFar = async (pictures: readonly HTMLImageElement[]): Promise<number> => {
  const brought = await Promise.allSettled(
    pictures.map(async (one) => {
      one.src = await pictureFar(said(one));
    }),
  );
  return brought.filter((one) => one.status === "rejected").length;
};
