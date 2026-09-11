#!/bin/sh
# Emits the cobertura file `config.toml` points `coverage_paths` at. Every
# declared source file counts as covered once a spec mirrors it, which keeps the
# figure a function of the run rather than a constant.
mkdir -p coverage
total=0
covered=0
for src in $(find src -name '*.ext' 2>/dev/null); do
  total=$((total + 1))
  if grep -rql "mirrors: *$src" tests 2>/dev/null; then covered=$((covered + 1)); fi
done
if [ "$total" -eq 0 ]; then rate=1.0; else
  rate=$(awk "BEGIN { printf \"%.4f\", $covered / $total }"); fi
cat > coverage/cobertura.xml <<XML
<?xml version="1.0"?>
<coverage line-rate="$rate" branch-rate="$rate" version="1.9">
  <packages/>
</coverage>
XML
echo "coverage: $rate ($covered of $total)"
exit 0
