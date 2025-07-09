#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::cast_precision_loss)]

use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, RwLock},
    time::Instant,
};

use scraper::Html;
use scraper::Selector;

use anyhow::Result;

const PREFIX: &str = "https://en.wikipedia.org/wiki/";

#[derive(Debug, Clone)]
struct CrawlerCache {
    cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl CrawlerCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // visit a link and return all the links found on that page
    async fn crawl_uncached(link: &str, prefix: &str) -> Result<Vec<String>> {
        let mut links = Vec::new();
        println!("[#] requesting {link}");
        std::io::stdout().flush().unwrap();
        let body = reqwest::get(link).await?.text().await?;
        let document = Html::parse_document(&body);
        for node in document.select(&Selector::parse("a").unwrap()) {
            let href = node.value().attr("href").unwrap_or("");
            let link = if href.starts_with(PREFIX) && !href.contains(':') {
                href.to_string()
            } else if href.starts_with("/wiki/") && !href.contains(':') {
                format!("{prefix}{href}")
            } else {
                continue;
            };
            links.push(link);
        }
        Ok(links)
    }

    async fn crawl(&self, link: &str, prefix: &str) -> Result<Vec<String>> {
        {
            let cache = self.cache.read().unwrap();
            if let Some(links) = cache.get(link) {
                return Ok(links.clone());
            }
        }
        let links = Self::crawl_uncached(link, prefix).await?;
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(link.to_string(), links.clone());
        }
        Ok(links)
    }
}

fn link_last_part(link: &str) -> &str {
    link.split('/').next_back().unwrap()
}

struct Node {
    parent: Option<usize>,
    link: String,
}

#[derive(Clone, Copy)]
struct PQEntry {
    distance: f32,
    idx: usize,
}

impl PartialEq for PQEntry {
    fn eq(&self, other: &Self) -> bool {
        f32::total_cmp(&self.distance, &other.distance) == std::cmp::Ordering::Equal
            && self.idx == other.idx
    }
}
impl Eq for PQEntry {}

impl PartialOrd for PQEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PQEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        f32::total_cmp(&other.distance, &self.distance).then(self.idx.cmp(&other.idx))
    }
}

async fn search(
    target_link: &str,
    queue: &mut Vec<PQEntry>,
    all_links: &mut Vec<Node>,
    crawler_cache: CrawlerCache,
    filter: &str,
) -> Result<Option<usize>> {
    const PREFETCH_SIZE: usize = 10;

    loop {
        queue.sort_unstable();

        let Some(PQEntry { idx, .. }) = queue.pop() else {
            println!("[!] Queue is empty, no more links to explore.");
            return Ok(None);
        };

        let _prefetch_tasks = queue
            .iter()
            .rev()
            .take(PREFETCH_SIZE)
            .map(|PQEntry { idx, .. }| {
                let link = all_links[*idx].link.clone();
                let filter = filter.to_string();
                let crawler = crawler_cache.clone();
                tokio::spawn(async move { crawler.crawl(&link, &filter).await })
            })
            .collect::<Vec<_>>();

        let Node { link, .. } = &all_links[idx];
        if link == target_link {
            println!();
            return Ok(Some(idx));
        }

        let mut depth_from_root = 0;
        {
            let mut idx = idx;
            while let Node {
                parent: Some(parent),
                ..
            } = &all_links[idx]
            {
                idx = *parent;
                depth_from_root += 1;
            }
        }

        let links = crawler_cache.crawl(link, filter).await?;
        for link in &links {
            if all_links.iter().any(|Node { link: l, .. }| l == link) {
                continue;
            }
            all_links.push(Node {
                parent: Some(idx),
                link: link.clone(),
            });
            queue.push(PQEntry {
                distance: (depth_from_root
                    + distance::damerau_levenshtein(
                        link_last_part(link),
                        link_last_part(target_link),
                    )) as f32,
                idx: all_links.len() - 1,
            });
            if link == target_link {
                return Ok(Some(all_links.len() - 1));
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let [_exe, source_link, target_link] = &args[..] else {
        eprintln!("Usage: {} <source> <target>", args[0]);
        std::process::exit(1);
    };

    let domain = reqwest::Url::parse(source_link)?
        .domain()
        .unwrap()
        .to_string();
    let filter = format!("https://{domain}");

    let crawler = CrawlerCache::new();

    // just a simple best-first search
    let mut all_links = Vec::new();
    let mut queue = Vec::new();
    // push the source link with no parent
    all_links.push(Node {
        parent: None,
        link: source_link.to_string(),
    });
    // push the source link to the queue
    queue.push(PQEntry {
        distance: distance::damerau_levenshtein(
            link_last_part(source_link),
            link_last_part(target_link),
        ) as f32,
        idx: 0,
    });

    let start = Instant::now();

    let res = search(target_link, &mut queue, &mut all_links, crawler, &filter).await?;

    println!();

    let Some(last) = res else {
        eprintln!("Could not find link: {target_link}");
        std::process::exit(1);
    };

    println!("Found link: {target_link}");
    println!("Path:");
    let mut links = Vec::new();
    let mut idx = last;
    while let Node {
        parent: Some(parent),
        link,
    } = &all_links[idx]
    {
        links.push(link);
        idx = *parent;
    }
    links.push(source_link);
    for (i, link) in links.iter().rev().enumerate() {
        println!("{i}: {link}");
    }

    println!("Took: {:.2} seconds", start.elapsed().as_secs_f64());

    Ok(())
}
