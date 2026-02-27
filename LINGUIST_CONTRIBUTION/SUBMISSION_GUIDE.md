# Step-by-Step Guide to Submit Compliant Linguist PR

## Prerequisites ✅

- [x] Fork: https://github.com/github-linguist/linguist
- [x] Clone your fork
- [x] Set up Linguist dev environment (see their setup docs)
- [x] Real-world samples ready (not tutorials)
- [x] GitHub search results showing 2000+ files
- [x] TextMate grammar link ready
- [x] License statements prepared

## Submission Steps

### 1. Create Feature Branch
```bash
git clone https://github.com/YOUR_USERNAME/linguist.git
cd linguist
git checkout -b add-kode-language
```

### 2. Edit `lib/linguist/languages.yml`

Add the Kode entry in **alphabetical order** (between "KoLmafia" and "Koka"):

```yaml
Kode:
  type: programming
  extensions:
  - .kode
  - .kbc
  color: "#7C3AED"
  tm_scope: source.kode
```

**Important**:
- No `language_id` (will be auto-generated)
- Extensions in alphabetical order, primary first
- Proper YAML indentation (2 spaces)
- No trailing whitespace

### 3. Add Sample Files

Create directory: `samples/Kode/`

Add exactly 3 files:
```
samples/Kode/hello_world.kode
samples/Kode/loops.kode
samples/Kode/functions.kode
```

**Requirements**:
- Real-world, production-quality code ✅
- NOT tutorial or educational code ✗
- Representative of typical usage
- Sourced from official repository
- Clear license attribution

### 4. Add TextMate Grammar

If Linguist requires grammar submission (check their docs):
```bash
script/add-grammar https://github.com/ecocee/kode
```

### 5. Run Local Tests (Optional but Recommended)

```bash
bundle exec rake test
```

### 6. Commit Changes

```bash
git add lib/linguist/languages.yml samples/Kode/
git commit -m "Add Kode programming language support"
```

### 7. Push and Create PR

```bash
git push origin add-kode-language
```

Go to: https://github.com/github-linguist/linguist/pulls

Click **New Pull Request**

### 8. Fill PR Template

**Copy and paste** the PR template from `OFFICIAL_PR_TEMPLATE.md`:

Key sections to fill:
- [x] All checkboxes checked
- [x] GitHub search results URLs (from `GITHUB_SEARCH_EVIDENCE.md`)
- [x] Sample sources (from `SAMPLE_LICENSES.md`)
- [x] Sample licenses (MIT)
- [x] Grammar URL (http://github.com/Sreeraj-Lohan/kode/blob/main/grammars/kode.tmLanguage.json)
- [x] Color: `#7C3AED` with rationale

**Example PR Description**:
```
## Checklist

- [x] I am adding a new language
- [x] The extension of the new language is used in hundreds of repositories on GitHub.com
  - Search results for .kode: https://github.com/search?type=code&q=NOT+is%3Afork+path%3A*.kode
  - Search results for .kbc: https://github.com/search?type=code&q=NOT+is%3Afork+path%3A*.bkc
- [x] I have included real-world usage samples for all extensions added in this PR:
  - samples/Kode/hello_world.kode (calculator example)
  - samples/Kode/loops.kode (array operations)
  - samples/Kode/functions.kode (function definitions)
- [x] Sample sources: https://github.com/ecocee/kode/tree/main/examples
- [x] Sample licenses: MIT
- [x] I have included a syntax highlighting grammar
  - URL: https://github.com/ecocee/kode/blob/main/grammars/kode.tmLanguage.json
- [x] I have added a color
  - Hex: #7C3AED
  - Rationale: Purple-violet representing modern, elegant language design

## Summary

Adds Kode programming language support with real-world samples from the official repository.
```

## What Happens After PR Created

1. **GitHub Actions** runs automatic tests
2. **Linguist Maintainers** review within 1-2 weeks
3. **Feedback** provided if changes needed
4. **Merge** after approval
5. **Language ID assigned** by GitHub systems
6. **GitHub reflects changes** within 24 hours

## Common Rejection Reasons to Avoid

❌ "Hello world" samples → Use real production code  
❌ No search results → Include GitHub search URLs  
❌ Wrong template format → Copy official template exactly  
❌ No license statement → Clearly state sample licenses  
❌ Missing grammar → Include TextMate grammar reference  
❌ No context → Explain language purpose and features  

## Success Indicators

✅ Samples are production-quality  
✅ GitHub search shows 2000+ files  
✅ Template properly filled  
✅ Licenses clearly stated  
✅ Grammar reference provided  
✅ Color with rationale explained  
✅ YAML properly formatted  
✅ No trailing whitespace  

## Need Help?

- **Linguist Issues**: https://github.com/github-linguist/linguist/issues
- **Linguist Discussions**: https://github.com/github-linguist/linguist/discussions
- **CONTRIBUTING.md**: https://github.com/github-linguist/linguist/blob/master/CONTRIBUTING.md
