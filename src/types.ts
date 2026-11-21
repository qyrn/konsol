export type GameSource = "steam" | "gog" | "epic" | "amazon" | "script" | "manual";

export type ImageSource = { kind: "file"; value: string } | { kind: "url"; value: string };

export interface Game {
  id: string;
  title: string;
  source: GameSource;
  cover: ImageSource | null;
  hero: ImageSource | null;
  accent: string | null;
  hidden: boolean;
}
