
#!/usr/bin/env bash
#
# Kode Documentation Website Builder
# Professional documentation website
#
# Dependencies: Bash, AWK, sed, find, standard Unix tools
#

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="$HERE/out"

VERSION="0.3.0"

if [[ -f "$ROOT/Cargo.toml" ]]; then
    DETECTED_VERSION="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -n 1)"
    [[ -n "$DETECTED_VERSION" ]] && VERSION="$DETECTED_VERSION"
fi

echo "Building Kode documentation..."
echo "Version: $VERSION"

rm -rf "$OUT"
mkdir -p "$OUT/assets"

# ============================================================
# CSS
# ============================================================

cat > "$OUT/style.css" <<'CSS'
:root {
    --background: #ffffff;
    --sidebar: #fafbfc;
    --surface: #f7f9fa;
    --border: #e5e9ec;
    --text: #202a30;
    --text-secondary: #52616a;
    --muted: #78868e;
    --accent: #087f68;
    --accent-hover: #066b58;
    --accent-soft: #eaf5f1;
    --code-background: #f4f6f7;
    --sidebar-width: 250px;
    --header-height: 62px;
    --content-width: 880px;
}

* {
    box-sizing: border-box;
}

html {
    scroll-behavior: smooth;
    scroll-padding-top: 90px;
}

body {
    margin: 0;
    background: var(--background);
    color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    font-size: 14px;
    line-height: 1.7;
    -webkit-font-smoothing: antialiased;
}

a {
    color: inherit;
    text-decoration: none;
}

button,
input {
    font: inherit;
}

/* Header */

.site-header {
    position: fixed;
    z-index: 100;
    top: 0;
    right: 0;
    left: 0;
    height: var(--header-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 28px;
    background: rgba(255, 255, 255, .97);
    border-bottom: 1px solid var(--border);
    backdrop-filter: blur(12px);
}

.brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -.3px;
}

.brand-mark {
    display: grid;
    width: 29px;
    height: 29px;
    place-items: center;
    border-radius: 5px;
    background: #17332d;
    color: white;
    font-size: 15px;
    font-weight: 750;
}

.brand-label {
    color: var(--muted);
    font-size: 13px;
    font-weight: 500;
}

.header-right {
    display: flex;
    align-items: center;
    gap: 22px;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 550;
}

.header-right a:hover {
    color: var(--accent);
}

.version {
    padding: 3px 8px;
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--muted);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
}

/* Layout */

.page-shell {
    min-height: 100vh;
    padding-top: var(--header-height);
}

.sidebar {
    position: fixed;
    top: var(--header-height);
    bottom: 0;
    left: 0;
    z-index: 20;
    width: var(--sidebar-width);
    overflow-y: auto;
    padding: 28px 17px;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
}

.sidebar-group {
    margin-bottom: 25px;
}

.sidebar-label {
    margin: 0 0 8px 10px;
    color: #87939a;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: .09em;
    text-transform: uppercase;
}

.sidebar a {
    display: block;
    margin: 2px 0;
    padding: 7px 10px;
    border-radius: 5px;
    color: var(--text-secondary);
    font-size: 12.5px;
    line-height: 1.5;
    transition: background .15s ease, color .15s ease;
}

.sidebar a:hover {
    background: #eef2f3;
    color: var(--text);
}

.sidebar a.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 650;
}

.main {
    min-width: 0;
    margin-left: var(--sidebar-width);
}

.content {
    width: min(calc(100% - 64px), var(--content-width));
    margin: 0 auto;
    padding: 42px 0 75px;
}

/* Breadcrumb */

.breadcrumb {
    margin-bottom: 25px;
    color: var(--muted);
    font-size: 11px;
}

.breadcrumb a {
    color: var(--text-secondary);
}

.breadcrumb a:hover {
    color: var(--accent);
}

.breadcrumb-separator {
    padding: 0 7px;
    color: #b4bdc2;
}

/* Page heading */

.page-heading {
    padding-bottom: 23px;
    margin-bottom: 30px;
    border-bottom: 1px solid var(--border);
}

.page-kicker {
    margin-bottom: 8px;
    color: var(--accent);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: .09em;
    text-transform: uppercase;
}

