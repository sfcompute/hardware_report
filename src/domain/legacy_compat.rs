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

//! Legacy compatibility layer for converting between old and new types
//!
//! This module provides conversion traits to maintain backward compatibility
//! while transitioning to the new Ports and Adapters architecture.

use crate::domain::entities as new;

/// Convert from legacy ServerInfo to new HardwareReport
impl From<crate::ServerInfo> for new::HardwareReport {
    fn from(legacy: crate::ServerInfo) -> Self {
        new::HardwareReport {
            summary: legacy.summary.into(),
            hostname: legacy.hostname,
            fqdn: legacy.fqdn,
            os_ip: legacy.os_ip.into_iter().map(|ip| ip.into()).collect(),
            bmc_ip: legacy.bmc_ip,
            bmc_mac: legacy.bmc_mac,
            hardware: legacy.hardware.into(),
            network: legacy.network.into(),
        }
    }
}

/// Convert from new HardwareReport to legacy ServerInfo
impl From<new::HardwareReport> for crate::ServerInfo {
    fn from(new_report: new::HardwareReport) -> Self {
        crate::ServerInfo {
            summary: new_report.summary.into(),
            hostname: new_report.hostname,
            fqdn: new_report.fqdn,
            os_ip: new_report.os_ip.into_iter().map(|ip| ip.into()).collect(),
            bmc_ip: new_report.bmc_ip,
            bmc_mac: new_report.bmc_mac,
            hardware: new_report.hardware.into(),
            network: new_report.network.into(),
        }
    }
}

/// Convert from legacy SystemSummary to new SystemSummary
impl From<crate::SystemSummary> for new::SystemSummary {
    fn from(legacy: crate::SystemSummary) -> Self {
        new::SystemSummary {
            system_info: legacy.system_info.into(),
            total_memory: legacy.total_memory,
            memory_config: legacy.memory_config,
            total_storage: legacy.total_storage,
            total_storage_tb: legacy.total_storage_tb,
            filesystems: legacy.filesystems,
            bios: legacy.bios.into(),
            chassis: legacy.chassis.into(),
            motherboard: legacy.motherboard.into(),
            total_gpus: legacy.total_gpus,
            total_nics: legacy.total_nics,
            numa_topology: legacy
                .numa_topology
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            cpu_topology: legacy.cpu_topology.into(),
            cpu_summary: legacy.cpu_summary,
        }
    }
}

/// Convert from new SystemSummary to legacy SystemSummary
impl From<new::SystemSummary> for crate::SystemSummary {
    fn from(new_summary: new::SystemSummary) -> Self {
        crate::SystemSummary {
            system_info: new_summary.system_info.into(),
            total_memory: new_summary.total_memory,
            memory_config: new_summary.memory_config,
            total_storage: new_summary.total_storage,
            total_storage_tb: new_summary.total_storage_tb,
            filesystems: new_summary.filesystems,
            bios: new_summary.bios.into(),
            chassis: new_summary.chassis.into(),
            motherboard: new_summary.motherboard.into(),
            total_gpus: new_summary.total_gpus,
            total_nics: new_summary.total_nics,
            numa_topology: new_summary
                .numa_topology
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            cpu_topology: new_summary.cpu_topology.into(),
            cpu_summary: new_summary.cpu_summary,
        }
    }
}

// Additional conversion impls for all the nested types would go here...
// For brevity, I'll implement the key ones and add TODO comments for others

impl From<crate::SystemInfo> for new::SystemInfo {
    fn from(legacy: crate::SystemInfo) -> Self {
        new::SystemInfo {
            uuid: legacy.uuid,
            serial: legacy.serial,
            product_name: legacy.product_name,
            product_manufacturer: legacy.product_manufacturer,
        }
    }
}

impl From<new::SystemInfo> for crate::SystemInfo {
    fn from(new_info: new::SystemInfo) -> Self {
        crate::SystemInfo {
            uuid: new_info.uuid,
            serial: new_info.serial,
            product_name: new_info.product_name,
            product_manufacturer: new_info.product_manufacturer,
        }
    }
}

