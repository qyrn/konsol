import type { ReactNode } from "react";

interface ModalProps {
  labelledBy: string;
  className: string;
  placement?: "center" | "top";
  onClose: () => void;
  children: ReactNode;
}

export function Modal({ labelledBy, className, placement = "center", onClose, children }: ModalProps) {
  return (
    <div className="modal-layer" data-placement={placement} onClick={onClose}>
      <div
        className={`modal-panel ${className}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby={labelledBy}
        onClick={(event) => event.stopPropagation()}
      >
        {children}
      </div>
    </div>
  );
}