.page-heading h1 {
    margin: 0;
    font-size: clamp(29px, 4vw, 38px);
    font-weight: 700;
    letter-spacing: -1.2px;
    line-height: 1.2;
}

.page-heading p {
    max-width: 680px;
    margin: 12px 0 0;
    color: var(--text-secondary);
    font-size: 14px;
}

.intro {
    max-width: 760px;
    margin: 0 0 32px;
    color: var(--text-secondary);
    font-size: 14px;
}

/* Documentation index */

.section {
    margin: 36px 0;
}

.section-heading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 10px;
    margin-bottom: 13px;
    border-bottom: 1px solid var(--border);
}

.section-heading h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 680;
    letter-spacing: -.3px;
}

.section-count {
    color: var(--muted);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 10px;
}

.doc-list {
    border: 1px solid var(--border);
    border-radius: 7px;
    overflow: hidden;
}

.doc-link {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 13px 16px;
    background: var(--background);
    border-bottom: 1px solid var(--border);
    transition: background .15s ease;
}

.doc-link:last-child {
    border-bottom: 0;
}

.doc-link:hover {
    background: var(--surface);
}

.doc-link-title {
    display: block;
    margin-bottom: 2px;
    color: var(--text);
    font-size: 13px;
    font-weight: 620;
}

.doc-link-description {
    display: block;
    color: var(--muted);
    font-size: 11px;
}

.doc-link-arrow {
    flex-shrink: 0;
    color: var(--muted);
    font-size: 15px;
}

.doc-link:hover .doc-link-arrow {
    color: var(--accent);
}

/* Search */

.search-area {
    margin: 25px 0 35px;
}

.search-box {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 13px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--background);
}

.search-box:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(8, 127, 104, .08);
}

.search-icon {
    color: var(--muted);
    font-size: 16px;
}

.search-input {
    width: 100%;
    padding: 12px 0;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
}

.search-input::placeholder {
    color: #9aa5ab;
}

.empty-state {
    display: none;
    padding: 22px;
    border: 1px dashed var(--border);
    border-radius: 6px;
    color: var(--muted);
    text-align: center;
    font-size: 12px;
}

/* Article */

.article {
    max-width: 820px;
}

.article h1 {
    margin: 0 0 25px;
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -1px;
    line-height: 1.25;
}

.article h2 {
    margin: 42px 0 14px;
    padding-bottom: 9px;
    border-bottom: 1px solid var(--border);
    font-size: 23px;
    font-weight: 680;
    letter-spacing: -.5px;
}

.article h3 {
    margin: 29px 0 10px;
    font-size: 17px;
    font-weight: 650;
}

.article h4 {
    margin: 24px 0 8px;
    font-size: 14px;
    font-weight: 650;
}

.article p {
    margin: 14px 0;
    color: var(--text-secondary);
}

.article ul,
.article ol {
    padding-left: 24px;
    color: var(--text-secondary);
}

.article li {
    margin: 7px 0;
    padding-left: 3px;
}

.article li::marker {
    color: var(--accent);
}

.article strong {
    color: var(--text);
    font-weight: 650;
}

.article em {
    color: var(--text-secondary);
}

.article a {
    color: var(--accent);
    text-decoration: underline;
    text-underline-offset: 3px;
}

.article a:hover {
    color: var(--accent-hover);
}

.article hr {
    height: 1px;
    margin: 30px 0;
    border: 0;
    background: var(--border);
}

.article pre {
    overflow-x: auto;
    padding: 17px 19px;
    margin: 20px 0;
    border: 1px solid var(--code-border);
    border-radius: 6px;
    background: var(--code-background);
    color: #263b36;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
    line-height: 1.8;
    tab-size: 4;
}

.article code {
    padding: 2px 5px;
    border: 1px solid var(--code-border);
    border-radius: 4px;
    background: var(--code-background);
    color: #176b59;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: .9em;
}

.article pre code {
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
}

.article blockquote {
    padding: 12px 17px;
    margin: 20px 0;
    border-left: 3px solid var(--accent);
    background: var(--surface);
    color: var(--text-secondary);
}

.article table {
    width: 100%;
    margin: 20px 0;
    border-collapse: collapse;
    font-size: 12px;
}

