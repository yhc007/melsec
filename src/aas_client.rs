use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// AAS REST API 클라이언트
pub struct AasClient {
    client: Client,
    base_url: String,
}

/// SubmodelElement 값 업데이트 요청
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyValue {
    pub value: serde_json::Value,
    pub value_type: String,
}

/// AAS 응답
#[derive(Debug, Deserialize)]
pub struct AasResponse {
    pub success: Option<bool>,
    pub error: Option<String>,
}

impl AasClient {
    /// 새 AAS 클라이언트 생성
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// SubmodelElement 값 업데이트 (BaSyx API)
    /// PUT /submodels/{submodelId}/submodel-elements/{idShortPath}/
    pub async fn update_property(
        &self,
        submodel_id: &str,
        property_path: &str,
        value: serde_json::Value,
        value_type: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Base64 encode submodel ID for URL
        let encoded_submodel_id = base64_url_encode(submodel_id);
        
        let url = format!(
            "{}/submodels/{}/submodel-elements/{}/",
            self.base_url,
            encoded_submodel_id,
            property_path
        );

        let payload = PropertyValue {
            value,
            value_type: value_type.to_string(),
        };

        let response = self.client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(format!("AAS update failed: {} - {}", status, body).into())
        }
    }

    /// 간단한 값 업데이트 (숫자)
    pub async fn update_numeric(
        &self,
        submodel_id: &str,
        property_path: &str,
        value: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.update_property(
            submodel_id,
            property_path,
            serde_json::json!(value),
            "xs:double",
        ).await
    }

    /// 정수 값 업데이트
    pub async fn update_integer(
        &self,
        submodel_id: &str,
        property_path: &str,
        value: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.update_property(
            submodel_id,
            property_path,
            serde_json::json!(value),
            "xs:integer",
        ).await
    }

    /// Boolean 값 업데이트
    pub async fn update_bool(
        &self,
        submodel_id: &str,
        property_path: &str,
        value: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.update_property(
            submodel_id,
            property_path,
            serde_json::json!(value),
            "xs:boolean",
        ).await
    }
}

/// Base64 URL-safe 인코딩
fn base64_url_encode(input: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.encode(input.as_bytes())
}
