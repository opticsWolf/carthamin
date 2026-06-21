# Carthamin Code Map

AST-derived structural overview of the Pygments codebase, mapped for Rust migration.

## 1. Core Engine (12 modules — must be ported to Rust)

### 1.1 `token.py` — Token Type Tree
- **Class**: `_TokenType(tuple)` — dynamic tree of token types via `__getattr__`
- **Singletons**: `Token`, `Text`, `Whitespace`, `Escape`, `Error`, `Other`, `Keyword`, `Name`, `Literal`, `String`, `Number`, `Punctuation`, `Operator`, `Comment`, `Generic`
- **Functions**: `is_token_subtype()`, `string_to_tokentype()`
- **Data**: `STANDARD_TYPES` dict (token → CSS class name mapping, ~90 entries)
- **Rust equivalent**: `Token` enum with nested variants, `STANDARD_TYPES` as const HashMap
- **Complexity**: Low (data structure + helpers)

### 1.2 `lexer.py` — Lexer Base Classes
- **Classes**:
  - `Lexer` — base lexer with `get_tokens()`, `get_tokens_unprocessed()`, `analyse_text()`, options handling
  - `RegexLexer` — regex-driven state machine; `tokens` attribute (state → rules), `get_tokens_unprocessed()` with state stack
  - `ExtendedRegexLexer` — supports `inherit` and `_tokens` for composition
  - `CompilationMode` — enum (End, Initial)
  - `LexerContext` — mutable context for debugging/profiling
  - `DelegatingLexer` — delegates to sub-lexers by detecting language blocks
  - `TextLexer` — returns all tokens as `Text`
- **Functions**: `inherit`, `bygroups`, `using`, `this`, `include`, `words()`, `do_insertions()`
- **Key methods**:
  - `RegexLexer.get_tokens_unprocessed()` — core state machine loop: iterate rules, match regex, push/pop states, yield tokens
  - `bygroups()` — filter that maps capture groups to token types
  - `using(OtherLexer)` — delegate a rule to another lexer
  - `words((keywords), tokentype)` — generate optimized keyword regex via `regex_opt`
- **Rust equivalent**: `Lexer` trait, `RegexLexer` struct, `ExtendedRegexLexer` struct with full feature support ✅
- **Bug fix**: `LexerAction::Noop` now emits `rule.pattern.token` (was silently consuming text)
- **Bug fix**: `PythonLexer` triple-quote constant corrected (`TRIPLE_DQ` = 3 quotes, not 2)
- **Complexity**: **High** (core engine logic)
- **Rust files**: `src/lexer/mod.rs` (Lexer, RegexLexer), `src/lexer/extended.rs` (ExtendedRegexLexer, DelegatingLexer, bygroups, using, include, inherit, combined)

### 1.3 `scanner.py` — Regex Scanner
- **Class**: `RegexScanner` — wraps compiled regex patterns with token type metadata
- **Methods**: `get_ranges()`, `search()` — return (start, end, token_type, match) tuples
- **Rust equivalent**: `RegexScanner` struct using `regex::RegexSet` or `aho_corasick`
- **Complexity**: Medium

### 1.4 `regexopt.py` — Regex Optimization
- **Functions**: `regex_opt()`, `regex_opt_inner()`, `commonprefix()`, `make_charset()`
- **Purpose**: Generate optimal regex from keyword lists (shared prefix/suffix extraction)
- **Rust equivalent**: Pure functions, straightforward port
- **Complexity**: Low

### 1.5 `style.py` — Style System
- **Class**: `StyleMeta` (metaclass) — builds style inheritance tree, `style_for_token()`, `colorformat()`
- **Class**: `Style` — base class; subclasses define `styles` dict
- **Data**: `_ansimap`, `ansicolors`
- **Rust equivalent**: `Style` struct with HashMap<Token, StyleAttributes>
- **Complexity**: Low

### 1.6 `formatter.py` — Formatter Base
- **Class**: `Formatter` — base with `format()`, `format_unencoded()`, `get_style_defs()`, encoding handling
- **Function**: `_lookup_style()`
- **Rust equivalent**: `Formatter` trait
- **Complexity**: Low

### 1.7 `filter.py` — Filter Base
- **Class**: `Filter` — base filter with `add_to lexer()`
- **Class**: `SimpleFilter` — applies transform to token stream
- **Rust equivalent**: `Filter` trait
- **Complexity**: Low

### 1.8 `filters/__init__.py` — Built-in Filters
- **Classes**: `KeywordCaseFilter`, `TokenTextFilter`, `CollapseWhitespace`, `VisibleWhitespace`, `BacktraceFilter`, `InsertTextFilter`, `StripCommentsFilter`, `StripStringsFilter`, `highlighting_filter`
- **Rust equivalent**: Structs implementing `Filter` trait
- **Complexity**: Low-Medium

