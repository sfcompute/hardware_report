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
    TOML,
    JSON,
}

impl std::str::FromStr for FileFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TOML" => Ok(FileFormat::TOML),
            "JSON" => Ok(FileFormat::JSON),
            _ => Err("File format must be either 'toml' or 'json'".to_string()),
        }
    }
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FileFormat::TOML => write!(f, "TOML"),
            FileFormat::JSON => write!(f, "JSON"),
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
    #[structopt(long, default_value = "https://example.com/toml")]
    endpoint: String,

    /// Authentication token
    #[structopt(long, env = "HARDWARE_REPORT_TOKEN")]
    auth_token: Option<String>,

    /// Labels in key=value format (only included in POST payload, not in output file)
    #[structopt(long = "label", parse(try_from_str = parse_label))]
    labels: Vec<(String, String)>,

    /// Output file format (toml or json)
    #[structopt(long, default_value = "toml")]
    file_format: FileFormat,

    /// Save POST payload to specified file for debugging (only works with --post)
    #[structopt(long)]
    save_payload: Option<String>,
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

    // Generate summary output for console
    println!("System Summary:");
    println!("==============");
    println!("Hostname: {}", server_info.hostname);
    println!("System UUID: {}", server_info.summary.system_uuid);
    println!("System Serial: {}", server_info.summary.system_serial);
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

    // Calculate total storage
    let total_storage = server_info
        .hardware
        .storage
        .devices
        .iter()
        .map(|device| device.size.clone())
        .collect::<Vec<String>>()
        .join(" + ");
    println!("Available Disks: {}", total_storage);

    // Get BIOS information from dmidecode
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
        println!(
            "  {} - {} {} ({}) [Speed: {}] [NUMA: {}]",
            nic.name,
            nic.vendor,
            nic.model,
            nic.pci_id,
            nic.speed.as_deref().unwrap_or("Unknown"),
            nic.numa_node
                .map_or("Unknown".to_string(), |n| n.to_string())
        );
    }

    println!("\nGPUs:");
    for gpu in &server_info.hardware.gpus.devices {
        println!(
            "  {} - {} ({}) [NUMA: {}]",
            gpu.name,
            gpu.vendor,
            gpu.pci_id,
            gpu.numa_node
                .map_or("Unknown".to_string(), |n| n.to_string())
        );
    }

    println!("\nNUMA Topology:");
    for (node_id, node) in &server_info.summary.numa_topology {
        println!("  Node {}:", node_id);
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

    println!(
        "\nCreating {} output for system serial number: {}",
        opt.file_format, safe_filename
    );

    let output_filename = format!(
        "{}_hardware_report.{}",
        safe_filename,
        match opt.file_format {
            FileFormat::TOML => "toml",
            FileFormat::JSON => "json",
        }
    );

    // Convert to specified format and write to file
    let output_string = match opt.file_format {
        FileFormat::TOML => toml::to_string_pretty(&server_info)?,
        FileFormat::JSON => serde_json::to_string_pretty(&server_info)?,
    };

    std::fs::write(&output_filename, output_string)?;
    println!("Configuration has been written to {}", output_filename);

    // Handle posting if enabled
    if opt.post {
        let labels: HashMap<String, String> = opt.labels.into_iter().collect();
        post_data(
            server_info,
            labels,
            &opt.endpoint,
            opt.auth_token.as_deref(),
        )
        .await?;
        println!("Successfully posted data to remote server");
    }

    Ok(())
}
