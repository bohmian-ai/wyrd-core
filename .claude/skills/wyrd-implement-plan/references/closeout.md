# Foundation Closeout

At each declared milestone, run its integrated checks sequentially and record
their results. Reopen the earliest responsible task when later work changes an
accepted invariant or integrated proof fails.

After all tasks are accepted, run the plan-required foundation gates: focused
tests, applicable Postgres integration tests, schema/codegen checks, then
`mise run pre-pr` when required by the plan or affected CI surface. Inspect the
complete branch diff and generated provenance.

Obtain terminal static review against the committed baseline using the approved
plan as intent. Validate and repair confirmed findings in the root session,
rerun affected task and integrated proof, and repeat terminal review after
material or cross-task remediation. Report completion only with accepted tasks,
passing required proof, clean review, and no unrelated changes.
