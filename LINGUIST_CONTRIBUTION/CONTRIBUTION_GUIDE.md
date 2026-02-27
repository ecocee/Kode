# Kode Language - GitHub Linguist Contribution

This directory contains the files necessary to add Kode language support to the GitHub Linguist project.

## Files to Contribute

### 1. lib/linguist/languages.yml

Add the following entry in alphabetical order in the `languages.yml` file:

```yaml
Kode:
  type: programming
  extensions:
  - .kode
  - .kbc
  color: "#7C3AED"
  tm_scope: source.kode
```

**Notes:**
- Maintains alphabetical ordering
- `type: programming` identifies Kode as a programming language
- Extensions: `.kode` (source files), `.kbc` (bytecode files)
- Color: `#7C3AED` - a modern purple-violet shade (Violet-600 equivalent)
- `tm_scope: source.kode` matches the TextMate grammar defined in the Kode repository

### 2. samples/Kode/

Create a new directory `samples/Kode/` with at least three representative samples:

- `hello_world.kode` - Basic output example
- `loops.kode` - Loop constructs demonstration
- `functions.kode` - Function definition and recursion example

## PR Title and Description Template

### Title
```
Add Kode programming language support
```

### Description
```
This PR adds support for the Kode programming language to Linguist.

## Changes Made
- Added Kode language entry to `lib/linguist/languages.yml`
- Added three representative samples in `samples/Kode/`:
  - `hello_world.kode` - Basic example with print statement
  - `loops.kode` - Demonstrates for/while loops and array iteration
  - `functions.kode` - Shows function definitions and recursion

## Language Details
- **Extensions**: `.kode`, `.kbc`
- **Type**: Programming
- **Color**: `#7C3AED` (Purple-Violet)
- **TextMate Scope**: `source.kode`

## TextMate Grammar
The Kode language includes a TextMate grammar file (`kode.tmLanguage.json`) in the [Kode repository](https://github.com/Sreeraj-Lohan/kode).

Fixes: N/A (initial addition)
```

## Contribution Steps

1. **Fork the Linguist repository**: https://github.com/github-linguist/linguist

2. **Clone your fork** and create a feature branch:
   ```bash
   git clone https://github.com/YOUR_USERNAME/linguist.git
   cd linguist
   git checkout -b add-kode-language
   ```

3. **Add the language entry**:
   - Edit `lib/linguist/languages.yml`
   - Add the Kode entry in alphabetical order

4. **Add sample files**:
   - Create `samples/Kode/` directory
   - Add `hello_world.kode`, `loops.kode`, and `functions.kode`

5. **Run tests** (on the linguist repo):
   ```bash
   bundle exec rake
   ```

6. **Commit and push**:
   ```bash
   git add lib/linguist/languages.yml samples/Kode/
   git commit -m "Add Kode programming language support"
   git push origin add-kode-language
   ```

7. **Create a Pull Request** on the Linguist repository with the description above

## Verification

After the PR is merged, GitHub will automatically:
- Recognize `.kode` files with the Kode language label
- Display files with the purple-violet color (`#7C3AED`)
- Provide syntax highlighting via the TextMate grammar

## Reference Files

- `languages.yml.entry` - Exact entry to add to Linguist's languages.yml
- `samples/` - Representative code examples
