interface UpdateBannerProps {
  version: string;
  onRestart: () => void;
}

export function UpdateBanner({ version, onRestart }: UpdateBannerProps) {
  return (
    <div className="update-banner glass" role="status">
      <span>La version {version} est prête !</span>
      <button type="button" className="launch-button compact" onClick={onRestart}>
        Redémarrer
      </button>
    </div>
  );
}
