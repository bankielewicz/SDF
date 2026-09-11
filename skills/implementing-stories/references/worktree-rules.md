# Worktrees, commits, and parallel builds

Read this at workflow step 4. Git is reached through four calls and no other path: `devforgeai worktree ensure`, `devforgeai worktree list`, `devforgeai worktree remove`, and `devforgeai commit`. A commit made outside `devforgeai commit` would skip the declared-file-set check that runs before staging, which is why the run makes none.

## What `worktree ensure` creates

```
devforgeai worktree ensure <STORY-nnn>
```

The path is `<config.toml [build].worktree_root>/<STORY-nnn>`, resolved against the project root, default `../wt`. The branch is `<[build].branch_prefix><STORY-nnn>`, default prefix `story/`, cut from `[build].base_ref`, default `HEAD`. The command prints the path on stdout, and appends a `[[worktree]]` entry — `story`, `path`, `branch`, `created_at` — to the main checkout's `state.toml`. One entry per live worktree; the registry is what every later resolution reads.

The call is idempotent. A path that is already a worktree of this repository on the matching branch is printed and the command exits 0, which is what a `--remedy` and a `--resume` run rely on: neither creates a second worktree, and both keep every commit the earlier run made.

## What lives where, and which directory a command runs in

The worktree holds sources and tests: the paths the story's `## Files` table declares, and nothing else the run writes. Every `.devforgeai/` document, every report, and `state.toml` live in the main checkout and are read and written there.

The split decides one thing for the model: `sh` and the test command of step 6 run in the worktree, because that is where the code under test sits. A `devforgeai` command resolves the same state from either directory, so the run issues its CLI calls from wherever it stands and the gate's command checks reach the worktree on their own through its `[[worktree]]` entry.

`state.toml` exists once, in the main checkout, and the `[[worktree]]` registry is what keeps two concurrent builds apart inside it. The `PreToolUse` write-scope check resolves the story from the path being written: a path under a registered worktree takes that entry's `story`, and a path under neither falls back to `[active].build`. So each run's writes are tested against its own story's `## Files` set however `[active].build` last moved. `phase set build` run from inside a registered worktree sets `[active].build` to that entry's story, so the fallback names the run that is actually working.

## Parallel builds

Two `/build` runs in two worktrees under `[build].worktree_root` are the supported parallel shape. They share the repository and the main checkout's `.devforgeai/`, and are separated inside it by their `[[worktree]]` entries: separate branches, separate working trees, separate write scopes resolved per path, separate reports named after their own stories.

Two guards hold the shape, and each covers what the other cannot. The registry separates the runs once both exist, by resolving every write to the story whose worktree the path sits under. `DFA-E272` separates the file sets before the second worktree exists, by refusing a story whose `## Files` set intersects a live one's — which is what stops two runs from both being entitled to a path the registry would then happily grant each of them on its own branch.

What keeps them from colliding is that a path belongs to at most one of them. `devforgeai story validate --scope sprint` check 9 already forbids two stories of one sprint declaring a shared `Path`, raising `DFA-E237` at the plan gate. That check covers `stories[]` and excludes `deferred[]`, so an overlap can still survive it: a deferred story built anyway, a `## Files` table edited after the plan gate passed, or two sprints open at once.

`worktree ensure` closes those three. Before creating anything it reads `git worktree list`, takes the `STORY-nnn` encoded in each worktree directory name under `worktree_root`, runs the `story files --list` set for each, and intersects the requested story's set with them.

## The `DFA-E272` refusal

A non-empty intersection is `DFA-E272`, exit 1, naming both story ids and the first shared `Path`. The run stops there with one `Blocked` line carrying that text, and it writes nothing: no worktree, no `[[worktree]]` entry, no `phase set`, no file under the project.

What clears it is the other worktree finishing or going away. `devforgeai worktree list` prints one line per worktree — `<STORY-nnn>  <path>  <branch>  <clean|dirty>  <n> ahead` — which is how the other story is found. `devforgeai worktree remove <STORY-nnn>` removes it, refusing with `DFA-E273` when that worktree holds uncommitted changes or commits absent from `[build].base_ref` unless `--force` is passed, and deleting the branch when it is merged into `base_ref`.

Detection reads worktree directory names rather than a story's frontmatter `status`. The story file is a `.devforgeai/` document and lives in the main checkout alone, so both concurrent runs see one `status` value and it cannot tell them apart.

A worktree created outside `worktree ensure` bypasses the refusal. Both guards then allow the shared path, each run commits it on its own branch, and the collision surfaces as a merge conflict when the second branch merges. A cross-worktree lock is outside the primitives this framework builds on, so none is claimed.

## The other git failure

`DFA-E271`, exit 1, means the project root is not a git work tree. It reaches `worktree ensure` and `commit` alike. The run stops with one `Blocked` line naming `git init`, which is the user's call to make.

## What `devforgeai commit` does

```
devforgeai commit <STORY-nnn> -m "<message>" [--paths <path>,...]
```

`--paths` defaults to every path git reports as changed in the current work tree. Each path is tested by the `story files --check` rule before anything is staged; a path outside the story's `## Files` set is `DFA-E239`, exit 1, and nothing is staged. The message is committed as `<STORY-nnn>: <message>` when it does not already hold the story id, and verbatim when it does, which satisfies the `commit-msg` hook by construction.

The command stages and commits, so the `pre-commit` hook runs its `doc validate` over staged `.devforgeai/` files and its `context audit`. A non-zero hook exit becomes exit 1 with that hook's stderr, and nothing is committed; the run stops with one `Blocked` line carrying the stderr. An empty stage set is `DFA-W243`, exit 0, and nothing is committed.

## What a send-back leaves behind

The worktree stays on disk with every commit the run made. The story file keeps `status: building`, the main checkout's `state.toml` keeps `[current].phase` of `build`, `[active].build` of the story id, and the story's `[[worktree]]` entry, and the returning `/build STORY-nnn --resume` finds the branch where the stopped run left it.
