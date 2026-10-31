import { useEffect, useState } from 'react';

const KEY = 'specdeck.sideWidth';

function stored(): number {
  try {
    const width = Number(localStorage.getItem(KEY));
    return width >= 260 && width <= 900 ? width : 380;
  } catch {
    return 380;
  }
}

/** The side panel's width, draggable and remembered. */
export function useSideWidth(): [number, (width: number) => void] {
  const [width, setWidth] = useState(stored);
  useEffect(() => {
    try {
      localStorage.setItem(KEY, String(width));
    } catch {
      // Not remembering the width is fine.
    }
  }, [width]);
  return [width, (w) => setWidth(Math.min(900, Math.max(260, w)))];
}

export function Splitter({ width, onWidth }: { width: number; onWidth: (width: number) => void }) {
  const [dragging, setDragging] = useState(false);
  return (
    <div
      className={`splitter${dragging ? ' dragging' : ''}`}
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize the side panel"
      aria-valuenow={width}
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === 'ArrowLeft') onWidth(width + 20);
        if (event.key === 'ArrowRight') onWidth(width - 20);
      }}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        setDragging(true);
      }}
      onPointerMove={(event) => dragging && onWidth(window.innerWidth - event.clientX)}
      onPointerUp={() => setDragging(false)}
    />
  );
}
