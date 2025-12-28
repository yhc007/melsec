pub mod client;
pub mod protocol;
pub mod error;
pub mod device;
pub mod kafka_types;
pub mod kafka_producer;

pub use client::MelsecClient;
pub use error::{MelsecError, Result};
pub use device::{Device, BitDevice, WordDevice};
pub use kafka_producer::KafkaProducer;
pub use kafka_types::{PlcReadResult, AddressData, BitAddressData};

