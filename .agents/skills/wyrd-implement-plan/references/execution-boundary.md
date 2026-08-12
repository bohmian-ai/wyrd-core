# Execution Boundary

Use a dedicated local branch and worktree from the caller's resolved `HEAD`
when it can include the caller-selected plan artifacts without absorbing
unrelated work. Preserve all caller-owned tracked and untracked changes; never
stash, reset, clean, force-add ignored files, push, merge, amend, rebase, or
rewrite accepted commits. Stage exact paths and inspect staged name-status.

The canonical plan, task packets, task evidence, Git history, current source,
and recorded verification are durable execution state. A task stays `Ready`
until focused proof passes and `$wyrd-review` approves it.

Repair stale private paths and equivalent verification recipes from repository
evidence. A public/persisted contract, migration, dependency/feature,
auth/tenancy/policy/audit semantic, ownership direction, requirement, or
acceptance change is material: revise affected plan and task authority before
implementation resumes.

Only unavailable external credentials, permissions, infrastructure, irreversible
external action, or an artifact with no discernible outcome is a genuine
blocker. Tests, review findings, plan defects, context resets, and elapsed time
are not blockers.
