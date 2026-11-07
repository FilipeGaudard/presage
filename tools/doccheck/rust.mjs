// A Rust scanner, not a parser: enough of the grammar to find items, fields,
// tests, commands and string literals, and nothing more. It runs on the code
// and on the ```rust blocks of the docs, so both sides are read by the same
// rules (ADR-0020).
//
// What it understands: line and nested block comments, strings (escaped, raw
// with any number of #, byte), char literals versus lifetimes, attributes, and
// the brace structure of mod / impl / trait / fn / struct / enum. What it does
// not: macros that declare items, items inside fn bodies, `use` re-exports.

const IDENT = /[A-Za-z0-9_]/;

/** Replaces comments, strings and chars, keeping every newline in place.
 *  A string becomes `"@k"`, where k indexes `strings`. */
export function strip(src) {
  let out = '';
  const strings = [];
  const comments = [];
  let i = 0;
  let line = 1;
  const n = src.length;
  const pushString = (value, from, to) => {
    const k = strings.length;
    strings.push({ value, line });
    out += `"@${k}"`;
    const nl = countNewlines(src, from, to);
    out += '\n'.repeat(nl);
    line += nl;
  };
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
      const startLine = line;
      let depth = 1;
      let j = i + 2;
      while (j < n && depth > 0) {
        if (src[j] === '/' && src[j + 1] === '*') { depth++; j += 2; continue; }
        if (src[j] === '*' && src[j + 1] === '/') { depth--; j += 2; continue; }
        if (src[j] === '\n') { out += '\n'; line++; }
        j++;
      }
      comments.push({ text: src.slice(i, j), line: startLine });
      i = j;
      continue;
    }
    const prevIdent = i > 0 && IDENT.test(src[i - 1]);
    if (!prevIdent && (c === 'r' || (c === 'b' && d === 'r'))) {
      const k = c === 'b' ? i + 2 : i + 1;
      let h = 0;
      while (src[k + h] === '#') h++;
      if (src[k + h] === '"') {
        const close = '"' + '#'.repeat(h);
        let end = src.indexOf(close, k + h + 1);
        if (end < 0) end = n;
        pushString(src.slice(k + h + 1, end), i, end + close.length);
        i = end + close.length;
        continue;
      }
    }
    if (c === '"' || (!prevIdent && c === 'b' && d === '"')) {
      let j = c === 'b' ? i + 2 : i + 1;
      let value = '';
      while (j < n && src[j] !== '"') {
        if (src[j] === '\\') {
          value += unescapeOne(src[j + 1]);
          j += 2;
        } else {
          value += src[j];
          j++;
        }
      }
      pushString(value, i, j + 1);
      i = j + 1;
      continue;
    }
    if (c === "'") {
      if (d === '\\') {
        let end = src.indexOf("'", i + 3);
        if (end < 0) end = n;
        out += "'_'";
        i = end + 1;
        continue;
      }
      const cp = src.codePointAt(i + 1);
      const len = cp !== undefined && cp > 0xffff ? 2 : 1;
      if (src[i + 1 + len] === "'" && src[i + 1] !== '\n') {
        out += "'_'";
        i = i + 2 + len;
        continue;
      }
      out += c; // a lifetime or a label
      i++;
      continue;
    }
    if (c === '\n') line++;
    out += c;
    i++;
  }
  return { code: out, strings, comments };
}

function countNewlines(s, from, to) {
  let k = 0;
  for (let i = from; i < to && i < s.length; i++) if (s[i] === '\n') k++;
  return k;
}

function unescapeOne(ch) {
  return { n: '\n', t: '\t', r: '\r', '0': '\0', '\\': '\\', '"': '"', "'": "'" }[ch] ?? ch;
}

/** Splits leading attributes off an item header. */
export function splitAttrs(header, strings) {
  const attrs = [];
  let rest = '';
  let i = 0;
  while (i < header.length) {
    if (header[i] === '#' && (header[i + 1] === '[' || (header[i + 1] === '!' && header[i + 2] === '['))) {
      const open = header.indexOf('[', i);
      let depth = 0;
      let j = open;
      for (; j < header.length; j++) {
        if (header[j] === '[') depth++;
        else if (header[j] === ']') { depth--; if (depth === 0) break; }
      }
      attrs.push(resolveStrings(header.slice(open + 1, j), strings).trim());
      i = j + 1;
      continue;
    }
    rest += header[i];
    i++;
  }
  return { attrs, rest: rest.replace(/\s+/g, ' ').trim() };
}

