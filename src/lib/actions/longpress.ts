/**
 * Svelte action that dispatches a 'longpress' event after a sustained press.
 * Uses Pointer Events API for cross-platform support.
 */

export interface LongpressOptions {
  /** Duration in ms before longpress fires (default: 500) */
  duration?: number;
  /** Movement threshold in px that cancels the longpress (default: 10) */
  threshold?: number;
}

export function longpress(node: HTMLElement, options: LongpressOptions = {}) {
  const duration = options.duration ?? 500;
  const threshold = options.threshold ?? 10;

  let timer: ReturnType<typeof setTimeout> | null = null;
  let startX = 0;
  let startY = 0;

  function handlePointerDown(e: PointerEvent) {
    // Only handle primary button (left click / touch)
    if (e.button !== 0) return;

    startX = e.clientX;
    startY = e.clientY;

    timer = setTimeout(() => {
      node.dispatchEvent(
        new CustomEvent('longpress', {
          detail: { clientX: e.clientX, clientY: e.clientY, originalEvent: e },
        })
      );
      timer = null;
    }, duration);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!timer) return;

    const dx = Math.abs(e.clientX - startX);
    const dy = Math.abs(e.clientY - startY);

    if (dx > threshold || dy > threshold) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function handlePointerUp() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function handlePointerCancel() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  node.addEventListener('pointerdown', handlePointerDown);
  node.addEventListener('pointermove', handlePointerMove);
  node.addEventListener('pointerup', handlePointerUp);
  node.addEventListener('pointercancel', handlePointerCancel);

  return {
    update(newOptions: LongpressOptions) {
      // Options are captured at pointerdown, so no runtime update needed
      // But we could extend this if needed
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
