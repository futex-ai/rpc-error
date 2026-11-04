# Accounting guide describes an uncommitted workspace

- **Severity:** Low. **Effort:** Small. **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-04, from the branch `conductor/agents-md-rpc-error`.
- **Location:** `docs/integration/accounting.md:3`,
  `docs/integration/accounting.md:4`, `docs/integration/accounting.md:10`,
  `docs/integration/accounting.md:22`, `docs/integration/accounting.md:30`,
  `docs/integration/accounting.md:60`, and
  `docs/integration/accounting.md:112`
- **Reason:** The fix rewrites a guide, and the branch that found the problem
  may change only the agent rules and the files that follow from them.

- **Terms:**
  - **Accounting:** The first downstream service of `rpc-error`, with its
    API in the `bkf-api` crate of the private repository
    `futex-ai/accounting`.
  - **Pin:** A git dependency with a fixed `rev`, the commit that a
    downstream service builds against.
- **Background:**
  - The guide `docs/integration/accounting.md` tells Accounting how to adopt
    `rpc-error`. The root `README.md`, `docs/README.md`, and
    `docs/integration/README.md` link to it.
  - Line 3 names the local path `/Users/calummoore/projects/futex/accounting`.
  - Line 4 says that `bkf-api` "currently uses Axum and renders manual
    `(StatusCode, reason, message)` tuples".
  - Line 10 says "After this repository is committed, Accounting should pin
    the exact revision".
  - Lines 22 to 26 say "This plan cannot validate a final git `rev` from
    inside the uncommitted workspace".
  - The same paragraph says that the example
    `crates/rpc-error/examples/accounting_axum.rs` compiles with the
    features `derive` and `axum`. The example still compiles with them.
  - The paragraph ends with the check after the commit: replace the local
    path with the commit hash, then run the Accounting API tests.
  - Line 30 calls the old Accounting envelope "the current shape".
  - The section "Targeted Tests", at lines 102 to 115, lists the Accounting
    API test commands.
  - The workspace was committed and merged to `main` in PR #1 on
    2026-06-22.
  - The plan that line 22 calls "This plan" was
    `plans/rpc-error-workspace-implementation.md`. The branch
    `conductor/agents-md-rpc-error` deletes that plan, with the user's
    approval of 2026-11-04.
  - Accounting adopted the `rpc-error` envelope in its PR #287, merged on
    2026-06-23.
  - On 2026-11-04 the workspace `Cargo.toml` of Accounting pins `rpc-error`
    at `ecf4e367342138911b9f45dbb97f73c9534c3136`. That commit is the merge
    commit of PR #1. The `bkf-api` crate enables the features `derive` and
    `axum`.
  - Line 60 and lines 112 to 115 also still treat the migration as future
    work.
- **Problem:** Line 10 and lines 22 to 26 describe the state before the merge
  of 2026-06-22, and "This plan" names a deleted file. Line 3 names a path
  on one developer's computer. Lines 4 and 30 call the old Accounting
  envelope current, but Accounting adopted the `rpc-error` envelope on
  2026-06-23.
- **Impact if no action:** A reader may think that Accounting still has to
  adopt `rpc-error` and that no commit exists to pin. The local path and the
  deleted plan do not help any reader.
- **Root question:** How does the guide give current guidance now that
  Accounting uses `rpc-error` at a pinned commit?
- **Options:**
  - **A.** Rewrite the stale passages as current guidance. Tell Accounting
    to pin a commit of `main` and to run its API tests after each pin
    change. Say that the example compiles with `derive` and `axum`. Replace
    the local path with the repository name `futex-ai/accounting`. Its risk:
    the guide can go stale again when Accounting changes. Effort: Small.
  - **B.** Delete the guide and its three links, because Accounting adopted
    `rpc-error` on 2026-06-23. Its cost: it deletes a doc on `main`, which
    needs the user's approval. No guide then stays for a later pin change.
    Effort: Small.
  - **C.** Turn the guide into a general guide for any downstream service,
    with Accounting as the finished example. Its cost: a rewrite of the
    whole guide. Effort: Medium.
- **Left out:**
  - Change nothing. The guide keeps a state that ended on 2026-06-22, so
    this fails the goal.
  - Move the guide into the `futex-ai/accounting` repository. Option B gives
    this repository the same result without work in another repository.
- **Wider fix:** None. No other doc under `docs/` or in a README names a plan
  or the uncommitted workspace. The related item below covers the other
  local paths.
- **Previous examples:** Commit `38c115b` removed plan references from the
  READMEs on 2026-11-01, after the user decided that a README holds no plan
  reference.
- **Related:** `plans/follow-ups/protocol-doc-old-review-step.md`. Both items
  replace the local path of Accounting, so one branch can fix both. That
  item may also move Accounting detail from the protocol doc into this
  guide.
- **Recommendation:** A. It makes the guide current with the smallest change
  and keeps the steps for a later pin change.
- **Action:** fix after merge, apply A, because the guide is outside the
  files that the agent-rules branch may change.
