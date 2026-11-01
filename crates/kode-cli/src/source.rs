//! Several source files (a program and its imports) share one offset space,
//! so every span in the compiler points at exactly one place.

use kode_diagnostics::{Diagnostic, Span};
use kode_mir::LineMap;

pub struct SourceFile {
    pub name: String,
    pub text: String,
    /// Offset of the first byte of this file in the shared space.
    pub start: usize,
}

#[derive(Default)]
pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

impl SourceMap {
    /// Add a file and return its start offset.
    pub fn add(&mut self, name: String, text: String) -> usize {
        let start = self.files.last().map(|f| f.start + f.text.len() + 1).unwrap_or(0);
        self.files.push(SourceFile { name, text, start });
        start
    }

    fn locate(&self, offset: usize) -> &SourceFile {
        self.files
            .iter()
            .rev()
            .find(|f| f.start <= offset)
            .unwrap_or(&self.files[0])
    }

    fn local(&self, d: &Diagnostic) -> (&SourceFile, Diagnostic) {
        let f = self.locate(d.span.start);
        let mut d2 = d.clone();
        d2.span = Span::new(
            d.span.start.saturating_sub(f.start),
            d.span.end.saturating_sub(f.start),
        );
        (f, d2)
    }

    pub fn render(&self, d: &Diagnostic) -> String {
        let (f, d2) = self.local(d);
        d2.render(&f.name, &f.text)
    }

    pub fn json(&self, d: &Diagnostic) -> String {
        let (f, d2) = self.local(d);
        d2.to_json(&f.name, &f.text)
    }
}

impl LineMap for SourceMap {
    fn line_of(&self, offset: usize) -> i64 {
        let f = self.locate(offset);
        let local = offset.saturating_sub(f.start).min(f.text.len());
        f.text[..local].matches('\n').count() as i64 + 1
    }
}