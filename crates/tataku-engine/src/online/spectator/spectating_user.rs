
/// helper for spectating users since we only care about the user_id and username
#[derive(Clone)]
pub struct SpectatingUser {
    pub user_id: u32,
    pub username: String,
}
impl SpectatingUser {
    pub fn new(user_id: u32, username: impl ToString) -> Self {
        Self {
            user_id,
            username: username.to_string()
        }
    }
}
