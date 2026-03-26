# AGT-60: makefile better descriptive cmds

## Research

### Current Makefile targets

| Target     | What it does                                                                   |
| ---------- | ------------------------------------------------------------------------------ |
| `dev`      | Starts Rust API server + Vite dev server                                       |
| `build`    | Builds web assets, then Rust binary (release mode)                             |
| `web`      | Builds Vue frontend only                                                       |
| `test`     | Runs Rust tests (`cargo test`)                                                 |
| `lint`     | Runs oxlint + oxfmt on web source                                              |
| `install`  | Quick release build + copies binary to `~/.local/bin` (no web assets)          |
| `deploy`   | Full release build (web + Rust) + installs binary + web assets to `~/.local/`  |
| `undeploy` | Removes `agt` binary and web data from `~/.local/`                             |

### The problem

"deploy" implies pushing to a remote server or production environment. Here it just means "build everything and copy to `~/.local/bin`" — a local install. Similarly "undeploy" just means "uninstall." The naming is misleading.

There's also an overlap: both `install` and `deploy` exist. `install` is a quick build without web assets; `deploy` is a full build with web assets. The distinction is useful but the names don't communicate it.

### References to update

- `CLAUDE.md` line 49: `agt` after `make deploy`.
- `CLAUDE.md` line 133: `Makefile  # make deploy builds standalone binary`

### Conventions in similar projects

- Homebrew formulae and most CLI tools use `make install` / `make uninstall` for putting a binary on your PATH.
- `deploy` typically means pushing to a remote (server, cloud, registry).
- Some projects use `make release` for "build in release mode."

## Approach Options

### Option 1: Rename `deploy` -> `install`, drop old `install`

- `install` = full build + install (what `deploy` does now)
- Remove the current `install` target entirely
- `uninstall` replaces `undeploy`
- Pros: Standard naming, simple, one obvious way to install
- Cons: Loses the "quick build without web" shortcut

### Option 2: Rename `deploy` -> `install`, rename old `install` -> `install-fast`

- `install` = full build + install with web assets
- `install-fast` = quick build without web assets (for iteration)
- `uninstall` replaces `undeploy`
- Pros: Standard naming, preserves the fast-iteration shortcut
- Cons: Two install targets could still be confusing

### Option 3: Rename `deploy` -> `install`, rename old `install` -> `quick`

- `install` = full build + install
- `quick` = quick release build, no web assets
- `uninstall` replaces `undeploy`
- Pros: Clear distinction, `make quick` is fast to type
- Cons: `quick` is vague — quick what?

### Option 4: Keep both, just rename deploy/undeploy

- `install` (existing) stays as-is (quick build, no web)
- `install-full` or `install-all` = what `deploy` does now
- `uninstall` replaces `undeploy`
- Pros: Preserves existing behavior, no ambiguity about scope
- Cons: `install-full` is longer to type than `deploy`

## Recommendation

**Option 2** is the best balance:

- `deploy` -> `install` (standard name for "put on my system")
- `install` -> `install-fast` (communicates it's a quicker variant)
- `undeploy` -> `uninstall`

This follows Unix/Makefile conventions where `make install` is the standard way to install a locally-built tool. The `-fast` suffix clearly signals the tradeoff (skips web assets for faster iteration).

### Implementation steps

1. Rename targets in `Makefile` and update `.PHONY` line
2. Update `CLAUDE.md` references (lines 49 and 133): `make deploy` -> `make install`
3. Optionally update Makefile comment descriptions for clarity

## Questions

- Do you still use `make install` (the quick, no-web-assets variant) regularly? If not, we could just drop it and keep things simpler (Option 1).
- Any preference on the name for the quick variant? (`install-fast`, `install-quick`, `quick`, etc.)

## Answers

- I use make install a lot ot quick install. make 'make' by itself make install, the cli installation. let's keep deploy. uninstall is too many letters 2 type. make install should just become make then.