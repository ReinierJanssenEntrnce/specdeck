#!/usr/bin/env bash
# Fails when an installer in the Tauri bundle directory is larger than the limit
# from the native-experience spec (Resource footprint).
set -euo pipefail
limit_mb=${1:-25}
bundle_dir=${2:-target/release/bundle}
limit=$((limit_mb * 1024 * 1024))
status=0
found=0
while IFS= read -r -d '' file; do
  found=1
  size=$(wc -c < "$file" | tr -d ' ')
  # awk, not bc: Git Bash on Windows has no bc.
  awk -v size="$size" -v file="$file" 'BEGIN { printf "%8.1f MB  %s\n", size / 1048576, file }'
  if [ "$size" -gt "$limit" ]; then
    echo "  larger than ${limit_mb} MB" >&2
    status=1
  fi
done < <(find "$bundle_dir" -type f \( -name '*.dmg' -o -name '*.exe' -o -name '*.msi' -o -name '*.AppImage' -o -name '*.deb' \) -print0)
if [ "$found" -eq 0 ]; then
  echo "no installers found in $bundle_dir" >&2
  exit 1
fi
exit "$status"
