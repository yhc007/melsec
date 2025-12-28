use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// PLC 읽기 결과 (Kafka 전송용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcReadResult {
    pub timestamp: DateTime<Utc>,
    pub word_data: Vec<AddressData>,
    pub bit_data: Vec<BitAddressData>,
}

/// 워드 디바이스 주소 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressData {
    pub address: String,
    pub value: u16,
    pub hex: String,
}

/// 비트 디바이스 주소 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitAddressData {
    pub address: String,
    pub value: bool,
}

impl PlcReadResult {
    /// 워드 데이터와 비트 데이터로부터 생성
    pub fn new(
        word_data: Vec<(u16, u16)>,
        bit_data: Vec<(u16, bool)>,
        addresses: &[String],
    ) -> Self {
        let mut word_results = Vec::new();
        let mut bit_results = Vec::new();

        // 워드 데이터 매핑
        for (addr_num, value) in word_data {
            if let Some(addr_str) = addresses.iter().find(|a| {
                if let Some((_, num)) = crate::device::Device::from_str(a) {
                    num == addr_num
                } else {
                    false
                }
            }) {
                word_results.push(AddressData {
                    address: addr_str.clone(),
                    value,
                    hex: format!("0x{:04X}", value),
                });
            }
        }

        // 비트 데이터 매핑
        for (addr_num, value) in bit_data {
            if let Some(addr_str) = addresses.iter().find(|a| {
                if let Some((_, num)) = crate::device::Device::from_str(a) {
                    num == addr_num
                } else {
                    false
                }
            }) {
                bit_results.push(BitAddressData {
                    address: addr_str.clone(),
                    value,
                });
            }
        }

        Self {
            timestamp: Utc::now(),
            word_data: word_results,
            bit_data: bit_results,
        }
    }
}

