import { LAUNCH_LABELS, type LaunchStatus } from "../launchStatus";

interface LaunchButtonProps {
  status: LaunchStatus;
  onLaunch: () => void;
}

export function LaunchButton({ status, onLaunch }: LaunchButtonProps) {
  return (
    <button
      type="button"
      className="launch-button"
      data-status={status}
      disabled={status !== "idle"}
      onClick={onLaunch}
    >
      {LAUNCH_LABELS[status]}
    </button>
  );
}
