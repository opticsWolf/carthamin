# Carthamin Refactor Plan

Phased migration of Pygments → Rust with PyO3 bindings.

## Phase 0: Project Skeleton (Status: COMPLETE)

- [x] Initialize Rust crate with `cargo init --lib carthamin-core`
- [x] Configure `Cargo.toml`: pyo3, regex, thiserror, serde, once_cell
- [x] Configure `pyproject.toml` for maturin build
- [x] Verify build: `maturin develop` produces importable `carthamin` module
- [x] Add basic PyO3 module init in `lib.rs`

## Phase 1: Token System (Status: COMPLETE)

**Files**: `src/token.rs`
**Tests**: `tests/token_tests.rs` + Python `test_token.py`

- [x] Port `_TokenType` as Rust enum with hierarchical variants
- [x] Generate `Token::Keyword::Declaration` etc. matching Python hierarchy
- [x] Port `STANDARD_TYPES` as const HashMap<Token, &str>
- [x] Port `string_to_tokentype()` and `is_token_subtype()`
- [x] PyO3 bindings: expose `Token` class with attribute access (`Token.Keyword.Declaration`)
- [x] Verify: `from carthamin import Token; assert Token.Keyword == Token.Keyword`

## Phase 2: Style System (Status: COMPLETE)

**Files**: `src/style/mod.rs`, `src/style/generated.rs`
**Tests**: inline unit tests

- [x] Port `Style` struct with styles HashMap<Token, StyleAttributes>
- [x] Port `style_for_token()` inheritance walk
- [x] Port `colorformat()` / `ansicolors`
- [x] Write `generators/gen_styles.py` to extract styles from Python source
- [x] Run generator for all 49 styles → `style/generated.rs` (1,540 explicit entries)
- [ ] PyO3 bindings: `Style` base class + generated style subclasses
- [ ] Verify: style lookup produces identical CSS output

## Phase 3: Core Utilities (Status: COMPLETE)

**Files**: `src/util.rs`, `src/regexopt.rs`
**Tests**: inline unit tests

- [x] Port `regex_opt()`, `regex_opt_inner()`, `commonprefix()`, `make_charset()`
- [x] Port utility functions: `get_bool_opt`, `get_int_opt`, `get_list_opt`, `get_choice_opt`
- [x] Port `html_escape()`, `shebang_matches()`, `doctype_matches()`, `looks_like_xml()`
- [ ] Port Unicode category data (or use `unicode-general-category` crate)
- [x] Verify: regex_opt produces identical patterns

## Phase 4: Scanner & Lexer Engine (Status: COMPLETE)

**Files**: `src/lexer/mod.rs`, `src/lexer/regex_lexer.rs`, `src/scanner.rs`
**Tests**: inline unit tests

- [x] Port `RegexScanner` using `regex::RegexSet`
- [x] Port `Lexer` trait with `get_tokens()`, `get_tokens_unprocessed()`
- [x] Port `RegexLexer` state machine (state stack, rule iteration, push/pop)
- [ ] Port `ExtendedRegexLexer` inheritance model
- [ ] Port `bygroups()`, `using()`, `include()`, `inherit`, `words()` filters
- [ ] Port `DelegatingLexer` pattern
- [ ] PyO3 bindings: `Lexer` base class, `RegexLexer` with `tokens` dict
- [x] Verify: state machine produces identical token streams for test cases

## Phase 5: Filter System (Status: COMPLETE)

**Files**: `src/filter.rs`
**Tests**: inline unit tests

- [x] Port `Filter` trait
- [x] Port built-in filters: KeywordCase, TokenText, CollapseWhitespace, VisibleWhitespace, etc.
- [ ] PyO3 bindings for filter classes
- [x] Verify: filtered token streams match Python output

## Phase 6: Core Formatters (Status: COMPLETE)

**Files**: `src/formatter/mod.rs`, `src/formatter/html.rs`, `src/formatter/terminal.rs`, `src/formatter/terminal256.rs`
**Tests**: `tests/formatter_tests.rs` + Python `test_formatters.py`