.article th,
.article td {
    padding: 10px 12px;
    border: 1px solid var(--border);
    text-align: left;
}

.article th {
    background: var(--surface);
    color: var(--text);
    font-weight: 650;
}

.article td {
    color: var(--text-secondary);
}

/* Footer */

.footer {
    padding-top: 20px;
    margin-top: 65px;
    border-top: 1px solid var(--border);
    color: var(--muted);
    font-size: 11px;
}

.footer strong {
    color: var(--text);
}

/* Mobile */

.mobile-menu {
    display: none;
    padding: 6px 9px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: white;
    color: var(--text);
    cursor: pointer;
}

@media (max-width: 850px) {
    :root {
        --sidebar-width: 215px;
    }

    .content {
        width: calc(100% - 48px);
    }
}

@media (max-width: 650px) {
    :root {
        --header-height: 58px;
    }

    .site-header {
        padding: 0 15px;
    }

    .brand {
        min-width: 0;
    }

    .header-right {
        gap: 12px;
    }

    .header-right .hide-mobile {
        display: none;
    }

    .mobile-menu {
        display: inline-block;
    }

    .page-shell {
        display: block;
    }

    .sidebar {
        display: none;
        position: fixed;
        top: var(--header-height);
        right: 0;
        bottom: auto;
        width: 100%;
        max-height: 65vh;
        overflow-y: auto;
        padding: 16px;
        border-right: 0;
        border-bottom: 1px solid var(--border);
        box-shadow: 0 10px 20px rgba(0,0,0,.06);
    }

    .sidebar.open {
        display: block;
    }

    .sidebar-group {
        margin-bottom: 17px;
    }

    .main {
        margin-left: 0;
    }

    .content {
        width: calc(100% - 34px);
        padding: 27px 0 55px;
    }

    .article h1 {
        font-size: 29px;
    }

    .article h2 {
        font-size: 21px;
    }

    .article pre {
        padding: 13px;
        font-size: 11px;
    }

    .doc-link {
        padding: 12px;
    }
}
CSS

# ============================================================
# HTML HELPERS
# ============================================================

page_header() {
    local title="${1:-Kode Documentation}"
    local description="${2:-Official documentation for the Kode programming language.}"

    cat <<HTML
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="theme-color" content="#ffffff">
    <meta name="description" content="$description">
    <title>$title | Kode Documentation</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>

<header class="site-header">
    <a class="brand" href="index.html">
        <span class="brand-mark">K</span>
        <span>Kode</span>
        <span class="brand-label">Documentation</span>
    </a>

    <nav class="header-right">
        <a href="index.html">Docs</a>
        <a class="hide-mobile" href="errors.html">Errors</a>
        <a class="hide-mobile" href="api/index.html">API</a>
        <span class="version">v$VERSION</span>
        <button class="mobile-menu" id="menuToggle" type="button" aria-label="Toggle navigation">Menu</button>
    </nav>
</header>

<div class="page-shell">

<aside class="sidebar" id="sidebar">
    <div class="sidebar-group">
        <div class="sidebar-label">Overview</div>
        <a href="index.html">Introduction</a>
    </div>

    <div class="sidebar-group">
        <div class="sidebar-label">Language</div>
        <a href="cheatsheet.html">Quick reference</a>
        <a href="generics.html">Generics</a>
    </div>

    <div class="sidebar-group">
        <div class="sidebar-label">Developer guides</div>
        <a href="editors.html">Editor setup</a>
    </div>

    <div class="sidebar-group">
        <div class="sidebar-label">Reference</div>
        <a href="errors.html">Compiler errors</a>
        <a href="api/index.html">Compiler API</a>
    </div>

    <div class="sidebar-group">
        <div class="sidebar-label">Project</div>
        <a href="whatsnew.html">What's new</a>
    </div>
</aside>

<main class="main">
<div class="content">
HTML
}

page_footer() {
    cat <<'HTML'
<footer class="footer">
    <strong>Kode</strong> · Programming Language Documentation
    <br>
    Technical documentation for developers.
</footer>
</div>
</main>
</div>

<script>
const menuToggle = document.getElementById("menuToggle");
const sidebar = document.getElementById("sidebar");

if (menuToggle && sidebar) {
    menuToggle.addEventListener("click", () => {
        sidebar.classList.toggle("open");
    });
}
</script>

</body>
</html>
HTML
}

