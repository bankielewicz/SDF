# Brand mode

Read before workflow step 5. Brand mode turns the Explore brand sketch, the `PERSONA-nnn` records of `requirements.yaml`, and the answers to five fixed questions into `.devforgeai/brand/tokens.json`, `.devforgeai/brand/logo.svg`, and `.devforgeai/brand/brand-kit.md`.

## The five questions

`templates/brand-questions.md` holds the two `AskUserQuestion` calls verbatim — the question text, the headers, the option labels, and the option descriptions. Copy them; the fixed set is what makes the same five answers produce the same token set.

The split is four questions in call 1 (`Tone`, `Color`, `Type`, `Density`) and one in call 2 (`Logo`), because the tool takes at most four questions per call and the `Logo` answer depends on none of the first four.

Show the step 5 candidates — the candidate name, palette, and type pair — as one line of prose above the first call, so the user answers against what the sketch already suggested rather than in the abstract.

A free-text answer matching no label re-asks that one question once. A second unmatched answer takes the first option of that question and adds one `open_questions` line to `brand-kit.md` naming the header.

## The answer-to-leaf mapping

The table at the end of `templates/brand-questions.md` maps each label to the leaves it sets. `brand-designer` receives that table and applies it:

- `Tone` sets `radius.sm`, `radius.md`, `elevation.raised`, and `type.weight-bold`.
- `Color` sets the 12 colour leaves, with the hue ranges the label names.
- `Type` sets `family-sans` and `family-mono`, and for `Sans and serif` puts the serif face on `size-xl` and `size-xxl`.
- `Density` sets the six spacing leaves.
- `Logo` sets the `viewBox` content of `logo.svg`.

Leaves no answer names keep the value in `templates/tokens.json`.

## Token shape

Top-level keys in this order, with no other top-level key: `meta`, `color`, `type`, `spacing`, `radius`, `elevation`, `motion`. `meta` carries the seven document keys in document order. The other six are token groups.

A group's leaf names match `^[a-z][a-z0-9-]*$`, which is why the scales read `xxl` rather than `2xl`: a leading digit would sit where the CSS custom-property grammar reads a name. A leaf of `color` is an object with exactly the keys `light` and `dark`. A leaf of every other group is a string.

The complete leaf sets, and the counts `design lint --tokens` checks:

| Group | Count | Leaves |
|---|---|---|
| `color` | 12 | `bg`, `surface`, `surface-raised`, `border`, `text`, `text-muted`, `primary`, `on-primary`, `accent`, `success`, `warning`, `danger` |
| `type` | 12 | `family-sans`, `family-mono`, `size-xs`, `size-sm`, `size-base`, `size-lg`, `size-xl`, `size-xxl`, `line-tight`, `line-base`, `weight-regular`, `weight-bold` |
| `spacing` | 6 | `xs`, `sm`, `md`, `lg`, `xl`, `xxl` |
| `radius` | 4 | `none`, `sm`, `md`, `full` |
| `elevation` | 3 | `flat`, `raised`, `overlay` |
| `motion` | 4 | `duration-fast`, `duration-base`, `duration-slow`, `ease-standard` |

The flattened token name is `TOKEN-<group>-<leaf>` in every group — one name for a colour with two values — and the CSS custom property a frontend file references is `--<group>-<leaf>`. `TOKEN-<name>` is a token reference rather than an ID-index entry: the index pattern is `[A-Z]+-[0-9]{3}`, which a lowercase leaf name does not match, so `doc validate` neither defines nor resolves one and `consumes` holds none. `design lint --tokens` resolves them.

`templates/tokens.json` holds the default values. Step 7 replaces every value; the file is the shape, not the palette.

## The contrast floors

Each text leaf clears 4.5 against `TOKEN-color-bg` in the same theme, in both themes. The border leaf clears 3.0. `brand-designer` returns the measured ratios in `contrast[]`, one entry per token and theme.

