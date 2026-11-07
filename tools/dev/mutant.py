#!/usr/bin/env python3
"""mutant.py FILE OLD NEW [FILE OLD NEW ...] -- TEST-COMMAND...

Aplica as trocas (cada OLD tem de aparecer exatamente uma vez), roda o teste,
restaura cada arquivo byte a byte e confere. Saída 0 = mutante morto.

Um mutante morto por erro de compilação ou de tipo não conta: quem tem de
pegar é uma asserção (CLAUDE.md, "Prova").

Exemplos:
  python3 tools/dev/mutant.py core/src/engine/stats.rs "1.0 /" "2.0 /" -- cargo test -p presage-core shrink
  python3 tools/dev/mutant.py src/lib/types.ts "inPool: boolean;" "" -- npm run doccheck -- -strict
"""
import os
import re
import subprocess
import sys
import time


def stamp(f):
    # No WSL, o mtime de /mnt/c fica uns 6 s atrás do relógio do Linux: uma
    # escrita logo depois do último build parece mais velha que ele, e o cargo
    # roda o binário velho (L-28). O carimbo explícito põe o mtime no presente.
    now = time.time() + 1
    os.utime(f, (now, now))

sep = sys.argv.index('--')
args, cmd = sys.argv[1:sep], sys.argv[sep + 1:]
assert len(args) % 3 == 0, "trios FILE OLD NEW"
orig = {}
r = None
try:
    for i in range(0, len(args), 3):
        f, old, new = args[i:i + 3]
        raw = orig.setdefault(f, open(f, 'rb').read())
        cur = open(f, 'rb').read().decode('utf-8')
        if cur.count(old) != 1:
            sys.exit(f"OLD aparece {cur.count(old)} vezes em {f}: {old[:60]!r}")
        open(f, 'wb').write(cur.replace(old, new).encode('utf-8'))
        stamp(f)
    r = subprocess.run(cmd, capture_output=True, text=True, encoding='utf-8', errors='replace',
                       shell=(sys.platform == 'win32'))
    out = re.sub(r'\x1b\[[0-9;]*m', '', r.stdout + r.stderr).strip().splitlines()
    # cargo: panicked, FAILED, test result; doccheck: pendentes, avisos; node: not ok
    keys = ('FAIL', 'panicked', 'test result', 'error', 'Error', 'pendentes', 'avisos', 'not ok', 'assert')
    keep = [l for l in out if any(k in l for k in keys)]
    print("\n".join(keep[:14]))
finally:
    for f, raw in orig.items():
        open(f, 'wb').write(raw)
        stamp(f)
for f, raw in orig.items():
    assert open(f, 'rb').read() == raw, f"a restauração falhou: {f}"
print("restaurado, idêntico: True")
if r is None:
    sys.exit(2)
print("MUTANT", "KILLED" if r.returncode != 0 else "SURVIVED")
sys.exit(0 if r.returncode != 0 else 1)
