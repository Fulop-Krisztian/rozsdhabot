use chrono::NaiveDateTime;

#[derive(Debug)]
pub enum ListingType {
    Featured,
    Bazar,
    Regular,
}
pub type ListingId = i64;

#[derive(Debug)]
pub struct Listing {
    // Should be unique
    pub id: ListingId,
    // pub img_url: String,
    pub url: String,
    pub title: String,
    pub price: f64,
    pub cities: Vec<String>,
    pub date: NaiveDateTime,

    pub frozen: bool,

    // === Seller information ===
    pub seller_name: String,
    // We only store the positive ratings. Only that matters mostly on hardverapro.
    pub seller_ratings: i64,
    pub seller_url: String,

    pub listing_type: ListingType,
}