### 1.9 `util.py` — Utilities
- **Classes**: `ClassNotFound`, `OptionError`, `Future`, `UnclosingTextIOWrapper`
- **Functions**: `get_bool_opt()`, `get_int_opt()`, `get_list_opt()`, `get_choice_opt()`, `docstring_headline()`, `make_analysator()`, `text_analyse()`, `shebang_matches()`, `doctype_matches()`, `looks_like_xml()`, `surrogatepair()`, `format_lines()`, `duplicates_removed()`, `guess_decode()`, `html_escape()`
- **Rust equivalent**: Utility functions, error types
- **Complexity**: Low

### 1.10 `unistring.py` — Unicode Data
- **Purpose**: Pre-computed Unicode character category strings (Cc, Cf, Nd, etc.)
- **Rust equivalent**: Embedded static strings or use `unicode-general-category` crate
- **Complexity**: Low (data only)

### 1.11 `console.py` — Console Helpers
- **Functions**: `colorize()`, `ansiformat()`, `reset_color()`
- **Rust equivalent**: Simple ANSI escape functions
- **Complexity**: Low

### 1.12 `cmdline.py` — CLI Entry Point
- **Functions**: `main()`, `OutputError`, `UsageError`, `_parse_options()`, `return_code()`
- **Rust equivalent**: CLI via `clap` or keep as Python wrapper
- **Complexity**: Medium (I/O heavy)

## 2. Registry Modules (4 modules — lookup/discovery)

### 2.1 `lexers/__init__.py` — Lexer Registry
- **Functions**: `get_all_lexers()`, `find_lexer_class()`, `find_lexer_class_by_name()`, `get_lexer_by_name()`, `get_lexer_for_filename()`, `get_lexer_for_mimetype()`, `guess_lexer()`, `load_lexer_from_file()`
- **Data**: Uses `LEXERS` mapping from `_mapping.py`
- **Rust equivalent**: Registry struct with lazy loading

### 2.2 `lexers/_mapping.py` — Lexer Mapping
- **Data**: `LEXERS` dict — (name, aliases, filenames, mimetypes) → class path for 263+ lexers
- **Rust equivalent**: Generated from Python source or embedded as static data

### 2.3 `formatters/__init__.py` — Formatter Registry
- **Functions**: `get_all_formatters()`, `find_formatter_class()`, `get_formatter_by_name()`, `get_formatter_for_filename()`
- **Data**: Uses `FORMATTERS` mapping from `_mapping.py`

### 2.4 `styles/__init__.py` — Style Registry
- **Functions**: `get_all_styles()`, `get_style_by_name()`, `get_style_by_archive_css()`
- **Data**: Uses `STYLES` mapping

## 3. Formatters (14 files)

