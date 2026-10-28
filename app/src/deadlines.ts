export type Standing = "ahead" | "today" | "late" | "underway" | "due_soon" | "overdue";

export type Pace = "on_track" | "slow" | "behind";

export interface Paced {
  pace: Pace;
  time: number;
  done: number;
}

export interface Dated {
  standings?: Record<string, Standing>;
  paces?: Record<string, Paced>;
}
