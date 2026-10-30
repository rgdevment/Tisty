import type { EditWord } from "./hosted";

export const TYPES: readonly [string, EditWord][] = [
  ["text", "tableTypeText"],
  ["number", "tableTypeNumber"],
  ["money", "tableTypeMoney"],
  ["percent", "tableTypePercent"],
  ["date", "tableTypeDate"],
  ["duration", "tableTypeDuration"],
  ["check", "tableTypeCheck"],
  ["select", "tableTypeSelect"],
  ["tags", "tableTypeTags"],
  ["rating", "tableTypeRating"],
  ["link", "tableTypeLink"],
  ["url", "tableTypeUrl"],
  ["email", "tableTypeEmail"],
  ["phone", "tableTypePhone"],
];
