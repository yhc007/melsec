// D1000부터 데이터 읽기 테스트 프로그램
use melsec::{Device, MelsecClient, WordDevice};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!("  PLC D1000 주소 읽기 테스트");
    println!("=================================\n");
    
    // PLC 연결 설정 (환경에 맞게 수정 가능)
    let ip = std::env::var("PLC_IP").unwrap_or_else(|_| "192.168.21.112".to_string());
    let port: u16 = std::env::var("PLC_PORT")
        .unwrap_or_else(|_| "5010".to_string())
        .parse()
        .unwrap_or(5010);
    
    println!("PLC 연결 시도 중...");
    println!("  IP: {}", ip);
    println!("  Port: {}", port);
    println!();
    
    // PLC 연결
    match MelsecClient::connect_str(&ip, port, 0, 0xFF).await {
        Ok(mut client) => {
            client.set_timeout(Duration::from_secs(3));
            println!("✓ PLC 연결 성공!\n");
            
            // D1000부터 10개의 워드 읽기
            let start_address = 1000u16;
            let count = 10u16;
            
            println!("D{}부터 {}개의 워드 읽기 중...", start_address, count);
            println!("─────────────────────────────────");
            
            match client.read_words(Device::Word(WordDevice::D), start_address, count).await {
                Ok(data) => {
                    println!("✓ 데이터 읽기 성공!\n");
                    println!("주소       | 10진수 값  | 16진수 값  | 2진수 값");
                    println!("───────────┼───────────┼───────────┼──────────────────");
                    
                    for (i, value) in data.iter().enumerate() {
                        let addr = start_address + i as u16;
                        println!(
                            "D{:<8} | {:<10} | 0x{:04X}     | {:016b}",
                            addr, value, value, value
                        );
                    }
                    
                    println!("\n=== 요약 ===");
                    println!("총 {}개의 워드를 성공적으로 읽었습니다.", data.len());
                    println!("주소 범위: D{} ~ D{}", start_address, start_address + count - 1);
                }
                Err(e) => {
                    eprintln!("✗ 데이터 읽기 실패: {}", e);
                    eprintln!("\n가능한 원인:");
                    eprintln!("  - PLC가 실행 중이 아님");
                    eprintln!("  - 네트워크 연결 문제");
                    eprintln!("  - 주소 범위가 PLC 설정을 벗어남");
                }
            }
            
            // 연결 종료
            client.disconnect().await?;
            println!("\n✓ PLC 연결 종료");
        }
        Err(e) => {
            eprintln!("✗ PLC 연결 실패: {}", e);
            eprintln!("\n연결 정보를 확인하세요:");
            eprintln!("  export PLC_IP=<PLC_IP_주소>");
            eprintln!("  export PLC_PORT=<PLC_포트>");
            return Err(e.into());
        }
    }
    
    Ok(())
}
