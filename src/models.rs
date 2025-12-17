use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Listing {
    // Should be unique
    pub id: i32,
    pub url: String,
    pub title: String,
    pub price: f64,
    pub cities: Vec<String>,
    pub date: DateTime<Utc>,

    pub frozen: bool,

    // === Seller information ===
    pub seller_name: String,
    // We only store the positive ratings. Only that matters mostly on hardverapro.
    pub seller_ratings: String,
    pub seller_url: String,
}

impl Listing {
    fn format_markdown_message(&self) -> String {
        format!("{}\n[{}]({})",self.price, self.title, self.url)



        String::from("")
    }
}



