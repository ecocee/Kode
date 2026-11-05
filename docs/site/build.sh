#!/usr/bin/env bash
# Build the documentation site: the spec, the ADRs, the error index and the API
# page that `kode doc` generates. No dependencies beyond awk and the compiler.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out="$here/out"

rm -rf "$out"
mkdir -p "$out"

# A deliberately small Markdown reader: headings, bullets, fenced code.
md2html() { # source.md output.html title
  awk -v title="$3" '
    BEGIN {
      print "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">"
      print "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
      print "<title>" title "</title><style>"
      print "body{font:16px/1.6 system-ui,sans-serif;max-width:52rem;margin:3rem auto;padding:0 1rem;color:#232323;background:#fbfaf7}"
      print "pre{background:#f3f0e9;padding:.8rem;overflow:auto;border-radius:4px;white-space:pre-wrap}"
      print "code{background:#f3f0e9;padding:0 .2rem;border-radius:3px}"
      print "h1{margin-top:0}h2{margin-top:2rem;border-bottom:1px solid #e2ddd2;padding-bottom:.3rem;font-size:1.1rem}"
      print "a{color:#3c6e47}table{border-collapse:collapse}td,th{border:1px solid #ddd;padding:.3rem .6rem}"
      print "</style></head><body><p><a href=\"index.html\">index</a></p>"
      in_code = 0
    }
    /^```/ { if (in_code) { print "</code></pre>"; in_code = 0 } else { print "<pre><code>"; in_code = 1 } next }
    in_code { print; next }
    /^# / { print "<h1>" substr($0, 3) "</h1>"; next }
    /^## / { print "<h2>" substr($0, 4) "</h2>"; next }
    /^### / { print "<h3>" substr($0, 5) "</h3>"; next }
    /^- / { print "<li>" substr($0, 3) "</li>"; next }
    /^[[:space:]]*$/ { print ""; next }
    { print "<p>" $0 "</p>" }
    END { if (in_code) print "</code></pre>"; print "</body></html>" }
  ' "$1" > "$2"
}

index="$out/index.html"
{
  echo '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Kode docs</title>'
  echo '<style>body{font:16px/1.6 system-ui,sans-serif;max-width:52rem;margin:3rem auto;padding:0 1rem;background:#fbfaf7;color:#232323}a{color:#3c6e47}li{margin:.2rem 0}</style>'
  echo '</head><body><h1>Kode</h1><h2>Spec</h2><ul>'
  for f in "$root"/docs/spec/*.md; do
    name=$(basename "$f" .md)
    md2html "$f" "$name" "$out/spec-$name.html"
    echo "<li><a href=\"spec-$name.html\">$name</a></li>"
  done
  echo '</ul><h2>Decisions</h2><ul>'
  for f in "$root"/docs/adr/*.md; do
    name=$(basename "$f" .md)
    md2html "$f" "ADR $name" "$out/adr-$name.html"
    echo "<li><a href=\"adr-$name.html\">$name</a></li>"
  done
  echo '</ul><h2>Errors</h2><ul>'
  md2html "$root/docs/errors/index.md" "Error codes" "$out/errors.html"
  echo '<li><a href="errors.html">error index</a></li>'
  echo '</ul>'
  if [ -d "$out/api" ]; then
    echo '<h2>API</h2><ul><li><a href="api/index.html">generated reference</a></li></ul>'
  fi
  echo '</body></html>'
} > "$index"

if [ -x "$root/target/debug/kode" ]; then
  "$root/target/debug/kode" doc "$root/examples" -o "$out/api" 2>/dev/null || \
    "$root/target/debug/kode" doc "$root/tests/run-pass" -o "$out/api" || true
fi

echo "docs: $index"
