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

use hardware_report::posting::post_data;
use hardware_report::ServerInfo;
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use structopt::StructOpt;

#[derive(Debug)]
enum FileFormat {
    Toml,
    Json,
}

impl std::str::FromStr for FileFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TOML" => Ok(FileFormat::Toml),
            "JSON" => Ok(FileFormat::Json),
            _ => Err("File format must be either 'toml' or 'json'".to_string()),
        }
    }
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FileFormat::Toml => write!(f, "TOML"),
            FileFormat::Json => write!(f, "JSON"),
        }
    }
}

#[derive(StructOpt)]
#[structopt(name = "hardware_report")]
struct Opt {
    /// Enable posting to remote server
    #[structopt(long)]
    post: bool,

    /// Remote endpoint URL
    #[structopt(long, default_value = "")]
    endpoint: String,

    /// Authentication token
    #[structopt(long, env = "HARDWARE_REPORT_TOKEN")]
    auth_token: Option<String>,

    /// Labels in key=value format (only included in POST payload, not in output file)
    #[structopt(long = "label", parse(try_from_str = parse_label))]
    labels: Vec<(String, String)>,

    /// Output file format (toml or json)
    #[structopt(long, default_value = "toml")]
    _file_format: FileFormat,

    /// Save POST payload to specified file for debugging (only works with --post)
    #[structopt(long)]
    save_payload: Option<String>,

    /// Skip TLS certificate verification (not recommended for production use)
    #[structopt(long)]
    skip_tls_verify: bool,

    /// No summary output to console
    #[structopt(long)]
    noout: bool,
}