A returned palette whose `text` on `bg` ratio falls below 4.5 in either theme comes back with the `text` value darkened or lightened until it clears, recorded in `adjustments[]` as `{token, from, to, reason}`. Write that adjustment into `brand-kit.md` `## Do not` as one sentence, so the reason the palette differs from the sketch stays with the brand rather than in a transcript.

`## Color` in `brand-kit.md` carries the ratio against `TOKEN-color-bg` to one decimal, the lower of the two themes.

## The logo

`.devforgeai/brand/logo.svg`: one `<svg>` element with a `viewBox` of `0 0 256 64`, `role="img"`, and one `<title>` holding the brand name. Every `fill` and `stroke` attribute is one of `currentColor`, `none`, or a hex value equal to a `color` leaf's `light` value. No `<image>`, no `<script>`, no external reference, no embedded font: text in the mark is drawn as `<path>` data, so the file renders the same wherever it is opened. Byte size at most 32768.

On `logo_written: false` with a `reason` string, write a one-line wordmark from `templates/logo.svg`, with the brand name in the `<title>` and the `TOKEN-color-text` light value as the fill, and continue to step 9.

## The brand kit

`templates/brand-kit.md` is the shape. Nine sections, in this order, with these headings verbatim:

| # | Heading | Content |
|---|---|---|
| 1 | `## Brand name` | One line, the name. Then one line, the one-sentence descriptor used under the logo. |
| 2 | `## Voice` | Table `Trait \| Sounds like \| Does not sound like`. Exactly 3 rows, one per trait chosen at step 6. |
| 3 | `## Logo` | Table `Field \| Value`, exactly the rows `Form`, `Path`, `Clear space` (a `TOKEN-spacing-<leaf>` name), `Smallest size` (a px value). |
| 4 | `## Color` | Table `Token \| Light \| Dark \| Used for \| Contrast on bg`. 12 rows, one per colour leaf, in `tokens.json` key order. |
| 5 | `## Type` | Table `Token \| Value \| Used for`. 12 rows, one per type leaf, in `tokens.json` key order. |
| 6 | `## Spacing and shape` | Table `Token \| Value \| Used for`. 10 rows, one per spacing and radius leaf. |
| 7 | `## Motion` | Table `Token \| Value \| Used for`. 4 rows. Then one line stating what `prefers-reduced-motion: reduce` changes. |
| 8 | `## Figma` | Table `Field \| Value`, exactly the rows `Status`, `Collection`, `Reason`. |
| 9 | `## Do not` | Unnumbered lines, 3 to 8, each one sentence in the present tense naming one thing the brand excludes. |

The `Smallest size` value in `## Logo` is a px measurement of the printed mark, and it is the one numeric literal in the file. `design lint` reads `tokens.json` and files matching `[frontend].globs`, and reads neither `brand-kit.md` nor a `UI-nnn.md` except under `--tokens`, so the value resolves to no token and needs none.

The frontmatter carries `id: TOKEN-001`, the same id as `tokens.json`: the kit is the prose face of the token file and shares its subject. `status` is `approved`, the only value; the file is written once per brand run and overwritten on the next one.

## The Figma mirror and its fallback

With the Figma plugin's tools present in the session and authenticated, the `figma:figma-use` and `figma:figma-generate-library` skills write the six groups as a Figma variable collection named `<brand name> tokens`, with a light mode and a dark mode carrying the two values of each colour leaf. `## Figma` then reads:

```
| Status | mirrored |
| Collection | <brand name> tokens |
| Reason | - |
```

The plugin reaches the session through MCP, which this framework does not require. With the tools absent, or with a call returning an authentication error, `## Figma` reads `Status: not mirrored` with `Reason` set to `figma plugin not present` or `figma plugin not authenticated` as the case fits. Every other output of step 9 is unchanged and the run continues to the handoff: no output of this skill depends on the Figma path.

The three Figma skills this skill invokes are `figma:figma-use` and `figma:figma-generate-library` at step 9, and `figma:figma-design-to-code` at spec-mode step 13, where the skill runs it in the main conversation and passes the result to `ui-spec-writer` as `figma_context`. The other Figma skills in the plugin are outside this skill's workflow.
