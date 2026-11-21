import { useEffect, useRef } from "react";

export interface GamepadActions {
  onHorizontal: (step: number) => void;
  onConfirm: () => void;
  onBack: () => void;
  onDetail: () => void;
  onTab: (step: number) => void;
  onSearch: () => void;
}

const BUTTON = { a: 0, b: 1, y: 3, lb: 4, rb: 5, start: 9, left: 14, right: 15 } as const;
const STICK_DEADZONE = 0.6;
const REPEAT_DELAY_MS = 380;
const REPEAT_EVERY_MS = 140;

export function useGamepad(actions: GamepadActions) {
  const latest = useRef(actions);
  latest.current = actions;

  useEffect(() => {
    let frame = 0;
    const pressed = new Set<number>();
    let heldDirection = 0;
    let nextRepeatAt = 0;

    function press(button: number, isDown: boolean, action: () => void) {
      if (isDown && !pressed.has(button)) {
        pressed.add(button);
        action();
      } else if (!isDown) {
        pressed.delete(button);
      }
    }

    function poll(time: number) {
      frame = requestAnimationFrame(poll);
      const pad = navigator.getGamepads?.().find((candidate) => candidate?.connected);
      if (!pad || !document.hasFocus()) return;
      const isDown = (index: number) => pad.buttons[index]?.pressed ?? false;
      const stick = pad.axes[0] ?? 0;
      const direction =
        isDown(BUTTON.left) || stick < -STICK_DEADZONE
          ? -1
          : isDown(BUTTON.right) || stick > STICK_DEADZONE
            ? 1
            : 0;
      if (direction !== heldDirection) {
        heldDirection = direction;
        if (direction !== 0) {
          latest.current.onHorizontal(direction);
          nextRepeatAt = time + REPEAT_DELAY_MS;
        }
      } else if (direction !== 0 && time >= nextRepeatAt) {
        latest.current.onHorizontal(direction);
        nextRepeatAt = time + REPEAT_EVERY_MS;
      }
      press(BUTTON.a, isDown(BUTTON.a), () => latest.current.onConfirm());
      press(BUTTON.b, isDown(BUTTON.b), () => latest.current.onBack());
      press(BUTTON.y, isDown(BUTTON.y), () => latest.current.onDetail());
      press(BUTTON.lb, isDown(BUTTON.lb), () => latest.current.onTab(-1));
      press(BUTTON.rb, isDown(BUTTON.rb), () => latest.current.onTab(1));
      press(BUTTON.start, isDown(BUTTON.start), () => latest.current.onSearch());
    }

    frame = requestAnimationFrame(poll);
    return () => cancelAnimationFrame(frame);
  }, []);
}