fn parse_label(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err("Label must be in key=value format".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    // Collect server information
    let server_info = ServerInfo::collect()?;

    // Generate summary output for console only if no_summary is false
    if !opt.noout {
        println!("System Summary:");
        println!("==============");
        println!("Hostname: {}", server_info.hostname);
        println!("FQDN: {}", server_info.fqdn);
        println!("System UUID: {}", server_info.summary.system_info.uuid);
        println!("System Serial: {}", server_info.summary.system_info.serial);
        println!("CPU: {}", server_info.summary.cpu_summary);
        println!(
            "Total: {} Cores, {} Threads",
            server_info.summary.cpu_topology.total_cores,
            server_info.summary.cpu_topology.total_threads
        );

        // Fix memory output format - add the missing format specifier
        println!(
            "Memory: {} {} @ {}",
            server_info.hardware.memory.total,
            server_info.hardware.memory.type_,
            server_info.hardware.memory.speed
        );

        println!(
            "Storage: {} (Total: {:.2} TB)",
            server_info.summary.total_storage, server_info.summary.total_storage_tb
        );

        // Calculate total storage - show clean disk sizes
        let disk_sizes: Vec<String> = server_info
            .hardware
            .storage
            .devices
            .iter()
            .map(|device| {
                // Extract clean size from macOS format or use as-is for Linux
                if device.size.contains("TB (") {
                    // Extract "2.0 TB" from "2.0 TB (2001111162880 Bytes) (exactly...)"
                    device
                        .size
                        .split(" (")
                        .next()
                        .unwrap_or(&device.size)
                        .to_string()
                } else {
                    device.size.clone()
                }
            })
            .collect();
        if !disk_sizes.is_empty() {
            println!("Available Disks: {}", disk_sizes.join(" + "));
        }

        // Get BIOS/Firmware information (platform-specific)
        if cfg!(target_os = "macos") {
            println!(
                "BIOS: {} {} ({})",
                server_info.summary.bios.vendor,
                server_info.summary.bios.version,
                server_info.summary.bios.release_date
            );
            println!(
                "Chassis: {} {} (S/N: {})",
                server_info.summary.chassis.manufacturer,
                server_info.summary.chassis.type_,
                server_info.summary.chassis.serial
            );
        } else {
            // Linux - use dmidecode
            let output = Command::new("dmidecode").args(["-t", "bios"]).output()?;
            let bios_str = String::from_utf8(output.stdout)?;
            println!(
                "BIOS: {} {} ({})",
                ServerInfo::extract_dmidecode_value(&bios_str, "Vendor")?,
                ServerInfo::extract_dmidecode_value(&bios_str, "Version")?,
                ServerInfo::extract_dmidecode_value(&bios_str, "Release Date")?
            );

            // Get chassis information from dmidecode
            let output = Command::new("dmidecode").args(["-t", "chassis"]).output()?;
            let chassis_str = String::from_utf8(output.stdout)?;
            println!(
                "Chassis: {} {} (S/N: {})",
                ServerInfo::extract_dmidecode_value(&chassis_str, "Manufacturer")?,
                ServerInfo::extract_dmidecode_value(&chassis_str, "Type")?,
                ServerInfo::extract_dmidecode_value(&chassis_str, "Serial Number")?
            );
        }

        // Get motherboard information from server_info
        println!(
            "Motherboard: {} {} v{} (S/N: {})",
            server_info.summary.motherboard.manufacturer,
            server_info.summary.motherboard.product_name,
            server_info.summary.motherboard.version,
            server_info.summary.motherboard.serial
        );

        println!("\nNetwork Interfaces:");
        for nic in &server_info.network.interfaces {
            let numa_info = if cfg!(target_os = "macos") || nic.numa_node.is_none() {
                String::new() // No NUMA info on macOS or when not detected
            } else {
                format!(
                    " [NUMA: {}]",
                    nic.numa_node
                        .map_or("Unknown".to_string(), |n| n.to_string())
                )
            };

            let pci_info = if cfg!(target_os = "macos") && nic.pci_id == "Unknown" {
                String::new() // Hide PCI ID on macOS when not available
            } else {
                format!(" ({})", nic.pci_id)
            };

            println!(
                "  {} - {} {}{} [Speed: {}]{}",
                nic.name,
                nic.vendor,
                nic.model,
                pci_info,
                nic.speed.as_deref().unwrap_or("Unknown"),
                numa_info
            );
        }

        println!("\nGPUs:");
        for gpu in &server_info.hardware.gpus.devices {
            let numa_info = if cfg!(target_os = "macos") || gpu.numa_node.is_none() {
                String::new() // No NUMA info on macOS or when not detected
            } else {
                format!(
                    " [NUMA: {}]",
                    gpu.numa_node
                        .map_or("Unknown".to_string(), |n| n.to_string())
                )
            };

            let pci_info = if cfg!(target_os = "macos") && gpu.pci_id == "Unknown" {
                String::new() // Hide PCI ID on macOS when not available
            } else {
                format!(" ({})", gpu.pci_id)
            };

            let memory_info = if gpu.memory != "Unknown" {
                format!(" [{}]", gpu.memory)
            } else {
                String::new()
            };

            println!(
                "  {} - {}{}{}{}",
                gpu.name, gpu.vendor, memory_info, pci_info, numa_info
            );
        }

        // On macOS, show display information summary
        if cfg!(target_os = "macos") {
            println!("\nDisplays:");
            // Run system_profiler to get display info
            if let Ok(output) = std::process::Command::new("system_profiler")
                .args(["SPDisplaysDataType", "-detailLevel", "mini"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut in_displays_section = false;
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("Displays:") {
                        in_displays_section = true;
                        continue;
                    }
                    if in_displays_section && line.starts_with("        ") && trimmed.ends_with(":")
                    {
                        // This is a display name
                        let display_name = trimmed.trim_end_matches(':');
                        println!("  {display_name}");
                    }
                }
            }
        }

        // Only show NUMA topology on Linux where it's relevant
        if !cfg!(target_os = "macos") && !server_info.summary.numa_topology.is_empty() {
            println!("\nNUMA Topology:");
            for (node_id, node) in &server_info.summary.numa_topology {
                println!("  Node {node_id}:");
                println!("    Memory: {}", node.memory);
                println!("    CPUs: {:?}", node.cpus);

                if !node.devices.is_empty() {
                    println!("    Devices:");
                    for device in &node.devices {
                        println!(
                            "      {} - {} (PCI ID: {})",
                            device.type_, device.name, device.pci_id
                        );
                    }
                }

                println!("    Distances:");
                let mut distances: Vec<_> = node.distances.iter().collect();
                distances.sort_by_key(|&(k, _)| k);
                for (to_node, distance) in distances {
                    println!("      To Node {}: {}", to_node, distance);
                }
            }
        }

        // Get filesystem information
        println!("\nFilesystems:");
        let output = Command::new("df")
            .args(["-h", "--output=source,fstype,size,used,avail,target"])
            .output()?;
        let fs_str = String::from_utf8(output.stdout)?;
        for line in fs_str.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 6 {
                println!(
                    "  {} ({}) - {} total, {} used, {} available, mounted on {}",
                    fields[0], fields[1], fields[2], fields[3], fields[4], fields[5]
                );
            }
        }
    }

    // Get chassis serial number and sanitize it for use as the file_name
    let chassis_serial = server_info.summary.chassis.serial.clone();
    let safe_filename = sanitize_filename(&chassis_serial);

    fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
    }

    println!("\nCreating output files for system serial number: {safe_filename}");

    // Generate both TOML and JSON files
    let toml_filename = format!("{safe_filename}_hardware_report.toml");
    let json_filename = format!("{safe_filename}_hardware_report.json");

    // Write TOML file
    let toml_string = toml::to_string_pretty(&server_info)?;
    std::fs::write(&toml_filename, toml_string)?;

    // Write JSON file
    let json_string = serde_json::to_string_pretty(&server_info)?;
    std::fs::write(&json_filename, json_string)?;

    println!("Configuration files have been written:");

    // Handle posting if enabled
    if opt.post {
        let labels: HashMap<String, String> = opt.labels.into_iter().collect();
        post_data(
            server_info,
            labels,
            &opt.endpoint,
            opt.auth_token.as_deref(),
            opt.save_payload.as_deref(),
            opt.skip_tls_verify,
        )
        .await?;
        println!("\nSuccessfully posted data to remote server");
    }

    // Final message about available output formats
    println!("\nHardware report files are available in both JSON and TOML formats:");
    println!("  - {toml_filename}");
    println!("  - {json_filename}");

    Ok(())
}