# ============================================================
# MARKDOWN RENDERER
# ============================================================

md2html() {
    local source="$1"
    local destination="$2"
    local title="$3"
    local relative_css="${4:-style.css}"

    {
        cat <<HTML
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="theme-color" content="#ffffff">
    <title>$title | Kode Documentation</title>
    <link rel="stylesheet" href="$relative_css">
</head>
<body>
<header class="site-header">
    <a class="brand" href="$([[ "$relative_css" == "style.css" ]] && echo "index.html" || echo "../index.html")">
        <span class="brand-mark">K</span>
        <span>Kode</span>
        <span class="brand-label">Documentation</span>
    </a>
    <nav class="header-right">
        <a href="$([[ "$relative_css" == "style.css" ]] && echo "index.html" || echo "../index.html")">Docs</a>
        <span class="version">v$VERSION</span>
    </nav>
</header>
<div class="page-shell">
<aside class="sidebar">
    <div class="sidebar-group">
        <div class="sidebar-label">Overview</div>
        <a href="$([[ "$relative_css" == "style.css" ]] && echo "index.html" || echo "../index.html")">Introduction</a>
    </div>
    <div class="sidebar-group">
        <div class="sidebar-label">Reference</div>
        <a href="$([[ "$relative_css" == "style.css" ]] && echo "errors.html" || echo "../errors.html")">Compiler errors</a>
    </div>
</aside>
<main class="main">
<div class="content">
<div class="breadcrumb">
    <a href="$([[ "$relative_css" == "style.css" ]] && echo "index.html" || echo "../index.html")">Documentation</a>
    <span class="breadcrumb-separator">/</span>
    $title
</div>
<article class="article">
HTML
    } > "$destination"

    awk '
    function html_escape(value) {
        gsub(/&/, "\\&amp;", value)
        gsub(/</, "\\&lt;", value)
        gsub(/>/, "\\&gt;", value)
        gsub(/"/, "\\&quot;", value)
        return value
    }

    function inline_format(value) {
        value = html_escape(value)

        # Inline code
        gsub(/`[^`]+`/, "<code>&</code>", value)

        # Bold
        gsub(/\*\*[^*]+\*\*/, "<strong>&</strong>", value)
        gsub(/__[^_]+__/, "<strong>&</strong>", value)

        # Italic
        gsub(/\*[^*]+\*/, "<em>&</em>", value)

        return value
    }

    function close_paragraph() {
        if (in_paragraph) {
            print "</p>"
            in_paragraph = 0
        }
    }

    function close_list() {
        if (list_type == "ul") print "</ul>"
        if (list_type == "ol") print "</ol>"
        list_type = ""
    }

    function close_code() {
        if (in_code) {
            print "</code></pre>"
            in_code = 0
        }
    }

    BEGIN {
        in_code = 0
        in_paragraph = 0
        list_type = ""
        first_heading = 0
    }

    /^```/ {
        close_paragraph()
        close_list()

        if (in_code) {
            close_code()
        } else {
            language = substr($0, 4)
            print "<pre><code>"
            in_code = 1
        }
        next
    }

    in_code {
        print html_escape($0)
        next
    }

    /^#### / {
        close_paragraph()
        close_list()
        print "<h4>" inline_format(substr($0, 6)) "</h4>"
        next
    }

    /^### / {
        close_paragraph()
        close_list()
        print "<h3>" inline_format(substr($0, 5)) "</h3>"
        next
    }

    /^## / {
        close_paragraph()
        close_list()
        print "<h2>" inline_format(substr($0, 4)) "</h2>"
        next
    }

    /^# / {
        close_paragraph()
        close_list()
        heading = substr($0, 3)
        if (!first_heading) {
            print "<h1>" inline_format(heading) "</h1>"
            first_heading = 1
        } else {
            print "<h2>" inline_format(heading) "</h2>"
        }
        next
    }

    /^---+$/ {
        close_paragraph()
        close_list()
        print "<hr>"
        next
    }

    /^> / {
        close_paragraph()
        close_list()
        print "<blockquote>" inline_format(substr($0, 3)) "</blockquote>"
        next
    }

    /^- / || /^\* / || /^\+ / {
        close_paragraph()
        if (list_type != "ul") {
            close_list()
            print "<ul>"
            list_type = "ul"
        }
        print "<li>" inline_format(substr($0, 3)) "</li>"
        next
    }

    /^[0-9]+[.)] / {
        close_paragraph()
        if (list_type != "ol") {
            close_list()
            print "<ol>"
            list_type = "ol"
        }
        line = $0
        sub(/^[0-9]+[.)] /, "", line)
        print "<li>" inline_format(line) "</li>"
        next
    }

    /^[[:space:]]*$/ {
        close_paragraph()
        close_list()
        next
    }

    {
        close_list()
        if (!in_paragraph) {
            print "<p>"
            in_paragraph = 1
        } else {
            printf " "
        }
        printf "%s", inline_format($0)
        print ""
    }

    END {
        close_paragraph()
        close_list()
        close_code()
    }
    ' "$source" >> "$destination"

    {
        echo '</article>'
        page_footer
    } >> "$destination"
}

