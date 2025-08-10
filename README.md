# [Wikipedia](wikipedia.org) as a [Semantic Network](https://wikipedia.org/wiki/Semantic_network)!

Made by Waffleleroo <3

---

A simple crate providing an easy way to access a Wikipedia page
and other Wikipedia links on that page

Please don't spam requests at Wikipedia just because you can!

Here's a little code example:
```rust
let url = WikipediaUrl::from_path("/wiki/Waffles").unwrap(); // Parse the url
let mut waffles_page = Page::new(url); // Turn it into the more flexible page struct

// Load the body into the struct and get the title
let title: String = waffles_page.get_title()?; 
assert_eq!(title.as_str(), "Waffle");

// Get all the Wikipedia links on the page
let connections: Vec<Page> = waffles_page.get_connections()?; 

// Remove the body from the struct for memory efficiency
waffles_page.unload_body(); 

for mut page in connections {
    // Print the names and URLs of all of the pages
    println!("Page {} at url {}", page.get_title()?, page.get_url()) 
}
```

Have fun!

Licensed under the [MIT License](LICENSE-MIT) or [Unlicense](UNLICENSE)

This project is neither official nor recognized by the Wikimedia Foundation