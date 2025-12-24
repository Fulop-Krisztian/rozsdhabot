mod hardverapro_parser;

use crate::models::Listing;

pub use self::hardverapro_parser::parse_hardverapro;

pub struct ParsedPage {
    pub metadata: ScrapeMetadata,
    pub listings: Vec<Listing>,
    pub failures: Vec<ParseFailure>,
}

#[derive(Debug, Clone)]
pub struct ScrapeMetadata {
    pub name: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

impl ParsedPage {
    pub fn skipped_listings_count(&self) -> usize {
        self.failures
            .iter()
            .filter(|f| f.kind == ParseFailureKind::Skipped)
            .count()
    }
    pub fn missing_field_listings_count(&self) -> usize {
        self.failures
            .iter()
            .filter(|f| f.kind == ParseFailureKind::Missing)
            .count()
    }
    pub fn invalid_filed_listing_count(&self) -> usize {
        self.failures
            .iter()
            .filter(|f| f.kind == ParseFailureKind::Invalid)
            .count()
    }

    pub fn unparsable_listing_count(&self) -> usize {
        self.failures
            .iter()
            .filter(|f| f.kind == ParseFailureKind::Invalid || f.kind == ParseFailureKind::Missing)
            .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Title,
    Id,
    Price,
    Cities,
    Date,
    Url,
    UrlHref,
    SellerName,
    SellerRatings,
    SellerUrl,
    SellerUrlHref,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Title => write!(f, "title"),
            Field::Id => write!(f, "listing id"),
            Field::Price => write!(f, "price"),
            Field::Cities => write!(f, "cities"),
            Field::Date => write!(f, "date"),
            Field::Url => write!(f, "listing link"),
            // This is a very unlikely edgecase.
            Field::UrlHref => write!(f, "listing link href"),
            Field::SellerName => write!(f, "seller name"),
            Field::SellerRatings => write!(f, "seller ratings"),
            Field::SellerUrl => write!(f, "seller link "),
            // This is a very unlikely edgecase.
            Field::SellerUrlHref => write!(f, "seller link href"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseFailureKind {
    Missing,
    Invalid,
    Skipped,
}

pub struct ParseFailure {
    pub field: Field,
    pub kind: ParseFailureKind,
    pub value: Option<String>,
}

impl ParseFailure {
    fn new(field: Field, kind: ParseFailureKind, value: Option<String>) -> Self {
        Self { field, kind, value }
    }

    fn missing(field: Field) -> Self {
        Self::new(field, ParseFailureKind::Missing, None)
    }

    fn invalid(field: Field, value: Option<String>) -> Self {
        Self::new(field, ParseFailureKind::Invalid, value)
    }

    fn skipped(field: Field) -> Self {
        Self::new(field, ParseFailureKind::Skipped, None)
    }
}
