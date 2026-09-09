pub mod client;
pub mod device;
pub mod error;
pub mod protocol;
pub mod kafka_producer;
pub mod kafka_types;
pub mod config;
pub mod aas_client;
pub mod aas_bridge;

pub use client::MelsecClient;
pub use device::{Device, BitDevice, WordDevice};
pub use error::{MelsecError, Result};
pub use kafka_producer::KafkaProducer;
pub use kafka_types::{PlcReadResult, AddressData, BitAddressData};
pub use config::Config;
pub use aas_client::AasClient;
pub use aas_bridge::AasBridge;
