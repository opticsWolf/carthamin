Carthamin — Pygments in Rust
=============================

**Carthamin** is a Rust reimplementation of `Pygments <https://pygments.org/>`_,
the generic syntax highlighter. It aims to provide **identical token output** to
the original Python library while delivering significantly better performance
through Rust's zero-cost abstractions.

Carthamin can be used:

* As a **native Rust library** for CLI tools, web backends, and editors.
* As a **Python package** via ``pyo3`` bindings, drop-in compatible with Pygments'
  API (``lex()``, ``format()``, ``highlight()``).

The original Pygments Python source lives in this repository under ``pygments/``
for reference and compatibility testing.

Quick Start
-----------

**Rust** — add to your ``Cargo.toml``::

    [dependencies]
    carthamin = { path = "carthamin-core" }

**Python** — build and install via maturin::

    cd carthamin-core
    pip install maturin
    maturin develop

Then use identically to Pygments::

    from carthamin import lex, format, highlight
    from carthamin import Token, Style

Architecture
------------

Carthamin mirrors Pygments' modular architecture, ported to idiomatic Rust:

+---------------------+-----------------------------+---------------------+
| Component           | Rust files                  | Status              |
+=====================+=============================+=====================+
| **Token System**    | ``src/token.rs``            | ✅ Complete         |
+---------------------+-----------------------------+---------------------+
| **Style System**    | ``src/style/mod.rs``        | ✅ Complete         |
|                     | ``src/style/generated.rs``  |                     |
+---------------------+-----------------------------+---------------------+
| **Core Utilities**  | ``src/util.rs``             | ✅ Complete         |
|                     | ``src/regexopt.rs``         |                     |
+---------------------+-----------------------------+---------------------+
| **Scanner & Lexer** | ``src/scanner.rs``          | ✅ Complete         |
|                     | ``src/lexer/mod.rs``        |                     |
|                     | ``src/lexer/regex_lexer.rs``|                     |
|                     | ``src/lexer/extended.rs``   | ✅ ExtendedRegexLexer |
+---------------------+-----------------------------+---------------------+
| **Filter System**   | ``src/filter.rs``           | ✅ Complete         |
+---------------------+-----------------------------+---------------------+
| **Formatters**      | ``src/formatter/mod.rs``    | ✅ Complete         |
|                     | ``src/formatter/html.rs``   |                     |
|                     | ``src/formatter/terminal.rs``|                    |
|                     | ``src/formatter/terminal256.rs``|                 |
|                     | ``src/formatter/other.rs``  |                     |
|                     | ``src/formatter/irc_bbcode.rs``|                  |
+---------------------+-----------------------------+---------------------+
| **PyO3 Bindings**   | ``src/bindings/mod.rs``     | ✅ Complete         |
|                     | ``src/bindings/lex.rs``     |                     |
|                     | ``src/bindings/classes.rs`` |                     |
+---------------------+-----------------------------+---------------------+

Lexers
------

**30 lexers ported**, **284 lexer tests**, **294 total tests** (all passing)::

    cargo test
    # test result: ok. 294 passed; 0 failed

    pytest ../tests/
    # 5327 passed, 16 skipped

    pytest ../tests/contrast/
    # 1 passed

### Core Languages (12)

Python, JavaScript, CSS, HTML/XML, C/C++, Rust, Go, Java, SQL, Bash, C#, Swift

### Scripting & Dynamic (7)

Kotlin, PHP, Ruby, Lua, Julia, R, PowerShell

### Data & Config (5)

JSON, YAML, Protobuf, Terraform, Makefile

### Infrastructure (1)

Docker

### Databases (1)

PostgreSQL

### Markup & Templates (2)

Markdown, Django

### JVM & Scala (1)

Scala

~140 lexers remaining (of ~598 in Pygments — 458 generated, 78 template, 61 custom). High-priority targets: TypeScript,
Perl, Haskell, Objective-C, Verilog.

Style System
------------

**49 styles generated** from Pygments' source via ``generators/gen_styles.py``,
covering **1,540 explicit style entries** across all token types.

The generator imports each Pygments style class at runtime, reads its ``styles``
dict (explicit definitions only — no inherited duplicates), maps Python token
reprs to Rust ``Token::CONSTANT`` names, and emits compact CSS strings parsed by
``StyleAttributes::from_css_string``.

Output: ``carthamin-core/src/style/generated.rs`` (~98 KB, 49 builder functions +
``get_style(name)`` registry + ``ALL_STYLE_NAMES`` constant).

WCAG AA color contrast compliance verified via ``tests/contrast/test_contrasts.py``
(requires ``wcag_contrast_ratio`` package).

Key styles: default, monokai, dracula, nord, solarized-dark/light, gruvbox,
one-dark, material, github-dark, zenburn, and 39 more.

Project Structure
-----------------