export function resolveStrings(text, strings) {
  return text.replace(/"@(\d+)"/g, (_, k) => JSON.stringify(strings[Number(k)]?.value ?? ''));
}

/** The serde options of an attribute list: rename_all, rename, skip, flatten. */
export function serdeOf(attrs) {
  const o = {};
  for (const a of attrs) {
    const m = a.match(/^serde\s*\((.*)\)$/s);
    if (!m) continue;
    for (const part of splitTop(m[1], ',')) {
      const kv = part.trim().match(/^(\w+)\s*(?:=\s*"((?:[^"\\]|\\.)*)")?$/);
      if (!kv) continue;
      o[kv[1]] = kv[2] === undefined ? true : kv[2];
    }
  }
  return o;
}

const VIS = String.raw`(?:pub(?:\s*\([^)]*\))?\s+)?`;
const RE = {
  fn: new RegExp(`^${VIS}(?:(?:const|async|unsafe|extern(?:\\s+"@\\d+")?)\\s+)*fn\\s+(\\w+)`),
  struct: new RegExp(`^${VIS}struct\\s+(\\w+)`),
  enum: new RegExp(`^${VIS}enum\\s+(\\w+)`),
  union: new RegExp(`^${VIS}union\\s+(\\w+)`),
  trait: new RegExp(`^${VIS}(?:unsafe\\s+)?(?:auto\\s+)?trait\\s+(\\w+)`),
  type: new RegExp(`^${VIS}type\\s+(\\w+)`),
  const: new RegExp(`^${VIS}(?:const|static)\\s+(?:mut\\s+)?(\\w+)\\s*:`),
  mod: new RegExp(`^${VIS}mod\\s+(\\w+)`),
  impl: /^(?:unsafe\s+)?impl\b/,
};

/** A fn signature with visibility and whitespace removed: `fnrank(x:u8)->Vec<Scored>`. */
export function signature(rest) {
  return rest.replace(new RegExp(`^${VIS}`), '').replace(/\s+/g, '');
}

/** A const initializer, when the header holds it whole (ended at ';', brackets balanced). */
export function constValue(rest, end) {
  const eq = rest.indexOf('=');
  if (eq < 0 || end !== ';') return null;
  const v = rest.slice(eq + 1).trim();
  let depth = 0;
  for (const c of v) {
    if ('([{'.includes(c)) depth++;
    else if (')]}'.includes(c)) depth--;
  }
  return depth === 0 && v ? v.replace(/\s+/g, '') : null;
}

