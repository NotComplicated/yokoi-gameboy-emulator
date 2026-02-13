use std::path::Path;

const ENTRY_POINT: usize = 0x0100;
const LOGO_START: usize = 0x0104;
const LOGO_END: usize = 0x0134;
const CHECKSUM_START: usize = 0x0134;
const TITLE_START: usize = 0x0134;
const CGB_FLAG: usize = 0x0143;
const TITLE_END: usize = 0x0144;
const NEW_LICENSEE_START: usize = 0x0144;
const NEW_LICENSEE_END: usize = 0x0146;
const FEATURES: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const OLD_LICENSEE: usize = 0x014B;
const CHECKSUM_END: usize = 0x014D;
const CHECKSUM_DIGEST: usize = 0x014D;
const HEADER_END: usize = 0x0150;

const CGB_COMPAT: u8 = 0x80;
const CGB_EXCL: u8 = 0xC0;

const USE_NEW_LICENSEE: u8 = 0x33;

const LOGO_BYTES: &[u8] = &[
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

#[derive(Debug)]
pub struct Cart(Vec<u8>);

#[derive(Debug)]
pub struct Error(pub &'static str);

#[derive(Debug)]
pub enum ColorSupport {
    BackwardsCompatible,
    Exclusive,
    No,
}

#[derive(Debug)]
pub enum Feature {
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    Mmm01,
    Ram,
    Battery,
    Timer,
    Rumble,
    Sensor,
    Camera,
    Tamagotchi,
    HuC1,
    HuC3,
}

impl Cart {
    pub fn new(data: Vec<u8>) -> Result<Self, Error> {
        if data.len() < HEADER_END {
            Err(Error("not enough data"))
        } else if &data[LOGO_START..LOGO_END] != LOGO_BYTES {
            Err(Error("missing Nintendo logo"))
        } else if !(data[TITLE_START..TITLE_END - 1].iter().all(u8::is_ascii)
            && (data[CGB_FLAG].is_ascii() || [CGB_COMPAT, CGB_EXCL].contains(&data[CGB_FLAG])))
        {
            Err(Error("missing title data"))
        } else {
            // validate checksum
            let digest = data[CHECKSUM_START..CHECKSUM_END]
                .iter()
                .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));
            if digest != data[CHECKSUM_DIGEST] {
                Err(Error("invalid checksum"))
            } else {
                Ok(Self(data))
            }
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.0
    }

    pub fn title(&self) -> &str {
        let region = &self.0[TITLE_START..TITLE_END];
        let end_pos = if let Some(pos) = region.iter().position(|&b| b == 0x00) {
            pos
        } else if [CGB_COMPAT, CGB_EXCL].contains(region.last().unwrap()) {
            region.len() - 1
        } else {
            region.len()
        };
        std::str::from_utf8(&region[0..end_pos]).expect("validated ascii")
    }

    pub fn color_supported(&self) -> ColorSupport {
        match self.0[CGB_FLAG] {
            CGB_COMPAT => ColorSupport::BackwardsCompatible,
            CGB_EXCL => ColorSupport::Exclusive,
            _ => ColorSupport::No,
        }
    }

    pub fn rom_size(&self) -> usize {
        32 * 1024 * 2usize.pow(self.0[ROM_SIZE] as _)
    }

    pub fn ram_size(&self) -> usize {
        match self.0[RAM_SIZE] {
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        }
    }

    pub fn features(&self) -> &'static [Feature] {
        match self.0[FEATURES] {
            0x01 => &[Feature::Mbc1],
            0x02 => &[Feature::Mbc1, Feature::Ram],
            0x03 => &[Feature::Mbc1, Feature::Ram, Feature::Battery],
            0x05 => &[Feature::Mbc2],
            0x06 => &[Feature::Mbc2, Feature::Battery],
            0x0B => &[Feature::Mmm01],
            0x0C => &[Feature::Mmm01, Feature::Ram],
            0x0D => &[Feature::Mmm01, Feature::Ram, Feature::Battery],
            0x0F => &[Feature::Mbc3, Feature::Timer, Feature::Battery],
            0x10 => &[
                Feature::Mbc3,
                Feature::Timer,
                Feature::Ram,
                Feature::Battery,
            ],
            0x11 => &[Feature::Mbc3],
            0x12 => &[Feature::Mbc3, Feature::Ram],
            0x13 => &[Feature::Mbc3, Feature::Ram, Feature::Battery],
            0x19 => &[Feature::Mbc5],
            0x1A => &[Feature::Mbc5, Feature::Ram],
            0x1B => &[Feature::Mbc5, Feature::Ram, Feature::Battery],
            0x1C => &[Feature::Mbc5, Feature::Rumble],
            0x1D => &[Feature::Mbc5, Feature::Rumble, Feature::Ram],
            0x1E => &[
                Feature::Mbc5,
                Feature::Rumble,
                Feature::Ram,
                Feature::Battery,
            ],
            0x20 => &[Feature::Mbc6],
            0x22 => &[
                Feature::Mbc7,
                Feature::Sensor,
                Feature::Rumble,
                Feature::Ram,
                Feature::Battery,
            ],
            0xFC => &[Feature::Camera],
            0xFD => &[Feature::Tamagotchi],
            0xFE => &[Feature::HuC3],
            0xFF => &[Feature::HuC1, Feature::Ram, Feature::Battery],
            _ => &[],
        }
    }

    pub fn licensee(&self) -> &'static str {
        match self.0[OLD_LICENSEE] {
            USE_NEW_LICENSEE => match &self.0[NEW_LICENSEE_START..NEW_LICENSEE_END] {
                b"01" => "Nintendo R&D",
                b"08" => "Capcom",
                b"13" | b"69" => "EA",
                b"18" | b"38" => "Hudson Soft",
                b"19" => "B-AI",
                b"20" => "KSS",
                b"22" => "Planning Office WADA",
                b"24" => "PCM Complete",
                b"25" => "San-X",
                b"28" => "Kemco",
                b"29" => "SETA Corporation",
                b"30" => "Viacom",
                b"31" => "Nintendo",
                b"32" => "Bandai",
                b"33" | b"93" => "Ocean Software",
                b"34" | b"54" => "Konami",
                b"35" => "HectorSoft",
                b"37" => "Taito",
                b"39" => "Banpresto",
                b"41" => "Ubi Soft",
                b"42" => "Atlus",
                b"44" => "Malibu Interactive",
                b"46" => "Angel",
                b"47" => "Bullet-Proof Software",
                b"49" => "Irem",
                b"50" => "Absolute",
                b"51" => "Acclaim Entertainment",
                b"52" => "Activision",
                b"53" => "Sammy USA Corporation",
                b"55" => "Hi Tech Expressions",
                b"56" => "LJN",
                b"57" => "Matchbox",
                b"58" => "Mattel",
                b"59" => "Milton Bradley Company",
                b"60" => "Titus Interactive",
                b"61" => "Virgin Games Ltd.",
                b"64" => "Lucasfilm Games",
                b"67" => "Ocean Software",
                b"70" => "Infogrames",
                b"71" => "Interplay Entertainment",
                b"72" => "Broderbund",
                b"73" => "Sculptured Software",
                b"75" => "The Sales Curve Limited",
                b"78" => "THQ",
                b"79" => "Accolade",
                b"80" => "Misawa Entertainment",
                b"83" => "LOZC G.",
                b"86" => "Tokuma Shoten",
                b"87" => "Tsukuda Original",
                b"91" => "Chunsoft Co.",
                b"92" => "Video System",
                b"95" => "Varie",
                b"96" => "Yonezawa",
                b"97" => "Kaneko",
                b"99" => "Pack-In-Video",
                b"9H" => "Bottom Up",
                b"A4" => "Konami (Yu-Gi-Oh!)",
                b"BL" => "MTO",
                b"DK" => "Kodansha",
                _ => "N/A",
            },
            0x01 | 0x31 => "Nintendo",
            0x08 | 0x38 => "Capcom",
            0x09 => "HOT-B",
            0x0A | 0xE0 => "Jaleco",
            0x0B => "Coconuts Japan",
            0x0C | 0x6E => "Elite Systems",
            0x13 | 0x69 => "EA",
            0x18 => "Hudson Soft",
            0x19 => "ITC Entertainment",
            0x1A => "Yanoman",
            0x1D => "Japan Clary",
            0x1F | 0x4A | 0x61 => "Virgin Games Ltd.",
            0x24 => "PCM Complete",
            0x25 => "San-X",
            0x28 | 0x7F | 0x97 | 0xC2 => "Kemco",
            0x29 => "SETA Corporation",
            0x30 | 0x70 => "Infogrames",
            0x32 | 0xA2 | 0xB2 => "Bandai",
            0x34 | 0xA4 => "Konami",
            0x35 => "HectorSoft",
            0x39 | 0x9D | 0xD9 => "Banpresto",
            0x3C => "Entertainment Interactive",
            0x3E => "Gremlin",
            0x41 => "Ubi Soft",
            0x42 | 0xEB => "Atlus",
            0x44 | 0x4D => "Malibu Interactive",
            0x46 | 0xCF => "Angel",
            0x47 => "Spectrum HoloByte",
            0x49 => "Irem",
            0x4F => "U.S. Gold",
            0x50 => "Absolute",
            0x51 | 0xB0 => "Acclaim Entertainment",
            0x52 => "Activision",
            0x53 => "Sammy USA Corporation",
            0x54 => "GameTek",
            0x55 => "Park Place",
            0x56 | 0xDB | 0xFF => "LJN",
            0x57 => "Matchbox",
            0x59 => "Milton Bradley Company",
            0x5A => "Mindscape",
            0x5B => "Romstar",
            0x5C | 0xD6 => "Naxat Soft",
            0x5D => "Tradewest",
            0x60 => "Titus Interactive",
            0x67 => "Ocean Software",
            0x6F => "Electro Brain",
            0x71 => "Interplay Entertainment",
            0x72 | 0xAA => "Broderbund",
            0x73 => "Sculptured Software",
            0x75 => "The Sales Curve Limited",
            0x78 => "THQ",
            0x79 => "Accolade",
            0x7A => "Triffix Entertainment",
            0x7C => "MicroProse",
            0x80 => "Misawa Entertainment",
            0x83 => "LOZC G.",
            0x86 | 0xC4 => "Tokuma Shoten",
            0x8B => "Bullet-Proof Software",
            0x8C => "Vic Tokai Corp.",
            0x8E => "Ape Inc.",
            0x8F => "IMax",
            0x91 => "Chunsoft Co.",
            0x92 => "Video System",
            0x93 => "Tsubaraya Productions",
            0x95 | 0xE3 => "Varie",
            0x96 => "Yonezawa",
            0x99 => "Arc",
            0x9A => "Nihon Bussan",
            0x9B => "Tecmo",
            0x9C => "Imagineer",
            0x9F => "Nova",
            0xA1 => "Hori Electric",
            0xA6 => "Kawada",
            0xA7 => "Takara",
            0xA9 => "Technos Japan",
            0xAC => "Toei Animation",
            0xAD => "Toho",
            0xAF => "Namco",
            0xB1 => "ASCII Corporation",
            0xB4 => "Square Enix",
            0xB6 => "HAL Laboratory",
            0xB7 => "SNK",
            0xB9 | 0xCE => "Pony Canyon",
            0xBA => "Culture Brain",
            0xBB => "Sunsoft",
            0xBD => "Sony Imagesoft",
            0xBF => "Sammy Corporation",
            0xC0 | 0xD0 => "Taito",
            0xC3 => "Square",
            0xC5 => "Data East",
            0xC6 => "Tonkin House",
            0xC8 => "Koei",
            0xC9 => "UFL",
            0xCA => "Ultra Games",
            0xCB => "VAP, Inc.",
            0xCC => "Use Corporation",
            0xCD => "Meldac",
            0xD1 => "SOFEL",
            0xD2 => "Quest",
            0xD3 => "Sigma Enterprises",
            0xD4 => "ASK Kodansha Co.",
            0xD7 => "Copya System",
            0xDA => "Tomy",
            0xDD => "Nippon Computer Systems",
            0xDE => "Human Ent.",
            0xDF => "Altron",
            0xE1 => "Towa Chiki",
            0xE2 => "Yutaka",
            0xE5 => "Epoch",
            0xE7 => "Athena",
            0xE8 => "Asmik Ace Entertainment",
            0xE9 => "Natsume",
            0xEA => "King Records",
            0xEC => "Epic/Sony Records",
            0xEE => "IGS",
            0xF0 => "A Wave",
            0xF3 => "Extreme Entertainment",
            _ => "N/A",
        }
    }
}
