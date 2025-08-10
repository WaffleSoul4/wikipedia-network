//! A simple crate to help turn [Wikipedia] into a [Semantic Network]
//!
//! [Wikipedia]: https://wikipedia.org
//! [Semantic Network]: https://wikipedia.org/wiki/Semantic_network
//!
//!
//! ```rust
//! # use wikipedia_network::{Page, WikipediaUrl};
//! # fn main() -> Result<(), reqwest::Error> {
//! let url = WikipediaUrl::from_path("/wiki/Waffles").unwrap(); // Parse the url
//! let mut waffles_page = Page::new(url); // Initialize the page struct
//! 
//! // Load the body into the struct and get the title
//! let title: String = waffles_page.get_title()?; 
//! assert_eq!(title.as_str(), "Waffle");
//! 
//! // Get all the Wikipedia links on the page
//! let connections: Vec<Page> = waffles_page.get_connections()?; 
//! 
//! // Remove the body from the struct for memory efficiency
//! waffles_page.unload_body();
//!
//! for mut page in connections {
//!     // Print the names and URLs of all of the pages
//!     println!("Page {} at url {}", page.get_title()?, page.get_url())
//! }
//! # Ok(())
//! # }
//! ```
//!

// TODO:
//     - Language support
//     - Async (mmm...)

use regex::Regex;
use reqwest::{IntoUrl, Url};
use thiserror::Error;

type ReqwestError = reqwest::Error;

#[derive(Debug, Clone)]
/// A parser struct containing the [Url] of a Wikipedia page
pub struct WikipediaUrl(Url);

impl WikipediaUrl {
    /// Creates a new [WikipediaUrl]
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

    #[doc = "Creates a new [WikipediaUrl] from the path of the url\nFor example: `https://en.wikipedia.org/wiki/Waffle` vs. `/wiki/Waffle`"]
    pub fn from_path<T: std::fmt::Display>(path: T) -> Result<Self, WikipediaUrlInvalidError> {
        let mut wikipedia_host: String;

        let path = path.to_string();

        if path.starts_with("/") || path.starts_with("\\") {
            wikipedia_host = "https://en.wikipedia.org".to_string()
        } else {
            wikipedia_host = "https://en.wikipedia.org/".to_string()
        }

        wikipedia_host.push_str(path.as_str());

        WikipediaUrl::new(wikipedia_host)
    }

    /// Get the raw [Url] from the [WikipediaUrl]
    pub fn get_url(&self) -> &Url {
        &self.0
    }
}

/// This error covers all failures related to the parsing of a [WikipediaUrl]
#[derive(Error, Debug)]
pub enum WikipediaUrlInvalidError {
    #[error("'{0}' is not a valid wikipedia url")]
    InvalidHostError(String),
    #[error("'{invalid_url}' failed to be parsed as a url: '{source}'")]
    InvalidUrlError {
        source: ReqwestError,
        invalid_url: String,
    },
}

/// A struct representing a Wikipedia page, optionally containing the title and body of the page
#[derive(Debug)]
pub struct Page {
    title: Option<String>,
    url: WikipediaUrl,
    body: Option<String>,
}

impl Page {
    /// Get the [Url] of the page
    pub fn get_url(&self) -> &Url {
        self.url.get_url()
    }

    /// Create a new [Page] with only a [WikipediaUrl]
    pub fn new(url: WikipediaUrl) -> Self {
        Page {
            title: None,
            url,
            body: None,
        }
    }

    /// Load the body of the wikipedia page into the struct
    pub fn load_body(&mut self) -> Result<(), ReqwestError> {
        if self.body.is_some() {
            return Ok(());
        }

        let url = self.get_url();

        self.body = Some(reqwest::blocking::get(url.clone())?.text()?);

        Ok(())
    }

    /// Get the body of the wikipedia page
    fn get_body(&mut self) -> Result<&String, ReqwestError> {
        self.load_body()?;

        match &self.body {
            Some(body) => Ok(body),
            None => panic!("Failed to load body for an unknown reason"),
        }
    }

    /// Remove the body from the wikipedia page to preserve memory
    pub fn unload_body(&mut self) {
        self.body = None
    }

