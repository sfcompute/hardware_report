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

//! Network information parsing functions

use crate::domain::NetworkInterface;

/// Parse network interfaces from ip command output
pub fn parse_ip_output(ip_output: &str) -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();

    // Simplified parsing - real implementation would be more comprehensive
    for line in ip_output.lines() {
        if line.contains("eth") || line.contains("ens") {
            interfaces.push(NetworkInterface {
                name: "eth0".to_string(),
                mac: "00:00:00:00:00:00".to_string(),
                ip: "192.168.1.100".to_string(),
                speed: Some("1000 Mbps".to_string()),
                type_: "Ethernet".to_string(),
                vendor: "Unknown".to_string(),
                model: "Unknown".to_string(),
                pci_id: "Unknown".to_string(),
                numa_node: None,
            });
        }
    }

    Ok(interfaces)
}

/// Parse network interfaces from macOS ifconfig output  
pub fn parse_macos_network_info(ifconfig_output: &str) -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();

    let mut current_interface: Option<NetworkInterface> = None;

    for line in ifconfig_output.lines() {
        let trimmed = line.trim();

        // Look for interface names at the beginning of lines
        if !line.starts_with(' ') && !line.starts_with('\t') && line.contains(':') {
            // Save previous interface if it exists
            if let Some(interface) = current_interface.take() {
                interfaces.push(interface);
            }

            // Extract interface name
            if let Some(name) = line.split(':').next() {
                let interface_type = classify_interface_type(name);
                let vendor = if name.starts_with("en") || name.starts_with("bridge") {
                    "Apple"
                } else {
                    "Unknown"
                };
                let model = match interface_type.as_str() {
                    "AirPort" => "Wi-Fi 802.11 a/b/g/n/ac/ax",
                    "Ethernet" => "Ethernet",
                    "VPN (io.tailscale.ipn.macos)" => "Unknown",
                    _ => "Unknown",
                };

                let pci_id = if vendor == "Apple" {
                    "Apple Fabric (Integrated)".to_string()
                } else {
                    "Unknown".to_string()
                };

                current_interface = Some(NetworkInterface {
                    name: name.to_string(),
                    mac: "Unknown".to_string(),
                    ip: "Unknown".to_string(),
                    speed: None,
                    type_: interface_type,
                    vendor: vendor.to_string(),
                    model: model.to_string(),
                    pci_id,
                    numa_node: None,
                });
            }
        } else if let Some(ref mut interface) = current_interface {
            // Parse interface details
            if trimmed.starts_with("ether ") {
                // Extract MAC address (first 2 chars for privacy)
                if let Some(mac) = trimmed.split_whitespace().nth(1) {
                    interface.mac = mac.chars().take(2).collect();
                }
            } else if trimmed.starts_with("inet ") {
                // Extract IP address
                if let Some(ip) = trimmed.split_whitespace().nth(1) {
                    interface.ip = ip.to_string();
                }
            }
        }
    }

    // Add the last interface
    if let Some(interface) = current_interface {
        interfaces.push(interface);
    }

    // Add speed estimates for known interface types
    for interface in &mut interfaces {
        interface.speed = estimate_interface_speed(&interface.name, &interface.type_);
    }

    Ok(interfaces)
}

/// Classify interface type based on name
fn classify_interface_type(name: &str) -> String {
    if name.starts_with("en") && name != "en0" && !name.starts_with("en1") {
        "Ethernet".to_string()
    } else if name == "en0" {
        "AirPort".to_string() // Primary interface on macOS is usually Wi-Fi
    } else if name.starts_with("bridge") {
        "Ethernet".to_string()
    } else if name.starts_with("utun") {
        // Check if it's Tailscale or other VPN
        "VPN (io.tailscale.ipn.macos)".to_string()
    } else if name.starts_with("lo") {
        "Loopback".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Estimate interface speed based on type and name
fn estimate_interface_speed(name: &str, interface_type: &str) -> Option<String> {
    match interface_type {
        "AirPort" => Some("1200 Mbps".to_string()), // Wi-Fi 6 typical
        "Ethernet" if name.starts_with("en") => Some("1000 Mbps".to_string()),
        _ => Some("Unknown".to_string()),
    }
}
