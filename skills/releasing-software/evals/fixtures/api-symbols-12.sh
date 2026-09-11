#!/bin/sh
set -eu

printf 'fn\topen_project\tsrc/project.ext\n'
printf 'fn\tload_config\tsrc/config.ext\n'
printf 'type\tGate\tsrc/gate.ext\n'
printf 'type\tGateResult\tsrc/gate.ext\n'
printf 'fn\tcheck_gate\tsrc/gate.ext\n'
printf 'fn\trender_handoff\tsrc/handoff.ext\n'
printf 'type\tReport\tsrc/report.ext\n'
printf 'type\tReportKind\tsrc/report.ext\n'
printf 'fn\tingest\tsrc/report.ext\n'
printf 'fn\tallocate_id\tsrc/ids.ext\n'
printf 'fn\tvalidate_doc\tsrc/doc.ext\n'
printf 'fn\tresolve_platform\tsrc/release.ext\n'
