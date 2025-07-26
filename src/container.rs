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

//! Dependency injection container for hardware reporting services

use crate::adapters::{
    HttpDataPublisher, LinuxSystemInfoProvider, MacOSSystemInfoProvider,
    UnixCommandExecutor,
};
use crate::domain::{HardwareCollectionService, ReportConfig};
use crate::ports::{
    CommandExecutor, ConfigurationProvider, DataPublisher, HardwareReportingService,
    SystemInfoProvider,
};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for the dependency injection container
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Command execution timeout
    pub command_timeout: Duration,
    /// Command retry count
    pub retry_count: u32,
    /// Enable verbose logging
    pub verbose: bool,
    /// HTTP timeout for publishing
    pub http_timeout: Duration,
    /// Skip TLS verification for HTTP publishing
    pub skip_tls_verify: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            retry_count: 2,
            verbose: false,
            http_timeout: Duration::from_secs(30),
            skip_tls_verify: false,
        }
    }
}

/// Simple configuration provider implementation
pub struct SimpleConfigurationProvider {
    config: ReportConfig,
}

impl SimpleConfigurationProvider {
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl ConfigurationProvider for SimpleConfigurationProvider {
    async fn get_report_config(&self) -> Result<ReportConfig, crate::domain::DomainError> {
        Ok(self.config.clone())
    }

    async fn get_publish_config(
        &self,
    ) -> Result<Option<crate::domain::PublishConfig>, crate::domain::DomainError> {
        Ok(None) // No publishing by default
    }

    async fn get_output_format(
        &self,
    ) -> Result<crate::ports::OutputFormat, crate::domain::DomainError> {
        Ok(crate::ports::OutputFormat::Both)
    }

    async fn get_command_timeout(&self) -> Result<u64, crate::domain::DomainError> {
        Ok(self.config.command_timeout)
    }

    async fn is_verbose_enabled(&self) -> Result<bool, crate::domain::DomainError> {
        Ok(self.config.verbose)
    }

    async fn get_labels(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, crate::domain::DomainError> {
        Ok(std::collections::HashMap::new())
    }
}

/// Dependency injection container
pub struct ServiceContainer {
    config: ContainerConfig,
}

impl ServiceContainer {
    /// Create a new service container with configuration
    pub fn new(config: ContainerConfig) -> Self {
        Self { config }
    }

    /// Create a service container with default configuration
    pub fn default() -> Self {
        Self::new(ContainerConfig::default())
    }

    /// Create the command executor
    pub fn create_command_executor(&self) -> Arc<dyn CommandExecutor> {
        Arc::new(UnixCommandExecutor::new(
            self.config.command_timeout,
            self.config.retry_count,
            self.config.verbose,
        ))
    }

    /// Create the platform-specific system info provider
    pub fn create_system_info_provider(
        &self,
    ) -> Result<Arc<dyn SystemInfoProvider>, Box<dyn Error>> {
        let command_executor = self.create_command_executor();

        let provider: Arc<dyn SystemInfoProvider> = if cfg!(target_os = "macos") {
            Arc::new(MacOSSystemInfoProvider::new(command_executor))
        } else if cfg!(target_os = "linux") {
            Arc::new(LinuxSystemInfoProvider::new(command_executor))
        } else {
            return Err("Unsupported operating system".into());
        };

        Ok(provider)
    }

    /// Create the data publisher
    pub fn create_data_publisher(&self) -> Result<Arc<dyn DataPublisher>, Box<dyn Error>> {
        let http_publisher =
            HttpDataPublisher::new(self.config.http_timeout, self.config.skip_tls_verify)?;

        Ok(Arc::new(http_publisher))
    }

    /// Create the configuration provider
    pub fn create_configuration_provider(
        &self,
        report_config: ReportConfig,
    ) -> Arc<dyn ConfigurationProvider> {
        Arc::new(SimpleConfigurationProvider::new(report_config))
    }

    /// Create the complete hardware reporting service
    pub fn create_hardware_reporting_service(
        &self,
        report_config: Option<ReportConfig>,
    ) -> Result<Arc<dyn HardwareReportingService>, Box<dyn Error>> {
        let system_provider = self.create_system_info_provider()?;
        let data_publisher = self.create_data_publisher()?;
        let config_provider = self.create_configuration_provider(report_config.unwrap_or_default());

        let service =
            HardwareCollectionService::new(system_provider, data_publisher, config_provider);

        Ok(Arc::new(service))
    }

    /// Get platform name for logging
    pub fn get_platform_name(&self) -> &'static str {
        if cfg!(target_os = "macos") {
            "macOS"
        } else if cfg!(target_os = "linux") {
            "Linux"
        } else {
            "Unknown"
        }
    }

    /// Validate that required system dependencies are available
    pub async fn validate_dependencies(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let system_provider = self.create_system_info_provider()?;
        let missing = system_provider
            .get_missing_dependencies()
            .await
            .map_err(|e| format!("Failed to check dependencies: {}", e))?;
        Ok(missing)
    }

    /// Check if the system has required privileges
    pub async fn check_privileges(&self) -> Result<bool, Box<dyn Error>> {
        let system_provider = self.create_system_info_provider()?;
        let has_privileges = system_provider
            .has_required_privileges()
            .await
            .map_err(|e| format!("Failed to check privileges: {}", e))?;
        Ok(has_privileges)
    }
}

/// Builder pattern for container configuration
pub struct ContainerConfigBuilder {
    config: ContainerConfig,
}

impl ContainerConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: ContainerConfig::default(),
        }
    }

    /// Set command timeout
    pub fn command_timeout(mut self, timeout: Duration) -> Self {
        self.config.command_timeout = timeout;
        self
    }

    /// Set retry count
    pub fn retry_count(mut self, count: u32) -> Self {
        self.config.retry_count = count;
        self
    }

    /// Enable verbose logging
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    /// Set HTTP timeout
    pub fn http_timeout(mut self, timeout: Duration) -> Self {
        self.config.http_timeout = timeout;
        self
    }

    /// Skip TLS verification
    pub fn skip_tls_verify(mut self, skip: bool) -> Self {
        self.config.skip_tls_verify = skip;
        self
    }

    /// Build the configuration
    pub fn build(self) -> ContainerConfig {
        self.config
    }
}

impl Default for ContainerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_creation() {
        let container = ServiceContainer::default();
        assert_eq!(
            container.get_platform_name(),
            if cfg!(target_os = "macos") {
                "macOS"
            } else {
                "Linux"
            }
        );
    }

    #[test]
    fn test_config_builder() {
        let config = ContainerConfigBuilder::new()
            .command_timeout(Duration::from_secs(60))
            .retry_count(3)
            .verbose(true)
            .build();

        assert_eq!(config.command_timeout, Duration::from_secs(60));
        assert_eq!(config.retry_count, 3);
        assert!(config.verbose);
    }

    #[test]
    fn test_command_executor_creation() {
        let container = ServiceContainer::default();
        let executor = container.create_command_executor();

        // Verify we got an executor (basic smoke test)
        assert!(Arc::strong_count(&executor) >= 1);
    }

    #[tokio::test]
    async fn test_system_info_provider_creation() {
        let container = ServiceContainer::default();
        let result = container.create_system_info_provider();

        // Should succeed on supported platforms
        if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_data_publisher_creation() {
        let container = ServiceContainer::default();
        let result = container.create_data_publisher();
        assert!(result.is_ok());
    }

    #[test]
    fn test_complete_service_creation() {
        let container = ServiceContainer::default();
        let result = container.create_hardware_reporting_service(None);

        // Should succeed on supported platforms
        if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            assert!(result.is_ok());
        }
    }
}
