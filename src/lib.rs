use regex::Regex;
use reqwest::{IntoUrl, Url};
use thiserror::Error;

#[derive(Debug, Clone)]

/// A struct representing the url of a wikipedia page
/// Doesn't use any network features
pub struct WikipediaUrl(Url);

impl WikipediaUrl {
    pub fn new<T: IntoUrl + std::fmt::Display + Clone>(
        input_url: T,
    ) -> Result<Self, WikipediaUrlInvalidError> {
        let url = match input_url.clone().into_url() {
            Ok(t) => Ok(t),
            Err(e) => Err(WikipediaUrlInvalidError::InvalidUrlError {
                source: e,
                invalid_url: input_url.to_string(),
            }),
        }?;

        // TODO: Multiple language support

        if let Some(host) = url.host_str() {
            if host != "en.wikipedia.org" {
                return Err(WikipediaUrlInvalidError::InvalidHostError(
                    input_url.to_string(),
                ));
            }
        }

        Ok(WikipediaUrl(url))
    }

    pub fn from_path(path: String) -> Result<Self, WikipediaUrlInvalidError> {
        let mut wikipedia_host: String;

        if path.starts_with("/") || path.starts_with("\\") {
            wikipedia_host = "https://en.wikipedia.org".to_string()
        } else {
            wikipedia_host = "https://en.wikipedia.org/".to_string()
        }

        wikipedia_host.push_str(path.as_str());

        WikipediaUrl::new(wikipedia_host)
    }

    pub fn get_url(&self) -> &Url {
        &self.0
    }

    pub fn try_into_page(self) -> Result<Page, reqwest::Error> {
        Page::from_url(self)
    }
}

#[derive(Error, Debug)]
pub enum WikipediaUrlInvalidError {
    #[error("'{0}' is not a valid wikipedia url")]
    InvalidHostError(String),
    #[error("'{invalid_url}' failed to be parsed as a url: '{source}'")]
    InvalidUrlError {
        source: reqwest::Error,
        invalid_url: String,
    },
}

#[derive(Debug)]
pub struct Page {
    title: String,
    url: WikipediaUrl,
}

impl Page {
    fn new<U: std::fmt::Display>(
        title: U,
        url: String,
    ) -> Result<Self, WikipediaUrlInvalidError> {
        Ok(Page {
            title: title.to_string(),
            url: WikipediaUrl::from_path(url)?,
        })
    }

    pub fn from_url(wiki_url: WikipediaUrl) -> Result<Self, reqwest::Error> {
        let url = wiki_url.get_url();

        let body = reqwest::blocking::get(url.clone())?.text()?;

        let body = body
            .lines()
            .filter(|l| l.contains("<title>"))
            .collect::<String>();

        let regex = Regex::new(r"([a-zA-Z ]+) - Wikipedia").expect("Title regex failed to compile");

        if let Some(title) = regex.captures(&body) {
            Ok(Page {
                title: String::from(title.extract::<1>().1[0]),
                url: wiki_url,
            })
        } else {
            todo!()
        }
    }

    pub fn try_get_connections(&self) -> Result<Vec<Page>, reqwest::Error> {
        try_get_connections_from_url(self.url.clone())
    }
}

pub fn extend_wikipedia_url(short: &str) -> String {
    format!("https://en.wikipedia.org{short}")
}

fn try_get_connections_from_url(url: WikipediaUrl) -> Result<Vec<Page>, reqwest::Error> {
    let response = reqwest::blocking::get(url.get_url().clone())?;

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
        //.map(|c| dbg!(c))
        .filter_map(|c| Page::new(c.1[0], extend_wikipedia_url(c.1[1])).ok())        
        //.map(|c| dbg!(c))
        .collect::<Vec<Page>>();

    Ok(captures)
}

#[cfg(test)]
mod tests {
    use crate::{Page, WikipediaUrl, try_get_connections_from_url};

    #[test]
    fn test_get_connections() {
        let url = WikipediaUrl::from_path("/wiki/Waffle".to_string()).unwrap();

        let connections = try_get_connections_from_url(dbg!(url)).unwrap();
    }

    #[test]
    fn test_page_initialization() {
        let url = WikipediaUrl::from_path("/wiki/Waffle".to_string()).unwrap();

        let waffle_page = Page::from_url(url).unwrap();
        assert_eq!(waffle_page.title.as_str(), "Waffle")
    }
}
