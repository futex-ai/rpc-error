# Protocol doc still requires the old review step

- **Severity:** Low. **Effort:** Medium. **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-04, from the branch `conductor/agents-md-rpc-error`.
- **Location:** `docs/protocol/rpc-error/README.md:252`
- **Reason:** The fix also splits the 253-line doc and moves its code names,
  so it is larger than the agent-rules change that found it.

- **Terms:**
  - **Protocol doc:** A spec file under `docs/protocol/` that holds the
    requirements and the decisions in force for one area.
  - **Shared skills:** The rule files of the `futex-ai/skills` repository,
    one for each kind of work, such as `documentation`.
  - **`cargo xtask review`:** A command of the task runner under `xtask/`
    that asks the `codex` CLI for a read-only AI review of the local diff.
  - **`implementation-review`:** The shared skill for the review after a
    push, with a fix rule that says which findings the agent fixes itself.
- **Background:**
  - The protocol doc `docs/protocol/rpc-error/README.md` has 253 lines.
  - Its last sentence, at lines 252 and 253, says "Before implementation is
    considered complete, run tests, then `cargo xtask check`, then
    `cargo xtask review`."
  - On 2026-11-04 the user chose the shared skills over the rules of the
    old `AGENTS.md`. The agent-rules branch
    `conductor/agents-md-rpc-error` applied that choice.
  - Since that decision, the review after the push follows the
    `implementation-review` skill and its fix rule.
  - `cargo xtask review` stays only as an optional manual command. The
    section "Developer Get Started" of the root `README.md` says so.
  - The `documentation` skill says that a protocol doc holds requirements
    and the decisions in force, not process steps, file names, or
    mechanics.
  - The same skill says that a change that touches a doc over about 250
    lines splits the doc into linked pages.
  - It also says that a change that touches a protocol doc moves its code
    names and mechanics to their homes in the same change.
  - The section "Workspace Shape", at lines 22 to 39, names
    `crates/rpc-error`, `crates/rpc-error-derive`, and `xtask`. The section
    "Key Code" of the root `README.md` already names all three.
  - The section "Downstream Compatibility", at lines 224 to 235, names three
    local paths under `/Users/calummoore/projects/`. The first path points
    to the Accounting service, whose repository is `futex-ai/accounting`.
    The other two point to the packages `pkg/rpc` and
    `pkg/rpc-error-convert` of a `zk-rollup-noir` repository.
  - The section "Verification", at lines 237 to 250, lists the tests that
    the implementation must provide.
  - Five files link to the doc: the root `README.md`, `docs/README.md`,
    `docs/protocol/README.md`, `crates/rpc-error/README.md`, and
    `crates/rpc-error-derive/README.md`.
  - The coordinator of the agent-rules branch set a rule on 2026-11-04. A
    protocol doc fix waits for a follow-up item when the doc has 250 lines
    or more or names code.
- **Problem:** The last sentence of the doc is a process step, not a
  requirement. It makes `cargo xtask review` a required step, but the user
  made that command optional on 2026-11-04. The doc also names code and
  local paths, and it is over the length target of about 250 lines.
- **Impact if no action:** An agent that reads the doc treats
  `cargo xtask review` as required, and that command needs the `codex` CLI.
  The doc and the `implementation-review` skill then give two different
  review steps.
- **Root question:** How does the protocol doc hold only the requirements and
  the decisions in force of `rpc-error`, within the length target?
- **Options:**
  - **A.** Delete the sentence. Drop the three code paths, which the root
    `README.md` already names. Replace each local path with the name of its
    repository. Delete the test list after a check that the tests under
    `crates/rpc-error/` cover each entry. These moves bring the doc under
    240 lines, so this option makes no split. Its risk: the `documentation`
    skill asks for a split when a change touches a doc over the target.
    Effort: Small.
  - **B.** Do all that option A does, and also split the doc into linked
    pages under `docs/protocol/rpc-error/`, one for each area. Possible
    areas are the error model and the envelope, the derive conversion, and
    the adapters and the logging. When `README.md` stays the first page,
    the five links to the doc stay valid. Its cost: more pages to keep
    current. Effort: Medium.
- **Left out:**
  - Rewrite the sentence to name the new review process. A protocol doc
    holds no process steps, so this fails the goal.
  - Delete only the sentence. The `documentation` skill moves code names in
    the same change, so this breaks a rule in force.
  - Move the sentence to the root `README.md`. This makes the optional
    command a required step again, against the user's decision of
    2026-11-04.
  - Change nothing. The doc then keeps a step that the user's decision of
    2026-11-04 replaced, so this fails the goal.
- **Wider fix:** None. The wider fix is already in force: the `documentation`
  skill keeps process steps, code names, and mechanics out of protocol docs.
- **Previous examples:** Commit `38c115b` removed plan references from the
  READMEs on 2026-11-01, after the user decided that a README holds no plan
  reference.
- **Related:**
  `plans/follow-ups/accounting-guide-describes-uncommitted-workspace.md`.
  Both items replace the local path of the Accounting service, so one
  branch can fix both. Accounting detail that this item moves out of the
  doc can go into that guide.
- **Recommendation:** B. It follows the split rule of the `documentation`
  skill as written, and pages of one area each leave room for new
  requirements.
- **Action:** fix after merge, apply B, because the split and the moves are
  larger than the agent-rules change that found the problem.
