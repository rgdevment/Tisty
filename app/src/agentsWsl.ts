import { invoke } from "@tauri-apps/api/core";

export interface InDistro {
  name: string;
  agents: boolean;
  skills: boolean;
  running: boolean;
  looked: boolean;
  wrote: number;
  left: number;
}

export const agentsWsl = (names: string[]): Promise<InDistro[]> => invoke("agents_wsl", { names });

export const agentsWslSet = (
  name: string,
  yes: { agents?: boolean; skills?: boolean },
  wake = false,
): Promise<InDistro> =>
  invoke("agents_wsl_set", {
    name,
    agents: yes.agents ?? null,
    skills: yes.skills ?? null,
    wake,
  });

export const agentsWslRound = (name: string, wake: boolean): Promise<InDistro> =>
  invoke("agents_wsl_round", { name, wake });
