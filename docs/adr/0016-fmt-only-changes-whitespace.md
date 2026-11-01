# ADR 0016: `kode fmt` only changes whitespace
Status: accepted

The formatter re-indents lines and collapses blank lines. It never rewrites tokens, so it cannot
change meaning and cannot lose a comment. A fuller formatter that also normalizes spacing inside
lines can come later, once comments are part of the syntax tree.
