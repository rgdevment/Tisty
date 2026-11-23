const UNSEEN = /​/g;

export const shownText = (said: string) => said.replace(UNSEEN, "");
