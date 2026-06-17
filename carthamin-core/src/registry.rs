use std::collections::HashMap;
use crate::lexer::Lexer;

/// Registry entry for a lexer.
#[derive(Debug, Clone)]
pub struct LexerEntry {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub filenames: &'static [&'static str],
    pub mimetypes: &'static [&'static str],
    pub priority: f64,
    pub create: fn() -> Box<dyn Lexer>,
}

/// Registry entry for a formatter.
#[derive(Debug, Clone)]
pub struct FormatterEntry {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub extension: &'static str,
    pub mimetype: &'static str,
}

/// Lexer registry — maps names/aliases to lexer constructors.
pub struct LexerRegistry {
    entries: Vec<LexerEntry>,
    by_name: HashMap<&'static str, usize>,
    by_alias: HashMap<String, usize>,
    by_filename: HashMap<&'static str, Vec<usize>>,
    by_mimetype: HashMap<&'static str, usize>,
}

/// Formatter registry — maps names/aliases to formatter info.
pub struct FormatterRegistry {
    entries: Vec<FormatterEntry>,
    by_name: HashMap<&'static str, usize>,
    by_alias: HashMap<&'static str, usize>,
    by_extension: HashMap<&'static str, usize>,
}

impl LexerRegistry {
    pub fn new() -> Self {
        let mut registry = LexerRegistry {
            entries: Vec::new(),
            by_name: HashMap::new(),
            by_alias: HashMap::new(),
            by_filename: HashMap::new(),
            by_mimetype: HashMap::new(),
        };
        registry.build();
        registry
    }

