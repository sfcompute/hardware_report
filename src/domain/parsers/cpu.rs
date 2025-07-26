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

//! CPU information parsing functions

use super::common::{clean_value, extract_dmidecode_value, parse_key_value};
use crate::domain::{CpuInfo, CpuTopology};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref CPU_SPEED_RE: Regex = Regex::new(r"(\d+(?:\.\d+)?)\s*(MHz|GHz)").unwrap();
    static ref CORE_COUNT_RE: Regex = Regex::new(r"(\d+)").unwrap();
}

/// Parse CPU information from Linux lscpu output
///
/// # Arguments
/// * `lscpu_output` - Raw output from lscpu command
///
/// # Returns
/// * `Ok(CpuInfo)` - Parsed CPU information
/// * `Err(String)` - Parse error description
pub fn parse_lscpu_output(lscpu_output: &str) -> Result<CpuInfo, String> {
    let mut model = "Unknown CPU".to_string();
    let mut cores = 1u32;
    let mut threads = 1u32;
    let mut sockets = 1u32;
    let mut speed = "Unknown".to_string();

    for line in lscpu_output.lines() {
        if let Ok((key, value)) = parse_key_value(line, ':') {
            match key.as_str() {
                "Model name" => {
                    model = clean_value(&value);
                }
                "CPU(s)" => {
                    if let Ok(total_cpus) = value.parse::<u32>() {
                        // This gives us total logical CPUs
                        threads = total_cpus;
                    }
                }
                "Core(s) per socket" => {
                    if let Ok(cores_per_socket) = value.parse::<u32>() {
                        cores = cores_per_socket;
                    }
                }
                "Socket(s)" => {
                    if let Ok(socket_count) = value.parse::<u32>() {
                        sockets = socket_count;
                    }
                }
                "Thread(s) per core" => {
                    if let Ok(threads_per_core) = value.parse::<u32>() {
                        // Recalculate threads based on cores and threads per core
                        threads = threads_per_core;
                    }
                }
                "CPU MHz" | "CPU max MHz" => {
                    speed = format!("{} MHz", clean_value(&value));
                }
                _ => {}
            }
        }
    }

    Ok(CpuInfo {
        model,
        cores,
        threads,
        sockets,
        speed,
    })
}

/// Parse CPU information from dmidecode processor output
///
/// # Arguments
/// * `dmidecode_output` - Raw output from dmidecode -t processor
///
/// # Returns
/// * `Ok(CpuInfo)` - Parsed CPU information
/// * `Err(String)` - Parse error description
pub fn parse_dmidecode_cpu(dmidecode_output: &str) -> Result<CpuInfo, String> {
    let model = extract_dmidecode_value(dmidecode_output, "Version")
        .unwrap_or_else(|_| "Unknown CPU".to_string());

    let speed = extract_dmidecode_value(dmidecode_output, "Current Speed")
        .or_else(|_| extract_dmidecode_value(dmidecode_output, "Max Speed"))
        .unwrap_or_else(|_| "Unknown".to_string());

    let core_count_str =
        extract_dmidecode_value(dmidecode_output, "Core Count").unwrap_or_else(|_| "1".to_string());
    let cores = core_count_str.parse::<u32>().unwrap_or(1);

    let thread_count_str = extract_dmidecode_value(dmidecode_output, "Thread Count")
        .unwrap_or_else(|_| "1".to_string());
    let threads = thread_count_str.parse::<u32>().unwrap_or(1);

    Ok(CpuInfo {
        model: clean_value(&model),
        cores,
        threads,
        sockets: 1, // dmidecode typically shows per-socket info
        speed: clean_value(&speed),
    })
}

