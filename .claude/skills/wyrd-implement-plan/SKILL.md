---
name: wyrd-implement-plan
description: Autonomously execute or resume a complete approved wyrd-core foundation implementation plan in one persistent root session. Use when asked to implement, continue, finish, or close a plan supplied in conversation or by file path. The root executes every dependency-ordered task through $wyrd-implement and invokes $wyrd-review after each task and terminal review at closeout.
---

# Wyrd Core Implement Plan

Execute the complete approved plan in one persistent root session. The root is
the only writer and owns plan truth, task order, implementation, verification,
material decisions, task status, commits, remediation, and the user outcome.
Use subagents only as independent read-only reviewers.

Before edits, read the complete plan and every task; `AGENTS.md`,
`architecture/agent-rules.md`, `architecture/wyrd-design.md`,
`architecture/wyrd-doctrine.mdx`; and every authority named by the plan. Read
`architecture/references/README.md` and only relevant existing reference slices.
Read the bundled references when their stage begins. This skill executes only
foundation-crate work; server, UI, SDK, Skald-runtime, and Vala/Bifrost changes
are external dependencies and must not enter its write set.

Create an isolated local branch/worktree when the caller worktree can be
preserved without losing caller-selected plan artifacts. Otherwise preserve
unrelated user changes, make only task-scoped edits, and record the constraint.
Do not push, merge, rewrite history, or alter Git identity.

For each dependency-ordered task, apply `$wyrd-implement`; run focused proof
and a complete diff audit; then obtain a fresh `$wyrd-review` binding-mode
review. Keep a task `Ready` until focused verification passes and review returns
`APPROVE`. Validate findings against source, repair confirmed bounded findings,
rerun affected proof, and re-review. For material conflicts, revise the
canonical plan/task before continuing; do not make the decision during coding.

After every task is accepted, execute plan-defined integrated checks and the
foundation closeout gates. Run terminal static review against the committed
baseline, fix confirmed findings, and repeat required proof. Complete only when
every task is accepted, required focused and integrated checks pass, no required
review finding remains, and the branch contains no unrelated change.