| # | File | Key Classes | Complexity | Priority |
|---|------|-------------|------------|----------|
| 1 | `html.py` | `HtmlFormatter` | High | **Critical** |
| 2 | `terminal.py` | `TerminalFormatter` | Medium | **Critical** |
| 3 | `terminal256.py` | `Terminal256Formatter`, `TerminalTrueColorFormatter`, `EscapeSequence` | Medium | **Critical** |
| 4 | `latex.py` | `LatexFormatter`, `LatexEmbeddedLexer` | High | High |
| 5 | `rtf.py` | `RtfFormatter` | Medium | High |
| 6 | `irc.py` | `IRCFormatter` | Low | **✅ Done** |
| 7 | `groff.py` | `GroffFormatter` | Low | Medium |
| 7 | `groff.py` | `GroffFormatter` | Low | Medium |
| 8 | `pangomarkup.py` | `PangoMarkupFormatter` | Low | Medium |
| 9 | `svg.py` | `SvgFormatter` | Low | Medium |
| 10 | `bbcode.py` | `BBCodeFormatter` | Low | **✅ Done** |
| 11 | `rst.py` | `RstFormatter` | Low | Low |
| 12 | `glsl.py` | `GlslFormatter` | Low | Low |
| 13 | `html_witer.py` | `HtmlWi`terFormatter` | Low | Low |
| 14 | `other.py` | `NullFormatter`, `RawTokenFormatter`, `TestcaseFormatter` | Low | **✅ Done** |

## 4. Styles (48 files)

All styles follow identical pattern: subclass `Style`, define `styles` dict mapping `Token.X` → CSS property string.

**High-priority styles**: default, emacs, friendly, monokai, vim, vs, xcode, tango, perldoc, autumn, abap, paraiso-light, paraiso-dark, solarized-dark, solarized-light, fruity, bw, pastie, parmed, rainbow_dash, manni, material, nord, gruvbox, one-dark, nord-darker, zenground, lilypond, git, solarized-dark, sas, staroffice, stata, stata-light, stata-dark, inkpot, mlx, native, emacs,friendly-grayscale, github, duotone-dark, duotone-light, lightz, neon

**Rust approach**: Code-gen from Python source — extract `styles` dict, emit Rust HashMap.

## 5. Lexers (263 files in 30+ subdirectories)

### 5.1 Lexer Categories

| Category | Files | Complexity | Notes |
|----------|-------|------------|-------|
| **Data-driven** (keywords + patterns) | ~180 | Low | Auto-generatable |
| **State-machine** (nested states) | ~50 | Medium | Manual port needed |
| **Composition** (delegating/embedded) | ~20 | High | Complex inheritance |
| **Template lexers** | ~13 | Medium | Django, Jinja, Mako, etc. |

### 5.2 Lexer Status

**458 lexers total** (28 manual + 430 auto-generated) out of 598 in Pygments.

### 5.2 Key Lexer Files (by priority)

**Critical** (port first):
- `python.py` — Python2Lexer, Python3Lexer, PythonConsoleLexer, etc.
- `javascript.py` — JavaScriptLexer, CoffeeScriptLexer
- `css.py` — CssLexer, ScssLexer, LessCssLexer
- `html.py` — HtmlLexer, RstLexer, XmlLexer, DTDLexer
- `c_like.py` — CLexer, GLSLLexer, CudaLexer, ObjectiveCLexer
- `c_cpp.py` — CppLexer
- `rust.py` — RustLexer
- `go.py` — GoLexer
- `java.py` — JavaLexer
- `sql.py` — SqlLexer, MySqlLexer, PostgresLexer, TransactSqlLexer

**High-priority**:
- `perl6.py`, `ruby.py`, `php.py`, `tcl.py`, `lua.py`, `dart.py`, `kotlin.py`, `swift.py`, `r.py`, `julia.py`, `scala.py`, `clojure.py`, `lisp.py`, `scheme.py`, `elixir.py`, `erlang.py`, `haskell.py`, `idlang.py`, `apl.py`, `ada.py`, `d.py`, `nim.py`, `crystal.py`, `v.py`, `zig.py`

**Medium-priority** (automation candidates):
- Config parsers: `python.py` (ini), `yaml.py`, `toml_lx.py`, `json.py`
- Markup: `markdown.py`, `restructuredtext.py`, `tex.py`, `bibtex.py`
- Data: `csv.py`, `xml.py`, `html.py`
- Shell: `bash.py`, `powershell.py`, `bat.py`
- Build: `make.py`, `cmake.py`, `m4.py`
- DevOps: `docker.py`, `saltstack.py`, `terraform_lx.py`

**Low-priority** (niche):
- Legacy: `cobol.py`, `fortran.py`, `plpgsql.py`
- Domain-specific: `tnt.py`, `func.py`, `moc.py`
- Embedded: `scilab.py`, `octave.py`

### 5.3 Lexer Inheritance Graph (key chains)

```
Lexer
├── RegexLexer
│   ├── ExtendedRegexLexer
│   │   ├── PythonLexer → Python3Lexer
│   │   ├── CLexer → GLSLLexer, CudaLexer
│   │   └── ... (~150 subclasses)
│   ├── DelegatingLexer
│   └── ScriptingLexer (template host)
├── TextLexer
└── Lexer (direct subclasses for non-regex)
```

## 6. Supporting Modules

| Module | Purpose | Port? |
|--------|---------|-------|
| `plugin.py` | Entry-point based plugin discovery | Python-only wrapper |
| `modeline.py` | Editor modeline detection | Yes (regex) |
| `sphinxext.py` | Sphinx documentation directive | No (doc tool) |
| `__init__.py` | Public API: `lex()`, `format()`, `highlight()` | PyO3 bindings |

## 7. Dependency Graph

```
token.py ← lexer.py ← scanner.py
            ↑           ↑
         regexopt.py  style.py
            ↑
         lexer.py ← filter.py ← filters/__init__.py
            ↑
         formatter.py ← formatters/*.py
            ↑
         util.py ← (everything)
            ↑
         unistring.py ← lexer.py
```

## 8. Rust Module Structure (Target)

```
carthamin/
├── src/
│   ├── lib.rs              # PyO3 module init
│   ├── token.rs            # Token enum + STANDARD_TYPES
│   ├── lexer/
│   │   ├── mod.rs          # Lexer trait, RegexLexer
│   │   ├── regex_lexer.rs  # Re-export
│   │   ├── extended.rs     # ExtendedRegexLexer, DelegatingLexer, bygroups, using, include, inherit, combined
│   │   ├── scanner.rs      # RegexScanner
│   │   └── registry.rs     # Lexer lookup
│   ├── formatter/
│   │   ├── mod.rs          # Formatter trait
│   │   ├── html.rs
│   │   ├── terminal.rs
│   │   ├── terminal256.rs
│   │   ├── latex.rs
│   │   ├── rtf.rs
│   │   └── ...
│   ├── style/
│   │   ├── mod.rs          # Style struct
│   │   └── generated.rs    # Auto-generated style data
│   ├── filter/
│   │   ├── mod.rs          # Filter trait
│   │   └── builtins.rs
│   ├── regexopt.rs         # regex_opt functions
│   ├── util.rs             # Utility functions
│   └── bindings/
│       ├── lex.rs          # lex(), highlight()
│       └── classes.rs      # Lexer/Formatter Python classes
├── generators/             # Code-gen tools
│   ├── gen_lexers.py       # Python → Rust lexer converter
│   └── gen_styles.py       # Python → Rust style converter
└── tests/
    ├── token_tests.rs
    ├── lexer_tests.rs
    └── python/             # Pytest compatibility tests
```
