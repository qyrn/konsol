import type { AchievementSummary } from "../types";

interface AchievementProgressProps {
  summary: AchievementSummary;
  onOpen: () => void;
}

export function achievementCountLabel(summary: AchievementSummary): string {
  return summary.total ? `${summary.unlocked} / ${summary.total}` : `${summary.unlocked}`;
}

export function AchievementProgress({ summary, onOpen }: AchievementProgressProps) {
  const label = summary.total
    ? `${achievementCountLabel(summary)} succès`
    : `${summary.unlocked} succès débloqués`;

  return (
    <button type="button" className="glass-button achievement-pill" onClick={onOpen}>
      {label}
    </button>
  );
}
