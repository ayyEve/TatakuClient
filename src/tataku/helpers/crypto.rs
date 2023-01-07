pub fn md5<B:AsRef<[u8]>>(body: B) -> String {
    format!("{:x}", md5::compute(body).to_owned())
}

pub fn sha512<B:AsRef<[u8]>>(body: B) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha512::new();
    hasher.update(body.as_ref());
    let hash = hasher.finalize();
    format!("{:02x?}", &hash[..])
        .replace(", ", "")
        .trim_start_matches("[")
        .trim_end_matches("]")
        .to_owned()
}

fn check_all_hex(s:&String) -> bool {
    const HEX_CHARS:[char;22] = [
        '0','1','2','3','4','5','6','7','8','9',
        'a','b','c','d','e','f',
        'A','B','C','D','E','F',
    ];
    for c in s.chars() {
        if !HEX_CHARS.contains(&c) { return false }
    }

    true
}


pub fn check_md5(s:String) -> String {
    if s.len() != 32 || !check_all_hex(&s) {
        md5(s)
    } else {
        s
    }
}
pub fn check_sha512(s:String) -> String {
    if s.len() != 128 || !check_all_hex(&s) {
        sha512(s)
    } else {
        s
    }
}
