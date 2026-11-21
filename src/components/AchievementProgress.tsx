import { cssVariable } from "../images";
import type { AchievementSummary } from "../types";

interface AchievementProgressProps {
  summary: AchievementSummary;
  onOpen: () => void;
}

export function AchievementProgress({ summary, onOpen }: AchievementProgressProps) {
  const ratio = summary.total ? summary.unlocked / summary.total : null;
  const label = summary.total
    ? `${summary.unlocked} / ${summary.total} succès`
    : `${summary.unlocked} succès débloqués`;

  return (
    <button type="button" className="achievement-progress glass" onClick={onOpen}>
      <span className="achievement-progress-label">{label}</span>
      {ratio !== null && (
        <span
          className="achievement-progress-bar"
          style={cssVariable("--progress", `${Math.round(ratio * 100)}%`)}
          aria-hidden="true"
        />
      )}
    </button>
  );
}