/// Parse CPU information from macOS system_profiler output
///
/// # Arguments
/// * `system_profiler_output` - Raw output from system_profiler SPHardwareDataType
///
/// # Returns
/// * `Ok(CpuInfo)` - Parsed CPU information
/// * `Err(String)` - Parse error description
pub fn parse_macos_cpu_info(system_profiler_output: &str) -> Result<CpuInfo, String> {
    let mut model = "Unknown CPU".to_string();
    let mut cores = 1u32;
    let mut speed = "Unknown".to_string();

    for line in system_profiler_output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Chip:") {
            model = trimmed
                .split(':')
                .nth(1)
                .unwrap_or("Unknown CPU")
                .trim()
                .to_string();
        } else if trimmed.starts_with("Processor Name:") {
            // Fallback for Intel Macs
            model = trimmed
                .split(':')
                .nth(1)
                .unwrap_or("Unknown CPU")
                .trim()
                .to_string();
        } else if trimmed.starts_with("Total Number of Cores:") {
            let core_str = trimmed
                .split(':')
                .nth(1)
                .unwrap_or("1")
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("1");
            cores = core_str.parse::<u32>().unwrap_or(1);
        } else if trimmed.starts_with("Processor Speed:") {
            speed = trimmed
                .split(':')
                .nth(1)
                .unwrap_or("Unknown")
                .trim()
                .to_string();
        }
    }

    Ok(CpuInfo {
        model: clean_value(&model),
        cores,
        threads: 1, // Apple Silicon doesn't expose thread count the same way
        sockets: 1, // Apple Silicon is single socket
        speed: clean_value(&speed),
    })
}

/// Combine CPU information from multiple sources
///
/// # Arguments
/// * `primary` - Primary CPU info (e.g., from lscpu)
/// * `secondary` - Secondary CPU info (e.g., from dmidecode)
///
/// # Returns
/// * Combined and enhanced CPU information
pub fn combine_cpu_info(primary: CpuInfo, secondary: CpuInfo) -> CpuInfo {
    CpuInfo {
        model: if primary.model != "Unknown CPU" && !primary.model.is_empty() {
            primary.model
        } else {
            secondary.model
        },
        cores: if primary.cores > 0 {
            primary.cores
        } else {
            secondary.cores
        },
        threads: if primary.threads > 0 {
            primary.threads
        } else {
            secondary.threads
        },
        sockets: if primary.sockets > 0 {
            primary.sockets
        } else {
            secondary.sockets
        },
        speed: if primary.speed != "Unknown" && !primary.speed.is_empty() {
            primary.speed
        } else {
            secondary.speed
        },
    }
}

/// Create CPU topology from CPU info
///
/// # Arguments
/// * `cpu_info` - Basic CPU information
/// * `numa_nodes` - Number of NUMA nodes (optional)
///
/// # Returns
/// * CPU topology information
pub fn create_cpu_topology(cpu_info: &CpuInfo, numa_nodes: Option<u32>) -> CpuTopology {
    let total_cores = cpu_info.cores * cpu_info.sockets;
    let total_threads = total_cores * cpu_info.threads;

    CpuTopology {
        total_cores,
        total_threads,
        sockets: cpu_info.sockets,
        cores_per_socket: cpu_info.cores,
        threads_per_core: cpu_info.threads,
        numa_nodes: numa_nodes.unwrap_or(1),
        cpu_model: cpu_info.model.clone(),
    }
}

