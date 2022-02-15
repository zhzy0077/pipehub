use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex = Regex::new(r"^【(?P<Sender>.+)】.*?(?P<Number>\d{4,8})").unwrap();
}

pub fn captcha(content: String) -> String {
    let re: Regex = Regex::new(r"^【(?P<Sender>.+)】.*?(?P<Number>\d{4,8})").unwrap();

    if let Some(captures) = re.captures(&content) {
        return format!(
            "{} - {}\n{}",
            &captures["Number"], &captures["Sender"], content
        );
    }
    content
}
