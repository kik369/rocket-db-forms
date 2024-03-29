use chrono::NaiveDateTime;
use rocket::request::FlashMessage;
use std::fmt::Debug;

pub fn serialise_data<T, E, I>(items: I) -> Vec<T>
where
    I: Iterator<Item = Result<T, E>>,
    E: Debug,
{
    items
        .filter_map(|item| item.map_err(|_e| {}).ok())
        .collect()
}

// parses from "2020-01-01T00:00:00" to "2020-01-01 00:00:00"
// "2020-01-01T00:00:00" is the format that the datepicker returns
// "2020-01-01 00:00:00" is the format generated by 'DATETIME DEFAULT CURRENT_TIMESTAMP' in sqlite
pub fn parse_date(date: &str) -> Result<String, ()> {
    let parsed_end_date = NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse date string");
    Ok(parsed_end_date.format("%Y-%m-%d %H:%M:%S").to_string())
}

pub fn get_flash_msg(flash: Option<FlashMessage<'_>>) -> String {
    flash
        .map(|flash| format!("{}: {}", flash.kind(), flash.message()))
        .unwrap_or_default()
}