/** What an item header declares, or null. */
export function parseHeader(rest) {
  const isPub = /^pub\b/.test(rest) && !/^pub\s*\(\s*(?:crate|super|self|in\b)/.test(rest);
  let m;
  if (RE.impl.test(rest)) return { kind: 'impl', ...parseImpl(rest), isPub: false };
  if ((m = rest.match(RE.mod))) return { kind: 'mod', name: m[1], isPub };
  if ((m = rest.match(RE.fn))) return { kind: 'fn', name: m[1], isPub };
  if ((m = rest.match(RE.struct))) return { kind: 'struct', name: m[1], isPub };
  if ((m = rest.match(RE.enum))) return { kind: 'enum', name: m[1], isPub };
  if ((m = rest.match(RE.union))) return { kind: 'struct', name: m[1], isPub };
  if ((m = rest.match(RE.trait))) return { kind: 'trait', name: m[1], isPub };
  if ((m = rest.match(RE.type))) return { kind: 'type', name: m[1], isPub };
  if ((m = rest.match(RE.const))) return { kind: 'const', name: m[1], isPub };
  return null;
}

function parseImpl(rest) {
  let s = rest.replace(/^(?:unsafe\s+)?impl\s*/, '');
  if (s.startsWith('<')) s = s.slice(matchAngle(s, 0) + 1);
  s = s.split(/\swhere\s/)[0].trim();
  const parts = splitTop(s, ' for ');
  const typePart = (parts.length > 1 ? parts[parts.length - 1] : parts[0]).trim();
  const traitPart = parts.length > 1 ? parts[0].trim() : null;
  return { type: lastIdent(typePart), trait: traitPart ? lastIdent(traitPart) : null };
}

function lastIdent(path) {
  const base = path.replace(/^[&\s]*(?:'\w+\s+)?(?:mut\s+)?(?:dyn\s+)?/, '');
  const noGen = base.split('<')[0];
  const segs = noGen.split('::').map((x) => x.trim()).filter(Boolean);
  return segs[segs.length - 1] ?? noGen;
}

function matchAngle(s, open) {
  let depth = 0;
  for (let j = open; j < s.length; j++) {
    if (s[j] === '<') depth++;
    else if (s[j] === '>' && s[j - 1] !== '-') { depth--; if (depth === 0) return j; }
  }
  return s.length - 1;
}

/** Splits on `sep` outside (), [], {}, <>. */
export function splitTop(s, sep) {
  const out = [];
  let depth = 0;
  let cur = '';
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if ('([{'.includes(c)) depth++;
    else if (')]}'.includes(c)) depth--;
    else if (c === '<') depth++;
    else if (c === '>' && s[i - 1] !== '-' && s[i - 1] !== '=') depth--;
    if (depth === 0 && s.startsWith(sep, i)) {
      out.push(cur);
      cur = '';
      i += sep.length - 1;
      continue;
    }
    cur += c;
  }
  if (cur.trim()) out.push(cur);
  return out;
}

function matchBrace(code, open) {
  let depth = 0;
  for (let j = open; j < code.length; j++) {
    if (code[j] === '{') depth++;
    else if (code[j] === '}') { depth--; if (depth === 0) return j; }
  }
  return code.length - 1;
}

const CASE = {
  lowercase: (s) => s.toLowerCase(),
  UPPERCASE: (s) => s.toUpperCase(),
  camelCase: (s, variant) => (variant ? s[0].toLowerCase() + s.slice(1) : s.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase())),
  PascalCase: (s, variant) => (variant ? s : s.replace(/(?:^|_)([a-z0-9])/g, (_, c) => c.toUpperCase())),
  snake_case: (s, variant) => (variant ? s.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase() : s),
  SCREAMING_SNAKE_CASE: (s, variant) => (variant ? s.replace(/([a-z0-9])([A-Z])/g, '$1_$2') : s).toUpperCase(),
  'kebab-case': (s, variant) => (variant ? s.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase() : s.replace(/_/g, '-')),
};

/** The serialized name, under serde's rename_all, of a field or variant. */
export function wireName(name, renameAll, variant) {
  if (!renameAll) return name;
  const f = CASE[renameAll];
  return f ? f(name, variant) : name;
}

function parseFields(body, strings, renameAll) {
  const fields = [];
  for (const raw of splitTop(body, ',')) {
    const { attrs, rest } = splitAttrs(raw, strings);
    const m = rest.match(new RegExp(`^${VIS}(?:r#)?(\\w+)\\s*:`));
    if (!m) continue;
    const s = serdeOf(attrs);
    fields.push({
      name: m[1],
      ty: rest.slice(m[0].length).replace(/\s+/g, ''),
      wire: s.skip || s.skip_serializing ? null : typeof s.rename === 'string' ? s.rename : wireName(m[1], renameAll, false),
      flatten: !!s.flatten,
    });
  }
  return fields;
}

function parseVariants(body, strings, renameAll) {
  const variants = [];
  for (const raw of splitTop(body, ',')) {
    const { attrs, rest } = splitAttrs(raw, strings);
    const m = rest.match(/^(\w+)\s*(\(|\{|=|$)/);
    if (!m) continue;
    const s = serdeOf(attrs);
    variants.push({
      name: m[1],
      unit: m[2] !== '(' && m[2] !== '{',
      wire: s.skip ? null : typeof s.rename === 'string' ? s.rename : wireName(m[1], renameAll, true),
    });
  }
  return variants;
}

/**
 * Scans one Rust source. `module` is the path of the file's module, e.g.
 * `core::engine::minimax`. Returns the items (fn, method, struct, enum,
 * trait, type, const) with their line, the tests, the tauri commands, the
 * generate_handler list, the string literals and the comments.
 */
export function scanRust(src, { file = '', module = '' } = {}) {
  const { code, strings, comments } = strip(src);
  const lineStarts = [0];
  for (let i = 0; i < code.length; i++) if (code[i] === '\n') lineStarts.push(i + 1);
  const lineOf = (pos) => {
    let lo = 0;
    let hi = lineStarts.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (lineStarts[mid] <= pos) lo = mid;
      else hi = mid - 1;
    }
    return lo + 1;
  };

  const items = [];
  const tests = [];
  const commands = [];
  const stack = [{ kind: 'mod', path: module, cfgTest: false }];
  let segStart = 0;

  let end = '';
  const record = (info, attrs, top, pos, rest) => {
    const line = lineOf(pos);
    const cfgTest = top.cfgTest || attrs.some((a) => /^cfg\s*\(\s*test\s*\)$/.test(a));
    if (info.kind === 'fn') {
      if (attrs.some((a) => a === 'test' || /^(?:tokio::)?test\b/.test(a))) tests.push({ name: info.name, file, line, module: top.path });
      if (attrs.some((a) => /^tauri::command\b/.test(a))) commands.push({ name: info.name, file, line });
    }
    if (cfgTest) return;
    let name = info.name;
    let kind = info.kind;
    if (top.kind === 'impl') {
      if (info.kind !== 'fn' && info.kind !== 'const' && info.kind !== 'type') return;
      name = `${top.type}::${info.name}`;
      kind = info.kind === 'fn' ? 'method' : info.kind;
    } else if (top.kind === 'trait') {
      if (info.kind !== 'fn') return;
      name = `${top.name}::${info.name}`;
      kind = 'method';
    }
    const isPub = top.kind === 'trait' ? top.isPub : top.kind === 'impl' && top.trait ? top.typePub : info.isPub;
    const it = { kind, name, module: top.path, file, line, isPub, trait: top.kind === 'impl' ? top.trait : null, attrs };
    // strings are placeholders ("@k") numbered per file: resolve them before comparing
    if (info.kind === 'fn') it.sig = signature(resolveStrings(rest, strings));
    if (info.kind === 'const') it.value = constValue(resolveStrings(rest, strings), end);
    items.push(it);
    return it;
  };

  for (let i = 0; i < code.length; i++) {
    const ch = code[i];
    if (ch !== '{' && ch !== ';' && ch !== '}') continue;
    if (ch === '}') {
      if (stack.length > 1) stack.pop();
      segStart = i + 1;
      continue;
    }
    const header = code.slice(segStart, i);
    end = ch;
    const lead = header.length - header.trimStart().length;
    const { attrs, rest } = splitAttrs(header, strings);
    const top = stack[stack.length - 1];
    const recording = top.kind === 'mod' || top.kind === 'impl' || top.kind === 'trait';
    const info = recording ? parseHeader(rest) : null;
    let rec = null;
    if (info && info.kind !== 'impl' && info.kind !== 'mod') rec = record(info, attrs, top, segStart + lead, rest);
    if (ch === '{') {
      if (info && (info.kind === 'struct' || info.kind === 'enum')) {
        const end = matchBrace(code, i);
        const body = code.slice(i + 1, end);
        const s = serdeOf(attrs);
        if (rec) {
          if (info.kind === 'struct') rec.fields = parseFields(body, strings, s.rename_all);
          else rec.variants = parseVariants(body, strings, s.rename_all);
          rec.serde = s;
        }
        i = end;
        segStart = end + 1;
        continue;
      }
      const cfgTest = top.cfgTest || attrs.some((a) => /^cfg\s*\(\s*test\s*\)$/.test(a));
      if (info?.kind === 'mod') stack.push({ kind: 'mod', path: top.path ? `${top.path}::${info.name}` : info.name, cfgTest });
      else if (info?.kind === 'impl') stack.push({ kind: 'impl', type: info.type, trait: info.trait, path: top.path, cfgTest, typePub: true });
      else if (info?.kind === 'trait') stack.push({ kind: 'trait', name: info.name, path: top.path, cfgTest, isPub: info.isPub });
      else stack.push({ kind: 'block', path: top.path, cfgTest });
    }
    segStart = i + 1;
  }

  // tuple structs end at ';' and carry no named fields: mark them
  for (const it of items) if (it.kind === 'struct' && !it.fields) it.fields = null;

  const handlers = [];
  for (const m of code.matchAll(/generate_handler!\s*\[([^\]]*)\]/g)) {
    for (const name of m[1].split(',').map((x) => x.trim()).filter(Boolean)) handlers.push(lastIdent(name));
  }
  return { items, tests, commands, handlers, strings, comments };
}

/** The module path of a source file, e.g. core/src/engine/minimax.rs → core::engine::minimax. */
export function modulePath(crate, relFromSrc) {
  const parts = relFromSrc.replace(/\\/g, '/').replace(/\.rs$/, '').split('/');
  const last = parts[parts.length - 1];
  if (last === 'mod' || ((last === 'lib' || last === 'main') && parts.length === 1)) parts.pop();
  return [crate, ...parts].join('::');
}
