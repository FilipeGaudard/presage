#!/usr/bin/env bash
# Varredura antes do push, de cada linha adicionada no intervalo (o histórico,
# não só a ponta):
#   bash tools/dev/scan.sh [INTERVALO]      padrão origin/main..HEAD
# Chaves e tokens (a da Riot primeiro), JWTs, senhas, o e-mail de quem commita
# fora da autoria, Riot ID e puuid de jogador, e arquivos que nunca podem ser
# versionados. Cada padrão foi conferido disparando numa amostra plantada.
R=$(cd "$(dirname "$0")/../.." && pwd)
cd "$R" || exit 9
RANGE=${1:-origin/main..HEAD}
ADDED=$(mktemp)
trap 'rm -f "$ADDED"' EXIT
echo "intervalo $RANGE: $(git rev-list --count $RANGE) commits"
git log -p --no-color --format='@@COMMIT %h' $RANGE | awk '/^@@COMMIT/{c=$2} /^\+\+\+ /{f=$2} /^\+/ && !/^\+\+\+ /{print c" "f": "substr($0,2,160)}' > "$ADDED"
echo "linhas adicionadas: $(wc -l < "$ADDED")"
check() { # rótulo regex [máximo]
  local n; n=$(grep -cE -- "$2" "$ADDED")
  echo "== $1: $n"
  [ "$n" -gt 0 ] && grep -E -- "$2" "$ADDED" | cut -c1-200 | head -${3:-15}
}
check "chave da Riot" 'RGAPI-[0-9a-fA-F]{8}-'
check "chaves privadas" 'BEGIN [A-Z ]*PRIVATE KEY'
check "chave do atualizador (minisign)" 'untrusted comment: (rsign|minisign) encrypted secret key'
check "tokens do github" '(gho|ghp|ghs|ghu|ghr)_[A-Za-z0-9]{20,}|github[_]pat_'
check "jwts" 'eyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.'
check "slack/openai/anthropic" 'xox[abpr]-|sk-[A-Za-z0-9]{20,}|sk[-]ant-'
check "senhas" '(password|passwd|senha)[A-Za-z_]*["'"'"']?\s*[:=]\s*["'"'"']?[^ "$'"'"'{(<]{6,}' 30
# puuid: 78 caracteres base64url; Riot ID: nome#TAG com a tag em maiúsculas
# As linhas de integridade do package-lock são hashes sha512, não puuid.
PUUID=$(grep -E -- '[A-Za-z0-9_-]{78}' "$ADDED" | grep -v '"integrity": "sha')
echo "== puuid: $(printf '%s' "$PUUID" | grep -c .)"
[ -n "$PUUID" ] && printf '%s\n' "$PUUID" | cut -c1-200 | head -15
check "riot id (nome#TAG)" '\b[A-Za-z][A-Za-z0-9]{2,15}#[A-Z0-9]{2,5}\b' 30
# entre aspas, com espaço e tag minúscula: "nome com espaço#tag" (fixture fictícia como alguem#BR1 também aparece)
check "riot id entre aspas" '"[^"#]{3,16}#[A-Za-z0-9]{2,5}"' 30
{ git config user.email; git log --format='%ae%n%ce' $RANGE; } | sort -u | while read -r EMAIL; do
  [ -n "$EMAIL" ] || continue
  n=$(grep -cF -- "$EMAIL" "$ADDED")
  echo "== o e-mail $EMAIL no conteúdo: $n"
  [ "$n" -gt 0 ] && grep -F -- "$EMAIL" "$ADDED" | cut -c1-200 | head -15
done
echo "== arquivos do intervalo que nunca podem ser versionados:"
git log --name-only --format= $RANGE | sort -u | grep -E '(^|/)\.pentest/|(^|/)\.presage/|\.key$|\.pfx$|(^|/)\.env|gravacao\.json|ultima-partida\.json|personal-v2\.json|base-de-partidas\.json|coleta-em-andamento\.json' || echo nenhum
echo "== arquivos versionados que o .gitignore ignora:"
git ls-files -ci --exclude-standard || true
echo "== autoria no intervalo:"
git log --format='%ae %ce' $RANGE | sort | uniq -c
