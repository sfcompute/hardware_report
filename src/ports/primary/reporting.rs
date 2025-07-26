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

use crate::domain::{HardwareReport, PublishConfig, PublishError, ReportConfig, ReportError};
use async_trait::async_trait;

/// Primary port - Main interface offered by the hardware reporting domain
///
/// This is what external systems (CLI, Web API, library consumers) use to interact
/// with the hardware reporting functionality.
#[async_trait]
pub trait HardwareReportingService: Send + Sync {
    /// Generate a complete hardware report for the current system
    ///
    /// # Arguments
    /// * `config` - Configuration options for report generation
    ///
    /// # Returns
    /// * `Ok(HardwareReport)` - Complete hardware report
    /// * `Err(ReportError)` - Error occurred during collection
    async fn generate_report(&self, config: ReportConfig) -> Result<HardwareReport, ReportError>;

    /// Publish a hardware report to a remote endpoint
    ///
    /// # Arguments
    /// * `report` - The hardware report to publish
    /// * `config` - Publishing configuration (endpoint, auth, etc.)
    ///
    /// # Returns
    /// * `Ok(())` - Report successfully published
    /// * `Err(PublishError)` - Error occurred during publishing
    async fn publish_report(
        &self,
        report: &HardwareReport,
        config: &PublishConfig,
    ) -> Result<(), PublishError>;

    /// Validate system dependencies and return missing requirements
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of missing dependencies (empty if all present)
    /// * `Err(ReportError)` - Error occurred during validation
    async fn validate_dependencies(&self) -> Result<Vec<String>, ReportError>;

    /// Check if the current user has sufficient privileges for hardware collection
    ///
    /// # Returns
    /// * `Ok(bool)` - true if sufficient privileges, false otherwise
    /// * `Err(ReportError)` - Error occurred during privilege check
    async fn check_privileges(&self) -> Result<bool, ReportError>;
}

/// Primary port - System monitoring interface for real-time hardware monitoring
///
/// This interface provides streaming capabilities for continuous hardware monitoring.
/// Currently not implemented but defined for future extensibility.
#[async_trait]
pub trait HardwareMonitoringService: Send + Sync {
    /// Start continuous hardware monitoring
    ///
    /// # Arguments
    /// * `interval_seconds` - Monitoring interval in seconds
    /// * `config` - Monitoring configuration
    ///
    /// # Returns
    /// * `Ok(MonitoringHandle)` - Handle to control monitoring
    /// * `Err(ReportError)` - Error occurred starting monitoring
    async fn start_monitoring(
        &self,
        interval_seconds: u64,
        config: ReportConfig,
    ) -> Result<MonitoringHandle, ReportError>;
}

/// Handle for controlling hardware monitoring sessions
#[derive(Debug)]
pub struct MonitoringHandle {
    /// Unique session identifier
    pub session_id: String,
}

impl MonitoringHandle {
    /// Stop the monitoring session
    pub async fn stop(&self) -> Result<(), ReportError> {
        // Implementation would be added when monitoring is implemented
        Ok(())
    }
}
