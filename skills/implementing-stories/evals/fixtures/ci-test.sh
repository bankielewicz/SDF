#!/bin/sh
# The eval harness's stand-in test runner.
#
# `config.toml` names this script as `test_command`, and Build's red-green loop
# turns on its exit code, so a fixture that names a command the workspace does
# not hold stops the run at step 7.2. It is language-agnostic, like the rest of
# the framework: nothing here names a runner, an assertion library, or a file
# extension beyond the `.ext` the fixtures use.
#
# Per criterion, not per file. An earlier version passed a spec whenever the
# source file it mirrors was non-empty, which made every criterion after the
# first pass before its implementation existed: step 7.2 reads a zero exit as
# "the test passed before any implementation", so the loop stopped at AC-002 and
# the run sent the story back to Plan. The story is not at fault and neither is
# the skill — `references/test-shapes.md` puts sibling criteria in one declared
# test file on purpose ("State the criterion shares with a sibling criterion is
# built by each test for itself"). The proxy was.
#
# The rule now: a spec block passes when every literal its `expect` lines fix
# appears in the source the file mirrors. AC-001 fixing 201 and AC-002 fixing
# 422 therefore go red independently, in one file, and each goes green only when
# its own outcome is implemented.

status=0
specs=0
cases=0

for spec in $(find tests -name '*_spec.ext' -o -name '*_test.ext' 2>/dev/null); do
  specs=$((specs + 1))
  target=$(sed -n 's/^# *mirrors: *//p' "$spec" | head -1)
  if [ -z "$target" ]; then
    echo "FAIL $spec: no '# mirrors:' line naming the source under test"
    status=1
    continue
  fi
  if [ ! -s "$target" ]; then
    echo "FAIL $spec: $target is absent or empty"
    status=1
    continue
  fi

  # Every block opened by a line holding `AC-nnn`, up to the next such line.
  # `awk` prints "<ac>\t<literal>" for each `expect ... == <literal>`.
  awk '
    /AC-[0-9][0-9][0-9]/ {
      if (match($0, /AC-[0-9][0-9][0-9]/)) ac = substr($0, RSTART, RLENGTH)
    }
    /==/ {
      line = $0
      sub(/.*==[ \t]*/, "", line)
      sub(/[ \t]*[});]*[ \t]*$/, "", line)
      gsub(/"/, "", line)
      if (ac != "" && line != "") printf "%s\t%s\n", ac, line
    }
  ' "$spec" > .dfa-expect.tmp

  if [ ! -s .dfa-expect.tmp ]; then
    # No readable assertion: the spec exists but fixes no value, which is the
    # `then_names_no_readable_outcome` shape rather than a pass.
    echo "FAIL $spec: no 'expect <something> == <value>' line to read"
    status=1
    rm -f .dfa-expect.tmp
    continue
  fi

  while IFS='	' read -r ac literal; do
    [ -z "$literal" ] && continue
    cases=$((cases + 1))
    if grep -qF -- "$literal" "$target"; then
      echo "PASS $spec $ac: $target holds $literal"
    else
      echo "FAIL $spec $ac: $target does not hold $literal"
      status=1
    fi
  done < .dfa-expect.tmp
  rm -f .dfa-expect.tmp
done

echo "specs: $specs  assertions: $cases"
exit $status
