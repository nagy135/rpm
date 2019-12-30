pub mod constants {
    pub static PASSWORD_HASH_HOLDER: &'static str = "/home/infiniter/pass_hash";
    pub const PASS_DELAY: u32 = 5;
    pub static STORAGE: &'static str = "/home/infiniter/storage.db";

    // {{{ Enum Event
    #[derive(Debug, Copy, Clone)]
    pub enum Event {
        New = 1,
        Get = 2
    }
    impl Event {
        pub fn to_u8(&self) -> u8 {
            *self as u8
        }
    }

    // }}}

}
