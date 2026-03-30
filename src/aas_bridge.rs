use crate::aas_client::AasClient;
use crate::client::MelsecClient;
use crate::config::{Config, DataType, PlcAasMapping};
use crate::device::Device;
use std::time::Duration;
use tokio::time;

/// PLC-AAS 브릿지
pub struct AasBridge {
    config: Config,
    melsec: Option<MelsecClient>,
    aas_client: AasClient,
}

impl AasBridge {
    /// 새 브릿지 생성
    pub fn new(config: Config) -> Self {
        let aas_client = AasClient::new(&config.aas.server);
        
        Self {
            config,
            melsec: None,
            aas_client,
        }
    }

    /// PLC 연결
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[INFO] Connecting to PLC {}:{}...", self.config.plc.ip, self.config.plc.port);
        
        let client = MelsecClient::connect_str(
            &self.config.plc.ip,
            self.config.plc.port,
            self.config.plc.network,
            self.config.plc.pc,
        ).await?;
        
        self.melsec = Some(client);
        println!("[INFO] PLC connected!");
        Ok(())
    }

    /// 폴링 루프 시작
    pub async fn start_polling(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let interval = Duration::from_millis(self.config.plc.interval_ms);
        let mut ticker = time::interval(interval);

        println!("[INFO] Starting polling loop (interval: {}ms)...", self.config.plc.interval_ms);
        println!("[INFO] Mappings: {} devices", self.config.mappings.len());

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.poll_and_update().await {
                eprintln!("[ERROR] Poll error: {}", e);
                // 연결 끊김 시 재연결 시도
                if e.to_string().contains("connection") || e.to_string().contains("broken") {
                    eprintln!("[WARN] Attempting reconnection...");
                    time::sleep(Duration::from_secs(5)).await;
                    if let Err(e) = self.connect().await {
                        eprintln!("[ERROR] Reconnection failed: {}", e);
                    }
                }
            }
        }
    }

    /// 한 번의 폴링 및 AAS 업데이트
    async fn poll_and_update(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mappings = self.config.mappings.clone();
        
        for mapping in &mappings {
            match self.read_and_update_single(&mapping).await {
                Ok(_) => {
                    println!("[OK] {} -> {}/{}", 
                        mapping.device, 
                        mapping.aas_submodel, 
                        mapping.aas_property
                    );
                }
                Err(e) => {
                    eprintln!("[FAIL] {} error: {}", mapping.device, e);
                }
            }
        }

        Ok(())
    }

    /// 단일 매핑 읽기 및 업데이트
    async fn read_and_update_single(
        &mut self,
        mapping: &PlcAasMapping,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 디바이스 주소 파싱 (예: "D1000" -> Device, 1000)
        let (device, addr) = Device::from_str(&mapping.device)
            .ok_or_else(|| format!("Invalid device address: {}", mapping.device))?;
        
        // PLC에서 읽기
        let melsec = self.melsec.as_mut()
            .ok_or("PLC not connected")?;
        let words = melsec.read_words(device, addr, mapping.count).await?;
        
        // 값 변환 및 스케일 적용
        let value = convert_value(&words, &mapping.data_type, mapping.scale, mapping.offset);
        
        // AAS에 업데이트
        match mapping.data_type {
            DataType::Bool => {
                let bool_val = words.first().map(|&w| w != 0).unwrap_or(false);
                self.aas_client.update_bool(
                    &mapping.aas_submodel,
                    &mapping.aas_property,
                    bool_val,
                ).await?;
            }
            DataType::Int16 | DataType::Int32 => {
                self.aas_client.update_integer(
                    &mapping.aas_submodel,
                    &mapping.aas_property,
                    value as i64,
                ).await?;
            }
            _ => {
                self.aas_client.update_numeric(
                    &mapping.aas_submodel,
                    &mapping.aas_property,
                    value,
                ).await?;
            }
        }

        Ok(())
    }
}

/// 워드 배열을 값으로 변환
fn convert_value(words: &[u16], data_type: &DataType, scale: f64, offset: f64) -> f64 {
    let raw = match data_type {
        DataType::Int16 => words.first().copied().unwrap_or(0) as i16 as f64,
        DataType::Uint16 => words.first().copied().unwrap_or(0) as f64,
        DataType::Int32 => {
            let low = words.first().copied().unwrap_or(0) as u32;
            let high = words.get(1).copied().unwrap_or(0) as u32;
            ((high << 16) | low) as i32 as f64
        }
        DataType::Uint32 => {
            let low = words.first().copied().unwrap_or(0) as u32;
            let high = words.get(1).copied().unwrap_or(0) as u32;
            ((high << 16) | low) as f64
        }
        DataType::Float32 => {
            let low = words.first().copied().unwrap_or(0) as u32;
            let high = words.get(1).copied().unwrap_or(0) as u32;
            let bits = (high << 16) | low;
            f32::from_bits(bits) as f64
        }
        DataType::Bool => {
            if words.first().copied().unwrap_or(0) != 0 { 1.0 } else { 0.0 }
        }
    };
    
    raw * scale + offset
}
