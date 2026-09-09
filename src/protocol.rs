use crate::device::Device;
use crate::error::{MelsecError, Result};
use bytes::{BufMut, Bytes, BytesMut};

/// MC Protocol 명령 코드
#[derive(Debug, Clone, Copy)]
pub enum Command {
    BatchRead = 0x0401,      // 배치 읽기
    BatchWrite = 0x1401,     // 배치 쓰기
    RandomRead = 0x0403,     // 랜덤 읽기
    RandomWrite = 0x1402,    // 랜덤 쓰기
    MonitorOn = 0x0801,      // 모니터링 ON
    MonitorOff = 0x0802,     // 모니터링 OFF
}

/// MC Protocol 프레임 빌더
pub struct FrameBuilder {
    network: u8,
    pc: u8,
    request_id: u16,
}

impl FrameBuilder {
    pub fn new(network: u8, pc: u8, request_id: u16) -> Self {
        Self {
            network,
            pc,
            request_id,
        }
    }

    /// 읽기 프레임 생성
    pub fn build_read_frame(
        &self,
        device: Device,
        start_addr: u16,
        count: u16,
    ) -> Bytes {
        let mut buf = BytesMut::new();
        
        // MC Protocol 3E Binary Header (11 bytes)
        buf.put_u8(0x50); // Sub-header
        buf.put_u8(0x00); // Sub-header
        buf.put_u8(self.network); // Network number
        buf.put_u8(self.pc); // PC (Station) number
        buf.put_u16_le(0x03FF); // Request destination IO number
        buf.put_u8(0x00); // Request destination station number
        
        // Placeholder for data length (will be filled later)
        let len_pos = buf.len();
        buf.put_u16_le(0x0000);
        
        buf.put_u16_le(0x0000); // CPU monitoring timer
        
        // Request data
        buf.put_u16_le(Command::BatchRead as u16); // Command code
        buf.put_u16_le(0x0000); // Sub-command
        
        // Start address (3 bytes, little endian) - 주소를 먼저!
        let addr_24bit = start_addr as u32;
        buf.put_u8((addr_24bit & 0xFF) as u8);
        buf.put_u8(((addr_24bit >> 8) & 0xFF) as u8);
        buf.put_u8(((addr_24bit >> 16) & 0xFF) as u8);
        
        // Device code (1 byte) - 디바이스 코드를 나중에!
        buf.put_u8(device.code() as u8);
        
        // Device count
        buf.put_u16_le(count);
        
        // Update data length (timer 2 bytes + request data)
        let data_len = (buf.len() - 9) as u16;  // 9 = header without timer and data length
        buf[len_pos] = (data_len & 0xFF) as u8;
        buf[len_pos + 1] = ((data_len >> 8) & 0xFF) as u8;
        
        buf.freeze()
    }

    /// 쓰기 프레임 생성 (워드 데이터)
    pub fn build_write_frame(
        &self,
        device: Device,
        start_addr: u16,
        data: &[u16],
    ) -> Bytes {
        let mut buf = BytesMut::new();
        
        // 헤더 (11 bytes)
        buf.put_u8(0x50);
        buf.put_u8(0x00);
        buf.put_u8(self.network);
        buf.put_u8(self.pc);
        buf.put_u16(self.request_id);
        buf.put_u16(0x0000); // 데이터 길이 (나중에 계산)
        buf.put_u8(0xFF);
        buf.put_u8(0x03);
        buf.put_u8(0x00);
        buf.put_u8(0x04);
        
        let data_start = buf.len();
        
        // 서브헤더
        buf.put_u8(0x00);
        
        // 명령 코드
        buf.put_u16_le(Command::BatchWrite as u16);
        
        // 디바이스 코드
        buf.put_u16_le(device.code());
        
        // 시작 주소
        buf.put_u32_le(start_addr as u32);
        
        // 디바이스 포인트
        buf.put_u16_le(data.len() as u16);
        
        // 데이터
        for &word in data {
            buf.put_u16_le(word);
        }
        
        // 데이터 길이 업데이트
        let data_len = (buf.len() - data_start) as u16;
        buf[7] = (data_len & 0xFF) as u8;
        buf[8] = ((data_len >> 8) & 0xFF) as u8;
        
        buf.freeze()
    }

