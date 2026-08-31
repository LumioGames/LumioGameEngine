# Retained Worktree Evidence

This directory preserves the only non-noise uncommitted artifacts found in
review worktrees before closeout cleanup on 2026-08-31. The copies are evidence
only; none is an accepted implementation input.

## Sources

| Source worktree | Source HEAD | Copied paths | Disposition |
|---|---|---|---|
| `gas-a2-review-snapshot` | `a4c1c57` (detached) | `.review-generated/` and `review-report.tmp.md` | review snapshot; not accepted |
| `spec-lint-containment-fix` | `0e5ea2f` | `.sdd/spec-lint-containment-fix-report.md` | review evidence; candidate not accepted |
| `w0-architecture-review` | `753920e` | `.sdd/w0-task-review.md` | rejected W0 candidate evidence |
| `w0-byte-authority-fix` | `e1705e9` | `.sdd/review-w0-byte-authority-uncommitted.diff` | uncommitted review evidence |
| `w0-byte-authority-review` | `b7db298` | `.sdd/w0-byte-authority-review.md`, `.gitattributes` | unaccepted review overlay |

The R-00316 dirty worktree (`f317b92`, detached) is not copied: its 42 tracked
changes and nine untracked fixtures were compared byte-for-byte with the
published `512da15` commit and contained no additional implementation. That
comparison is recorded in the closeout report.

## Integrity

There are 76 files totalling 251,552 bytes. Verify the retained files with:

```text
Get-ChildItem .sdd/retained-worktrees -Recurse -File |
  Get-FileHash -Algorithm SHA256
```

The closeout report records the source commits, file counts, and cleanup
decision. The copied generated tree is deliberately kept under `.sdd` and is
not part of any package or public contract.
