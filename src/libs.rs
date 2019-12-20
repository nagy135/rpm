static PASSWORD_HASH_HOLDER: &'static str = "/home/infiniter/pass_hash";
const PASS_DELAY: u32 = 5;
static STORAGE: &'static str = "/home/infiniter/storage.db";

#[derive(Debug)]
struct Record {
    key: String,
    value: String
}
