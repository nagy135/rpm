use std::convert::From;

pub mod constants {
    pub static PASSWORD_HASH_HOLDER: &'static str = "/home/infiniter/pass_hash";
    pub const PASS_DELAY: u64 = 600;
    pub static STORAGE: &'static str = "/home/infiniter/storage.db";
    pub static IV: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";

    // {{{ Enum Event
    #[derive(Debug, Copy, Clone)]
    pub enum Event {
        New = 1,
        Get = 2,
        Validate = 3
    }
    impl Event {
        pub fn to_u8(&self) -> u8 {
            *self as u8
        }
    }
    impl From<&u8> for Event {
        fn from(i: &u8) -> Self {
            match i {
                1 => Event::New,
                2 => Event::Get,
                3 => Event::Validate,
                _ => Event::Validate
            }
        }
    }

    // }}}

}
