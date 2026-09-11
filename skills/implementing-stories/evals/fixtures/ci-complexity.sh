#!/bin/sh
# One number per source file: its line count stands in for complexity, so the
# gate's `complexity_max` has something real to compare against.
for f in $(find src -name '*.ext' 2>/dev/null); do
  printf "%s %s
" "$f" "$(wc -l < "$f" | tr -d ' ')"
done
echo "complexity: done"
exit 0
