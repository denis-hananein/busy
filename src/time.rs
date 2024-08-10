use chrono::{ParseResult, TimeZone};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DateTimeInterval {
  pub start_time: chrono::DateTime<chrono::Local>,
  pub stop_time: Option<chrono::DateTime<chrono::Local>>,
}

impl DateTimeInterval {
  pub fn duration(&self) -> chrono::Duration {
    return self
      .stop_time
      .unwrap_or(chrono::Local::now())
      .signed_duration_since(self.start_time);
  }
}

pub fn parse_datetime(datetime: &str) -> ParseResult<chrono::DateTime<chrono::Local>> {
  let mut input = datetime.to_owned();
  if !input.contains(' ') {
    input = format!("{} {}", chrono::Local::now().format("%Y-%m-%d"), input);
  }
  return chrono::Local.datetime_from_str(&input, "%Y-%m-%d %H:%M");
}

#[cfg(test)]
mod tests {
  use crate::time::parse_datetime;

  #[test]
  fn test_parse_datetime() {
    let datetime = parse_datetime("2020-01-01 00:00");
    assert!(datetime.unwrap().format("%Y-%m-%d %H:%M:%S").to_string() == "2020-01-01 00:00:00");
  }

  #[test]
  fn test_parse_time() {
    let datetime = parse_datetime("11:00");
    assert!(datetime.is_ok());

    assert_eq!(
      datetime.unwrap().format("%Y-%m-%d %H:%M").to_string(),
      format!("{} {}", chrono::Local::now().format("%Y-%m-%d"), "11:00")
    );
  }
}
