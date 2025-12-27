use crate::device::Device;
use crate::error::{MelsecError, Result};
use crate::protocol::FrameBuilder;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// MELSEC PLC 클라이언트
pub struct MelsecClient {
    stream: TcpStream,
    frame_builder: FrameBuilder,
    timeout: Duration,
}

impl MelsecClient {
    /// 새 PLC 클라이언트 생성 및 연결
    pub async fn connect(
        addr: SocketAddr,
        network: u8,
        pc: u8,
    ) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        
        Ok(Self {
            stream,
            frame_builder: FrameBuilder::new(network, pc, 0x0001),
            timeout: Duration::from_secs(5),
        })
    }

    /// IP 주소와 포트로 연결
    pub async fn connect_str(
        ip: &str,
        port: u16,
        network: u8,
        pc: u8,
    ) -> Result<Self> {
        let addr = format!("{}:{}", ip, port);
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| MelsecError::ProtocolError(format!("주소 파싱 오류: {}", e)))?;
        
        Self::connect(socket_addr, network, pc).await
    }

    /// 타임아웃 설정
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// 워드 디바이스 읽기
    pub async fn read_words(
        &mut self,
        device: Device,
        start_addr: u16,
        count: u16,
    ) -> Result<Vec<u16>> {
        let frame = self.frame_builder.build_read_frame(device, start_addr, count);
        
        // 프레임 전송
        timeout(self.timeout, self.stream.write_all(&frame))
            .await??;
        
        // 응답 수신
        let mut header = vec![0u8; 15];
        timeout(self.timeout, self.stream.read_exact(&mut header))
            .await??;
        
        // 데이터 길이 확인
        let data_len = (header[13] as usize) | ((header[14] as usize) << 8);
        let mut response = header;
        response.resize(15 + data_len, 0);
        
        if data_len > 0 {
            timeout(
                self.timeout,
                self.stream.read_exact(&mut response[15..])
            )
            .await??;
        }
        
        // 응답 파싱
        FrameBuilder::parse_response(&response)
    }

    /// 비트 디바이스 읽기
    pub async fn read_bits(
        &mut self,
        device: Device,
        start_addr: u16,
        count: u16,
    ) -> Result<Vec<bool>> {
        let frame = self.frame_builder.build_read_frame(device, start_addr, count);
        
        timeout(self.timeout, self.stream.write_all(&frame))
            .await??;
        
        let mut header = vec![0u8; 15];
        timeout(self.timeout, self.stream.read_exact(&mut header))
            .await??;
        
        let data_len = (header[13] as usize) | ((header[14] as usize) << 8);
        let mut response = header;
        response.resize(15 + data_len, 0);
        
        if data_len > 0 {
            timeout(
                self.timeout,
                self.stream.read_exact(&mut response[15..])
            )
            .await??;
        }
        
        FrameBuilder::parse_bit_response(&response)
    }

    /// 워드 디바이스 쓰기
    pub async fn write_words(
        &mut self,
        device: Device,
        start_addr: u16,
        data: &[u16],
    ) -> Result<()> {
        let frame = self.frame_builder.build_write_frame(device, start_addr, data);
        
        timeout(self.timeout, self.stream.write_all(&frame))
            .await??;
        
        let mut response = vec![0u8; 15];
        timeout(self.timeout, self.stream.read_exact(&mut response))
            .await??;
        
        // 에러 체크만 수행
        if response.len() > 10 && response[10] != 0x00 {
            return Err(MelsecError::PlcError(response[10]));
        }
        
        Ok(())
    }

    /// 비트 디바이스 쓰기
    pub async fn write_bits(
        &mut self,
        device: Device,
        start_addr: u16,
        bits: &[bool],
    ) -> Result<()> {
        let frame = self.frame_builder.build_bit_write_frame(device, start_addr, bits);
        
        timeout(self.timeout, self.stream.write_all(&frame))
            .await??;
        
        let mut response = vec![0u8; 15];
        timeout(self.timeout, self.stream.read_exact(&mut response))
            .await??;
        
        if response.len() > 10 && response[10] != 0x00 {
            return Err(MelsecError::PlcError(response[10]));
        }
        
        Ok(())
    }

    /// 단일 워드 읽기
    pub async fn read_word(&mut self, device: Device, addr: u16) -> Result<u16> {
        let words = self.read_words(device, addr, 1).await?;
        words.first()
            .copied()
            .ok_or_else(|| MelsecError::ProtocolError("응답 데이터가 비어있습니다".to_string()))
    }

    /// 단일 워드 쓰기
    pub async fn write_word(&mut self, device: Device, addr: u16, value: u16) -> Result<()> {
        self.write_words(device, addr, &[value]).await
    }

    /// 단일 비트 읽기
    pub async fn read_bit(&mut self, device: Device, addr: u16) -> Result<bool> {
        let bits = self.read_bits(device, addr, 1).await?;
        bits.first()
            .copied()
            .ok_or_else(|| MelsecError::ProtocolError("응답 데이터가 비어있습니다".to_string()))
    }

    /// 단일 비트 쓰기
    pub async fn write_bit(&mut self, device: Device, addr: u16, value: bool) -> Result<()> {
        self.write_bits(device, addr, &[value]).await
    }

    /// 연결 종료
    pub async fn disconnect(mut self) -> Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }
}

