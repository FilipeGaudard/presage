#!/usr/bin/env bash
# Os portões que rodam no WSL, todos, antes do commit (CLAUDE.md, "Gates").
# A casca não compila aqui: ela fica para o CI do Windows.
#   bash tools/dev/gates.sh && git commit …
cd "$(dirname "$0")/../.." || exit 9
falhou=0
passo() { echo "== $1"; shift; "$@" >/tmp/gates-$$.log 2>&1 || { echo "FALHOU"; tail -20 /tmp/gates-$$.log; falhou=1; }; }
passo "doccheck -strict" node tools/doccheck/doccheck.mjs -strict
passo "testes do doccheck" node --test tools/doccheck/doccheck.test.mjs
passo "testes do front (tipos e vitest)" npm test
passo "build do front" npm run -s build
passo "testes do core" bash tools/dev/core-test.sh
rm -f /tmp/gates-$$.log
[ $falhou -eq 0 ] && echo "todos verdes" || { echo "algum portão falhou"; exit 1; }