impl From<crate::BiosInfo> for new::BiosInfo {
    fn from(legacy: crate::BiosInfo) -> Self {
        new::BiosInfo {
            vendor: legacy.vendor,
            version: legacy.version,
            release_date: legacy.release_date,
            firmware_version: legacy.firmware_version,
        }
    }
}

impl From<new::BiosInfo> for crate::BiosInfo {
    fn from(new_bios: new::BiosInfo) -> Self {
        crate::BiosInfo {
            vendor: new_bios.vendor,
            version: new_bios.version,
            release_date: new_bios.release_date,
            firmware_version: new_bios.firmware_version,
        }
    }
}

impl From<crate::ChassisInfo> for new::ChassisInfo {
    fn from(legacy: crate::ChassisInfo) -> Self {
        new::ChassisInfo {
            manufacturer: legacy.manufacturer,
            type_: legacy.type_,
            serial: legacy.serial,
        }
    }
}

impl From<new::ChassisInfo> for crate::ChassisInfo {
    fn from(new_chassis: new::ChassisInfo) -> Self {
        crate::ChassisInfo {
            manufacturer: new_chassis.manufacturer,
            type_: new_chassis.type_,
            serial: new_chassis.serial,
        }
    }
}

impl From<crate::MotherboardInfo> for new::MotherboardInfo {
    fn from(legacy: crate::MotherboardInfo) -> Self {
        new::MotherboardInfo {
            manufacturer: legacy.manufacturer,
            product_name: legacy.product_name,
            version: legacy.version,
            serial: legacy.serial,
            features: legacy.features,
            location: legacy.location,
            type_: legacy.type_,
        }
    }
}

impl From<new::MotherboardInfo> for crate::MotherboardInfo {
    fn from(new_mb: new::MotherboardInfo) -> Self {
        crate::MotherboardInfo {
            manufacturer: new_mb.manufacturer,
            product_name: new_mb.product_name,
            version: new_mb.version,
            serial: new_mb.serial,
            features: new_mb.features,
            location: new_mb.location,
            type_: new_mb.type_,
        }
    }
}

impl From<crate::CpuTopology> for new::CpuTopology {
    fn from(legacy: crate::CpuTopology) -> Self {
        new::CpuTopology {
            total_cores: legacy.total_cores,
            total_threads: legacy.total_threads,
            sockets: legacy.sockets,
            cores_per_socket: legacy.cores_per_socket,
            threads_per_core: legacy.threads_per_core,
            numa_nodes: legacy.numa_nodes,
            cpu_model: legacy.cpu_model,
        }
    }
}

impl From<new::CpuTopology> for crate::CpuTopology {
    fn from(new_topo: new::CpuTopology) -> Self {
        crate::CpuTopology {
            total_cores: new_topo.total_cores,
            total_threads: new_topo.total_threads,
            sockets: new_topo.sockets,
            cores_per_socket: new_topo.cores_per_socket,
            threads_per_core: new_topo.threads_per_core,
            numa_nodes: new_topo.numa_nodes,
            cpu_model: new_topo.cpu_model,
        }
    }
}

impl From<crate::HardwareInfo> for new::HardwareInfo {
    fn from(legacy: crate::HardwareInfo) -> Self {
        new::HardwareInfo {
            cpu: legacy.cpu.into(),
            memory: legacy.memory.into(),
            storage: legacy.storage.into(),
            gpus: legacy.gpus.into(),
        }
    }
}

impl From<new::HardwareInfo> for crate::HardwareInfo {
    fn from(new_hw: new::HardwareInfo) -> Self {
        crate::HardwareInfo {
            cpu: new_hw.cpu.into(),
            memory: new_hw.memory.into(),
            storage: new_hw.storage.into(),
            gpus: new_hw.gpus.into(),
        }
    }
}

impl From<crate::CpuInfo> for new::CpuInfo {
    fn from(legacy: crate::CpuInfo) -> Self {
        new::CpuInfo {
            model: legacy.model,
            cores: legacy.cores,
            threads: legacy.threads,
            sockets: legacy.sockets,
            speed: legacy.speed,
            ..Default::default()
        }
    }
}

