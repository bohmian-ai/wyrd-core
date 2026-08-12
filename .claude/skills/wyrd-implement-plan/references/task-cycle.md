# Root Task Cycle

The root applies `$wyrd-implement` to one `Ready` task at a time. It establishes
the complete task contract, current owners and callers, accepted feature set,
test tier, and material stop conditions before edits. It records exact focused
commands, results, generated provenance, and diff-audit evidence.

Every implementation-finished task receives a fresh independent
`$wyrd-review` in plan-execution binding mode. The reviewer gets the approved
plan, active task, last accepted commit, complete tracked/untracked delta, and
canonical completion and verification evidence. It performs static analysis
only and must provide requirement traceability, source-derived enforcement,
regression assertions, evidence status, inspected surfaces, and adversarial
probes.

For `RESUME_IMPLEMENTATION`, implement confirmed bounded corrections, rerun
affected proof, and request focused re-review. For `ROOT_DECISION_REQUIRED`,
revise the canonical plan/task before implementation continues. For
`REVIEW_BLOCKED`, provide resolvable authority or proof; apply the execution
boundary only when external authority is genuinely unavailable. Mark complete
and checkpoint-commit only after substantiated `APPROVE`.
