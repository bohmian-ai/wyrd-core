#!/usr/bin/env python3
"""Symmetric-miss check: every wyrd-spec vala file, 3-way + shipped disposition."""
import subprocess
WYRD="/Users/stevenforrester/Documents/GitHub/wyrd"
FOUND="/Users/stevenforrester/Documents/GitHub/wyrd-foundation"
SURF="f2553269"; ORACLE="6b450184"; MB="f58ec630"
BASE="crates/wyrd-spec/src/vala"

def rp(repo, ref_path):
    r=subprocess.run(["git","rev-parse",ref_path],cwd=repo,capture_output=True,text=True)
    return r.stdout.strip() if r.returncode==0 else None
def changed(a,b,path):
    r=subprocess.run(["git","diff","--quiet",f"{a}..{b}","--",path],cwd=WYRD,capture_output=True)
    return r.returncode!=0  # nonzero => differs
def lstree(ref):
    r=subprocess.run(["git","ls-tree","-r","--name-only",ref,"--",BASE],cwd=WYRD,capture_output=True,text=True)
    return set(r.stdout.split())

files=sorted(lstree(SURF)|lstree(ORACLE))
defects=[]; reconcile=[]
print(f"{'file':<34} {'shipped':<7} {'sAdv':<5} {'oAdv':<5} verdict")
for f in files:
    rel=f[len(BASE)+1:]
    sfb=rp(WYRD,f"{SURF}:{f}"); orb=rp(WYRD,f"{ORACLE}:{f}"); fnb=rp(FOUND,f"HEAD:{f}")
    ship="ABSENT" if fnb is None else ("surf" if fnb==sfb else ("oracle" if fnb==orb else "OTHER"))
    sadv=changed(MB,SURF,f); oadv=changed(MB,ORACLE,f)
    verdict="ok"
    if ship=="oracle" and sadv and not oadv:
        verdict="** DEFECT: surfaces-advanced but shipped ORACLE **"; defects.append(rel)
    elif ship=="oracle" and sadv and oadv:
        verdict="~ both-advanced, shipped oracle (reconcile?)"; reconcile.append(rel)
    elif ship=="surf" and oadv and not sadv:
        verdict="note: oracle-advanced but shipped surfaces"
    elif ship=="OTHER":
        verdict="OTHER (neither surf nor oracle blob)"
    print(f"{rel:<34} {ship:<7} {str(sadv):<5} {str(oadv):<5} {verdict}")
print(f"\nDEFECTS(surfaces-advanced->shipped-oracle): {defects or 'NONE'}")
print(f"BOTH-ADVANCED shipped-oracle (needs reconcile review): {reconcile or 'NONE'}")
