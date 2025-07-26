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

use crate::domain::{HardwareReport, PublishConfig, PublishError};
use async_trait::async_trait;
use std::path::Path;

/// Secondary port - Data publishing abstraction
///
/// This interface abstracts how hardware reports are published/stored,
/// allowing for different implementations (HTTP, file system, databases, etc.)
#[async_trait]
pub trait DataPublisher: Send + Sync {
    /// Publish a hardware report to a remote endpoint
    ///
    /// # Arguments
    /// * `report` - The hardware report to publish
    /// * `config` - Publishing configuration
    ///
    /// # Returns
    /// * `Ok(())` - Report successfully published
    /// * `Err(PublishError)` - Error occurred during publishing
    async fn publish(
        &self,
        report: &HardwareReport,
        config: &PublishConfig,
    ) -> Result<(), PublishError>;

    /// Test connectivity to the publishing endpoint
    ///
    /// # Arguments
    /// * `config` - Publishing configuration
    ///
    /// # Returns
    /// * `Ok(bool)` - true if endpoint is reachable
    /// * `Err(PublishError)` - Error testing connectivity
    async fn test_connectivity(&self, config: &PublishConfig) -> Result<bool, PublishError>;
}

/// Secondary port - File repository abstraction
///
/// This interface abstracts file-based storage of hardware reports
#[async_trait]
pub trait FileRepository: Send + Sync {
    /// Save hardware report to a file in JSON format
    ///
    /// # Arguments
    /// * `report` - The hardware report to save
    /// * `path` - File path to save to
    ///
    /// # Returns
    /// * `Ok(())` - Report successfully saved
    /// * `Err(PublishError)` - Error occurred during save
    async fn save_json(&self, report: &HardwareReport, path: &Path) -> Result<(), PublishError>;

    /// Save hardware report to a file in TOML format
    ///
    /// # Arguments
    /// * `report` - The hardware report to save
    /// * `path` - File path to save to
    ///
    /// # Returns
    /// * `Ok(())` - Report successfully saved
    /// * `Err(PublishError)` - Error occurred during save
    async fn save_toml(&self, report: &HardwareReport, path: &Path) -> Result<(), PublishError>;

    /// Load hardware report from a JSON file
    ///
    /// # Arguments
    /// * `path` - File path to load from
    ///
    /// # Returns
    /// * `Ok(HardwareReport)` - Loaded hardware report
    /// * `Err(PublishError)` - Error occurred during load
    async fn load_json(&self, path: &Path) -> Result<HardwareReport, PublishError>;

    /// Load hardware report from a TOML file
    ///
    /// # Arguments
    /// * `path` - File path to load from
    ///
    /// # Returns
    /// * `Ok(HardwareReport)` - Loaded hardware report
    /// * `Err(PublishError)` - Error occurred during load
    async fn load_toml(&self, path: &Path) -> Result<HardwareReport, PublishError>;

    /// Check if file exists
    ///
    /// # Arguments
    /// * `path` - File path to check
    ///
    /// # Returns
    /// * `Ok(bool)` - true if file exists
    /// * `Err(PublishError)` - Error checking file existence
    async fn file_exists(&self, path: &Path) -> Result<bool, PublishError>;
}
