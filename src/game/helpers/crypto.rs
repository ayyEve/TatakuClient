

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