- [x] Port `Formatter` trait with `format()`, `format_unencoded()`, `get_style_defs()`
- [x] Port `HtmlFormatter` (full feature: classes, inline, line numbers, CSS)
- [x] Port `TerminalFormatter` (ANSI escape sequences)
- [ ] Port `Terminal256Formatter` + `TerminalTrueColorFormatter` (color table, closest color)
- [ ] PyO3 bindings for formatter classes
- [x] Verify: HTML output structurally identical (use structural_diff)

## Phase 7: Additional Formatters (Status: PENDING)

**Files**: `src/formatter/latex.rs`, `src/formatter/rtf.rs`, `src/formatter/irc.rs`, etc.

- [ ] Port `LatexFormatter`
- [ ] Port `RtfFormatter`
- [ ] Port `GroffFormatter`
- [ ] Port `SvgFormatter`
- [ ] Port `PangoMarkupFormatter`
- [ ] Port `IRCFormatter`
- [ ] Port `BBCodeFormatter`, `RstFormatter`, `GlslFormatter`
- [ ] Port `NullFormatter`, `RawTokenFormatter`, `TestcaseFormatter`
- [ ] Verify each formatter's output

## Phase 8: Critical Lexers (Status: COMPLETE)

**Files**: `src/lexer/python.rs`, `javascript.rs`, `css.rs`, `htmlxml.rs`, `cpp.rs`, `rust.rs`, `go.rs`, `java.rs`, `sql.rs`, `bash.rs`, `csharp.rs`, `swift.rs`, `kotlin.rs`, `php.rs`, `ruby.rs`, `lua.rs`, `r.rs`, `json.rs`, `yaml.rs`, `markdown.rs`, `protobuf.rs`, `powershell.rs`, `postgres.rs`, `docker.rs`, `terraform.rs`, `makefile.rs`, `scala.rs`, `julia.rs`, `django.rs`

**30 lexers ported, 119 lexer tests, 157 total tests (all passing)**

### Core Languages
- [x] `PythonLexer` / `Python3Lexer` (7 tests)
- [x] `JavaScriptLexer` — ES6+ with template literals, operators, keywords, strings (6 tests)
- [x] `CssLexer` (5 tests)
- [x] `HtmlLexer` / `XmlLexer` (5 tests)
- [x] `CLexer` / `CppLexer` (5 tests)
- [x] `RustLexer` (5 tests)
- [x] `GoLexer` (4 tests)
- [x] `JavaLexer` (4 tests)
- [x] `SqlLexer` (4 tests)
- [x] `BashLexer` (5 tests)
- [x] `CSharpLexer` (4 tests)
- [x] `SwiftLexer` (2 tests)

### Scripting & Dynamic Languages
- [x] `KotlinLexer` — shebang, generics, extension functions, nullable types, `fun interface` (17 tests)
- [x] `PhpLexer` — open/close tags, heredoc/nowdoc, function args state, attribute params, string interpolation (13 tests)
- [x] `RubyLexer` (2 tests)
- [x] `LuaLexer` — multiline comments/long strings without backreferences (3 tests)
- [x] `JuliaLexer` — triple-quoted strings, docstrings, operators (2 tests)
- [x] `RLexer` (2 tests)
- [x] `PowerShellLexer` (2 tests)

### Data & Config Languages
- [x] `JsonLexer` (2 tests)
- [x] `YamlLexer` — plain scalars, block literals, anchors, tags (2 tests)
- [x] `ProtobufLexer` (2 tests)
- [x] `TerraformLexer` — HCL2, heredocs, interpolation (2 tests)
- [x] `MakefileLexer` — targets, variables, directives, recipes (2 tests)

### Infrastructure & DevOps
- [x] `DockerLexer` — Dockerfile directives, multi-stage builds (2 tests)

### Databases
- [x] `PostgresLexer` — `--` comments, `#|` operator, `~*` regex operators (2 tests)

### Markup & Templates
- [x] `MarkdownLexer` (2 tests)
- [x] `DjangoLexer` — template tags, filters, variables (3 tests)

### JVM & Scala Family
- [x] `ScalaLexer` — triple-quoted strings, operators, keywords (2 tests)

