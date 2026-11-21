export const MILESTONE_HOURS = [1, 5, 10, 25, 50, 100, 250, 500] as const;

export function milestoneTitle(hours: number): string {
  return hours === 1 ? "Première heure" : `${hours} heures de jeu`;
}
