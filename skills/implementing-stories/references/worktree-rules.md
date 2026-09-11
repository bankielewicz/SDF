# Worktrees, commits, and parallel builds

Read this at workflow step 4. Git is reached through four calls and no other path: `devforgeai worktree ensure`, `devforgeai worktree list`, `devforgeai worktree remove`, and `devforgeai commit`. A commit made outside `devforgeai commit` would skip the declared-file-set check that runs before staging, which is why the run makes none.

## What `worktree ensure` creates

```
devforgeai worktree ensure <STORY-nnn>
```

The path is `<config.toml [build].worktree_root>/<STORY-nnn>`, resolved against the project root, default `../wt`. The branch is `<[build].branch_prefix><STORY-nnn>`, default prefix `story/`, cut from `[build].base_ref`, default `HEAD`. The command prints the path on stdout, and that path is where the rest of the run works.

The call is idempotent. A path that is already a worktree of this repository on the matching branch is printed and the command exits 0, which is what a `--remedy` and a `--resume` run rely on: neither creates a second worktree, and both keep every commit the earlier run made.

`ensure` also copies the main checkout's `.devforgeai/state.toml` into the new worktree. `state.toml` is the one untracked path under `.devforgeai/`; every other path there is tracked and therefore already present in the checkout. Without the copy, a `phase set` inside a fresh worktree would start from the default phase.

## Why each worktree carries its own state

`--project` resolves to the worktree, because `.devforgeai/` is checked out there. Each worktree therefore keeps its own `[active].build`, and the `PreToolUse` `story files --check` hook resolves `--id` from that value. Two runs in two worktrees each guard against their own story's `## Files` set, and a write allowed in one is unaffected by the other. Two concurrent builds also produce no merge conflict on a state file, because the file is untracked in both.

## Parallel builds

Two `/build` runs in two worktrees under `[build].worktree_root` are the supported parallel shape. They share the repository and share nothing else: separate branches, separate working trees, separate `[active].build`, separate reports named after their own stories.

What keeps them from colliding is that a path belongs to at most one of them. `devforgeai story validate --scope sprint` check 9 already forbids two stories of one sprint declaring a shared `Path`, raising `DFA-E237` at the plan gate. That check covers `stories[]` and excludes `deferred[]`, so an overlap can still survive it: a deferred story built anyway, a `## Files` table edited after the plan gate passed, or two sprints open at once.

`worktree ensure` closes those three. Before creating anything it reads `git worktree list`, takes the `STORY-nnn` encoded in each worktree directory name under `worktree_root`, runs the `story files --list` set for each, and intersects the requested story's set with them.

## The `DFA-E272` refusal

A non-empty intersection is `DFA-E272`, exit 1, naming both story ids and the first shared `Path`. The run stops there with one `Blocked` line carrying that text, and it writes nothing: no worktree, no `phase set`, no file under the project.

What clears it is the other worktree finishing or going away. `devforgeai worktree list` prints one line per worktree — `<STORY-nnn>  <path>  <branch>  <clean|dirty>  <n> ahead` — which is how the other story is found. `devforgeai worktree remove <STORY-nnn>` removes it, refusing with `DFA-E273` when that worktree holds uncommitted changes or commits absent from `[build].base_ref` unless `--force` is passed, and deleting the branch when it is merged into `base_ref`.

Detection reads worktree directory names rather than a story's frontmatter `status`, because `phase set build` writes `building` into the worktree's copy of the story file while the main checkout keeps the value it had.

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

The worktree stays on disk with every commit the run made. The story file keeps `status: building`, `state.toml` keeps `[current].phase` of `build` and `[active].build` of the story id, and the returning `/build STORY-nnn --resume` finds the branch where the stopped run left it.