impl From<new::CpuInfo> for crate::CpuInfo {
    fn from(new_cpu: new::CpuInfo) -> Self {
        crate::CpuInfo {
            model: new_cpu.model,
            cores: new_cpu.cores,
            threads: new_cpu.threads,
            sockets: new_cpu.sockets,
            speed: new_cpu.speed,
        }
    }
}

impl From<crate::MemoryInfo> for new::MemoryInfo {
    fn from(legacy: crate::MemoryInfo) -> Self {
        new::MemoryInfo {
            total: legacy.total,
            type_: legacy.type_,
            speed: legacy.speed,
            modules: legacy.modules.into_iter().map(|m| m.into()).collect(),
        }
    }
}

impl From<new::MemoryInfo> for crate::MemoryInfo {
    fn from(new_mem: new::MemoryInfo) -> Self {
        crate::MemoryInfo {
            total: new_mem.total,
            type_: new_mem.type_,
            speed: new_mem.speed,
            modules: new_mem.modules.into_iter().map(|m| m.into()).collect(),
        }
    }
}

impl From<crate::MemoryModule> for new::MemoryModule {
    fn from(legacy: crate::MemoryModule) -> Self {
        new::MemoryModule {
            size: legacy.size,
            type_: legacy.type_,
            speed: legacy.speed,
            location: legacy.location,
            manufacturer: legacy.manufacturer,
            serial: legacy.serial,
        }
    }
}

impl From<new::MemoryModule> for crate::MemoryModule {
    fn from(new_mod: new::MemoryModule) -> Self {
        crate::MemoryModule {
            size: new_mod.size,
            type_: new_mod.type_,
            speed: new_mod.speed,
            location: new_mod.location,
            manufacturer: new_mod.manufacturer,
            serial: new_mod.serial,
        }
    }
}

