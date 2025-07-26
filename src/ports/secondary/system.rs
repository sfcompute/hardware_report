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
    BiosInfo, ChassisInfo, CpuInfo, GpuInfo, MemoryInfo, MotherboardInfo, NetworkInfo, NumaNode,
    StorageInfo, SystemError, SystemInfo,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Secondary port - System information provider
///
/// This interface abstracts platform-specific system information collection.
/// Different adapters can implement this for Linux, macOS, Windows, etc.
#[async_trait]
pub trait SystemInfoProvider: Send + Sync {
    /// Collect CPU information
    ///
    /// # Returns
    /// * `Ok(CpuInfo)` - CPU details
    /// * `Err(SystemError)` - Error collecting CPU info
    async fn get_cpu_info(&self) -> Result<CpuInfo, SystemError>;

    /// Collect memory information
    ///
    /// # Returns
    /// * `Ok(MemoryInfo)` - Memory details including modules
    /// * `Err(SystemError)` - Error collecting memory info
    async fn get_memory_info(&self) -> Result<MemoryInfo, SystemError>;

    /// Collect storage information
    ///
    /// # Returns
    /// * `Ok(StorageInfo)` - Storage device details
    /// * `Err(SystemError)` - Error collecting storage info
    async fn get_storage_info(&self) -> Result<StorageInfo, SystemError>;

    /// Collect GPU information
    ///
    /// # Returns
    /// * `Ok(GpuInfo)` - GPU device details
    /// * `Err(SystemError)` - Error collecting GPU info
    async fn get_gpu_info(&self) -> Result<GpuInfo, SystemError>;

    /// Collect network interface information
    ///
    /// # Returns
    /// * `Ok(NetworkInfo)` - Network interface details
    /// * `Err(SystemError)` - Error collecting network info
    async fn get_network_info(&self) -> Result<NetworkInfo, SystemError>;

    /// Collect BIOS/firmware information
    ///
    /// # Returns
    /// * `Ok(BiosInfo)` - BIOS/firmware details
    /// * `Err(SystemError)` - Error collecting BIOS info
    async fn get_bios_info(&self) -> Result<BiosInfo, SystemError>;

    /// Collect chassis information
    ///
    /// # Returns
    /// * `Ok(ChassisInfo)` - Chassis details
    /// * `Err(SystemError)` - Error collecting chassis info
    async fn get_chassis_info(&self) -> Result<ChassisInfo, SystemError>;

    /// Collect motherboard information
    ///
    /// # Returns
    /// * `Ok(MotherboardInfo)` - Motherboard details
    /// * `Err(SystemError)` - Error collecting motherboard info
    async fn get_motherboard_info(&self) -> Result<MotherboardInfo, SystemError>;

    /// Collect basic system information
    ///
    /// # Returns
    /// * `Ok(SystemInfo)` - System UUID, serial, product info
    /// * `Err(SystemError)` - Error collecting system info
    async fn get_system_info(&self) -> Result<SystemInfo, SystemError>;

    /// Collect NUMA topology information
    ///
    /// # Returns
    /// * `Ok(HashMap<String, NumaNode>)` - NUMA topology mapping
    /// * `Err(SystemError)` - Error collecting NUMA info
    async fn get_numa_topology(&self) -> Result<HashMap<String, NumaNode>, SystemError>;

    /// Get system hostname
    ///
    /// # Returns
    /// * `Ok(String)` - System hostname
    /// * `Err(SystemError)` - Error getting hostname
    async fn get_hostname(&self) -> Result<String, SystemError>;

    /// Get fully qualified domain name
    ///
    /// # Returns
    /// * `Ok(String)` - FQDN
    /// * `Err(SystemError)` - Error getting FQDN
    async fn get_fqdn(&self) -> Result<String, SystemError>;

    /// Get filesystem information
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of filesystem descriptions
    /// * `Err(SystemError)` - Error collecting filesystem info
    async fn get_filesystems(&self) -> Result<Vec<String>, SystemError>;

    /// Check if running with sufficient privileges
    ///
    /// # Returns
    /// * `Ok(bool)` - true if running with adequate privileges
    /// * `Err(SystemError)` - Error checking privileges
    async fn has_required_privileges(&self) -> Result<bool, SystemError>;

    /// Get list of missing system dependencies
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of missing commands/tools
    /// * `Err(SystemError)` - Error checking dependencies
    async fn get_missing_dependencies(&self) -> Result<Vec<String>, SystemError>;
}
