
/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_number(num: &impl num_format::ToFormattedStr) -> String {
    use num_format::{ Buffer, Locale };
    let mut buf = Buffer::default();
    buf.write_formatted(num, &Locale::en);

    buf.as_str().to_owned()
}

/// format a float into a locale string ie 1000.1 -> 1,000.100
pub fn format_float(num: &impl ToString, precis: usize) -> String {
    let num = num.to_string();
    let mut split = num.split(".");
    let Some(num) = split
        .next()
        .and_then(|a| a.parse::<i64>().ok())
        .as_ref()
        .map(format_number) else { 
            return String::new() 
        };

    let Some(dec) = split.next() else {
        return format!("{num}.{}", "0".repeat(precis));
    };

    let dec = if dec.len() > precis {
        dec.split_at(precis).0.to_owned()
    } else {
        format!("{dec:0precis$}")
    };

    format!("{num}.{dec}")
}
