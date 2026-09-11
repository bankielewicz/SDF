#!/bin/sh
# A trailing-whitespace and tab check over the declared source set. Enough for
# the gate's `lint_clean` kind to mean something without inventing a language.
status=0
for f in $(find src tests -name '*.ext' 2>/dev/null); do
  if grep -nq "[ 	]$" "$f"; then echo "$f: trailing whitespace"; status=1; fi
done
echo "lint: done"
exit $status
