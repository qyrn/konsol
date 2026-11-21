export type LaunchStatus = "idle" | "launching" | "running";

export const LAUNCH_LABELS: Record<LaunchStatus, string> = {
  idle: "Lancer",
  launching: "Ça se lance…",
  running: "En jeu",
};
