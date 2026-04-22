use std::{env::home_dir, fmt::Display, fs, path::PathBuf, process::Command};

use nanoserde::DeJson;
use regex::Regex;

#[derive(DeJson)]
struct Config {
    edition: usize,
    release_date: Date,
    previews: Vec<Preview>,
    articles: Vec<Article>,
}

#[derive(DeJson)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
}
impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

#[derive(DeJson)]
struct Preview {
    title: String,
    body: String,
}

#[derive(DeJson)]
struct Article {
    path: String,
    header: bool,
    footer: bool,
    kürzel: String,
    language: String,
}

const HIGHLIGHT_FILTER: &str = include_str!("../highlight-filter.lua");
const EDITION_TEMPLATE: &str = include_str!("../templates/edition.typ");
const ARTICLE_TEMPLATE: &str = include_str!("../templates/article.typ");
const BRAINMADE_SVG: &[u8] = include_bytes!("../Brainmade.svg");
const SOURCE_CITATION_PATTERN: &str =
    r#"#link\("([^"]+)"\)\[#super\[\\\[\d+\\\]\];\]"#;

fn main() {
    let config_str = fs::read_to_string("config.json").expect("Failed to read config");
    let config = Config::deserialize_json(&config_str).expect("Failed to deserialize config");

    let previews_str = config
        .previews
        .into_iter()
        .map(|Preview { title, body }| format!("== {title}\n{body}\n"))
        .collect::<String>();

    // write lua filter to cache
    let highlight_filter_path = write_highlight_filter();

    let body = config
        .articles
        .into_iter()
        .map(
            |Article {
                 path,
                 header,
                 footer,
                 kürzel,
                 language,
             }| {
                let pandoc_err_msg = "Failed to run pandoc on article";
                let output = Command::new("pandoc")
                    .args([&path, "--to", "typst", "--lua-filter"])
                    .arg(&highlight_filter_path)
                    .output()
                    .expect(pandoc_err_msg);
                if !output.status.success() {
                    panic!(
                        "{}: {}",
                        pandoc_err_msg,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                let content =
                    String::from_utf8(output.stdout).expect("Pandoc output should be utf8");
                let (title, rest) = content
                    .trim()
                    .split_once("\n\n")
                    .expect("Article has no title");

                let centered = |content| format!("centered[\n{}\n],\nspacing,\n", content);

                let (header, rest) = if header {
                    let (header, rest) = rest
                        .trim()
                        .split_once("\n\n")
                        .expect("Article has no header");
                    (centered(header), rest)
                } else {
                    (String::new(), rest)
                };
                let (body, footer) = if footer {
                    let (body, footer) = rest
                        .trim()
                        .rsplit_once("\n\n")
                        .expect("Article has no footer");
                    (body, centered(footer))
                } else {
                    (rest, String::new())
                };

                ARTICLE_TEMPLATE
                    .replace("LANGUAGE", language.trim())
                    .replace("TITLE", title.trim())
                    .replace("HEADER", header.trim())
                    .replace("BODY", body.trim())
                    .replace("FOOTER", footer.trim())
                    .replace("KÜRZEL", kürzel.trim())
            },
        )
        .collect::<String>();

    let edition_str = EDITION_TEMPLATE
        .replace("EDITION", &config.edition.to_string())
        .replace("YEAR", &config.release_date.year.to_string())
        .replace("MONTH", &config.release_date.month.to_string())
        .replace("DAY", &config.release_date.day.to_string())
        .replace("PREVIEWS", &previews_str)
        .replace("BODY", &body);
    let (edition_str, refs_yaml) = rewrite_source_citations(edition_str);

    let typst_file = format!("{}.typ", config.release_date);
    fs::write(typst_file.clone(), edition_str).expect("Failed to write edition");
    fs::write("refs.yaml", refs_yaml).expect("Failed to write refs.yaml");
    fs::write("Brainmade.svg", BRAINMADE_SVG).expect("Failed to write Brainmade.svg");

    Command::new("typstyle")
        .args(["-i", &typst_file])
        .spawn()
        .expect("Failed to spawn typst formatter")
        .wait()
        .expect("Failed to format typst file");
}

fn rewrite_source_citations(content: String) -> (String, String) {
    let citation_regex = Regex::new(SOURCE_CITATION_PATTERN).expect("Citation regex should compile");
    let mut citations = Vec::<Citation>::new();
    let mut rewritten = String::with_capacity(content.len());
    let mut last_end = 0;

    for captures in citation_regex.captures_iter(&content) {
        let matched = captures.get(0).expect("Full citation match should exist");
        rewritten.push_str(&content[last_end..matched.start()]);

        let url = captures
            .get(1)
            .expect("Citation URL should exist")
            .as_str()
            .to_owned();

        let key = if let Some(existing) = citations.iter().find(|citation| citation.url == url) {
            existing.key()
        } else {
            let citation = Citation {
                number: citations.len() + 1,
                url,
            };
            let key = citation.key();
            citations.push(citation);
            key
        };

        rewritten.push_str(&format!("#cite(<{key}>)"));
        last_end = matched.end();
    }

    rewritten.push_str(&content[last_end..]);

    let refs_yaml = citations
        .into_iter()
        .map(|citation| citation.to_hayagriva_entry())
        .collect::<Vec<_>>()
        .join("\n");

    (rewritten, refs_yaml)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Citation {
    number: usize,
    url: String,
}

impl Citation {
    fn key(&self) -> String {
        format!("ref-{}", self.number)
    }

    fn to_hayagriva_entry(&self) -> String {
        let url = yaml_string(&self.url);
        format!(
            "{}:\n  type: web\n  title: {}\n  url: {}\n",
            self.key(),
            url,
            url
        )
    }
}

fn yaml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Writes the highlight lua filter and returns its path
fn write_highlight_filter() -> PathBuf {
    let home = home_dir().expect("Failed to get home dir for writing highlight lua filter");

    let dir = home.join(".cache/dieprobezeit-generate");
    fs::create_dir_all(dir.clone()).expect("Failed to create cache dir");

    let file = dir.join("highlight-filter.lua");
    fs::write(file.clone(), HIGHLIGHT_FILTER).expect("Failed to write highlight lua filter");

    file
}

#[cfg(test)]
mod tests {
    use super::{rewrite_source_citations, Citation};

    #[test]
    fn rewrites_source_citations_to_native_typst_citations() {
        let input = concat!(
            "Hello ",
            "#link(\"https://anthropic.com/research\")[#super[\\[1\\]];]",
            " and again ",
            "#link(\"https://anthropic.com/research\")[#super[\\[1\\]];]",
            "."
        );

        let (rewritten, refs_yaml) = rewrite_source_citations(input.to_owned());

        assert_eq!("Hello #cite(<ref-1>) and again #cite(<ref-1>).", rewritten);
        assert_eq!(
            Citation {
                number: 1,
                url: "https://anthropic.com/research".to_owned()
            }
            .to_hayagriva_entry(),
            refs_yaml
        );
    }

    #[test]
    fn ignores_source_numbering_and_uses_first_appearance_order() {
        let input = concat!(
            "#link(\"https://example.com/b\")[#super[\\[7\\]];] ",
            "#link(\"https://example.com/a\")[#super[\\[2\\]];] ",
            "#link(\"https://example.com/b\")[#super[\\[99\\]];]"
        );
        let (rewritten, refs_yaml) = rewrite_source_citations(input.to_owned());

        assert_eq!(
            "#cite(<ref-1>) #cite(<ref-2>) #cite(<ref-1>)",
            rewritten
        );
        assert_eq!(
            concat!(
                "ref-1:\n",
                "  type: web\n",
                "  title: \"https://example.com/b\"\n",
                "  url: \"https://example.com/b\"\n",
                "\n",
                "ref-2:\n",
                "  type: web\n",
                "  title: \"https://example.com/a\"\n",
                "  url: \"https://example.com/a\"\n"
            ),
            refs_yaml
        );
    }
}