impl From<crate::StorageInfo> for new::StorageInfo {
    fn from(legacy: crate::StorageInfo) -> Self {
        new::StorageInfo {
            devices: legacy.devices.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<new::StorageInfo> for crate::StorageInfo {
    fn from(new_storage: new::StorageInfo) -> Self {
        crate::StorageInfo {
            devices: new_storage.devices.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<crate::StorageDevice> for new::StorageDevice {
    fn from(legacy: crate::StorageDevice) -> Self {
        new::StorageDevice {
            name: legacy.name,
            type_: legacy.type_.clone(),
            size: legacy.size,
            model: legacy.model,
            ..Default::default()
        }
    }
}

impl From<new::StorageDevice> for crate::StorageDevice {
    fn from(new_dev: new::StorageDevice) -> Self {
        crate::StorageDevice {
            name: new_dev.name,
            type_: new_dev.type_,
            size: new_dev.size,
            model: new_dev.model,
        }
    }
}

impl From<crate::GpuInfo> for new::GpuInfo {
    fn from(legacy: crate::GpuInfo) -> Self {
        new::GpuInfo {
            devices: legacy.devices.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<new::GpuInfo> for crate::GpuInfo {
    fn from(new_gpu: new::GpuInfo) -> Self {
        crate::GpuInfo {
            devices: new_gpu.devices.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<crate::GpuDevice> for new::GpuDevice {
    fn from(legacy: crate::GpuDevice) -> Self {
        new::GpuDevice {
            index: legacy.index,
            name: legacy.name,
            uuid: legacy.uuid,
            memory: legacy.memory,
            pci_id: legacy.pci_id,
            vendor: legacy.vendor,
            numa_node: legacy.numa_node,
            ..Default::default()
        }
    }
}

impl From<new::GpuDevice> for crate::GpuDevice {
    fn from(new_gpu: new::GpuDevice) -> Self {
        crate::GpuDevice {
            index: new_gpu.index,
            name: new_gpu.name,
            uuid: new_gpu.uuid,
            memory: new_gpu.memory,
            pci_id: new_gpu.pci_id,
            vendor: new_gpu.vendor,
            numa_node: new_gpu.numa_node,
        }
    }
}

impl From<crate::NetworkInfo> for new::NetworkInfo {
    fn from(legacy: crate::NetworkInfo) -> Self {
        new::NetworkInfo {
            interfaces: legacy.interfaces.into_iter().map(|i| i.into()).collect(),
            infiniband: legacy.infiniband.map(|ib| ib.into()),
        }
    }
}

impl From<new::NetworkInfo> for crate::NetworkInfo {
    fn from(new_net: new::NetworkInfo) -> Self {
        crate::NetworkInfo {
            interfaces: new_net.interfaces.into_iter().map(|i| i.into()).collect(),
            infiniband: new_net.infiniband.map(|ib| ib.into()),
        }
    }
}

impl From<crate::NetworkInterface> for new::NetworkInterface {
    fn from(legacy: crate::NetworkInterface) -> Self {
        new::NetworkInterface {
            name: legacy.name,
            mac: legacy.mac,
            ip: legacy.ip,
            prefix: legacy.prefix,
            speed: legacy.speed,
            type_: legacy.type_,
            vendor: legacy.vendor,
            model: legacy.model,
            pci_id: legacy.pci_id,
            numa_node: legacy.numa_node,
            ..Default::default()
        }
    }
}

impl From<new::NetworkInterface> for crate::NetworkInterface {
    fn from(new_iface: new::NetworkInterface) -> Self {
        crate::NetworkInterface {
            name: new_iface.name,
            mac: new_iface.mac,
            ip: new_iface.ip,
            prefix: new_iface.prefix,
            speed: new_iface.speed,
            type_: new_iface.type_,
            vendor: new_iface.vendor,
            model: new_iface.model,
            pci_id: new_iface.pci_id,
            numa_node: new_iface.numa_node,
        }
    }
}

impl From<crate::InfinibandInfo> for new::InfinibandInfo {
    fn from(legacy: crate::InfinibandInfo) -> Self {
        new::InfinibandInfo {
            interfaces: legacy.interfaces.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<new::InfinibandInfo> for crate::InfinibandInfo {
    fn from(new_ib: new::InfinibandInfo) -> Self {
        crate::InfinibandInfo {
            interfaces: new_ib.interfaces.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl From<crate::IbInterface> for new::IbInterface {
    fn from(legacy: crate::IbInterface) -> Self {
        new::IbInterface {
            name: legacy.name,
            port: legacy.port,
            state: legacy.state,
            rate: legacy.rate,
        }
    }
}

impl From<new::IbInterface> for crate::IbInterface {
    fn from(new_ib: new::IbInterface) -> Self {
        crate::IbInterface {
            name: new_ib.name,
            port: new_ib.port,
            state: new_ib.state,
            rate: new_ib.rate,
        }
    }
}

impl From<crate::NumaNode> for new::NumaNode {
    fn from(legacy: crate::NumaNode) -> Self {
        new::NumaNode {
            id: legacy.id,
            cpus: legacy.cpus,
            memory: legacy.memory,
            devices: legacy.devices.into_iter().map(|d| d.into()).collect(),
            distances: legacy.distances,
        }
    }
}

impl From<new::NumaNode> for crate::NumaNode {
    fn from(new_node: new::NumaNode) -> Self {
        crate::NumaNode {
            id: new_node.id,
            cpus: new_node.cpus,
            memory: new_node.memory,
            devices: new_node.devices.into_iter().map(|d| d.into()).collect(),
            distances: new_node.distances,
        }
    }
}

impl From<crate::NumaDevice> for new::NumaDevice {
    fn from(legacy: crate::NumaDevice) -> Self {
        new::NumaDevice {
            type_: legacy.type_,
            pci_id: legacy.pci_id,
            name: legacy.name,
        }
    }
}

impl From<new::NumaDevice> for crate::NumaDevice {
    fn from(new_dev: new::NumaDevice) -> Self {
        crate::NumaDevice {
            type_: new_dev.type_,
            pci_id: new_dev.pci_id,
            name: new_dev.name,
        }
    }
}

impl From<crate::InterfaceIPs> for new::InterfaceIPs {
    fn from(legacy: crate::InterfaceIPs) -> Self {
        new::InterfaceIPs {
            interface: legacy.interface,
            ip_addresses: legacy.ip_addresses,
        }
    }
}

impl From<new::InterfaceIPs> for crate::InterfaceIPs {
    fn from(new_ips: new::InterfaceIPs) -> Self {
        crate::InterfaceIPs {
            interface: new_ips.interface,
            ip_addresses: new_ips.ip_addresses,
        }
    }
}
