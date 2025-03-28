
#[macro_export]
macro_rules! async_retain {
    ($list:ident, $item:ident, $check_fn:expr) => {{
        let mut to_remove = Vec::new();
        for (n, $item) in $list.iter().enumerate() {
            if !$check_fn {
                to_remove.push(n)
            }
        }

        for i in to_remove.into_iter().rev() {
            $list.remove(i);
        }
    }}
}



/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_number(num: impl num_format::ToFormattedStr) -> String {
    use num_format::{ Buffer, Locale };
    let mut buf = Buffer::default();
    buf.write_formatted(&num, &Locale::en);

    buf.as_str().to_owned()
}

/// format a float into a locale string ie 1000.1 -> 1,000.100
pub fn format_float(num: impl ToString, precis: usize) -> String {

    let num = num.to_string();
    let mut split = num.split(".");
    let Some(num) = split.next().and_then(|a| a.parse::<i64>().ok()).map(format_number) else { return String::new() };

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

