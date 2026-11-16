import { fell } from "../Standing";

export type LoadedDiary = typeof import("./DiaryView").default;

export const diaryLoaded = (): Promise<LoadedDiary> =>
  import("./DiaryView").catch(fell).then(({ default: ready }) => ready);
