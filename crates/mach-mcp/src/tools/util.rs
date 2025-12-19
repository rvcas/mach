use chrono::NaiveDate;
use machich::service::todo::ListScope;
use miette::{IntoDiagnostic, Result};

pub fn parse_scope(s: &str, today: NaiveDate) -> Result<ListScope> {
    match s.trim().to_lowercase().as_str() {
        "today" => Ok(ListScope::Day(today)),
        "backlog" | "someday" => Ok(ListScope::Backlog),
        date_str => {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .into_diagnostic()
                .map_err(|_| {
                    miette::miette!("invalid scope, expected 'today', 'backlog', or YYYY-MM-DD")
                })?;
            Ok(ListScope::Day(date))
        }
    }
}

pub fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .into_diagnostic()
        .map_err(|_| miette::miette!("invalid date format, expected YYYY-MM-DD"))
}
