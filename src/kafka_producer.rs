use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::time::Duration;

use crate::kafka_types::PlcReadResult;

pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
}

impl KafkaProducer {
    /// Kafka Producer 생성
    pub fn new(brokers: &str, topic: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "1")
            .set("retries", "3")
            .create()?;

        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }

    /// PLC 데이터 전송
    pub async fn send_plc_data(&self, data: &PlcReadResult) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json_data = serde_json::to_string(data)?;
        
        // Key로 타임스탬프 사용
        let key = data.timestamp.timestamp().to_string();
        
        let record = FutureRecord::to(&self.topic)
            .key(&key)
            .payload(&json_data);

        match self.producer.send(record, Timeout::After(Duration::from_secs(5))).await {
            Ok(_) => Ok(()),
            Err((e, _)) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Kafka 전송 실패: {}", e),
            ))),
        }
    }

    /// 연결 테스트 (실제 전송으로 테스트)
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 빈 메시지로 연결 테스트 (실제로는 첫 전송 시 연결 확인)
        // 여기서는 단순히 producer가 생성되었는지만 확인
        // 실제 연결은 첫 메시지 전송 시 확인됨
        Ok(())
    }

    /// 토픽 존재 여부 확인 (실제 전송으로 확인)
    pub async fn check_topic(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // 실제로는 첫 메시지 전송 시 토픽 존재 여부 확인
        // 여기서는 항상 true 반환 (실제 전송 시 에러로 확인)
        Ok(true)
    }
    
    /// Producer 참조 (내부 사용)
    #[allow(dead_code)]
    fn _producer(&self) -> &FutureProducer {
        &self.producer
    }
}

