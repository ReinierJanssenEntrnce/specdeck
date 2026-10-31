import { useRef, type KeyboardEvent, type ReactNode } from 'react';

export interface Tab {
  id: string;
  label: string;
  badge?: number;
  content: ReactNode;
}

/** Tabs beside the terminal: timeline, files and people. Arrow keys move between tabs. */
export function SidePanel({ tabs, selected, onSelect }: { tabs: Tab[]; selected: string; onSelect: (id: string) => void }) {
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const current = tabs.find((tab) => tab.id === selected) ?? tabs[0]!;
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
    const index = tabs.findIndex((tab) => tab.id === current.id);
    const next = tabs[(index + (event.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length]!;
    onSelect(next.id);
    refs.current[next.id]?.focus();
  };
  return (
    <aside className="side" aria-label="Session details">
      <div className="tabs" role="tablist" onKeyDown={onKeyDown}>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            ref={(element) => {
              refs.current[tab.id] = element;
            }}
            role="tab"
            id={`tab-${tab.id}`}
            className="tab"
            aria-selected={tab.id === current.id}
            aria-controls={`panel-${tab.id}`}
            tabIndex={tab.id === current.id ? 0 : -1}
            onClick={() => onSelect(tab.id)}
          >
            {tab.label}
            {tab.badge !== undefined && tab.badge > 0 && <span className="badge">{tab.badge}</span>}
          </button>
        ))}
      </div>
      <div className="panel" role="tabpanel" id={`panel-${current.id}`} aria-labelledby={`tab-${current.id}`}>
        {current.content}
      </div>
    </aside>
  );
}
