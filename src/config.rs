use serde::Deserialize;
use std::fs;
use std::path::Path;

/// PLC 설정
#[derive(Debug, Clone, Deserialize)]
pub struct PlcConfig {
    pub ip: String,
    pub port: u16,
    #[serde(default = "default_network")]
    pub network: u8,
    #[serde(default = "default_pc")]
    pub pc: u8,
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
}

fn default_network() -> u8 { 0 }
fn default_pc() -> u8 { 255 }
fn default_interval() -> u64 { 1000 }

/// AAS 서버 설정
#[derive(Debug, Clone, Deserialize)]
pub struct AasConfig {
    pub server: String,
    #[serde(default)]
    pub aas_id: Option<String>,
}

/// PLC → AAS 매핑
#[derive(Debug, Clone, Deserialize)]
pub struct PlcAasMapping {
    /// PLC 디바이스 주소 (예: "D1000", "M100")
    pub device: String,
    /// 읽을 워드 수 (기본값: 1)
    #[serde(default = "default_count")]
    pub count: u16,
    /// AAS Submodel ID
    pub aas_submodel: String,
    /// AAS Property (idShort)
    pub aas_property: String,
    /// 데이터 타입
    #[serde(default)]
    pub data_type: DataType,
    /// 스케일 팩터 (곱셈)
    #[serde(default = "default_scale")]
    pub scale: f64,
    /// 오프셋 (덧셈)
    #[serde(default)]
    pub offset: f64,
}

fn default_count() -> u16 { 1 }
fn default_scale() -> f64 { 1.0 }

/// 데이터 타입
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    #[default]
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Bool,
}

/// 전체 설정
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub plc: PlcConfig,
    pub aas: AasConfig,
    #[serde(default)]
    pub mappings: Vec<PlcAasMapping>,
}

impl Config {
    /// 설정 파일 로드
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
