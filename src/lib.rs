use std::fmt::format;

use regex::Regex;
use reqwest::Url;

#[derive(Debug)]
pub struct Page {
    title: String,
    url: Url,
}

impl Page {
    fn new<T: reqwest::IntoUrl, U: std::fmt::Display>(title: U,  url: T) -> Result<Self, reqwest::Error> {
        Ok(Page { title: title.to_string(), url: url.into_url()? })
    }

    pub fn from_url<T: reqwest::IntoUrl>(url: T) -> Result<Self, reqwest::Error> {
        let url = url.into_url()?;

        let body = reqwest::blocking::get(url.clone())?.text()?;

        let body = body
            .lines()
            .filter(|l| l.contains("<title>"))
            .collect::<String>();

        println!("{}", &body);

        let regex = Regex::new(r"([a-zA-Z ]+) - Wikipedia").expect("Title regex failed to compile");

        if let Some(title) = regex.captures(&body) {

            Ok(Page {
                title: String::from(title.extract::<1>().1[0]),
                url,
            })
        } else {
            todo!()
        }
    }

    fn try_get_connections(&self) -> Result<Vec<Page>, reqwest::Error> {
        get_connections_from_url(self.url.clone())
    }
}

pub fn extend_wikipedia_url(short: &str) -> String {
    format!("https://en.wikipedia.org{short}")
}

fn get_connections_from_url<T: reqwest::IntoUrl>(url: T) -> Result<Vec<Page>, reqwest::Error> {
    let response = reqwest::blocking::get(url.into_url()?)?;

        let body = response.text()?;

        //println!("{}", &body);

        let wiki_regex = Regex::new(
            "<a href=\"(/wiki/[a-zA-Z_\\(\\)]+)\"(?: class=\"[a-zA-Z-_]\")? title=\"([a-zA-Z ]+)\"",
        )
        .unwrap();

        let captures = wiki_regex
            .captures_iter(&body)
            .map(|c| c.extract::<2>())
            .filter(|c| !c.0.contains("Wayback Machine"))
            .filter_map(|c| {
                Page::new(c.1[0], extend_wikipedia_url(c.1[1])).ok()
            })
            .collect::<Vec<Page>>();

        Ok(captures)

}

#[cfg(test)]
mod tests {
    use crate::{get_connections_from_url, Page};

    #[test]
    fn test_get_connections() {
        let connections = get_connections_from_url("https://en.wikipedia.org/wiki/Waffle").unwrap();

        println!("{connections:?}");
    }

    #[test]
    fn test_page_initialization() {
        let waffle_page = Page::from_url("https://en.wikipedia.org/wiki/Waffle").unwrap();

        assert_eq!(waffle_page.title.as_str(), "Waffle")
    }
}