/// Create CPU summary string
///
/// # Arguments
/// * `cpu_topology` - CPU topology information
///
/// # Returns
/// * Human-readable CPU summary string
pub fn create_cpu_summary(cpu_topology: &CpuTopology) -> String {
    format!(
        "{} ({} Socket{}, {} Core{}/Socket, {} Thread{}/Core, {} NUMA Node{})",
        cpu_topology.cpu_model,
        cpu_topology.sockets,
        if cpu_topology.sockets == 1 { "" } else { "s" },
        cpu_topology.cores_per_socket,
        if cpu_topology.cores_per_socket == 1 {
            ""
        } else {
            "s"
        },
        cpu_topology.threads_per_core,
        if cpu_topology.threads_per_core == 1 {
            ""
        } else {
            "s"
        },
        cpu_topology.numa_nodes,
        if cpu_topology.numa_nodes == 1 {
            ""
        } else {
            "s"
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lscpu_output() {
        let lscpu_output = r#"Architecture:                    x86_64
CPU op-mode(s):                  32-bit, 64-bit
Byte Order:                      Little Endian
Address sizes:                   39 bits physical, 48 bits virtual
CPU(s):                          16
On-line CPU(s) list:             0-15
Thread(s) per core:              2
Core(s) per socket:              8
Socket(s):                       1
Model name:                      Intel(R) Core(TM) i7-10875H CPU @ 2.30GHz
CPU family:                      6
Model:                           165
Stepping:                        2
CPU MHz:                         2300.000"#;

        let cpu_info = parse_lscpu_output(lscpu_output).unwrap();
        assert_eq!(cpu_info.model, "Intel(R) Core(TM) i7-10875H CPU @ 2.30GHz");
        assert_eq!(cpu_info.cores, 8);
        assert_eq!(cpu_info.threads, 2);
        assert_eq!(cpu_info.sockets, 1);
        assert_eq!(cpu_info.speed, "2300.000 MHz");
    }

    #[test]
    fn test_parse_macos_cpu_info() {
        let macos_output = r#"Hardware Overview:

      Model Name: MacBook Pro
      Model Identifier: MacBookPro18,2
      Chip: Apple M1 Max
      Total Number of Cores: 10 (8 performance and 2 efficiency)
      Memory: 32 GB
      System Firmware Version: 8419.121.2
      OS Loader Version: 8419.121.2"#;

        let cpu_info = parse_macos_cpu_info(macos_output).unwrap();
        assert_eq!(cpu_info.model, "Apple M1 Max");
        assert_eq!(cpu_info.cores, 10);
        assert_eq!(cpu_info.sockets, 1);
    }

    #[test]
    fn test_combine_cpu_info() {
        let primary = CpuInfo {
            model: "Intel Core i7".to_string(),
            cores: 8,
            threads: 2,
            sockets: 1,
            speed: "Unknown".to_string(),
        };

        let secondary = CpuInfo {
            model: "Unknown CPU".to_string(),
            cores: 0,
            threads: 0,
            sockets: 0,
            speed: "2.3 GHz".to_string(),
        };

        let combined = combine_cpu_info(primary, secondary);
        assert_eq!(combined.model, "Intel Core i7");
        assert_eq!(combined.cores, 8);
        assert_eq!(combined.speed, "2.3 GHz");
    }

    #[test]
    fn test_create_cpu_topology() {
        let cpu_info = CpuInfo {
            model: "Intel Core i7".to_string(),
            cores: 8,
            threads: 2,
            sockets: 1,
            speed: "2.3 GHz".to_string(),
        };

        let topology = create_cpu_topology(&cpu_info, Some(1));
        assert_eq!(topology.total_cores, 8);
        assert_eq!(topology.total_threads, 16);
        assert_eq!(topology.sockets, 1);
        assert_eq!(topology.cores_per_socket, 8);
        assert_eq!(topology.threads_per_core, 2);
        assert_eq!(topology.numa_nodes, 1);
    }

    #[test]
    fn test_create_cpu_summary() {
        let topology = CpuTopology {
            total_cores: 16,
            total_threads: 32,
            sockets: 2,
            cores_per_socket: 8,
            threads_per_core: 2,
            numa_nodes: 2,
            cpu_model: "Intel Xeon Gold 6226R".to_string(),
        };

        let summary = create_cpu_summary(&topology);
        assert!(summary.contains("Intel Xeon Gold 6226R"));
        assert!(summary.contains("2 Sockets"));
        assert!(summary.contains("8 Cores/Socket"));
        assert!(summary.contains("2 Threads/Core"));
        assert!(summary.contains("2 NUMA Nodes"));
    }
}