::

    carthamin/
    ├── carthamin-core/          # Rust crate (carthamin)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs           # crate root + PyO3 module init
    │       ├── token.rs         # Token enum + hierarchy
    │       ├── style/           # Style system
    │       │   ├── mod.rs       # Style, StyleAttributes, ansi_color
    │       │   └── generated.rs # 49 generated style builders
    │       ├── scanner.rs       # RegexScanner (regex::RegexSet)
    │       ├── regexopt.rs      # regex_opt, commonprefix, make_charset
    │       ├── util.rs          # html_escape, shebang_matches, etc.
    │       ├── filter.rs        # Filter trait + built-in filters
    │       ├── formatter/       # Formatters
    │       │   ├── mod.rs       # Formatter trait
    │       │   ├── html.rs      # HtmlFormatter
    │       │   ├── terminal.rs  # TerminalFormatter (ANSI)
    │       │   ├── terminal256.rs
    │       │   ├── other.rs     # Null, RawToken, Testcase
    │       │   └── irc_bbcode.rs # IRC, BBCode
    │       ├── lexer/           # Lexer engine + 30 ported lexers
    │       │   ├── mod.rs       # Lexer trait, RegexLexer, LexerRule
    │       │   ├── regex_lexer.rs
    │       │   ├── extended.rs  # ExtendedRegexLexer, DelegatingLexer, bygroups, using, include, inherit
    │       │   ├── python.rs, javascript.rs, ...
    │       │   └── ...
    │       └── bindings/        # PyO3 Python bindings
    │           ├── mod.rs
    │           ├── lex.rs
    │           └── classes.rs
    ├── generators/              # Code generation scripts
    │   └── gen_styles.py        # Pygments style → Rust generator
    ├── pygments/                # Original Pygments source (reference)
    ├── tests/                   # Compatibility tests
    ├── refactor_plan.md         # Phased migration plan (14 phases)
    └── code_map.md              # AST-derived codebase map

Development
-----------

### Build & Test

**Rust**::

    cd carthamin-core
    cargo build
    cargo test

**Python** (via maturin)::

    cd carthamin-core
    maturin develop
    pytest ../tests/

### Add a New Lexer

1. Study the Python lexer in ``pygments/lexers/<lang>.py``.
2. Create ``carthamin-core/src/lexer/<lang>.rs`` following the ``RegexLexer``
   pattern (see ``python.rs`` for granular token types or ``javascript.rs`` for
   simpler state machines).
3. Add ``pub mod <lang>;`` to ``src/lexer/mod.rs``.
4. Write inline tests (``#[cfg(test)] mod tests``) and verify with ``cargo test``.
5. Add compatibility tests in ``tests/test_compatibility.py`` and verify with ``pytest``.

### Regenerate Styles

When Pygments adds new styles or modifies existing ones::

    python generators/gen_styles.py

This rewrites ``carthamin-core/src/style/generated.rs``.

Migration Roadmap
-----------------

See `refactor_plan.md <refactor_plan.md>`_ for the full phased plan.

+----------+---------------------+-------+----------------------+
| Phase    | Component           | Files | Tests                |
+==========+=====================+=======+======================+
| 0        | Project Skeleton    | ✅    | ✅                   |
+----------+---------------------+-------+----------------------+
| 1        | Token System        | ✅    | ✅                   |
+----------+---------------------+-------+----------------------+
| 2        | Style System        | ✅    | ✅ (4 tests)         |
+----------+---------------------+-------+----------------------+
| 3        | Core Utilities      | ✅    | ✅                   |
+----------+---------------------+-------+----------------------+
| 4        | Scanner & Lexer     | ⚠️ Partial | ✅ core + extended   |
+----------+---------------------+-------+----------------------+
| 5        | Filter System       | ✅    | ✅                   |
+----------+---------------------+-------+----------------------+
| 6        | Core Formatters     | ✅    | ✅                   |
+----------+---------------------+-------+----------------------+
| 7        | Extra Formatters    | ⚠️ Partial | 8 of 10 done       |
+----------+---------------------+-------+----------------------+
| 8        | Critical Lexers     | ✅    | ✅ (293/293)         |
+----------+---------------------+-------+----------------------+
| 9        | Lexer Code Gen      | ✅    | ✅ (430 generated)   |
+----------+---------------------+-------+----------------------+
| 10       | Registry & Public   | ✅    | ✅                   |
|          | API                 |       |                      |
+----------+---------------------+-------+----------------------+
| 11       | Compatibility Tests | ✅    | ✅ (5327 passed)     |
+----------+---------------------+-------+----------------------+
| 12       | Remaining Lexers    | ✅    | ✅ (458 total)       |
+----------+---------------------+-------+----------------------+
| 13       | Final Polish        | ⬜    |                      |
+----------+---------------------+-------+----------------------+

Security Considerations
-----------------------

Carthamin inherits Pygments' security model. Lexer regex patterns process
arbitrary user input, so:

* **Set timeouts** — highlight operations should be bounded (seconds at most for
  reasonably-sized input).
* **Limit concurrency** — avoid oversubscription of resources.
* **Validate input size** — reject excessively large inputs before tokenization.

The Rust implementation benefits from memory safety guarantees (no buffer
overflows, no use-after-free) but regex backtracking remains a potential
concern for crafted inputs.

License
-------

BSD 2-Clause — same license as the original Pygments.

The Original Pygments
---------------------

Pygments was created by **Georg Brandl** and is maintained by **Georg Brandl**,
**Matthäus Chajdas**, and **Jean Abou-Samra**.  Many lexers and fixes have been
contributed by **Armin Ronacher**, the Pocoo team, and **Tim Hatch**.

Pygments homepage: https://pygments.org/
Pygments GitHub: https://github.com/pygments/pygments
