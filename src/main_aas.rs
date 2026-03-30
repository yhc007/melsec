//! PLC-AAS Bridge
//! 
//! PLC 데이터를 읽어서 AAS 서버에 업데이트하는 브릿지 서비스
//! 
//! 사용법:
//!   melsec-aas                    # config.toml 사용
//!   melsec-aas /path/to/config.toml
//!   CONFIG_PATH=/etc/melsec.toml melsec-aas

use melsec::{AasBridge, Config};
use std::env;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    println!("╔═══════════════════════════════════════╗");
    println!("║     MELSEC PLC → AAS Bridge v1.0     ║");
    println!("╚═══════════════════════════════════════╝");

    // 설정 파일 경로
    let config_path = env::args().nth(1)
        .or_else(|| env::var("CONFIG_PATH").ok())
        .unwrap_or_else(|| "config.toml".to_string());

    // 설정 로드
    println!("[INFO] Loading config: {}", config_path);
    let config = Config::load(&config_path)?;
    
    println!("[INFO] PLC: {}:{}", config.plc.ip, config.plc.port);
    println!("[INFO] AAS Server: {}", config.aas.server);
    println!("[INFO] Poll Interval: {}ms", config.plc.interval_ms);
    println!("[INFO] Mappings: {} devices", config.mappings.len());

    for (i, m) in config.mappings.iter().enumerate() {
        println!("  [{}] {} → {}/{}", i + 1, m.device, m.aas_submodel, m.aas_property);
    }

    // 브릿지 생성 및 연결
    let mut bridge = AasBridge::new(config);
    
    println!("[INFO] Connecting to PLC...");
    bridge.connect().await?;

    // 폴링 시작
    println!("[INFO] Starting polling loop... (Ctrl+C to stop)");
    bridge.start_polling().await?;

    Ok(())
}
