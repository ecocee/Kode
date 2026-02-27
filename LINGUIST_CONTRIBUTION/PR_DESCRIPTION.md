## Summary

This PR adds full GitHub Linguist support for the Kode programming language.

## Changes

- Added Kode language entry to `lib/linguist/languages.yml` with purple-violet color `#7C3AED`
- Added file extensions: `.kode` and `.kbc`
- Created three representative sample files in `samples/Kode/`:
  - `hello_world.kode` - Basic "Hello, World!" example
  - `loops.kode` - Demonstrates for/while loops and array iteration
  - `functions.kode` - Shows function definitions and recursion

## Language Details

**Name**: Kode  
**Type**: Programming Language  
**Extensions**: `.kode`, `.kbc`  
**Color**: `#7C3AED` (Purple-Violet)  
**TextMate Scope**: `source.kode`  

The Kode language provides:
- Modern syntax with type inference
- First-class functions and closures
- Powerful module system
- Compile-to-bytecode architecture
- Comprehensive error handling

## Grammar Support

The Kode repository includes full TextMate grammar support (`kode.tmLanguage.json`) enabling syntax highlighting across compatible editors and GitHub's preview system.

**Repository**: https://github.com/ecocee/kode

## Testing

- YAML syntax validated
- No trailing whitespace
- Extensions properly formatted
- Sample files are syntactically correct and representative
- Color hex code verified
- Alphabetical ordering maintained in languages.yml

## After Merge

GitHub will automatically:
- Recognize `.kode` and `.kbc` files as Kode language
- Display repositories in language statistics with the purple-violet color
- Apply TextMate syntax highlighting to Kode files