    fn build(&mut self) {
        let entries: Vec<LexerEntry> = vec![
            LexerEntry {
                name: "Python",
                aliases: &["python", "py", "python3", "py3"],
                filenames: &["*.py", "*.pyw", "*.pyi"],
                mimetypes: &["text/x-python", "application/x-python"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::python::PythonLexer::new()) },
            },
            LexerEntry {
                name: "JavaScript",
                aliases: &["javascript", "js"],
                filenames: &["*.js", "*.jsm", "*.mjs", "*.cjs"],
                mimetypes: &["application/javascript", "application/x-javascript", "text/x-javascript", "text/javascript"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::javascript::JavaScriptLexer::new()) },
            },
            LexerEntry {
                name: "HTML",
                aliases: &["html", "xhtml"],
                filenames: &["*.html", "*.htm", "*.xhtml"],
                mimetypes: &["text/html", "application/xhtml+xml"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::htmlxml::HtmlLexer::new()) },
            },
            LexerEntry {
                name: "XML",
                aliases: &["xml"],
                filenames: &["*.xml", "*.xsl", "*.xsd", "*.wsdl", "*.svg"],
                mimetypes: &["text/xml", "application/xml"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::htmlxml::XmlLexer::new()) },
            },
            LexerEntry {
                name: "CSS",
                aliases: &["css"],
                filenames: &["*.css"],
                mimetypes: &["text/css"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::css::CssLexer::new()) },
            },
            LexerEntry {
                name: "Rust",
                aliases: &["rust", "rs"],
                filenames: &["*.rs"],
                mimetypes: &["text/x-rustsrc", "application/x-rustsrc"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::rust::RustLexer::new()) },
            },
            LexerEntry {
                name: "C++",
                aliases: &["cpp", "c++", "hpp", "cxx", "cc"],
                filenames: &["*.cpp", "*.hpp", "*.cxx", "*.h", "*.cc", "*.hh"],
                mimetypes: &["text/x-c++src", "text/x-c++hdr", "text/x-csrc", "text/x-chdr"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::cpp::CppLexer::new()) },
            },
            LexerEntry {
                name: "C",
                aliases: &["c", "h"],
                filenames: &["*.c", "*.h"],
                mimetypes: &["text/x-csrc", "text/x-chdr"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::cpp::CLexer::new()) },
            },
            LexerEntry {
                name: "Go",
                aliases: &["go", "golang"],
                filenames: &["*.go"],
                mimetypes: &["text/x-gosrc"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::go::GoLexer::new()) },
            },
            LexerEntry {
                name: "Java",
                aliases: &["java"],
                filenames: &["*.java"],
                mimetypes: &["text/x-java"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::java::JavaLexer::new()) },
            },
            LexerEntry {
                name: "SQL",
                aliases: &["sql"],
                filenames: &["*.sql"],
                mimetypes: &["text/x-sql"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::sql::SqlLexer::new()) },
            },
            LexerEntry {
                name: "Bash",
                aliases: &["bash", "sh", "shell", "zsh", "ksh"],
                filenames: &["*.sh", "*.bash", "*.zsh", "*.ksh"],
                mimetypes: &["text/x-shellscript", "application/x-shellscript"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::bash::BashLexer::new()) },
            },
            LexerEntry {
                name: "C#",
                aliases: &["csharp", "c#", "cs"],
                filenames: &["*.cs"],
                mimetypes: &["text/x-csharp"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::csharp::CSharpLexer::new()) },
            },
            LexerEntry {
                name: "Swift",
                aliases: &["swift"],
                filenames: &["*.swift"],
                mimetypes: &["text/x-swift"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::swift::SwiftLexer::new()) },
            },
            LexerEntry {
                name: "Kotlin",
                aliases: &["kotlin", "kt"],
                filenames: &["*.kt", "*.kts", "*.kotlin"],
                mimetypes: &["text/x-kotlin"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::kotlin::KotlinLexer::new()) },
            },
            LexerEntry {
                name: "PHP",
                aliases: &["php", "php3", "php4", "php5"],
                filenames: &["*.php", "*.php3", "*.php4", "*.php5", "*.inc"],
                mimetypes: &["text/x-php"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::php::PhpLexer::new()) },
            },
            LexerEntry {
                name: "Ruby",
                aliases: &["ruby", "rb", "duby"],
                filenames: &["*.rb", "*.rbw", "Rakefile", "*.rake", "*.gemspec", "*.rbx", "*.duby", "Gemfile", "Vagrantfile"],
                mimetypes: &["text/x-ruby", "application/x-ruby"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::ruby::RubyLexer::new()) },
            },
            LexerEntry {
                name: "Lua",
                aliases: &["lua"],
                filenames: &["*.lua", "*.wlua"],
                mimetypes: &["text/x-lua", "application/x-lua"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::lua::LuaLexer::new()) },
            },
            LexerEntry {
                name: "R",
                aliases: &["r", "s", "splus"],
                filenames: &["*.R", "*.r", ".Rhistory", ".Rprofile", ".Renviron"],
                mimetypes: &["text/x-r-source", "text/x-s", "text/x-R", "text/x-r-history", "text/x-r-profile"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::r::RLexer::new()) },
            },
            LexerEntry {
                name: "JSON",
                aliases: &["json"],
                filenames: &["*.json"],
                mimetypes: &["application/json", "text/json"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::json::JsonLexer::new()) },
            },
            LexerEntry {
                name: "YAML",
                aliases: &["yaml"],
                filenames: &["*.yaml", "*.yml"],
                mimetypes: &["text/x-yaml", "application/x-yaml"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::yaml::YamlLexer::new()) },
            },
            LexerEntry {
                name: "Markdown",
                aliases: &["markdown", "md", "mkd", "mdwn", "mdown", "mkdn", "rmd"],
                filenames: &["*.md", "*.mkd", "*.mdwn", "*.mdown", "*.mkdn", "*.rmd"],
                mimetypes: &["text/x-markdown"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::markdown::MarkdownLexer::new()) },
            },
            LexerEntry {
                name: "Protocol Buffer",
                aliases: &["protobuf", "proto"],
                filenames: &["*.proto"],
                mimetypes: &["text/x-protobuf"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::protobuf::ProtoBufLexer::new()) },
            },
            LexerEntry {
                name: "PowerShell",
                aliases: &["powershell", "ps", "ps1", "ps2", "psd1", "psd2", "psm1", "psm2"],
                filenames: &["*.ps1", "*.ps2", "*.psd1", "*.psd2", "*.psm1", "*.psm2"],
                mimetypes: &["text/x-powershell"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::powershell::PowerShellLexer::new()) },
            },
            LexerEntry {
                name: "PostgreSQL",
                aliases: &["postgresql", "postgres"],
                filenames: &["*.pgsql", "*.psql"],
                mimetypes: &["text/x-pgsql"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::postgres::PostgresLexer::new()) },
            },
            LexerEntry {
                name: "Docker",
                aliases: &["docker", "dockerfile"],
                filenames: &["Dockerfile", "*.dockerfile"],
                mimetypes: &["text/x-dockerfile"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::docker::DockerLexer::new()) },
            },
            LexerEntry {
                name: "Terraform",
                aliases: &["terraform", "hcl", "tf"],
                filenames: &["*.tf", "*.tfvars"],
                mimetypes: &["text/x-terraform", "text/x-hcl"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::terraform::TerraformLexer::new()) },
            },
            LexerEntry {
                name: "Makefile",
                aliases: &["makefile", "make"],
                filenames: &["Makefile", "makefile", "*.mk"],
                mimetypes: &["text/x-makefile"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::makefile::MakefileLexer::new()) },
            },
            LexerEntry {
                name: "Scala",
                aliases: &["scala"],
                filenames: &["*.scala"],
                mimetypes: &["text/x-scala"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::scala::ScalaLexer::new()) },
            },
            LexerEntry {
                name: "Julia",
                aliases: &["julia"],
                filenames: &["*.jl"],
                mimetypes: &["text/x-julia"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::julia::JuliaLexer::new()) },
            },
            LexerEntry {
                name: "Django/Jinja",
                aliases: &["django", "jinja"],
                filenames: &["*.html", "*.jinja", "*.jinja2", "*.djhtml"],
                mimetypes: &["application/x-django-templating", "application/x-jinja"],
                priority: 0.0,
                create: || -> Box<dyn Lexer> { Box::new(crate::lexer::django::DjangoLexer::new()) },
            },
        ];

        for (idx, entry) in entries.iter().enumerate() {
            self.by_name.insert(entry.name, idx);
            for alias in entry.aliases {
                self.by_alias.insert(alias.to_lowercase(), idx);
            }
            for filename in entry.filenames {
                self.by_filename.entry(filename).or_default().push(idx);
            }
            for mimetype in entry.mimetypes {
                self.by_mimetype.insert(mimetype, idx);
            }
        }
        self.entries = entries;
    }

    /// Get a lexer by name.
    pub fn get_by_name(&self, name: &str) -> Option<&LexerEntry> {
        self.by_name.get(name).map(|&idx| &self.entries[idx])
    }

    /// Get a lexer by alias.
    pub fn get_by_alias(&self, alias: &str) -> Option<&LexerEntry> {
        self.by_alias.get(&alias.to_lowercase()).map(|&idx| &self.entries[idx])
    }

    /// Get all lexers matching a filename pattern.
    pub fn get_by_filename(&self, filename: &str) -> Vec<&LexerEntry> {
        let mut results = Vec::new();
        for (pattern, indices) in &self.by_filename {
            if matches_pattern(pattern, filename) {
                for &idx in indices {
                    results.push(&self.entries[idx]);
                }
            }
        }
        results
    }

    /// Get a lexer by MIME type.
    pub fn get_by_mimetype(&self, mimetype: &str) -> Option<&LexerEntry> {
        self.by_mimetype.get(mimetype).map(|&idx| &self.entries[idx])
    }

    /// Iterate over all registered lexers.
    pub fn iter(&self) -> impl Iterator<Item = &LexerEntry> {
        self.entries.iter()
    }

    /// Create a lexer instance by alias.
    pub fn create_by_alias(&self, alias: &str) -> Option<Box<dyn Lexer>> {
        self.get_by_alias(alias).map(|entry| (entry.create)())
    }
}