    /// 응답 프레임 파싱
    pub fn parse_response(buf: &[u8]) -> Result<Vec<u16>> {
        if buf.len() < 11 {
            return Err(MelsecError::LengthError {
                expected: 11,
                actual: buf.len(),
            });
        }
        
        // 에러 체크 (9-10번째 바이트, little endian)
        let error_code = (buf[9] as u16) | ((buf[10] as u16) << 8);
        if error_code != 0x0000 {
            let error_msg = match error_code {
                0xC056 => "지정된 디바이스가 범위를 벗어났거나 잘못된 주소입니다",
                0xC050 => "데이터 형식 오류",
                0xC051 => "데이터 길이 오류",
                0xC059 => "지정된 디바이스가 존재하지 않습니다",
                0xC05B => "읽기/쓰기 요청 길이 오류",
                0xC05C => "ASCII 변환 오류",
                0xC05F => "프레임 구조 오류",
                0xC060 => "프레임 길이 오류",
                0xC061 => "잘못된 CPU 모델",
                _ => "알 수 없는 PLC 오류",
            };
            return Err(MelsecError::PlcError(error_code, error_msg.to_string()));
        }
        
        // 데이터 추출 (워드 단위) - 11번째 바이트부터 실제 데이터 시작
        let mut result = Vec::new();
        let data_start = 11;
        
        if buf.len() > data_start {
            let data_len = buf.len() - data_start;
            for i in 0..(data_len / 2) {
                let offset = data_start + i * 2;
                if offset + 1 < buf.len() {
                    let word = (buf[offset] as u16) | ((buf[offset + 1] as u16) << 8);
                    result.push(word);
                }
            }
        }
        
        Ok(result)
    }
    
    /// 비트 쓰기 프레임 생성
    pub fn build_bit_write_frame(
        &self,
        device: Device,
        start_addr: u16,
        bits: &[bool],
    ) -> Bytes {
        // 비트 데이터를 바이트로 변환
        let bit_count = bits.len();
        let byte_count = (bit_count + 7) / 8;
        let mut bit_bytes = vec![0u8; byte_count];
        
        for (i, &bit) in bits.iter().enumerate() {
            if bit {
                bit_bytes[i / 8] |= 1 << (i % 8);
            }
        }
        
        let mut buf = BytesMut::new();
        
        // 헤더
        buf.put_u8(0x50);
        buf.put_u8(0x00);
        buf.put_u8(self.network);
        buf.put_u8(self.pc);
        buf.put_u16(self.request_id);
        buf.put_u16(0x0000);
        buf.put_u8(0xFF);
        buf.put_u8(0x03);
        buf.put_u8(0x00);
        buf.put_u8(0x04);
        
        let data_start = buf.len();
        
        buf.put_u8(0x00);
        buf.put_u16_le(Command::BatchWrite as u16);
        buf.put_u16_le(device.code());
        buf.put_u32_le(start_addr as u32);
        buf.put_u16_le(bit_count as u16);
        
        // 비트 데이터
        buf.extend_from_slice(&bit_bytes);
        
        let data_len = (buf.len() - data_start) as u16;
        buf[7] = (data_len & 0xFF) as u8;
        buf[8] = ((data_len >> 8) & 0xFF) as u8;
        
        buf.freeze()
    }
    
    /// 비트 응답 파싱
    pub fn parse_bit_response(buf: &[u8]) -> Result<Vec<bool>> {
        if buf.len() < 15 {
            return Err(MelsecError::LengthError {
                expected: 15,
                actual: buf.len(),
            });
        }
        
        // 에러 체크 (9-10번째 바이트, little endian)
        let error_code = (buf[9] as u16) | ((buf[10] as u16) << 8);
        if error_code != 0x0000 {
            let error_msg = match error_code {
                0xC056 => "지정된 디바이스가 범위를 벗어났거나 잘못된 주소입니다",
                0xC050 => "데이터 형식 오류",
                0xC051 => "데이터 길이 오류",
                0xC059 => "지정된 디바이스가 존재하지 않습니다",
                0xC05B => "읽기/쓰기 요청 길이 오류",
                0xC05C => "ASCII 변환 오류",
                0xC05F => "프레임 구조 오류",
                0xC060 => "프레임 길이 오류",
                0xC061 => "잘못된 CPU 모델",
                _ => "알 수 없는 PLC 오류",
            };
            return Err(MelsecError::PlcError(error_code, error_msg.to_string()));
        }
        
        let data_len = (buf[13] as u16) | ((buf[14] as u16) << 8);
        
        if buf.len() < 15 + data_len as usize {
            return Err(MelsecError::LengthError {
                expected: 15 + data_len as usize,
                actual: buf.len(),
            });
        }
        
        let mut result = Vec::new();
        let data_start = 15;
        
        for i in 0..data_len {
            let byte_idx = i as usize / 8;
            let bit_idx = i as usize % 8;
            
            if data_start + byte_idx < buf.len() {
                let byte = buf[data_start + byte_idx];
                result.push((byte & (1 << bit_idx)) != 0);
            }
        }
        
        Ok(result)
    }
}

