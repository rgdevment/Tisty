/// A bracket left bare closes the label early, and the reference stops naming anything.
export const labelled = (said: string): string => said.replace(/([[\]\\])/g, "\\$1");