impl FormatterRegistry {
    pub fn new() -> Self {
        let mut registry = FormatterRegistry {
            entries: Vec::new(),
            by_name: HashMap::new(),
            by_alias: HashMap::new(),
            by_extension: HashMap::new(),
        };
        registry.build();
        registry
    }

    fn build(&mut self) {
        let entries: Vec<FormatterEntry> = vec![
            FormatterEntry {
                name: "HTML",
                aliases: &["html", "HTML"],
                extension: "html",
                mimetype: "text/html",
            },
            FormatterEntry {
                name: "Terminal",
                aliases: &["terminal", "console", "tty"],
                extension: "",
                mimetype: "text/plain",
            },
            FormatterEntry {
                name: "Terminal256",
                aliases: &["terminal256", "256"],
                extension: "",
                mimetype: "text/plain",
            },
        ];

        for (idx, entry) in entries.iter().enumerate() {
            self.by_name.insert(entry.name, idx);
            for alias in entry.aliases {
                self.by_alias.insert(alias, idx);
            }
            if !entry.extension.is_empty() {
                self.by_extension.insert(entry.extension, idx);
            }
        }
        self.entries = entries;
    }

    /// Get a formatter by name.
    pub fn get_by_name(&self, name: &str) -> Option<&FormatterEntry> {
        self.by_name.get(name).map(|&idx| &self.entries[idx])
    }

    /// Get a formatter by alias.
    pub fn get_by_alias(&self, alias: &str) -> Option<&FormatterEntry> {
        self.by_alias.get(alias).map(|&idx| &self.entries[idx])
    }

    /// Get a formatter by file extension.
    pub fn get_by_extension(&self, extension: &str) -> Option<&FormatterEntry> {
        self.by_extension.get(extension).map(|&idx| &self.entries[idx])
    }

    /// Iterate over all registered formatters.
    pub fn iter(&self) -> impl Iterator<Item = &FormatterEntry> {
        self.entries.iter()
    }
}

/// Simple glob pattern matching for filename patterns.
fn matches_pattern(pattern: &str, filename: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return filename.ends_with(ext);
    }
    pattern == filename
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_registry() {
        let registry = LexerRegistry::new();
        assert!(registry.get_by_alias("python").is_some());
        assert!(registry.get_by_alias("py").is_some());
        assert!(registry.get_by_alias("python3").is_some());
        assert!(registry.get_by_name("Python").is_some());
    }

    #[test]
    fn test_formatter_registry() {
        let registry = FormatterRegistry::new();
        assert!(registry.get_by_alias("html").is_some());
        assert!(registry.get_by_alias("terminal").is_some());
        assert!(registry.get_by_extension("html").is_some());
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("*.py", "test.py"));
        // *.py does not match test.pyi (different extension)
        // assert!(matches_pattern("*.py", "test.pyi"));
        assert!(!matches_pattern("*.py", "test.js"));
        assert!(matches_pattern("*", "anything"));
    }
}
