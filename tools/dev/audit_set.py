#!/usr/bin/env python3
"""audit_set.py ID STATE PROOF: muda o estado e a prova de uma linha de build/audit.md.

O estado é a célula antes da Fase (as tabelas A/B/D/F têm 7 células, a P tem 5);
a prova é a última."""
import sys

path = 'build/audit.md'
rid, state, proof = sys.argv[1], sys.argv[2], sys.argv[3]
assert state in ('OPEN', 'BUILT', 'DEFERRED', 'DOC'), state
text = open(path, encoding='utf-8', newline='').read()
nl = '\r\n' if '\r\n' in text else '\n'
lines = text.split(nl)
for i, l in enumerate(lines):
    if l.startswith('| %s |' % rid):
        c = l.split('|')
        # P: '', ID, Item, Evidência, Estado, Fase, Prova, ''  (8)
        # as outras: '', ID, Doc, Requisito, Evidência, Estado, Fase, Prova, '' (9)
        si = 4 if len(c) == 8 else 5
        c[si] = ' %s ' % state
        c[-2] = ' %s ' % proof
        lines[i] = '|'.join(c)
        break
else:
    sys.exit('nenhuma linha ' + rid)
open(path, 'w', encoding='utf-8', newline='').write(nl.join(lines))
