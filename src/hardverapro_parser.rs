use crate::models::Listing;
use chrono::Utc;
use scraper::{ElementRef, Html, Selector};

pub fn parse_hardverapro(body: &str) -> Vec<Listing> {
    let document = Html::parse_document(body);

    let mut results: Vec<Listing> = Vec::new();

    let ad_sel = Selector::parse("li.media").unwrap();
    let ads = document.select(&ad_sel);

    // Category for display
    let category_sel = Selector::parse("div.uad-categories-item.active>a").unwrap();
    let category = document
        .select(&category_sel)
        .next()
        .map(|catsel| catsel.inner_html());

    println!(
        "category: {}",
        category.unwrap_or("Unknown category".to_string())
    );

    // Min and max price
    let minprice_sel = Selector::parse("input[name=\"minprice\"]").unwrap();
    let minprice = document
        .select(&minprice_sel)
        .next()
        .and_then(|x| x.attr("value"))
        .and_then(|price| price.parse::<f64>().ok());

    let maxprice_sel = Selector::parse("input[name=\"maxprice\"]").unwrap();
    let maxprice = document
        .select(&maxprice_sel)
        .next()
        .and_then(|x| x.attr("value"))
        .and_then(|price| price.parse::<f64>().ok());

    println!(
        "minprice: {}, maxprice: {}",
        minprice.unwrap_or(0.0),
        maxprice.unwrap_or(0.0)
    );

    for ad in ads {
        match parse_hardverapro_listing(ad) {
            Ok(listing) => results.push(listing),
            Err(message) => eprintln!("{message}"),
        }
    }

    results
}

/// Parses
/// Expects an ad, which is defined by li.media
fn parse_hardverapro_listing(ad: ElementRef<'_>) -> Result<Listing, String> {
    const BASE_URL: &str = "https://hardverapro.hu";

    // TODO: replace Option with Result

    // TODO: replace unwraps with logging.
    //
    // let info_sel = Selector::parse("div.uad-col-info").unwrap();

    // let date_sel = Selector::parse("div.uad-time").unwrap();

    let url_sel = Selector::parse("div.uad-col-title>h1>a").unwrap();
    let url = match ad.select(&url_sel).next() {
        Some(url_fragment) => url_fragment
            .attr("href")
            .expect("Expected href in URL")
            .to_string(),
        None => return Err(format!("skipping ad: missing url {}", ad.html())),
    };

    let title_sel = Selector::parse("div.uad-col-title>h1>a").unwrap();
    let title = match ad.select(&title_sel).next() {
        Some(title_fragment) => title_fragment.inner_html(),
        None => return Err(format!("skipping ad: missing title {}", ad.html())),
    };

    // unwrap is for parsing the selector string itself.
    let price_sel = Selector::parse("div.uad-price>span").unwrap();
    let price = match ad.select(&price_sel).next() {
        Some(price_fragment) => price_fragment
            .inner_html()
            .replace(" ", "")
            .replace("Ft", "")
            .parse::<f64>()
            .unwrap_or(-1.0),
        None => return Err(format!("skipping ad: missing price {}", ad.html())),
    };

    let id = ad.attr("data-uadid").unwrap().parse::<i32>().unwrap();

    let frozed_sel = Selector::parse("div.uad-price-iced").unwrap();
    let frozen = ad.select(&frozed_sel).next().is_some();

    let cities_sel = Selector::parse("div.uad-cities").unwrap();
    let cities_string = match ad.select(&cities_sel).next() {
        Some(cities_fragment) => cities_fragment.inner_html(),
        None => return Err(format!("skipping ad: missing city {}", ad.html())),
    };

    let cities: Vec<String> = cities_string
        .split(", ")
        .map(|cities| cities.to_string())
        .collect();

    let seller_name_sel = Selector::parse("span.uad-user-text>a").unwrap();
    let seller_name = match ad.select(&seller_name_sel).next() {
        Some(seller_name_fragment) => seller_name_fragment.inner_html(),
        None => return Err(format!("skipping ad: missing seller name {}", ad.html())),
    };

    let seller_ratings_sel = Selector::parse("span.uad-rating-positive").unwrap();
    let seller_ratings = match ad.select(&seller_ratings_sel).next() {
        Some(seller_ratings_fragment) => seller_ratings_fragment.inner_html(),
        None => String::from("n.a."),
        // None => return Err(format!("skipping ad: missing seller ratings {}", ad.html())),
    };

    let seller_url_sel = Selector::parse("span.uad-user-text>a").unwrap();

    // We need to cat the seller urls with the base url since they are relative
    let seller_url = match ad.select(&seller_url_sel).next() {
        Some(seller_url_fragment) => {
            String::from(BASE_URL) + seller_url_fragment.attr("href").unwrap()
        }

        None => return Err(format!("skipping ad: missing seller url {}", ad.html())),
    };

    // TODO: maybe parse the date?
    let date = Utc::now();

    Ok(Listing {
        id,
        url,
        title,
        price,
        cities,
        date,
        frozen,
        seller_name,
        seller_ratings,
        seller_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hardverapro() {
        let body = include_str!("../index.html");
        let results = parse_hardverapro(body);
        assert_eq!(results.len(), 1);
    }
}
