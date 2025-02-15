pub trait Backend{
    fn draw_frame(&mut self, framebuffer: &[u8; 64 * 32]);
    fn poll_key(&mut self, key: Keys) -> bool;
    fn wait_for_key(&mut self);
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Keys{
    KEY1,
    KEY2,
    KEY3,
    KEY4,
    KEYQ,
    KEYW,
    KEYE,
    KEYR,
    KEYA,
    KEYS,
    KEYD,
    KEYF,
    KEYZ,
    KEYX,
    KEYC,
    KEYV,
}



impl From<u8> for Keys {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Keys::KEY1,
            0x1 => Keys::KEY2,
            0x2 => Keys::KEY3,
            0x3 => Keys::KEY4,
            0x4 => Keys::KEYQ,
            0x5 => Keys::KEYW,
            0x6 => Keys::KEYE,
            0x7 => Keys::KEYR,
            0x8 => Keys::KEYA,
            0x9 => Keys::KEYS,
            0xA => Keys::KEYD,
            0xB => Keys::KEYF,
            0xC => Keys::KEYZ,
            0xD => Keys::KEYX,
            0xE => Keys::KEYC,
            0xF => Keys::KEYV,
            _ => panic!("Invalid key value: {value}"),
        }
    }
}

impl From<Keys> for u8 {
    fn from(key: Keys) -> Self {
        match key {
            Keys::KEY1 => 0x0,
            Keys::KEY2 => 0x1,
            Keys::KEY3 => 0x2,
            Keys::KEY4 => 0x3,
            Keys::KEYQ => 0x4,
            Keys::KEYW => 0x5,
            Keys::KEYE => 0x6,
            Keys::KEYR => 0x7,
            Keys::KEYA => 0x8,
            Keys::KEYS => 0x9,
            Keys::KEYD => 0xA,
            Keys::KEYF => 0xB,
            Keys::KEYZ => 0xC,
            Keys::KEYX => 0xD,
            Keys::KEYC => 0xE,
            Keys::KEYV => 0xF,        }
    }
}