    /// Load the title of the Wikipedia page into the struct, loading the body as well if necessary
    pub fn load_title(&mut self) -> Result<(), ReqwestError> {
        if self.title.is_some() {
            return Ok(());
        }

        self.title = Some(Self::get_title_from_body(self.get_body()?)?);

        Ok(())
    }

    /// Only load the title of the Wikipedia page if the body is loaded as well
    pub fn try_load_title(&mut self) -> Result<(), ReqwestError> {
        if self.title.is_some() {
            return Ok(());
        }

        if let Some(body) = &self.body {
            self.title = Some(Self::get_title_from_body(body)?)
        }

        Ok(())
    }

    /// Get the title of the Wikipedia page, loading it and the body as well if necessary
    pub fn get_title(&mut self) -> Result<String, ReqwestError> {
        self.load_title()?;

        Ok(self
            .title
            .clone()
            .expect("Title failed to load for an unknown reason"))
    }

    /// Only get the title of the Wikipedia page if the title is already loaded
    pub fn try_get_title(&mut self) -> Result<Option<String>, ReqwestError> {
        self.try_load_title()?;

        Ok(self.title.clone())
    }

    /// Get the title from a body of HTML
    fn get_title_from_body(body: &String) -> Result<String, ReqwestError> {
        let title_unparsed = body
            .lines()
            .filter(|l| l.contains("<title>"))
            .collect::<String>();

        let regex = Regex::new(r"([a-zA-Z ]+) - Wikipedia").expect("Title regex failed to compile");

        match regex.captures(&title_unparsed) {
            Some(captures) => Ok(captures.extract::<1>().1[0].to_string()),
            None => panic!("Failed to find title of wikipedia page"),
        }
    }

    /// Create a new [Page] and immediatly load the title
    pub fn new_load_title(wiki_url: WikipediaUrl) -> Result<Self, ReqwestError> {
        let mut page = Page::new(wiki_url);

        page.load_title()?;

        Ok(page)
    }

    /// Create a new [Page], supplying a title for it
    fn new_with_title(wiki_url: WikipediaUrl, title: String) -> Page {
        Page {
            title: Some(title),
            url: wiki_url,
            body: None,
        }
    }

    /// Get a list of [Page]s for all of the Wikipedia links on the page, loading the body as well if necessary
    pub fn get_connections(&mut self) -> Result<Vec<Page>, ReqwestError> {
        Self::get_connections_from_body(self.get_body()?)
    }

    /// Only get a list of [Page]s for all of the Wikipedia links on the page if the body is already loaded
    pub fn try_get_connections(&self) -> Option<Result<Vec<Page>, ReqwestError>> {
        match &self.body {
            Some(body) => Some(Self::get_connections_from_body(body)),
            None => None,
        }
    }

    /// Get a list of [Page]s for all of the Wikipedia links on the page from a body of HTML
    fn get_connections_from_body(body: &String) -> Result<Vec<Page>, ReqwestError> {
        let wiki_regex = Regex::new(
            "<a href=\"(/wiki/[a-zA-Z_\\(\\)]+)\"(?: class=\"[a-zA-Z-_]\")? title=\"([a-zA-Z ]+)\"",
        )
        .unwrap();

        let captures = wiki_regex
            .captures_iter(&body)
            .map(|c| c.extract::<2>())
            .filter(|c| !c.0.contains("Wayback Machine"))
            //.map(|c| dbg!(c))
            .filter_map(|c| {
                Some(Page::new_with_title(
                    WikipediaUrl::from_path(c.1[0].to_string()).ok()?,
                    c.1[1].to_string(),
                ))
            })
            //.map(|c| dbg!(c))
            .collect::<Vec<Page>>();

        Ok(captures)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Page, WikipediaUrl};

    #[test]
    fn test_get_connections() {
        let url = WikipediaUrl::from_path("/wiki/Waffle".to_string()).unwrap();
        let mut waffle_page = Page::new(url);

        waffle_page.get_connections().unwrap();
    }

    #[test]
    fn test_page_initialization() {
        let url = WikipediaUrl::from_path("/wiki/Waffle".to_string()).unwrap();
        let waffle_page = Page::new_load_title(url).unwrap();

        assert_eq!(waffle_page.title.unwrap().as_str(), "Waffle")
    }
}
