/**
 * Svelte action that triggers a callback after a sustained press.
 * Uses Pointer Events API for cross-platform support.
 */

export interface LongpressOptions {
  /** Duration in ms before longpress fires (default: 500) */
  duration?: number;
  /** Movement threshold in px that cancels the longpress (default: 10) */
  threshold?: number;
  /** Callback to invoke on longpress */
  onLongpress?: (e: PointerEvent) => void;
}

export function longpress(node: HTMLElement, options: LongpressOptions = {}) {
  let duration = options.duration ?? 500;
  let threshold = options.threshold ?? 10;
  let onLongpress = options.onLongpress;

  let timer: ReturnType<typeof setTimeout> | null = null;
  let startX = 0;
  let startY = 0;
  let currentEvent: PointerEvent | null = null;

  function handlePointerDown(e: PointerEvent) {
    // Only handle primary button (left click / touch)
    if (e.button !== 0) return;

    startX = e.clientX;
    startY = e.clientY;
    currentEvent = e;

    timer = setTimeout(() => {
      if (onLongpress && currentEvent) {
        onLongpress(currentEvent);
      }
      timer = null;
      currentEvent = null;
    }, duration);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!timer) return;

    const dx = Math.abs(e.clientX - startX);
    const dy = Math.abs(e.clientY - startY);

    if (dx > threshold || dy > threshold) {
      clearTimeout(timer);
      timer = null;
      currentEvent = null;
    }
  }

  function handlePointerUp() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
      currentEvent = null;
    }
  }

  function handlePointerCancel() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
      currentEvent = null;
    }
  }

  node.addEventListener('pointerdown', handlePointerDown);
  node.addEventListener('pointermove', handlePointerMove);
  node.addEventListener('pointerup', handlePointerUp);
  node.addEventListener('pointercancel', handlePointerCancel);

  return {
    update(newOptions: LongpressOptions) {
      duration = newOptions.duration ?? 500;
      threshold = newOptions.threshold ?? 10;
      onLongpress = newOptions.onLongpress;
    },
    destroy() {
      if (timer) {
        clearTimeout(timer);
      }
      node.removeEventListener('pointerdown', handlePointerDown);
      node.removeEventListener('pointermove', handlePointerMove);
      node.removeEventListener('pointerup', handlePointerUp);
      node.removeEventListener('pointercancel', handlePointerCancel);
    },
  };
}
