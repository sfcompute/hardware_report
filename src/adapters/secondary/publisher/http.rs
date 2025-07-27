/*
Copyright 2024 San Francisco Compute Company

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! HTTP data publisher for sending reports to remote endpoints

use crate::domain::{HardwareReport, PublishConfig, PublishError};
use crate::ports::DataPublisher;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

/// HTTP data publisher that sends reports to remote endpoints
pub struct HttpDataPublisher {
    client: Client,
    #[allow(dead_code)]
    timeout: Duration,
}

impl HttpDataPublisher {
    /// Create a new HTTP data publisher
    ///
    /// # Arguments
    /// * `timeout` - HTTP request timeout
    /// * `skip_tls_verify` - Whether to skip TLS certificate verification
    pub fn new(timeout: Duration, skip_tls_verify: bool) -> Result<Self, PublishError> {
        let client = Client::builder()
            .timeout(timeout)
            .danger_accept_invalid_certs(skip_tls_verify)
            .build()
            .map_err(|e| {
                PublishError::NetworkFailed(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self { client, timeout })
    }

    /// Create with default settings
    pub fn with_defaults() -> Result<Self, PublishError> {
        Self::new(Duration::from_secs(30), false)
    }

    /// Create a payload with labels merged in
    fn create_payload(&self, report: &HardwareReport, config: &PublishConfig) -> serde_json::Value {
        let mut payload = serde_json::to_value(report).unwrap_or_else(|_| json!({}));

        // Add labels if provided
        if !config.labels.is_empty() {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert(
                    "labels".to_string(),
                    serde_json::to_value(&config.labels).unwrap_or(json!({})),
                );
            }
        }

        payload
    }
}

#[async_trait]
impl DataPublisher for HttpDataPublisher {
    async fn publish(
        &self,
        report: &HardwareReport,
        config: &PublishConfig,
    ) -> Result<(), PublishError> {
        if config.endpoint.is_empty() {
            return Err(PublishError::NetworkFailed(
                "No endpoint URL provided".to_string(),
            ));
        }

        let payload = self.create_payload(report, config);

        let mut request = self.client.post(&config.endpoint).json(&payload);

        // Add authentication if provided
        if let Some(ref token) = config.auth_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        // Add content type
        request = request.header("Content-Type", "application/json");

        // Send the request
        let response = request
            .send()
            .await
            .map_err(|e| PublishError::NetworkFailed(format!("Failed to send request: {e}")))?;

        // Check response status
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if status.as_u16() == 401 || status.as_u16() == 403 {
                Err(PublishError::AuthenticationFailed(format!(
                    "HTTP {status}: {error_text}"
                )))
            } else {
                Err(PublishError::NetworkFailed(format!(
                    "HTTP {status}: {error_text}"
                )))
            }
        }
    }

    async fn test_connectivity(&self, config: &PublishConfig) -> Result<bool, PublishError> {
        if config.endpoint.is_empty() {
            return Ok(false);
        }

        // Try a simple HEAD request to test connectivity
        let mut request = self.client.head(&config.endpoint);

        // Add authentication if provided
        if let Some(ref token) = config.auth_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        match request.send().await {
            Ok(response) => Ok(response.status().is_success() || response.status().as_u16() == 405), // 405 = Method Not Allowed is OK for HEAD
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{HardwareInfo, HardwareReport, NetworkInfo, SystemInfo, SystemSummary};
    use std::collections::HashMap;

    fn create_test_report() -> HardwareReport {
        HardwareReport {
            summary: SystemSummary {
                system_info: SystemInfo {
                    uuid: "test-uuid".to_string(),
                    serial: "test-serial".to_string(),
                    product_name: "Test System".to_string(),
                    product_manufacturer: "Test Corp".to_string(),
                },
                total_memory: "16GB".to_string(),
                memory_config: "DDR4 @ 3200MHz".to_string(),
                total_storage: "1TB".to_string(),
                total_storage_tb: 1.0,
                filesystems: vec![],
                bios: crate::domain::BiosInfo {
                    vendor: "Test BIOS".to_string(),
                    version: "1.0".to_string(),
                    release_date: "2024-01-01".to_string(),
                    firmware_version: "1.0".to_string(),
                },
                chassis: crate::domain::ChassisInfo {
                    manufacturer: "Test Corp".to_string(),
                    type_: "Desktop".to_string(),
                    serial: "test-chassis".to_string(),
                },
                motherboard: crate::domain::MotherboardInfo {
                    manufacturer: "Test Corp".to_string(),
                    product_name: "Test Board".to_string(),
                    version: "1.0".to_string(),
                    serial: "test-mb".to_string(),
                    features: "None".to_string(),
                    location: "System".to_string(),
                    type_: "Motherboard".to_string(),
                },
                total_gpus: 1,
                total_nics: 1,
                numa_topology: HashMap::new(),
                cpu_topology: crate::domain::CpuTopology {
                    total_cores: 8,
                    total_threads: 16,
                    sockets: 1,
                    cores_per_socket: 8,
                    threads_per_core: 2,
                    numa_nodes: 1,
                    cpu_model: "Test CPU".to_string(),
                },
                cpu_summary: "Test CPU (1 Socket, 8 Cores/Socket, 2 Threads/Core, 1 NUMA Node)"
                    .to_string(),
            },
            hostname: "test-host".to_string(),
            fqdn: "test-host.example.com".to_string(),
            os_ip: vec![],
            bmc_ip: None,
            bmc_mac: None,
            hardware: HardwareInfo {
                cpu: crate::domain::CpuInfo {
                    model: "Test CPU".to_string(),
                    cores: 8,
                    threads: 2,
                    sockets: 1,
                    speed: "3.0 GHz".to_string(),
                },
                memory: crate::domain::MemoryInfo {
                    total: "16GB".to_string(),
                    type_: "DDR4".to_string(),
                    speed: "3200 MHz".to_string(),
                    modules: vec![],
                },
                storage: crate::domain::StorageInfo { devices: vec![] },
                gpus: crate::domain::GpuInfo { devices: vec![] },
            },
            network: NetworkInfo {
                interfaces: vec![],
                infiniband: None,
            },
        }
    }

    #[tokio::test]
    async fn test_http_publisher_creation() {
        let publisher = HttpDataPublisher::with_defaults();
        assert!(publisher.is_ok());
    }

    #[tokio::test]
    async fn test_create_payload_with_labels() {
        let publisher = HttpDataPublisher::with_defaults().unwrap();
        let report = create_test_report();

        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "test".to_string());
        labels.insert("datacenter".to_string(), "dc1".to_string());

        let config = PublishConfig {
            endpoint: "http://example.com".to_string(),
            auth_token: None,
            skip_tls_verify: false,
            labels,
        };

        let payload = publisher.create_payload(&report, &config);
        assert!(payload.get("labels").is_some());
        assert_eq!(payload["labels"]["environment"], "test");
        assert_eq!(payload["labels"]["datacenter"], "dc1");
    }

    #[tokio::test]
    async fn test_empty_endpoint_error() {
        let publisher = HttpDataPublisher::with_defaults().unwrap();
        let report = create_test_report();
        let config = PublishConfig {
            endpoint: String::new(),
            auth_token: None,
            skip_tls_verify: false,
            labels: HashMap::new(),
        };

        let result = publisher.publish(&report, &config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PublishError::NetworkFailed(_)
        ));
    }
}