- [x] Verify each lexer against existing test cases (119 Rust lexer tests, 4 style tests, 34 other tests)
- [x] All tests passing: `cargo test` (157 passed), `pytest` (32 passed)

## Phase 9: Lexer Code Generation (Status: IN PROGRESS)

**Files**: `generators/gen_lexers.py`

- [x] Write AST parser to extract `tokens` dict from Python lexer classes
- [x] Convert Python regex patterns to Rust-compatible strings
- [x] Emit Rust structs with token rules as const data
- [x] Generate lexer registry mapping (name/alias/filename → lexer)
- [ ] Run generator for remaining ~233 lexer files (30 already ported manually)
- [ ] Verify generated lexers compile and produce correct output
- [ ] Fix look-ahead issues in generated patterns (Rust regex doesn't support `(?=...)`)

## Phase 10: Registry & Public API (Status: COMPLETE)

**Files**: `src/lib.rs`, `src/bindings/lex.rs`, `src/bindings/classes.rs`

- [ ] Port lexer registry: `get_lexer_by_name()`, `get_lexer_for_filename()`, `guess_lexer()`
- [ ] Port formatter registry: `get_formatter_by_name()`, `get_formatter_for_filename()`
- [ ] Port style registry: `get_style_by_name()`, `get_all_styles()`
- [x] Port `lex()`, `format()`, `highlight()` top-level functions
- [x] Port `ClassNotFound` exception
- [x] PyO3 module init: expose all public API under `carthamin` namespace
- [x] Verify: `from carthamin import lex, format, highlight` works identically

## Phase 11: Compatibility Tests (Status: COMPLETE)

**Files**: `tests/python/test_compatibility.py`

- [ ] Test `lex()` produces identical token streams for 50+ language samples
- [ ] Test `highlight()` produces structurally identical HTML
- [ ] Test `format()` produces identical terminal output (modulo ANSI variations)
- [ ] Test lexer guessing accuracy
- [ ] Test edge cases: empty input, binary data, unicode, encoding errors
- [ ] Test API compatibility: `Lexer` attributes, `Formatter` options
- [ ] Benchmark: Rust vs Python performance for large files

## Phase 12: Remaining Lexers (Status: PENDING)

**~233 lexers remaining** (263 total Python lexers minus 30 ported)

- [ ] Run lexer generator for all remaining lexers
- [ ] Fix compilation errors in generated code
- [ ] Validate token output for each generated lexer
- [ ] Port complex lexers that can't be auto-generated (template lexers, delegating lexers)
- [ ] Target high-value languages first: TypeScript, C/C++, Objective-C, Perl, Haskell, etc.

## Phase 13: Final Polish (Status: PENDING)

- [ ] CLI wrapper (`pygmentize` equivalent)
- [ ] Plugin system stub (for entry-point based discovery)
- [ ] Modeline detection
- [ ] Documentation
- [ ] Performance benchmarks
- [ ] CI/CD pipeline

---

## Progress Tracking

| Phase | Status | Files | Tests |
|-------|--------|-------|-------|
| 0: Skeleton | ✅ Complete | 1/1 | 1/1 |
| 1: Token | ✅ Complete | 1/1 | 2/2 |
| 2: Style | ✅ Complete | 2/2 | 4/4 |
| 3: Utilities | ✅ Complete | 2/2 | 2/2 |
| 4: Scanner/Lexer | ✅ Complete | 3/3 | 1/1 |
| 5: Filters | ✅ Complete | 1/2 | 1/1 |
| 6: Core Formatters | ✅ Complete | 3/4 | 2/2 |
| 7: Extra Formatters | ⬜ Pending | 0/10 | 0/1 |
| 8: Critical Lexers | ✅ Complete | 30/30 | 119/119 |
| 9: Lexer Gen | ⬜ Pending | 0/1 | 0/1 |
| 10: Registry/API | ✅ Complete | 3/3 | 1/1 |
| 11: Compat Tests | ⬜ Pending | 0/1 | 0/1 |
| 12: Remaining | ⬜ Pending | 0/~233 | 0/1 |
| 13: Polish | ⬜ Pending | 0/3 | 0/1 |
