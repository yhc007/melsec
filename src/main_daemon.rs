// MELSEC PLC 헤드리스 데몬
// PLC 데이터를 주기적으로 읽어 Kafka로 전송하는 백그라운드 서비스

use futures::stream::StreamExt;
use log::{debug, error, info, warn};
use melsec::{AddressData, Device, KafkaProducer, MelsecClient, PlcReadResult, WordDevice};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// 데몬 설정
#[derive(Debug, Clone)]
struct DaemonConfig {
    plc_ip: String,
    plc_port: u16,
    kafka_brokers: String,
    kafka_topic: String,
    read_interval_ms: u64,
    start_address: u16,
    count: u16,
}

impl DaemonConfig {
    /// 환경 변수에서 설정 로드
    fn from_env() -> Self {
        Self {
            plc_ip: std::env::var("PLC_IP").unwrap_or_else(|_| "192.168.21.112".to_string()),
            plc_port: std::env::var("PLC_PORT")
                .unwrap_or_else(|_| "5010".to_string())
                .parse()
                .unwrap_or(5010),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            kafka_topic: std::env::var("KAFKA_TOPIC")
                .unwrap_or_else(|_| "melsec-plc-data".to_string()),
            read_interval_ms: std::env::var("READ_INTERVAL_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            start_address: std::env::var("START_ADDRESS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            count: std::env::var("READ_COUNT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

/// PLC 데몬
struct PlcDaemon {
    config: DaemonConfig,
    client: Option<MelsecClient>,
    kafka_producer: Option<KafkaProducer>,
    running: Arc<AtomicBool>,
}

impl PlcDaemon {
    fn new(config: DaemonConfig, running: Arc<AtomicBool>) -> Self {
        Self {
            config,
            client: None,
            kafka_producer: None,
            running,
        }
    }

    /// PLC 연결
    async fn connect_plc(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("PLC 연결 시도: {}:{}", self.config.plc_ip, self.config.plc_port);
        
        match MelsecClient::connect_str(&self.config.plc_ip, self.config.plc_port, 0, 0xFF).await {
            Ok(mut client) => {
                client.set_timeout(Duration::from_secs(5));
                self.client = Some(client);
                info!("✓ PLC 연결 성공!");
                Ok(())
            }
            Err(e) => {
                error!("✗ PLC 연결 실패: {}", e);
                self.client = None;
                Err(e.into())
            }
        }
    }

    /// Kafka Producer 초기화
    async fn init_kafka(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Kafka Producer 초기화: {} -> {}",
            self.config.kafka_brokers, self.config.kafka_topic
        );

        match KafkaProducer::new(&self.config.kafka_brokers, &self.config.kafka_topic) {
            Ok(producer) => {
                self.kafka_producer = Some(producer);
                info!("✓ Kafka Producer 생성 성공!");
                Ok(())
            }
            Err(e) => {
                error!("✗ Kafka Producer 생성 실패: {}", e);
                self.kafka_producer = None;
                Err(e)
            }
        }
    }

    /// PLC 데이터 읽기
    async fn read_plc_data(&mut self) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
        if let Some(client) = &mut self.client {
            debug!(
                "D{}부터 {}개 읽기 중...",
                self.config.start_address, self.config.count
            );

            match client
                .read_words(
                    Device::Word(WordDevice::D),
                    self.config.start_address,
                    self.config.count,
                )
                .await
            {
                Ok(data) => {
                    debug!("✓ 데이터 읽기 성공: {} words", data.len());
                    Ok(data)
                }
                Err(e) => {
                    warn!("✗ 데이터 읽기 실패: {}", e);
                    // 연결이 끊어진 것으로 간주
                    self.client = None;
                    Err(e.into())
                }
            }
        } else {
            Err("PLC 연결 없음".into())
        }
    }

    /// Kafka로 데이터 전송
    async fn send_to_kafka(&self, data: &[u16]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(producer) = &self.kafka_producer {
            // PlcReadResult 생성
            let mut word_data = Vec::new();
            for (i, &value) in data.iter().enumerate() {
                let addr = self.config.start_address + i as u16;
                word_data.push(AddressData {
                    address: format!("D{}", addr),
                    value,
                    hex: format!("0x{:04X}", value),
                });
            }

            let plc_data = PlcReadResult {
                timestamp: chrono::Utc::now(),
                word_data,
                bit_data: vec![],
            };

            debug!("Kafka로 데이터 전송 중...");
            match producer.send_plc_data(&plc_data).await {
                Ok(_) => {
                    debug!("✓ Kafka 전송 성공");
                    Ok(())
                }
                Err(e) => {
                    warn!("✗ Kafka 전송 실패: {}", e);
                    Err(e)
                }
            }
        } else {
            Err("Kafka Producer 없음".into())
        }
    }

    /// 메인 루프
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("데몬 시작");
        info!("설정: {:?}", self.config);

        // Kafka 초기화
        if let Err(e) = self.init_kafka().await {
            error!("Kafka 초기화 실패, 계속 시도합니다: {}", e);
        }

        let mut reconnect_delay = 1;
        let mut kafka_reconnect_delay = 1;
        let mut success_count = 0u64;
        let mut error_count = 0u64;

        while self.running.load(Ordering::Relaxed) {
            // PLC 연결 확인 및 재연결
            if self.client.is_none() {
                info!("PLC 재연결 시도... ({}초 후)", reconnect_delay);
                sleep(Duration::from_secs(reconnect_delay)).await;

                if let Err(e) = self.connect_plc().await {
                    error!("PLC 재연결 실패: {}", e);
                    reconnect_delay = (reconnect_delay * 2).min(60); // 최대 60초
                    continue;
                } else {
                    reconnect_delay = 1; // 성공 시 리셋
                }
            }

            // Kafka Producer 확인 및 재연결
            if self.kafka_producer.is_none() {
                warn!("Kafka Producer 재연결 시도... ({}초 후)", kafka_reconnect_delay);
                sleep(Duration::from_secs(kafka_reconnect_delay)).await;

                if let Err(e) = self.init_kafka().await {
                    error!("Kafka 재연결 실패: {}", e);
                    kafka_reconnect_delay = (kafka_reconnect_delay * 2).min(60);
                } else {
                    kafka_reconnect_delay = 1;
                }
            }

            // 데이터 읽기 및 전송
            match self.read_plc_data().await {
                Ok(data) => {
                    // Kafka로 전송
                    if let Err(e) = self.send_to_kafka(&data).await {
                        error_count += 1;
                        warn!("Kafka 전송 실패 ({}번째): {}", error_count, e);
                        // Kafka Producer 재생성 시도
                        self.kafka_producer = None;
                    } else {
                        success_count += 1;
                        if success_count % 100 == 0 {
                            info!("누적 성공: {} 회, 실패: {} 회", success_count, error_count);
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    warn!("PLC 읽기 실패 ({}번째): {}", error_count, e);
                    // PLC 재연결 필요
                    self.client = None;
                }
            }

            // 대기
            sleep(Duration::from_millis(self.config.read_interval_ms)).await;
        }

        info!("데몬 종료");
        info!(
            "최종 통계 - 성공: {} 회, 실패: {} 회",
            success_count, error_count
        );
        Ok(())
    }
}

/// 시그널 핸들러
async fn handle_signals(mut signals: Signals, running: Arc<AtomicBool>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                info!("종료 시그널 수신: {}", signal);
                running.store(false, Ordering::Relaxed);
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 로거 초기화
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("===================================");
    info!("  MELSEC PLC Daemon v0.1.0");
    info!("===================================");

    // 설정 로드
    let config = DaemonConfig::from_env();

    // 실행 플래그
    let running = Arc::new(AtomicBool::new(true));

    // 시그널 핸들러 설정
    let signals = Signals::new(&[SIGTERM, SIGINT])?;
    let signals_handle = signals.handle();
    let running_clone = running.clone();

    tokio::spawn(async move {
        handle_signals(signals, running_clone).await;
    });

    // 데몬 실행
    let mut daemon = PlcDaemon::new(config, running.clone());
    let result = daemon.run().await;

    // 정리
    signals_handle.close();

    info!("프로그램 종료");
    result
}
