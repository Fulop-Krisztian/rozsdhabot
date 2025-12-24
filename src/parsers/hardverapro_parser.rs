use crate::{
    models::{Listing, ListingType},
    parsers::{Field, ParseFailure, ParsedPage, ScrapeMetadata},
};
use scraper::{ElementRef, Html, Selector};

pub fn parse_hardverapro(body: &str) -> ParsedPage {
    let document = Html::parse_document(body);

    let mut listings: Vec<Listing> = Vec::new();

    let ad_sel = Selector::parse("li.media").unwrap();
    let ads = document.select(&ad_sel);

    // Category for display
    let category_sel = Selector::parse("div.uad-categories-item.active>a").unwrap();
    let category = document
        .select(&category_sel)
        .next()
        .map(|catsel| catsel.inner_html());

    // Min and max price
    let minprice_sel = Selector::parse("input[name=\"minprice\"]").unwrap();
    let min_price = document
        .select(&minprice_sel)
        .next()
        .and_then(|x| x.attr("value"))
        .and_then(|price| price.parse::<f64>().ok());

    let maxprice_sel = Selector::parse("input[name=\"maxprice\"]").unwrap();
    let max_price = document
        .select(&maxprice_sel)
        .next()
        .and_then(|x| x.attr("value"))
        .and_then(|price| price.parse::<f64>().ok());

    // println!(
    //     "category: {}",
    //     category.unwrap_or("Unknown category".to_string())
    // );
    // println!(
    //     "minprice: {}, maxprice: {}",
    //     minprice.unwrap_or(0.0),
    //     maxprice.unwrap_or(0.0)
    // );

    let mut failures = Vec::new();

    for ad in ads {
        match parse_hardverapro_listing(ad) {
            Ok(listing) => listings.push(listing),
            // TODO: this should go to logging
            Err(e) => failures.push(e),
        }
    }

    ParsedPage {
        metadata: ScrapeMetadata {
            name: None,
            category,
            min_price,
            max_price,
        },
        listings,
        failures,
    }
}

/// Parses
/// Expects an ad, which is defined by li.media
/// Ignores listings without parsable price (keresem, ingyen)
fn parse_hardverapro_listing(ad: ElementRef<'_>) -> Result<Listing, ParseFailure> {
    // const BASE_URL: &str = "https://hardverapro.hu";

    let ribbon_sel = Selector::parse("a.uad-image>div.uad-corner-ribbon>span").unwrap();
    let listing_type = match ad.select(&ribbon_sel).next() {
        Some(ribbon) => match ribbon.inner_html().as_str() {
            "Bazár" => ListingType::Bazar,
            "Kiemelt" => ListingType::Featured,
            // Fresh listings are not differentiated
            "Friss" => ListingType::Regular,
            _ => ListingType::Regular,
        },
        None => ListingType::Regular,
    };

    let id_str = ad
        .attr("data-uadid")
        .ok_or(ParseFailure::missing(Field::Id))?;
    let id = id_str
        .parse::<i64>()
        .map_err(|_e| ParseFailure::invalid(Field::Id, Some(id_str.to_string())))?;

    let url_sel = Selector::parse("div.uad-col-title>h1>a").unwrap();
    let url = ad
        .select(&url_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::Url))?
        .attr("href")
        .ok_or(ParseFailure::missing(Field::UrlHref))?
        .to_string();

    let title_sel = Selector::parse("div.uad-col-title>h1>a").unwrap();
    let title = ad
        .select(&title_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::Title))?
        .inner_html();

    let price_sel = Selector::parse("div.uad-price>span").unwrap();
    let price_str = ad
        .select(&price_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::Price))?
        .inner_html()
        .replace(" ", "")
        .replace("Ft", "");
    let price = match price_str.as_str() {
        "Ingyenes" => 0.0,
        "Csere" => return Err(ParseFailure::skipped(Field::Price)),
        "Keresem" => return Err(ParseFailure::skipped(Field::Price)),
        _ => price_str
            .parse::<f64>()
            .map_err(|_e| ParseFailure::invalid(Field::Price, Some(price_str.to_string())))?,
    };

    let frozed_sel = Selector::parse("div.uad-price-iced").unwrap();
    let frozen = ad.select(&frozed_sel).next().is_some();

    let cities_sel = Selector::parse("div.uad-cities").unwrap();
    let cities = ad
        .select(&cities_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::Cities))?
        .inner_html()
        .split(", ")
        .map(|cities| cities.to_string())
        .collect();

    let seller_name_sel = Selector::parse("span.uad-user-text>a").unwrap();
    let seller_name = ad
        .select(&seller_name_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::SellerName))?
        .inner_html();

    // Seller ratings are 0 if the rating div is not present, but it can error out if the rating
    // div is not parseable.
    let seller_ratings_sel = Selector::parse("span.uad-rating-positive").unwrap();
    let seller_ratings_str = ad
        .select(&seller_ratings_sel)
        .next()
        .map(|x| x.inner_html());
    let seller_ratings = match seller_ratings_str {
        Some(ratings) => ratings
            // +123 => 123
            .replace("+", "")
            .parse::<i64>()
            .map_err(|_e| ParseFailure::invalid(Field::SellerRatings, Some(ratings.to_string())))?,
        None => 0,
    };

    let seller_url_sel = Selector::parse("span.uad-user-text>a").unwrap();
    // We need to cat the seller urls with the base url since they are relative
    let seller_url = ad
        .select(&seller_url_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::SellerUrl))?
        .attr("href")
        .ok_or(ParseFailure::missing(Field::SellerUrlHref))?
        .to_string();

    // TODO: maybe parse the date?
    let date_sel = Selector::parse("div.uad-time>time").unwrap();
    let date_str = ad
        .select(&date_sel)
        .next()
        .ok_or(ParseFailure::missing(Field::Date))?
        .inner_html();

    if date_str.contains("Előresorolva") {
        return Err(ParseFailure::skipped(Field::Date));
    }

    let date = convert_date(date_str.as_str())
        .map_err(|_e| ParseFailure::invalid(Field::Date, Some(date_str)))?;

    Ok(Listing {
        id,
        url,
        // img_url,
        title,
        price,
        cities,
        date,
        frozen,
        seller_name,
        seller_ratings,
        seller_url,
        listing_type,
    })
}

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
fn convert_date(expression: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    let now = Local::now().naive_local();

    if expression.starts_with("ma") {
        // e.g. "ma 14:30"
        let time_part = expression
            .split_whitespace()
            .nth(1)
            .expect("Unhandled date format: 'ma' without following hour and minute");
        let time = NaiveTime::parse_from_str(time_part, "%H:%M")?;

        Ok(NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
            .unwrap() // Should never fail.
            .and_time(time))
    } else if expression.starts_with("tegnap") {
        // e.g. "tegnap 09:15"
        let time_part = expression
            .split_whitespace()
            .nth(1)
            .expect("Unhandled date format: 'tegnap' without following hour and minute");
        let time = NaiveTime::parse_from_str(time_part, "%H:%M")?;

        Ok(NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
            .unwrap() // Should never fail.
            .and_time(time)
            - Duration::days(1))
    } else {
        // e.g. "2023-11-01"
        let date = NaiveDate::parse_from_str(expression, "%Y-%m-%d")?;
        // We can't know what the time is, so we assume 00:00:00
        Ok(date.and_hms_opt(0, 0, 0).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hardverapro() {
        let body = include_str!("../../tests/71_dated_listings.html");
        let results = parse_hardverapro(body);
        assert_eq!(results.listings.len(), 71);
        assert_eq!(results.failures.len(), 29);
    }
}
