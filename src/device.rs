use std::fmt;

/// MELSEC PLC 디바이스 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    /// 비트 디바이스 (X, Y, M, L, F, V, B, SB, DX, DY, D, W, SM, SD, SW, FD)
    Bit(BitDevice),
    /// 워드 디바이스 (D, W, SD, SW, FD, R, ZR)
    Word(WordDevice),
}

/// 비트 디바이스 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDevice {
    X,  // 입력 릴레이
    Y,  // 출력 릴레이
    M,  // 내부 릴레이
    L,  // 래치 릴레이
    F,  // 어넌시에이터
    V,  // 엣지 릴레이
    B,  // 링크 릴레이
    SB, // 링크 특수 릴레이
    DX, // 직접 입력 릴레이
    DY, // 직접 출력 릴레이
}

/// 워드 디바이스 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordDevice {
    D,  // 데이터 레지스터
    W,  // 링크 레지스터
    SD, // 링크 특수 레지스터
    SW, // 링크 특수 레지스터
    FD, // 파일 레지스터
    R,  // 파일 레지스터
    ZR, // 파일 레지스터
}

impl Device {
    /// 디바이스 코드 반환 (MC Protocol용)
    pub fn code(&self) -> u16 {
        match self {
            Device::Bit(bit) => bit.code(),
            Device::Word(word) => word.code(),
        }
    }
    
    /// 디바이스 이름 반환
    pub fn name(&self) -> &'static str {
        match self {
            Device::Bit(bit) => bit.name(),
            Device::Word(word) => word.name(),
        }
    }
    
    /// 주소 문자열에서 디바이스 파싱 (예: "D100", "M0", "X10")
    pub fn from_str(addr: &str) -> Option<(Device, u16)> {
        if addr.is_empty() {
            return None;
        }
        
        let addr_upper = addr.to_uppercase();
        
        // 2글자 prefix 확인
        if addr_upper.len() >= 2 {
            let prefix2 = &addr_upper[..2];
            if let Some(number_str) = addr_upper.get(2..) {
                if let Ok(number) = number_str.parse::<u16>() {
                    match prefix2 {
                        "SB" => return Some((Device::Bit(BitDevice::SB), number)),
                        "SD" => return Some((Device::Word(WordDevice::SD), number)),
                        "SW" => return Some((Device::Word(WordDevice::SW), number)),
                        "FD" => return Some((Device::Word(WordDevice::FD), number)),
                        "ZR" => return Some((Device::Word(WordDevice::ZR), number)),
                        "DX" => return Some((Device::Bit(BitDevice::DX), number)),
                        "DY" => return Some((Device::Bit(BitDevice::DY), number)),
                        _ => {}
                    }
                }
            }
        }
        
        // 1글자 prefix 확인
        let prefix = addr_upper.chars().next()?;
        let number_str = &addr_upper[1..];
        let number = number_str.parse::<u16>().ok()?;
        
        match prefix {
            'X' => Some((Device::Bit(BitDevice::X), number)),
            'Y' => Some((Device::Bit(BitDevice::Y), number)),
            'M' => Some((Device::Bit(BitDevice::M), number)),
            'L' => Some((Device::Bit(BitDevice::L), number)),
            'F' => Some((Device::Bit(BitDevice::F), number)),
            'V' => Some((Device::Bit(BitDevice::V), number)),
            'B' => Some((Device::Bit(BitDevice::B), number)),
            'D' => Some((Device::Word(WordDevice::D), number)),
            'W' => Some((Device::Word(WordDevice::W), number)),
            'R' => Some((Device::Word(WordDevice::R), number)),
            _ => None,
        }
    }
}

impl BitDevice {
    fn code(&self) -> u16 {
        match self {
            BitDevice::X => 0x9C,
            BitDevice::Y => 0x9D,
            BitDevice::M => 0x90,
            BitDevice::L => 0x92,
            BitDevice::F => 0x93,
            BitDevice::V => 0x94,
            BitDevice::B => 0xA0,
            BitDevice::SB => 0xA1,
            BitDevice::DX => 0xA2,
            BitDevice::DY => 0xA3,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            BitDevice::X => "X",
            BitDevice::Y => "Y",
            BitDevice::M => "M",
            BitDevice::L => "L",
            BitDevice::F => "F",
            BitDevice::V => "V",
            BitDevice::B => "B",
            BitDevice::SB => "SB",
            BitDevice::DX => "DX",
            BitDevice::DY => "DY",
        }
    }
}

impl WordDevice {
    fn code(&self) -> u16 {
        match self {
            WordDevice::D => 0xA8,
            WordDevice::W => 0xB4,
            WordDevice::SD => 0xA9,
            WordDevice::SW => 0xB5,
            WordDevice::FD => 0xCC,
            WordDevice::R => 0xAF,
            WordDevice::ZR => 0xB0,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            WordDevice::D => "D",
            WordDevice::W => "W",
            WordDevice::SD => "SD",
            WordDevice::SW => "SW",
            WordDevice::FD => "FD",
            WordDevice::R => "R",
            WordDevice::ZR => "ZR",
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

