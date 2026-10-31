#!/usr/bin/env bash
# Fails when an installer in the Tauri bundle directory is larger than its limit from the
# native-experience spec (Resource footprint): 25 MB, or 100 MB for an AppImage, which carries
# its own web engine and GTK libraries.
# Usage: check-installer-size.sh [limit MB] [AppImage limit MB] [bundle dir]
set -euo pipefail
limit_mb=${1:-25}
appimage_limit_mb=${2:-100}
bundle_dir=${3:-target/release/bundle}
status=0
found=0
while IFS= read -r -d '' file; do
  found=1
  size=$(wc -c < "$file" | tr -d ' ')
  # awk, not bc: Git Bash on Windows has no bc.
  awk -v size="$size" -v file="$file" 'BEGIN { printf "%8.1f MB  %s\n", size / 1048576, file }'
  case "$file" in
    *.AppImage) file_limit_mb=$appimage_limit_mb ;;
    *) file_limit_mb=$limit_mb ;;
  esac
  if [ "$size" -gt $((file_limit_mb * 1024 * 1024)) ]; then
    echo "  larger than ${file_limit_mb} MB" >&2
    status=1
  fi
done < <(find "$bundle_dir" -type f \( -name '*.dmg' -o -name '*.exe' -o -name '*.msi' -o -name '*.AppImage' -o -name '*.deb' \) -print0)
if [ "$found" -eq 0 ]; then
  echo "no installers found in $bundle_dir" >&2
  exit 1
fi
exit "$status"
