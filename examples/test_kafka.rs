// Kafka 연결 테스트 프로그램
use melsec::{Device, MelsecClient, WordDevice, KafkaProducer, PlcReadResult, AddressData};
use chrono::Utc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=================================");
    println!("  Kafka 연결 테스트");
    println!("=================================\n");
    
    // Kafka 설정
    let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let topic = std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "melsec-plc-data".to_string());
    
    println!("Kafka 설정:");
    println!("  Brokers: {}", brokers);
    println!("  Topic: {}", topic);
    println!();
    
    // Kafka Producer 생성
    println!("Kafka Producer 생성 중...");
    match KafkaProducer::new(&brokers, &topic) {
        Ok(producer) => {
            println!("✓ Kafka Producer 생성 성공!\n");
            
            // 연결 테스트
            println!("Kafka 연결 테스트 중...");
            match producer.test_connection().await {
                Ok(_) => println!("✓ Kafka 연결 테스트 성공!\n"),
                Err(e) => {
                    eprintln!("✗ Kafka 연결 테스트 실패: {}", e);
                    return Err(e);
                }
            }
            
            // 토픽 확인
            println!("Kafka 토픽 확인 중...");
            match producer.check_topic().await {
                Ok(exists) => {
                    if exists {
                        println!("✓ 토픽 '{}' 확인됨\n", topic);
                    } else {
                        eprintln!("✗ 토픽 '{}'이 존재하지 않습니다", topic);
                        eprintln!("토픽 생성 명령:");
                        eprintln!("  /usr/local/kafka/bin/kafka-topics.sh --create \\");
                        eprintln!("    --bootstrap-server localhost:9092 \\");
                        eprintln!("    --topic {} \\", topic);
                        eprintln!("    --partitions 1 \\");
                        eprintln!("    --replication-factor 1");
                        return Ok(());
                    }
                }
                Err(e) => {
                    eprintln!("✗ 토픽 확인 실패: {}", e);
                }
            }
            
            // 테스트 데이터 전송
            println!("테스트 데이터 전송 중...");
            let test_data = PlcReadResult {
                timestamp: Utc::now(),
                word_data: vec![
                    AddressData {
                        address: "D1000".to_string(),
                        value: 123,
                        hex: format!("0x{:04X}", 123),
                    },
                    AddressData {
                        address: "D1001".to_string(),
                        value: 456,
                        hex: format!("0x{:04X}", 456),
                    },
                ],
                bit_data: vec![],
            };
            
            match producer.send_plc_data(&test_data).await {
                Ok(_) => {
                    println!("✓ 테스트 데이터 전송 성공!");
                    println!("\n전송된 데이터:");
                    println!("  Timestamp: {}", test_data.timestamp);
                    for addr_data in &test_data.word_data {
                        println!("  {} = {} ({})", addr_data.address, addr_data.value, addr_data.hex);
                    }
                }
                Err(e) => {
                    eprintln!("✗ 데이터 전송 실패: {}", e);
                    return Err(e);
                }
            }
            
            println!("\n=== Kafka 연결 테스트 완료 ===");
            println!("✓ 모든 테스트 성공!");
            
            // 실제 PLC 연결 및 데이터 전송 테스트
            println!("\n=================================");
            println!("  PLC + Kafka 통합 테스트");
            println!("=================================\n");
            
            let plc_ip = std::env::var("PLC_IP").unwrap_or_else(|_| "192.168.21.112".to_string());
            let plc_port: u16 = std::env::var("PLC_PORT")
                .unwrap_or_else(|_| "5010".to_string())
                .parse()
                .unwrap_or(5010);
            
            println!("PLC 연결 시도: {}:{}", plc_ip, plc_port);
            
            match MelsecClient::connect_str(&plc_ip, plc_port, 0, 0xFF).await {
                Ok(mut client) => {
                    client.set_timeout(Duration::from_secs(3));
                    println!("✓ PLC 연결 성공!\n");
                    
                    // D1000부터 10개 읽기
                    println!("D1000~D1009 데이터 읽기 중...");
                    match client.read_words(Device::Word(WordDevice::D), 1000, 10).await {
                        Ok(data) => {
                            println!("✓ 데이터 읽기 성공!\n");
                            
                            // Kafka로 전송
                            let mut word_data = Vec::new();
                            for (i, &value) in data.iter().enumerate() {
                                let addr = 1000 + i as u16;
                                word_data.push(AddressData {
                                    address: format!("D{}", addr),
                                    value,
                                    hex: format!("0x{:04X}", value),
                                });
                            }
                            
                            let plc_data = PlcReadResult {
                                timestamp: Utc::now(),
                                word_data,
                                bit_data: vec![],
                            };
                            
                            println!("Kafka로 데이터 전송 중...");
                            match producer.send_plc_data(&plc_data).await {
                                Ok(_) => {
                                    println!("✓ Kafka 전송 성공!");
                                    println!("\n전송된 데이터:");
                                    for addr_data in &plc_data.word_data {
                                        println!("  {} = {} ({})", 
                                            addr_data.address, 
                                            addr_data.value,
                                            addr_data.hex
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("✗ Kafka 전송 실패: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("✗ PLC 데이터 읽기 실패: {}", e);
                        }
                    }
                    
                    client.disconnect().await?;
                }
                Err(e) => {
                    eprintln!("✗ PLC 연결 실패: {}", e);
                    println!("\nPLC 없이 Kafka 테스트만 완료되었습니다.");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Kafka Producer 생성 실패: {}", e);
            eprintln!("\n가능한 원인:");
            eprintln!("  - Kafka 서버가 실행 중이 아님");
            eprintln!("  - Broker 주소가 잘못됨: {}", brokers);
            eprintln!("  - 네트워크 연결 문제");
            eprintln!("\nKafka 상태 확인:");
            eprintln!("  ps aux | grep kafka");
            eprintln!("  netstat -tlnp | grep 9092");
            return Err(e);
        }
    }
    
    Ok(())
}
