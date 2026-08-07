#!/usr/bin/env python3
"""Faithful-port audit: compare each foundation crate's content against the
surfaces target (f2553269) and oracle (6b450184) using git blob SHAs.

Blob SHAs are content hashes identical across repos, so this needs no tar/awk.
"""
import subprocess, sys, os

WYRD = "/Users/stevenforrester/Documents/GitHub/wyrd"
FOUND = "/Users/stevenforrester/Documents/GitHub/wyrd-core"
SURF = "f2553269"
ORACLE = "6b450184"

CANDIDATES = ["crates/{n}", "crates/shared/{n}", "crates/skald/{n}",
              "crates/wyrd/{n}", "crates/vala/{n}"]

def run(cwd, *args):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True)

def tree_map(repo, sha, path):
    """relpath -> blob sha at repo:sha:path (empty if path absent)."""
    r = run(repo, "git", "ls-tree", "-r", sha, "--", path)
    m = {}
    for line in r.stdout.splitlines():
        meta, _, fname = line.partition("\t")
        parts = meta.split()
        if len(parts) >= 3 and parts[1] == "blob":
            rel = fname[len(path):].lstrip("/")
            m[rel] = parts[2]
    return m

def found_map(name):
    """relpath -> blob sha for foundation crates/<name> working tree (index)."""
    r = run(FOUND, "git", "ls-files", "-s", "--", f"crates/{name}")
    m = {}
    base = f"crates/{name}/"
    for line in r.stdout.splitlines():
        meta, _, fname = line.partition("\t")
        parts = meta.split()
        if len(parts) >= 2 and fname.startswith(base):
            m[fname[len(base):]] = parts[1]
    return m

def find_source_path(repo, sha, name):
    for c in CANDIDATES:
        p = c.format(n=name)
        if tree_map(repo, sha, p):
            return p
    return None

# discover foundation crate dirs
r = run(FOUND, "git", "ls-files", "crates/")
crates = sorted({p.split("/")[1] for p in r.stdout.splitlines() if p.startswith("crates/") and len(p.split("/")) > 2})

IGNORE = lambda rel: rel.startswith(".dev") or rel.startswith("target/")

print(f"{'CRATE':<28} {'surf':>4} {'diff':>4} {'sONLY':>5} {'fONLY':>5}  notes")
print("-"*90)
summary = []
for name in crates:
    sp = find_source_path(WYRD, SURF, name)
    op = find_source_path(WYRD, ORACLE, name)
    fm = {k:v for k,v in found_map(name).items() if not IGNORE(k)}
    sm = {k:v for k,v in (tree_map(WYRD, SURF, sp).items() if sp else []) if not IGNORE(k)}
    om = {k:v for k,v in (tree_map(WYRD, ORACLE, op).items() if op else []) if not IGNORE(k)}

    if not sp:
        note = "NEW (no surfaces source)" if not op else "oracle-only crate (no surfaces source!)"
        print(f"{name:<28} {'-':>4} {'-':>4} {'-':>5} {len(fm):>5}  {note}")
        summary.append((name, sp, op, [], [], [], om))
        continue

    differ = sorted(k for k in fm if k in sm and fm[k] != sm[k])
    s_only = sorted(k for k in sm if k not in fm)      # in surfaces target, missing from foundation
    f_only = sorted(k for k in fm if k not in sm)      # in foundation, not in surfaces target
    # for f_only, is it an oracle blob? (oracle-only addition carried into foundation)
    f_only_oracle = [k for k in f_only if k in om and fm[k]==om[k]]
    status = "OK" if not (differ or s_only or f_only) else "DIVERGES"
    note = status
    if f_only_oracle: note += f" | {len(f_only_oracle)} oracle-only file(s)"
    print(f"{name:<28} {len(sm):>4} {len(differ):>4} {len(s_only):>5} {len(f_only):>5}  {note}")
    summary.append((name, sp, op, differ, s_only, f_only, om))

print("\n\n=== DETAIL for diverging crates ===")
for name, sp, op, differ, s_only, f_only, om in summary:
    if not (differ or s_only or f_only):
        continue
    print(f"\n## {name}  (surfaces: {sp}  oracle: {op})")
    if differ:
        fm = {kk:vv for kk,vv in found_map(name).items()}
        n_oracle = sum(1 for k in differ if k in om and fm.get(k)==om[k])
        n_other = len(differ)-n_oracle
        print(f"  DIFFER (foundation != surfaces target) [{len(differ)}]  =O:{n_oracle} =X(reconciled/novel):{n_other}:")
        for k in differ:
            if k in om and fm.get(k)==om[k]:
                tag = " =O (pure oracle base -> FLIP to surfaces unless bifrost)"
            else:
                tag = " =X (already reconciled/modified -> inspect)"
            print(f"    ~ {k}{tag}")
    if s_only:
        print(f"  MISSING from foundation (surfaces has, port dropped) [{len(s_only)}]:")
        for k in s_only: print(f"    - {k}")
    if f_only:
        print(f"  EXTRA in foundation (not in surfaces target) [{len(f_only)}]:")
        for k in f_only:
            tag = " [==oracle: oracle-only addition to FLAG]" if (k in om) else " [novel: not in oracle either]"
            print(f"    + {k}{tag}")
