import type { FileChange } from '../model/timeline';

/** One change to one file, as unified diff hunks with line numbers. */
export function DiffView({ change, showPath = true }: { change: FileChange; showPath?: boolean }) {
  return (
    <div className="diff" aria-label={`${change.change === 'created' ? 'Created' : 'Changed'} ${change.path}`}>
      {showPath && (
        <div className="file">
          <span>{change.path}</span>
          <span className="kind">{change.change === 'created' ? 'new file' : 'edited'}</span>
        </div>
      )}
      {change.hunks.map((hunk, i) => {
        let oldLine = hunk.oldStart;
        let newLine = hunk.newStart;
        return (
          <div key={i}>
            {change.change === 'edited' && (
              <div className="hunk">
                @@ −{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
              </div>
            )}
            <div className="lines">
              {hunk.lines.map((line, j) => {
                const sign = line[0] ?? ' ';
                const kind = sign === '+' ? 'add' : sign === '-' ? 'del' : '';
                const left = sign === '+' ? '' : oldLine++;
                const right = sign === '-' ? '' : newLine++;
                return (
                  <div key={j} className={`line ${kind}`}>
                    <span className="n">{left}</span>
                    <span className="n">{right}</span>
                    <span className="sign">{sign === ' ' ? '' : sign}</span>
                    <span>{line.slice(1)}</span>
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}
