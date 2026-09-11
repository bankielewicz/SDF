#!/bin/sh
# The eval harness's stand-in test runner.
#
# `config.toml` names this script as `test_command`, and Build's red-green loop
# turns on its exit code, so a fixture that names a command the workspace does
# not hold stops the run at step 7.2. It is deliberately language-agnostic, like
# the rest of the framework: a spec file under `tests/` passes when the source
# file it names on its `# mirrors:` line exists and is not empty, and fails
# otherwise. That is red before the implementation and green after it, which is
# the only property the loop reads.
status=0
count=0
for spec in $(find tests -name '*_spec.ext' -o -name '*_test.ext' 2>/dev/null); do
  count=$((count + 1))
  target=$(sed -n 's/^# *mirrors: *//p' "$spec" | head -1)
  if [ -z "$target" ]; then
    echo "FAIL $spec: no '# mirrors:' line naming the source under test"
    status=1
  elif [ ! -s "$target" ]; then
    echo "FAIL $spec: $target is absent or empty"
    status=1
  else
    echo "PASS $spec"
  fi
done
echo "tests: $count"
exit $status
