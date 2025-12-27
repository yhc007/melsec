use thiserror::Error;

#[derive(Error, Debug)]
pub enum MelsecError {
    #[error("통신 오류: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("프로토콜 오류: {0}")]
    ProtocolError(String),
    
    #[error("PLC 응답 오류: 코드 0x{0:02X}")]
    PlcError(u8),
    
    #[error("타임아웃")]
    Timeout(#[from] tokio::time::error::Elapsed),
    
    #[error("데이터 길이 오류: 예상 {expected}, 실제 {actual}")]
    LengthError { expected: usize, actual: usize },
    
    #[error("디바이스 주소 파싱 오류: {0}")]
    DeviceAddressError(String),
}

pub type Result<T> = std::result::Result<T, MelsecError>;