# ============================================================
# DOCUMENTATION INDEX HELPERS
# ============================================================

add_document_section() {
    local pattern="$1"
    local prefix="$2"
    local heading="$3"
    local description="$4"

    shopt -s nullglob
    local files=( $pattern )
    shopt -u nullglob

    [[ ${#files[@]} -eq 0 ]] && return

    echo '<section class="section">'
    echo '<div class="section-heading">'
    echo "<h2>$heading</h2>"
    printf '<span class="section-count">%02d DOCUMENTS</span>\n' "${#files[@]}"
    echo '</div>'
    echo '<div class="doc-list">'

    for file in "${files[@]}"; do
        local name
        name="$(basename "$file" .md)"

        md2html "$file" "$OUT/$prefix-$name.html" "$name"

        echo '<a class="doc-link" href="'"$prefix-$name"'.html">'
        echo '<span>'
        echo '<span class="doc-link-title">'"$name"'</span>'
        echo '<span class="doc-link-description">'"$description"'</span>'
        echo '</span>'
        echo '<span class="doc-link-arrow">→</span>'
        echo '</a>'
    done

    echo '</div>'
    echo '</section>'
}

add_single_document() {
    local source="$1"
    local destination="$2"
    local title="$3"
    local section_title="$4"
    local description="$5"

    [[ -f "$source" ]] || return

    md2html "$source" "$OUT/$destination" "$title"

    cat <<HTML
<section class="section">
    <div class="section-heading">
        <h2>$section_title</h2>
        <span class="section-count">GUIDE</span>
    </div>
    <div class="doc-list">
        <a class="doc-link" href="$destination">
            <span>
                <span class="doc-link-title">$title</span>
                <span class="doc-link-description">$description</span>
            </span>
            <span class="doc-link-arrow">→</span>
        </a>
    </div>
</section>
HTML
}

# ============================================================
# API DOCUMENTATION
# ============================================================

if [[ -x "$ROOT/target/debug/kode" ]]; then
    echo "Generating compiler API documentation..."

    mkdir -p "$OUT/api"

    if ! "$ROOT/target/debug/kode" doc "$ROOT/examples" -o "$OUT/api" 2>/dev/null; then
        "$ROOT/target/debug/kode" doc "$ROOT/tests/run-pass" -o "$OUT/api" 2>/dev/null || true
    fi
else
    echo "Kode debug binary not found. Skipping generated API documentation."
fi

# ============================================================
# INDEX PAGE
# ============================================================

INDEX="$OUT/index.html"

{
    page_header "Documentation" "Official documentation for the Kode programming language."

    cat <<HTML
<div class="breadcrumb">Documentation <span class="breadcrumb-separator">/</span> Introduction</div>

<div class="page-heading">
    <div class="page-kicker">Kode Programming Language · v$VERSION</div>
    <h1>Documentation</h1>
    <p>
        Technical guides, language specifications, compiler references
        and project documentation for Kode.
    </p>
</div>

<p class="intro">
    Start with the language reference to understand Kode syntax and features.
    Use the guides and compiler references when building, testing and debugging
    Kode programs.
</p>

<div class="search-area">
    <div class="search-box">
        <span class="search-icon">⌕</span>
        <input
            id="docSearch"
            class="search-input"
            type="search"
            placeholder="Search documentation..."
            autocomplete="off"
            aria-label="Search documentation"
        >
    </div>
</div>

<div id="docSections">
HTML

    add_document_section \
        "$ROOT/docs/spec/*.md" \
        "spec" \
        "Language specification" \
        "Language rules, syntax and compiler behaviour."

    add_document_section \
        "$ROOT/docs/adr/*.md" \
        "adr" \
        "Architecture decisions" \
        "Technical decisions and design records."

    add_single_document \
        "$ROOT/docs/cheatsheet.md" \
        "cheatsheet.html" \
        "Quick reference" \
        "Quick reference" \
        "A compact reference for Kode syntax and examples."

    add_single_document \
        "$ROOT/docs/editors.md" \
        "editors.html" \
        "Editor setup" \
        "Developer guides" \
        "Configure your development environment for Kode."

    add_single_document \
        "$ROOT/docs/site/content/generics.md" \
        "generics.html" \
        "Generics" \
        "Language guides" \
        "Learn about generic types and functions."

    add_single_document \
        "$ROOT/docs/errors/index.md" \
        "errors.html" \
        "Compiler errors" \
        "Compiler reference" \
        "Understand compiler diagnostics and error codes."

    if [[ -f "$ROOT/CHANGELOG.md" ]]; then
        md2html "$ROOT/CHANGELOG.md" "$OUT/whatsnew.html" "What's new"
    fi

    if [[ -f "$OUT/api/index.html" ]]; then
        cat <<'HTML'
<section class="section">
    <div class="section-heading">
        <h2>Compiler API</h2>
        <span class="section-count">GENERATED</span>
    </div>
    <div class="doc-list">
        <a class="doc-link" href="api/index.html">
            <span>
                <span class="doc-link-title">API reference</span>
                <span class="doc-link-description">Generated documentation for the Kode compiler.</span>
            </span>
            <span class="doc-link-arrow">→</span>
        </a>
    </div>
</section>
HTML
    fi

    cat <<'HTML'
</div>

<div id="emptyState" class="empty-state">
    No matching documentation found.
</div>

<script>
const searchInput = document.getElementById("docSearch");
const sections = [...document.querySelectorAll("#docSections .section")];
const emptyState = document.getElementById("emptyState");

searchInput.addEventListener("input", () => {
    const query = searchInput.value.trim().toLowerCase();
    let visibleDocuments = 0;

    sections.forEach(section => {
        const links = [...section.querySelectorAll(".doc-link")];
        let visibleInSection = 0;

        links.forEach(link => {
            const matches = link.innerText.toLowerCase().includes(query);
            link.style.display = matches ? "flex" : "none";

            if (matches) {
                visibleInSection++;
                visibleDocuments++;
            }
        });

        section.style.display = visibleInSection > 0 ? "" : "none";
    });

    emptyState.style.display = visibleDocuments > 0 ? "none" : "block";
});
</script>
HTML

    page_footer
} > "$INDEX"

# ============================================================
# APPLY STYLESHEET TO GENERATED API PAGES
# ============================================================

if [[ -d "$OUT/api" ]]; then
    while IFS= read -r -d '' directory; do
        cp "$OUT/style.css" "$directory/style.css"
    done < <(find "$OUT/api" -type d -print0)

    while IFS= read -r -d '' file; do
        if ! grep -q 'stylesheet' "$file"; then
            sed -i.bak \
                's|</head>|<link rel="stylesheet" href="style.css"></head>|' \
                "$file"
            rm -f "$file.bak"
        fi
    done < <(find "$OUT/api" -type f -name '*.html' -print0)
fi

# ============================================================
# BUILD SUMMARY
# ============================================================

echo
echo "----------------------------------------"
echo " KODE DOCUMENTATION BUILD"
echo "----------------------------------------"
echo
echo " Version : $VERSION"
echo " Theme   : Professional documentation"
echo " Output  : $OUT"
echo " Status  : Complete"
echo
echo "Open documentation:"
echo "  open \"$INDEX\""
echo
