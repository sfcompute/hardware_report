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

use crate::domain::{DomainError, PublishConfig, ReportConfig};
use async_trait::async_trait;
use std::collections::HashMap;

/// Secondary port - Configuration provider abstraction
///
/// This interface abstracts how configuration is loaded and managed,
/// allowing for different sources (CLI args, files, environment, etc.)
#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    /// Get report generation configuration
    ///
    /// # Returns
    /// * `Ok(ReportConfig)` - Report configuration
    /// * `Err(DomainError)` - Error loading configuration
    async fn get_report_config(&self) -> Result<ReportConfig, DomainError>;

    /// Get publishing configuration if enabled
    ///
    /// # Returns
    /// * `Ok(Option<PublishConfig>)` - Publishing config if enabled
    /// * `Err(DomainError)` - Error loading configuration
    async fn get_publish_config(&self) -> Result<Option<PublishConfig>, DomainError>;

    /// Get output format preference
    ///
    /// # Returns
    /// * `Ok(OutputFormat)` - Preferred output format
    /// * `Err(DomainError)` - Error loading configuration
    async fn get_output_format(&self) -> Result<OutputFormat, DomainError>;

    /// Get command timeout in seconds
    ///
    /// # Returns
    /// * `Ok(u64)` - Timeout in seconds
    /// * `Err(DomainError)` - Error loading configuration
    async fn get_command_timeout(&self) -> Result<u64, DomainError>;

    /// Check if verbose logging is enabled
    ///
    /// # Returns
    /// * `Ok(bool)` - true if verbose logging enabled
    /// * `Err(DomainError)` - Error loading configuration
    async fn is_verbose_enabled(&self) -> Result<bool, DomainError>;

    /// Get additional labels/metadata
    ///
    /// # Returns
    /// * `Ok(HashMap<String, String>)` - Additional labels
    /// * `Err(DomainError)` - Error loading configuration
    async fn get_labels(&self) -> Result<HashMap<String, String>, DomainError>;
}

/// Output format options
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// Both JSON and TOML
    Both,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Both
    }
}
