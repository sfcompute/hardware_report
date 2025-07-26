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

use crate::domain::{
    CpuTopology, HardwareInfo, HardwareReport, InterfaceIPs, PublishConfig, PublishError,
    ReportConfig, ReportError, SystemSummary,
};
use crate::ports::{
    ConfigurationProvider, DataPublisher, HardwareReportingService, SystemInfoProvider,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Domain service that implements hardware report collection
///
/// This service coordinates the collection of hardware information from various
/// system sources and aggregates them into a complete hardware report.
pub struct HardwareCollectionService {
    /// System information provider (platform-specific)
    system_provider: Arc<dyn SystemInfoProvider>,
    /// Data publisher for remote endpoints
    data_publisher: Arc<dyn DataPublisher>,
    /// Configuration provider
    config_provider: Arc<dyn ConfigurationProvider>,
}

impl HardwareCollectionService {
    /// Create a new hardware collection service
    ///
    /// # Arguments
    /// * `system_provider` - Platform-specific system information provider
    /// * `data_publisher` - Publisher for sending reports to remote endpoints
    /// * `config_provider` - Configuration provider
    pub fn new(
        system_provider: Arc<dyn SystemInfoProvider>,
        data_publisher: Arc<dyn DataPublisher>,
        config_provider: Arc<dyn ConfigurationProvider>,
    ) -> Self {
        Self {
            system_provider,
            data_publisher,
            config_provider,
        }
    }

    /// Collect all hardware information and create summary
    async fn collect_hardware_info(&self) -> Result<(HardwareInfo, SystemSummary), ReportError> {
        // Collect all hardware components concurrently
        let (cpu_result, memory_result, storage_result, gpu_result, network_result) = tokio::join!(
            self.system_provider.get_cpu_info(),
            self.system_provider.get_memory_info(),
            self.system_provider.get_storage_info(),
            self.system_provider.get_gpu_info(),
            self.system_provider.get_network_info(),
        );

        let cpu = cpu_result
            .map_err(|e| ReportError::GenerationFailed(format!("CPU collection failed: {}", e)))?;
        let memory = memory_result.map_err(|e| {
            ReportError::GenerationFailed(format!("Memory collection failed: {}", e))
        })?;
        let storage = storage_result.map_err(|e| {
            ReportError::GenerationFailed(format!("Storage collection failed: {}", e))
        })?;
        let gpus = gpu_result
            .map_err(|e| ReportError::GenerationFailed(format!("GPU collection failed: {}", e)))?;
        let network = network_result.map_err(|e| {
            ReportError::GenerationFailed(format!("Network collection failed: {}", e))
        })?;

        let hardware = HardwareInfo {
            cpu: cpu.clone(),
            memory: memory.clone(),
            storage: storage.clone(),
            gpus: gpus.clone(),
        };

        // Collect system metadata concurrently
        let (
            system_info_result,
            bios_result,
            chassis_result,
            motherboard_result,
            numa_result,
            filesystems_result,
        ) = tokio::join!(
            self.system_provider.get_system_info(),
            self.system_provider.get_bios_info(),
            self.system_provider.get_chassis_info(),
            self.system_provider.get_motherboard_info(),
            self.system_provider.get_numa_topology(),
            self.system_provider.get_filesystems(),
        );

        let system_info = system_info_result.map_err(|e| {
            ReportError::GenerationFailed(format!("System info collection failed: {}", e))
        })?;
        let bios = bios_result
            .map_err(|e| ReportError::GenerationFailed(format!("BIOS collection failed: {}", e)))?;
        let chassis = chassis_result.map_err(|e| {
            ReportError::GenerationFailed(format!("Chassis collection failed: {}", e))
        })?;
        let motherboard = motherboard_result.map_err(|e| {
            ReportError::GenerationFailed(format!("Motherboard collection failed: {}", e))
        })?;
        let numa_topology = numa_result
            .map_err(|e| ReportError::GenerationFailed(format!("NUMA collection failed: {}", e)))?;
        let filesystems = filesystems_result.map_err(|e| {
            ReportError::GenerationFailed(format!("Filesystem collection failed: {}", e))
        })?;

        // Calculate summary information
        let summary = self
            .create_system_summary(
                system_info,
                &memory,
                &storage,
                &gpus,
                &network,
                bios,
                chassis,
                motherboard,
                numa_topology,
                filesystems,
                &cpu,
            )
            .await?;

        Ok((hardware, summary))
    }

    /// Create system summary from collected information
    async fn create_system_summary(
        &self,
        system_info: crate::domain::SystemInfo,
        memory: &crate::domain::MemoryInfo,
        storage: &crate::domain::StorageInfo,
        gpus: &crate::domain::GpuInfo,
        network: &crate::domain::NetworkInfo,
        bios: crate::domain::BiosInfo,
        chassis: crate::domain::ChassisInfo,
        motherboard: crate::domain::MotherboardInfo,
        numa_topology: HashMap<String, crate::domain::NumaNode>,
        filesystems: Vec<String>,
        cpu: &crate::domain::CpuInfo,
    ) -> Result<SystemSummary, ReportError> {
        // Calculate CPU topology
        let cpu_topology = CpuTopology {
            total_cores: cpu.cores * cpu.sockets,
            total_threads: cpu.cores * cpu.sockets * cpu.threads,
            sockets: cpu.sockets,
            cores_per_socket: cpu.cores,
            threads_per_core: cpu.threads,
            numa_nodes: numa_topology.len() as u32,
            cpu_model: cpu.model.clone(),
        };

        // Calculate total storage in TB
        let total_storage_tb = self.calculate_total_storage_tb(&storage.devices);

        // Create CPU summary string
        let cpu_summary = format!(
            "{} ({} Socket{}, {} Core{}/Socket, {} Thread{}/Core, {} NUMA Node{})",
            cpu.model,
            cpu.sockets,
            if cpu.sockets == 1 { "" } else { "s" },
            cpu.cores,
            if cpu.cores == 1 { "" } else { "s" },
            cpu.threads,
            if cpu.threads == 1 { "" } else { "s" },
            numa_topology.len(),
            if numa_topology.len() == 1 { "" } else { "s" }
        );

        // Create memory config string
        let memory_config = format!("{} @ {}", memory.type_, memory.speed);

        Ok(SystemSummary {
            system_info,
            total_memory: memory.total.clone(),
            memory_config,
            total_storage: self.format_total_storage(&storage.devices),
            total_storage_tb,
            filesystems,
            bios,
            chassis,
            motherboard,
            total_gpus: gpus.devices.len(),
            total_nics: network.interfaces.len(),
            numa_topology,
            cpu_topology,
            cpu_summary,
        })
    }

    /// Calculate total storage in TB
    fn calculate_total_storage_tb(&self, devices: &[crate::domain::StorageDevice]) -> f64 {
        devices
            .iter()
            .map(|device| self.parse_storage_size_to_bytes(&device.size))
            .sum::<u64>() as f64
            / (1024.0 * 1024.0 * 1024.0 * 1024.0) // Convert bytes to TB
    }

    /// Format total storage as human-readable string
    fn format_total_storage(&self, devices: &[crate::domain::StorageDevice]) -> String {
        if devices.is_empty() {
            return "No storage devices found".to_string();
        }

        let total_tb = self.calculate_total_storage_tb(devices);
        if total_tb >= 1.0 {
            format!("{:.1} TB", total_tb)
        } else {
            let total_gb = total_tb * 1024.0;
            format!("{:.0} GB", total_gb)
        }
    }

    /// Parse storage size string to bytes (simplified version)
    fn parse_storage_size_to_bytes(&self, size: &str) -> u64 {
        // This is a simplified implementation - in the real implementation,
        // we would use the more sophisticated parsing logic from the original code
        if size.contains("TB") {
            if let Some(num_str) = size.split_whitespace().next() {
                if let Ok(num) = num_str.parse::<f64>() {
                    return (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64;
                }
            }
        } else if size.contains("GB") {
            if let Some(num_str) = size.split_whitespace().next() {
                if let Ok(num) = num_str.parse::<f64>() {
                    return (num * 1024.0 * 1024.0 * 1024.0) as u64;
                }
            }
        }
        0
    }

    /// Get hostname and FQDN
    async fn get_network_identity(
        &self,
    ) -> Result<(String, String, Vec<InterfaceIPs>), ReportError> {
        let hostname = self.system_provider.get_hostname().await.map_err(|e| {
            ReportError::GenerationFailed(format!("Hostname collection failed: {}", e))
        })?;

        let fqdn =
            self.system_provider.get_fqdn().await.map_err(|e| {
                ReportError::GenerationFailed(format!("FQDN collection failed: {}", e))
            })?;

        // For now, create empty OS IP list - this would be populated by network adapter
        let os_ip = Vec::new();

        Ok((hostname, fqdn, os_ip))
    }
}

#[async_trait]
impl HardwareReportingService for HardwareCollectionService {
    async fn generate_report(&self, _config: ReportConfig) -> Result<HardwareReport, ReportError> {
        // Collect network identity and hardware info concurrently
        let (network_result, hardware_result) =
            tokio::join!(self.get_network_identity(), self.collect_hardware_info());

        let (hostname, fqdn, os_ip) = network_result?;
        let (hardware, summary) = hardware_result?;

        // Get network info for the report
        let network = self.system_provider.get_network_info().await.map_err(|e| {
            ReportError::GenerationFailed(format!("Network collection failed: {}", e))
        })?;

        let report = HardwareReport {
            summary,
            hostname,
            fqdn,
            os_ip,
            bmc_ip: None,  // Would be populated by BMC detection logic
            bmc_mac: None, // Would be populated by BMC detection logic
            hardware,
            network,
        };

        Ok(report)
    }

    async fn publish_report(
        &self,
        report: &HardwareReport,
        config: &PublishConfig,
    ) -> Result<(), PublishError> {
        self.data_publisher.publish(report, config).await
    }

    async fn validate_dependencies(&self) -> Result<Vec<String>, ReportError> {
        self.system_provider
            .get_missing_dependencies()
            .await
            .map_err(|e| {
                ReportError::GenerationFailed(format!("Dependency validation failed: {}", e))
            })
    }

    async fn check_privileges(&self) -> Result<bool, ReportError> {
        self.system_provider
            .has_required_privileges()
            .await
            .map_err(|e| ReportError::GenerationFailed(format!("Privilege check failed: {}", e)))
    }
}
