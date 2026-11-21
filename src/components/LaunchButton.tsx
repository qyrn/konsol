import { LAUNCH_LABELS, type LaunchStatus } from "../launchStatus";

interface LaunchButtonProps {
  status: LaunchStatus;
  installed?: boolean;
  onLaunch: () => void;
}

export function LaunchButton({ status, installed = true, onLaunch }: LaunchButtonProps) {
  if (!installed) {
    return (
      <button type="button" className="launch-button" onClick={onLaunch}>
        Installer
      </button>
    );
  }
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
