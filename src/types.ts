export type GameSource = "steam" | "gog" | "epic" | "amazon" | "script" | "manual";

export type Cover = { kind: "file"; value: string } | { kind: "url"; value: string };

export interface Game {
  id: string;
  title: string;
  source: GameSource;
  cover: Cover | null;
  hidden: boolean;
}
