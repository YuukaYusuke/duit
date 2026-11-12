use chrono::NaiveDate;

pub fn valid_date(date: &str) -> Result<(), String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| "Tanggal tidak valid.".into())
}

pub fn validate_amount(amount: i64) -> Result<(), String> {
    if amount > 0 {
        Ok(())
    } else {
        Err("Nominal harus lebih dari nol.".into())
    }
}
