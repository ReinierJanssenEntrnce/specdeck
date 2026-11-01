import { useState } from 'react';
import { ConfirmDialog } from './ConfirmDialog';

interface Props {
  /** The host name of the join link's sharing service. */
  serverName: string;
  /** Whether to offer keeping the service; not when the environment decides it. */
  canKeep: boolean;
  onJoin: (keep: boolean) => void;
  onCancel: () => void;
}

/**
 * Asks before joining through a sharing service other than the user's own
 * (`configurable-sharing-service` design decision 5): that service sees the viewer's network
 * address, though it cannot read the session.
 */
export function JoinThroughDialog({ serverName, canKeep, onJoin, onCancel }: Props) {
  const [keep, setKeep] = useState(false);
  return (
    <ConfirmDialog title={`Join through ${serverName}?`} confirm="Join" onConfirm={() => onJoin(keep)} onCancel={onCancel}>
      <p>
        This session uses a different sharing service than yours. It cannot read the session, but it sees your network
        address. Join only if you trust the link.
      </p>
      {canKeep && (
        <label className="check">
          <input type="checkbox" checked={keep} onChange={(event) => setKeep(event.target.checked)} />
          Use this service from now on
        </label>
      )}
    </ConfirmDialog>
  );
}
