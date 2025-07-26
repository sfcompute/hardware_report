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

//! File-based data publisher for saving reports to local files

use crate::domain::{HardwareReport, PublishError};
use crate::ports::FileRepository;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

/// File system repository for storing hardware reports
pub struct FileSystemRepository;

impl FileSystemRepository {
    /// Create a new file system repository
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileSystemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileRepository for FileSystemRepository {
    async fn save_json(&self, report: &HardwareReport, path: &Path) -> Result<(), PublishError> {
        let json_string = serde_json::to_string_pretty(report)
            .map_err(|e| PublishError::SerializationFailed(format!("JSON serialization failed: {}", e)))?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| PublishError::NetworkFailed(format!("Failed to create directory: {}", e)))?;
        }
        
        fs::write(path, json_string)
            .await
            .map_err(|e| PublishError::NetworkFailed(format!("Failed to write JSON file: {}", e)))?;
        
        Ok(())
    }
    
    async fn save_toml(&self, report: &HardwareReport, path: &Path) -> Result<(), PublishError> {
        let toml_string = toml::to_string_pretty(report)
            .map_err(|e| PublishError::SerializationFailed(format!("TOML serialization failed: {}", e)))?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| PublishError::NetworkFailed(format!("Failed to create directory: {}", e)))?;
        }
        
        fs::write(path, toml_string)
            .await
            .map_err(|e| PublishError::NetworkFailed(format!("Failed to write TOML file: {}", e)))?;
        
        Ok(())
    }
    
    async fn load_json(&self, path: &Path) -> Result<HardwareReport, PublishError> {
        let json_string = fs::read_to_string(path)
            .await
            .map_err(|e| PublishError::NetworkFailed(format!("Failed to read JSON file: {}", e)))?;
        
        serde_json::from_str(&json_string)
            .map_err(|e| PublishError::SerializationFailed(format!("JSON deserialization failed: {}", e)))
    }
    
    async fn load_toml(&self, path: &Path) -> Result<HardwareReport, PublishError> {
        let toml_string = fs::read_to_string(path)
            .await
            .map_err(|e| PublishError::NetworkFailed(format!("Failed to read TOML file: {}", e)))?;
        
        toml::from_str(&toml_string)
            .map_err(|e| PublishError::SerializationFailed(format!("TOML deserialization failed: {}", e)))
    }
    
    async fn file_exists(&self, path: &Path) -> Result<bool, PublishError> {
        Ok(path.exists())
    }
}

/// Composite data publisher that saves to both JSON and TOML files
pub struct FileDataPublisher {
    repository: FileSystemRepository,
}

impl FileDataPublisher {
    /// Create a new file data publisher
    pub fn new() -> Self {
        Self {
            repository: FileSystemRepository::new(),
        }
    }
    
    /// Save hardware report to both JSON and TOML files
    /// 
    /// # Arguments
    /// * `report` - The hardware report to save
    /// * `base_path` - Base path without extension (e.g., "/path/to/report")
    /// 
    /// # Returns
    /// * `Ok((json_path, toml_path))` - Paths to the saved files
    /// * `Err(PublishError)` - Error occurred during save
    pub async fn save_both_formats(&self, report: &HardwareReport, base_path: &str) -> Result<(String, String), PublishError> {
        let json_path = format!("{}.json", base_path);
        let toml_path = format!("{}.toml", base_path);
        
        // Save both formats
        let json_result = self.repository.save_json(report, Path::new(&json_path));
        let toml_result = self.repository.save_toml(report, Path::new(&toml_path));
        
        // Wait for both operations to complete
        let (json_res, toml_res) = tokio::join!(json_result, toml_result);
        
        json_res?;
        toml_res?;
        
        Ok((json_path, toml_path))
    }
}

impl Default for FileDataPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{HardwareReport, SystemSummary, SystemInfo, HardwareInfo, NetworkInfo};
    use std::collections::HashMap;
    use tempfile::tempdir;

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
                cpu_summary: "Test CPU (1 Socket, 8 Cores/Socket, 2 Threads/Core, 1 NUMA Node)".to_string(),
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
                storage: crate::domain::StorageInfo {
                    devices: vec![],
                },
                gpus: crate::domain::GpuInfo {
                    devices: vec![],
                },
            },
            network: NetworkInfo {
                interfaces: vec![],
                infiniband: None,
            },
        }
    }

    #[tokio::test]
    async fn test_save_load_json() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_report.json");
        
        let repository = FileSystemRepository::new();
        let original_report = create_test_report();
        
        // Save the report
        repository.save_json(&original_report, &file_path).await.unwrap();
        
        // Verify file exists
        assert!(repository.file_exists(&file_path).await.unwrap());
        
        // Load the report back
        let loaded_report = repository.load_json(&file_path).await.unwrap();
        
        // Verify key fields match
        assert_eq!(original_report.hostname, loaded_report.hostname);
        assert_eq!(original_report.summary.system_info.uuid, loaded_report.summary.system_info.uuid);
    }

    #[tokio::test]
    async fn test_save_load_toml() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_report.toml");
        
        let repository = FileSystemRepository::new();
        let original_report = create_test_report();
        
        // Save the report
        repository.save_toml(&original_report, &file_path).await.unwrap();
        
        // Verify file exists
        assert!(repository.file_exists(&file_path).await.unwrap());
        
        // Load the report back
        let loaded_report = repository.load_toml(&file_path).await.unwrap();
        
        // Verify key fields match
        assert_eq!(original_report.hostname, loaded_report.hostname);
        assert_eq!(original_report.summary.system_info.uuid, loaded_report.summary.system_info.uuid);
    }

    #[tokio::test]
    async fn test_save_both_formats() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path().join("test_report").to_string_lossy().to_string();
        
        let publisher = FileDataPublisher::new();
        let report = create_test_report();
        
        // Save both formats
        let (json_path, toml_path) = publisher.save_both_formats(&report, &base_path).await.unwrap();
        
        // Verify both files exist
        assert!(Path::new(&json_path).exists());
        assert!(Path::new(&toml_path).exists());
        
        // Verify we can load from both
        let json_report = publisher.repository.load_json(Path::new(&json_path)).await.unwrap();
        let toml_report = publisher.repository.load_toml(Path::new(&toml_path)).await.unwrap();
        
        assert_eq!(json_report.hostname, report.hostname);
        assert_eq!(toml_report.hostname, report.hostname);
    }

    #[tokio::test]
    async fn test_create_directory() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("directory").join("report.json");
        
        let repository = FileSystemRepository::new();
        let report = create_test_report();
        
        // This should create the nested directory structure
        repository.save_json(&report, &nested_path).await.unwrap();
        
        // Verify file was created
        assert!(nested_path.exists());
    }
}