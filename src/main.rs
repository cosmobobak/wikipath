#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::{collections::{HashMap, VecDeque}, time::Instant, io::Write};

use scraper::Html;
use scraper::Selector;

const PREFIX: &str = "https://en.wikipedia.org/wiki/";

struct Crawler {
    cache: HashMap<String, Vec<String>>,
}

impl Crawler {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    // visit a link and return all the links found on that page
    async fn crawl_uncached(link: &str, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut links = Vec::new();
        println!("[#] requesting {link}");
        std::io::stdout().flush().unwrap();
        let body = reqwest::get(link).await?.text().await?;
        let document = Html::parse_document(&body);
        for node in document.select(&Selector::parse("a").unwrap()) {
            let href = node.value().attr("href").unwrap_or("");
            if href.starts_with(PREFIX) && !href.contains(':') {
                links.push(href.to_string());
            } else if href.starts_with("/wiki/") && !href.contains(':') {
                links.push(format!("{prefix}{href}"));
            }
        }
        Ok(links)
    }

    async fn crawl(&mut self, link: &str, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if let Some(links) = self.cache.get(link) {
            return Ok(links.clone());
        }

        let res = Self::crawl_uncached(link, prefix).await?;
        self.cache.insert(link.to_string(), res.clone());
        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let [_exe, source_link, target_link] = &args[..] else {
        eprintln!("Usage: {} <source> <target>", args[0]);
        std::process::exit(1);
    };

    let domain = reqwest::Url::parse(source_link)?.domain().unwrap().to_string();
    let filter = format!("https://{domain}");

    let mut crawler = Crawler::new();

    // just a simple BFS
    let mut all_links = Vec::new();
    let mut queue = VecDeque::new();
    // push the source link with no parent
    all_links.push((None, source_link.to_string()));
    // push the source link to the queue
    queue.push_back(0);

    let start = Instant::now();

    let mut last = None;
    while let Some(idx) = queue.pop_front() {
        let (_, link) = &all_links[idx];
        if link == target_link {
            last = Some(idx);
            println!();
            break;
        }

        let links = crawler.crawl(link, &filter).await?;
        for link in links {
            if all_links.iter().any(|(_, l)| l == &link) {
                continue;
            }
            all_links.push((Some(idx), link.clone()));
            queue.push_back(all_links.len() - 1);
        }
    };

    let Some(last) = last else {
        eprintln!("Could not find link: {target_link}");
        std::process::exit(1);
    };
    println!("Found link: {target_link}");
    println!("Path:");
    let mut links = Vec::new();
    let mut idx = last;
    while let (Some(parent), link) = &all_links[idx] {
        links.push(link);
        idx = *parent;
    }
    for (i, link) in links.iter().rev().enumerate() {
        println!("{i}: {link}");
    }

    println!("Took: {} seconds", start.elapsed().as_secs());

    Ok(())
}
