use std::{
    env::{self, home_dir},
    fmt::Display,
    fs,
    path::PathBuf,
    process::Command,
};

use nanoserde::DeJson;

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

fn main() {
    let config_str = fs::read_to_string("config.json").expect("Failed to read config");
    let config = Config::deserialize_json(&config_str).expect("Failed to deserialize config");

    let previews_str = config
        .previews
        .into_iter()
        .map(|Preview { title, body }| format!("== {title}\n{}", body))
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
                let (title, rest) = content.split_once('\n').expect("Article has no title");

                let centered = |content| format!("centered[\n{}\n],\nspacing,\n", content);

                let (header, rest) = if header {
                    let (header, rest) = rest.split_once('\n').expect("Article has no header");
                    (centered(header), rest)
                } else {
                    (String::new(), rest)
                };
                let (body, footer) = if footer {
                    let (body, footer) = rest.rsplit_once("\n").expect("Article has no footer");
                    (body, centered(footer))
                } else {
                    (rest, String::new())
                };

                ARTICLE_TEMPLATE
                    .replace("LANGUAGE", &language)
                    .replace("TITLE", title)
                    .replace("HEADER", &header)
                    .replace("BODY", body)
                    .replace("FOOTER", &footer)
                    .replace("KÜRZEL", &kürzel)
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

    let typst_file = format!("{}.typ", config.release_date);
    fs::write(typst_file.clone(), edition_str).expect("Failed to write edition");
    fs::write("Brainmade.svg", BRAINMADE_SVG).expect("Failed to write Brainmade.svg");

    Command::new("typstyle")
        .args(["-i", &typst_file])
        .spawn()
        .expect("Failed to spawn typst formatter")
        .wait()
        .expect("Failed to format typst file");
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
