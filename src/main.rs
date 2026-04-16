use std::{env, fmt::Display, fs};

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
    year: u8,
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

fn main() {
    let config_str = fs::read_to_string("config.json").expect("Failed to read config");
    let config = Config::deserialize_json("config.json").expect("Failed to deserialize config");

    let template_path = env::args().nth(1).unwrap();
    let template = fs::read_to_string(template_path).expect("Failed to read template");

    let previews_str = config
        .previews
        .into_iter()
        .map(|Preview { title, body }| format!("== {title}\n{}", plain_text_to_typst(&body)))
        .collect::<String>();

    let body = config.articles.into_iter().map(
        |Article {
             path,
             header,
             footer,
             kürzel,
             language,
         }| {},
    );

    let edition_str = template
        .replace("EDITION", &config.edition.to_string())
        .replace("YEAR", &config.release_date.year.to_string())
        .replace("MONTH", &config.release_date.month.to_string())
        .replace("DAY", &config.release_date.day.to_string())
        .replace("PREVIEWS", &previews_str);

    fs::write(config.release_date.to_string(), edition_str).expect("Failed to write edition")
}

fn plain_text_to_typst(text: &str) -> String {
    text.replace("\n", "\\\n")
}
