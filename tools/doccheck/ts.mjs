// A TypeScript scanner for the front: interfaces and string unions (the
// mirror of the Rust wire types), `invoke("…")` calls, string literals and
// comments. Same spirit as rust.mjs: just enough grammar, checked by fixtures.

export function stripTs(src) {
  let out = '';
  const strings = [];
  const comments = [];
  let i = 0;
  let line = 1;
  const n = src.length;
  while (i < n) {
    const c = src[i];
    const d = src[i + 1];
    if (c === '/' && d === '/') {
      let j = src.indexOf('\n', i);
      if (j < 0) j = n;
      comments.push({ text: src.slice(i, j), line });
      i = j;
      continue;
    }
    if (c === '/' && d === '*') {
      let j = src.indexOf('*/', i + 2);
      if (j < 0) j = n;
      const text = src.slice(i, j + 2);
      comments.push({ text, line });
      for (const ch of text) if (ch === '\n') { out += '\n'; line++; }
      i = j + 2;
      continue;
    }
    if (c === '"' || c === "'" || c === '`') {
      let j = i + 1;
      let value = '';
      while (j < n && src[j] !== c) {
        if (src[j] === '\\') { value += src[j + 1]; j += 2; continue; }
        if (src[j] === '\n') line++;
        value += src[j];
        j++;
      }
      const k = strings.length;
      strings.push({ value, line, quote: c });
      out += `"@${k}"` + '\n'.repeat((value.match(/\n/g) || []).length);
      i = j + 1;
      continue;
    }
    if (c === '\n') line++;
    out += c;
    i++;
  }
  return { code: out, strings, comments };
}

function matchBrace(code, open) {
  let depth = 0;
  for (let j = open; j < code.length; j++) {
    if (code[j] === '{') depth++;
    else if (code[j] === '}') { depth--; if (depth === 0) return j; }
  }
  return code.length - 1;
}

function lineAt(code, pos) {
  let k = 1;
  for (let i = 0; i < pos; i++) if (code[i] === '\n') k++;
  return k;
}

/** Top-level member names of an object type body. */
function members(body, strings) {
  const names = [];
  let depth = 0;
  let cur = '';
  const flush = () => {
    const m = cur.trim().match(/^(?:readonly\s+)?(?:"@(\d+)"|(\w+))\??\s*:/);
    if (m) names.push(m[1] !== undefined ? strings[Number(m[1])].value : m[2]);
    cur = '';
  };
  let prev = '';
  for (const c of body) {
    if ('{([<'.includes(c)) depth++;
    else if ('})]'.includes(c) || (c === '>' && prev !== '=')) depth--;
    prev = c;
    if (depth === 0 && (c === ';' || c === ',' || c === '\n')) { flush(); continue; }
    cur += c;
  }
  flush();
  return names;
}

export function scanTs(src, { file = '' } = {}) {
  const { code, strings, comments } = stripTs(src);
  const interfaces = [];
  const unions = [];
  for (const m of code.matchAll(/\b(?:export\s+)?interface\s+(\w+)(?:<[^>{]*>)?\s*(?:extends\s+([^{]+))?\{/g)) {
    const open = m.index + m[0].length - 1;
    const end = matchBrace(code, open);
    interfaces.push({
      name: m[1],
      fields: members(code.slice(open + 1, end), strings),
      extends: m[2] ? m[2].split(',').map((s) => s.trim().split('<')[0]).filter(Boolean) : [],
      file,
      line: lineAt(code, m.index),
    });
  }
  for (const m of code.matchAll(/\b(?:export\s+)?type\s+(\w+)\s*=\s*([^;]+);/g)) {
    const rhs = m[2].trim();
    if (rhs.startsWith('{')) {
      const open = m.index + m[0].indexOf('{');
      const end = matchBrace(code, open);
      interfaces.push({ name: m[1], fields: members(code.slice(open + 1, end), strings), extends: [], file, line: lineAt(code, m.index) });
      continue;
    }
    const parts = rhs.split('|').map((s) => s.trim()).filter(Boolean);
    if (parts.length && parts.every((p) => /^"@\d+"$/.test(p))) {
      unions.push({ name: m[1], values: parts.map((p) => strings[Number(p.slice(2, -1))].value), file, line: lineAt(code, m.index) });
    }
  }
  const invokes = [];
  for (const m of code.matchAll(/\binvoke\s*(?:<[^(]*?>)?\s*\(\s*"@(\d+)"/g)) {
    invokes.push({ name: strings[Number(m[1])].value, file, line: lineAt(code, m.index) });
  }
  // O canal de erro do App (`chamar(rótulo, comando, args)` e `ler(rótulo,
  // comando, aplicar)`): o comando é o segundo texto.
  for (const m of code.matchAll(/\b(?:chamar|ler)\s*(?:<[^(]*?>)?\s*\(\s*"@\d+"\s*,\s*"@(\d+)"/g)) {
    invokes.push({ name: strings[Number(m[1])].value, file, line: lineAt(code, m.index) });
  }
  return { interfaces, unions, invokes, strings, comments };
}
