#!/bin/sh
# The stand-in API symbol lister `config.toml` names as `api_symbols_command`.
#
# One `<kind>	<symbol>	<path>` line per exported symbol, which is the shape
# the release gate's `release-docs-cover` check parses. Three symbols, not
# twelve: the check compares them against the H3 headings the run writes on the
# API page, so the set has to be small enough that a happy-path run documents
# all of it. rl-08 is the case that measures a twelve-symbol sweep.
printf 'fn	open_project	src/project.ext
'
printf 'fn	load_config	src/config.ext
'
printf 'type	Gate	src/gate.ext
'
