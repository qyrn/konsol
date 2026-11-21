import { type FormEvent, useState } from "react";
import { CloseButton } from "./CloseButton";
import { Modal } from "./Modal";

interface TextPromptProps {
  title: string;
  help: string;
  label: string;
  submitLabel: string;
  inputMode?: "text" | "decimal";
  onSubmit: (value: string) => void;
  onCancel: () => void;
}

export function TextPrompt({
  title,
  help,
  label,
  submitLabel,
  inputMode = "text",
  onSubmit,
  onCancel,
}: TextPromptProps) {
  const [value, setValue] = useState("");

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (value.trim()) onSubmit(value.trim());
  }

  return (
    <Modal labelledBy="text-prompt-title" className="form-panel form-panel-narrow" onClose={onCancel}>
      <form onSubmit={handleSubmit}>
        <header className="form-panel-header">
          <div>
            <h2 id="text-prompt-title">{title}</h2>
            <p>{help}</p>
          </div>
          <CloseButton onClose={onCancel} />
        </header>
        <div className="form-fields">
          <label>
            {label}
            <input
              value={value}
              inputMode={inputMode}
              maxLength={40}
              autoFocus
              required
              onChange={(event) => setValue(event.target.value)}
            />
          </label>
        </div>
        <div className="form-actions">
          <button type="button" className="glass-button" onClick={onCancel}>
            Annuler
          </button>
          <button type="submit" className="launch-button compact">
            {submitLabel}
          </button>
        </div>
      </form>
    </Modal>
  );
}
