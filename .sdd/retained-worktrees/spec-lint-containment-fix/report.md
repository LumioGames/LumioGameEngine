# Spec-lint Containment Fix Report

## Scope

Commit: `0e5ea2f621b8943e539377f19d2ef31b49e9e359` (`fix(spec-lint): accept Windows case-variant links`)

Changed files are limited to `.spec/tools/spec-lint.mjs` and
`.spec/tools/spec-lint.test.mjs`. The report is under the ignored `.sdd/`
directory and is not part of the commit.

## TDD Evidence

### RED

Before the implementation change, the new regression failed on this Windows
runtime:

```text
$ node --test --test-name-pattern="case-variant" .spec/tools/spec-lint.test.mjs
exit=1
spec-lint: 1 处不一致

  ✗ .claude\agents: 软链接未解析进 .spec/(实际指向 C:\Users\g923\AppData\Local\Temp\spec-lint-fixture-o7t8AP\.SPEC\agents)
✖ case-variant link target follows platform case semantics (59.3554ms)
ℹ tests 1
ℹ suites 0
ℹ pass 0
ℹ fail 1
```

The failure was expected: the Windows filesystem resolves `.SPEC` to the
existing `.spec` directory, while the old comparison required exact casing.

### GREEN

The focused regression passed after the change:

```text
$ node --test --test-name-pattern="case-variant" .spec/tools/spec-lint.test.mjs
exit=0
✔ case-variant link target follows platform case semantics (65.0652ms)
ℹ tests 1
ℹ suites 0
ℹ pass 1
ℹ fail 0
```

The retained sibling-prefix guard also passes:

```text
$ node --test --test-name-pattern="sibling-prefix" .spec/tools/spec-lint.test.mjs
exit=0
spec-lint: 1 处不一致

  ✗ .claude\agents: 软链接未解析进 .spec/(实际指向 C:\Users\g923\AppData\Local\Temp\spec-lint-fixture-yfBYz4\.spec-evil\agents)
✔ sibling-prefix link target is rejected (58.1356ms)
ℹ tests 1
ℹ pass 1
ℹ fail 0
```

The complete suite passed with the expected 15 tests:

```text
$ node --test .spec/tools/spec-lint.test.mjs
exit=0
ℹ tests 15
ℹ pass 15
ℹ fail 0
```

## Implementation

`normalizeContainmentPath` lowercases only when `process.platform === 'win32'`.
The containment check compares normalized paths for exact `.spec` equality or a
descendant beginning with the normalized spec path plus the platform separator;
the original resolved path remains in the existing error message and the catch
behavior is unchanged. POSIX comparisons remain case-sensitive, so a distinct
`.SPEC` directory is rejected; the separator boundary continues to reject
`.spec-evil` and dot-traversal targets resolved outside the tree. The same
component-boundary logic applies to drive and UNC paths after Windows case
normalization.

## Checkout and Gate Evidence

The changed script was run against a real-junction checkout:

```text
$ node .spec/tools/spec-lint.mjs C:\Work\LumioGames\LumioGameEngineArchitecture
exit=0
spec-lint: OK
```

This worktree itself is a known environment gap, not a pass signal:

```text
$ git config --get core.symlinks
false
$ node .spec/tools/spec-lint.mjs
exit=1
spec-lint: 3 处不一致

  ✗ .claude\agents: 软链接未解析进 .spec/(实际指向 C:\Users\g923\orca\workspaces\LumioGameEngineArchitecture\spec-lint-containment-fix\.claude\agents)
  ✗ .claude\skills: 软链接未解析进 .spec/(实际指向 C:\Users\g923\orca\workspaces\LumioGameEngineArchitecture\spec-lint-containment-fix\.claude\skills)
  ✗ .agents\skills: 软链接未解析进 .spec/(实际指向 C:\Users\g923\orca\workspaces\LumioGameEngineArchitecture\spec-lint-containment-fix\.agents\skills)
```

The three tracked link entries are plain files in that `core.symlinks=false`
projection. WSL is unavailable on this host: `wsl.exe --status` exits nonzero
and prints the Windows prompt to install WSL (`wsl.exe --install`), so no POSIX
runtime pass is claimed.

The cached diff contained exactly the two target files (21 insertions, 1
deletion); `git diff --cached --ignore-space-at-eol` showed only the normalizer,
comparison, and one regression test, and `git diff --cached --check` exited 0.
After commit, `git status --porcelain=v2 --branch` showed only the branch header
and no worktree or index entries.

## Known Gaps

- No WSL/POSIX runtime was available; the POSIX branch is covered by the
  platform-conditional regression and static separator/case review, not a WSL
  execution claim.
- The plain-file `core.symlinks=false` checkout cannot validate link health;
  the real-junction checkout above is the meaningful linter run.
