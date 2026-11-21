export function CloseButton({ onClose }: { onClose: () => void }) {
  return (
    <button type="button" className="close-button" aria-label="Fermer" onClick={onClose}>
      ×
    </button>
  );
}
