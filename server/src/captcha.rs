use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"^【(?P<Sender>.+)】.*?(?P<Number>[a-zA-Z\-0-9]{4,8})").unwrap();
}

pub fn captcha(content: &str) -> String {
    if let Some(captures) = RE.captures(&content) {
        return format!("{} - {}", &captures["Number"], &captures["Sender"]);
    }
    String::new()
}
