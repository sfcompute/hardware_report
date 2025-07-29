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

use crate::ServerInfo;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum NetBoxError {
    ApiError(String),
    ConnectionError(String),
    ValidationError(String),
}

impl fmt::Display for NetBoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetBoxError::ApiError(msg) => write!(f, "NetBox API error: {}", msg),
            NetBoxError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            NetBoxError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for NetBoxError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxDevice {
    pub name: String,
    pub device_type: u32, // ID of the device type
    pub device_role: u32, // ID of the device role
    pub platform: Option<u32>, // ID of the platform
    pub serial: String,
    pub asset_tag: Option<String>,
    pub site: u32, // ID of the site
    pub rack: Option<u32>, // ID of the rack
    pub position: Option<f32>,
    pub face: Option<String>, // "front" or "rear"
    pub status: String, // "active", "planned", "staged", etc.
    pub airflow: Option<String>, // "front-to-rear", "rear-to-front", etc.
    pub primary_ip4: Option<u32>, // ID of primary IPv4
    pub primary_ip6: Option<u32>, // ID of primary IPv6
    pub oob_ip: Option<u32>, // Out-of-band IP address ID (reference to IP address object)
    pub cluster: Option<u32>, // ID of cluster
    pub virtual_chassis: Option<u32>,
    pub vc_position: Option<u32>,
    pub vc_priority: Option<u32>,
    pub description: Option<String>,
    pub comments: Option<String>,
    pub config_template: Option<u32>,
    pub local_context_data: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxDeviceType {
    pub manufacturer: u32, // ID of manufacturer
    pub model: String,
    pub slug: String,
    pub part_number: Option<String>,
    pub u_height: f32, // Height in rack units (4.0 for Digital Ocean nodes)
    pub is_full_depth: bool,
    pub subdevice_role: Option<String>, // "parent", "child", or null
    pub airflow: Option<String>,
    pub front_image: Option<String>,
    pub rear_image: Option<String>,
    pub description: Option<String>,
    pub comments: Option<String>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxInterface {
    pub device: u32, // ID of the device
    pub name: String,
    pub type_: String, // Interface type (e.g., "1000base-t", "10gbase-x-sfpp")
    pub enabled: bool,
    pub parent: Option<u32>, // Parent interface ID
    pub bridge: Option<u32>, // Bridge interface ID
    pub lag: Option<u32>, // LAG interface ID
    pub mtu: Option<u32>,
    pub mac_address: Option<String>,
    pub speed: Option<u32>, // Speed in Kbps
    pub duplex: Option<String>, // "auto", "full", "half"
    pub wwn: Option<String>,
    pub mgmt_only: bool, // True for out-of-band management interfaces
    pub description: Option<String>,
    pub mode: Option<String>, // "access", "tagged", "tagged-all"
    pub rf_role: Option<String>,
    pub rf_channel: Option<String>,
    pub poe_mode: Option<String>,
    pub poe_type: Option<String>,
    pub rf_channel_frequency: Option<f32>,
    pub rf_channel_width: Option<f32>,
    pub tx_power: Option<u32>,
    pub untagged_vlan: Option<u32>,
    pub tagged_vlans: Option<Vec<u32>>,
    pub mark_connected: bool,
    pub cable: Option<u32>,
    pub cable_end: Option<String>,
    pub wireless_link: Option<u32>,
    pub link_peers: Option<Vec<HashMap<String, serde_json::Value>>>,
    pub link_peers_type: Option<String>,
    pub wireless_lans: Option<Vec<u32>>,
    pub vrf: Option<u32>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxIPAddress {
    pub address: String, // IP address in CIDR notation (e.g., "192.168.1.1/24")
    pub vrf: Option<u32>, // VRF ID
    pub tenant: Option<u32>, // Tenant ID
    pub status: String, // "active", "reserved", "deprecated", "dhcp", "slaac"
    pub role: Option<String>, // "loopback", "secondary", "anycast", "vip", "vrrp", "hsrp", "glbp", "carp"
    pub assigned_object_type: Option<String>, // "dcim.interface" or "virtualization.vminterface"
    pub assigned_object_id: Option<u32>, // ID of the interface
    pub nat_inside: Option<u32>, // ID of NAT inside IP
    pub nat_outside: Option<Vec<u32>>, // IDs of NAT outside IPs
    pub dns_name: Option<String>,
    pub description: Option<String>,
    pub comments: Option<String>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxInventoryItem {
    pub device: u32, // Device ID
    pub parent: Option<u32>, // Parent inventory item ID
    pub name: String,
    pub label: Option<String>,
    pub role: Option<u32>, // Inventory item role ID
    pub manufacturer: Option<u32>, // Manufacturer ID
    pub part_id: Option<String>,
    pub serial: Option<String>,
    pub asset_tag: Option<String>,
    pub discovered: bool,
    pub description: Option<String>,
    pub component_type: Option<String>,
    pub component_id: Option<u32>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxSite {
    pub name: String,
    pub slug: String,
    pub status: String, // "active", "planned", "retired"
    pub region: Option<u32>, // Region ID
    pub group: Option<u32>, // Site group ID
    pub tenant: Option<u32>, // Tenant ID
    pub facility: Option<String>,
    pub asns: Option<Vec<u32>>, // ASN IDs
    pub time_zone: Option<String>,
    pub description: Option<String>,
    pub physical_address: Option<String>,
    pub shipping_address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub comments: Option<String>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetBoxCluster {
    pub name: String,
    pub type_: u32, // Cluster type ID
    pub group: Option<u32>, // Cluster group ID
    pub tenant: Option<u32>, // Tenant ID
    pub site: Option<u32>, // Site ID
    pub status: String, // "planned", "staging", "active", "decommissioning", "offline"
    pub description: Option<String>,
    pub comments: Option<String>,
    pub tags: Option<Vec<u32>>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

pub struct NetBoxClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

impl NetBoxClient {
    pub fn new(base_url: String, token: String, skip_tls_verify: bool) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(skip_tls_verify)
            .build()?;
        
        Ok(NetBoxClient {
            base_url,
            token,
            client,
        })
    }

    pub async fn get_or_create_site(&self, name: &str, slug: &str) -> Result<u32, Box<dyn Error>> {
        // First try to find existing site
        let search_url = format!("{}/api/dcim/sites/?slug={}", self.base_url, slug);
        let response = self.client
            .get(&search_url)
            .header("Authorization", format!("Token {}", self.token))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        if let Some(results) = data["results"].as_array() {
            if !results.is_empty() {
                if let Some(id) = results[0]["id"].as_u64() {
                    return Ok(id as u32);
                }
            }
        }
        
        // Create new site if not found
        let site = NetBoxSite {
            name: name.to_string(),
            slug: slug.to_string(),
            status: "active".to_string(),
            region: None,
            group: None,
            tenant: None,
            facility: None,
            asns: None,
            time_zone: None,
            description: Some("Digital Ocean site".to_string()),
            physical_address: None,
            shipping_address: None,
            latitude: None,
            longitude: None,
            comments: None,
            tags: None,
            custom_fields: None,
        };
        
        let create_url = format!("{}/api/dcim/sites/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&site)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create site: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get site ID".into())
        }
    }

    pub async fn get_or_create_manufacturer(&self, name: &str) -> Result<u32, Box<dyn Error>> {
        // First try to find existing manufacturer
        let search_url = format!("{}/api/dcim/manufacturers/?name={}", self.base_url, name);
        let response = self.client
            .get(&search_url)
            .header("Authorization", format!("Token {}", self.token))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        if let Some(results) = data["results"].as_array() {
            if !results.is_empty() {
                if let Some(id) = results[0]["id"].as_u64() {
                    return Ok(id as u32);
                }
            }
        }
        
        // Create new manufacturer if not found
        let manufacturer = serde_json::json!({
            "name": name,
            "slug": name.to_lowercase().replace(" ", "-").replace(".", "")
        });
        
        let create_url = format!("{}/api/dcim/manufacturers/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&manufacturer)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create manufacturer: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get manufacturer ID".into())
        }
    }

    pub async fn get_or_create_device_type(&self, manufacturer_id: u32, model: &str, u_height: f32) -> Result<u32, Box<dyn Error>> {
        // First try to find existing device type
        let slug = model.to_lowercase().replace(" ", "-");
        let search_url = format!("{}/api/dcim/device-types/?manufacturer_id={}&model={}", self.base_url, manufacturer_id, model);
        let response = self.client
            .get(&search_url)
            .header("Authorization", format!("Token {}", self.token))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        if let Some(results) = data["results"].as_array() {
            if !results.is_empty() {
                if let Some(id) = results[0]["id"].as_u64() {
                    let device_type_id = id as u32;
                    
                    // Check if u_height needs to be updated to 4U
                    if let Some(current_height) = results[0]["u_height"].as_f64() {
                        if (current_height - u_height as f64).abs() > 0.1 {
                            // Update the device type height
                            let update_payload = serde_json::json!({
                                "u_height": u_height
                            });
                            
                            let update_url = format!("{}/api/dcim/device-types/{}/", self.base_url, device_type_id);
                            let update_response = self.client
                                .patch(&update_url)
                                .header("Authorization", format!("Token {}", self.token))
                                .json(&update_payload)
                                .send()
                                .await?;
                            
                            if update_response.status().is_success() {
                                println!("Updated device type {} height to {}U", model, u_height);
                            }
                        }
                    }
                    
                    return Ok(device_type_id);
                }
            }
        }
        
        // Create new device type if not found
        let device_type = NetBoxDeviceType {
            manufacturer: manufacturer_id,
            model: model.to_string(),
            slug,
            part_number: None,
            u_height,
            is_full_depth: true,
            subdevice_role: None,
            airflow: Some("front-to-rear".to_string()),
            front_image: None,
            rear_image: None,
            description: None,
            comments: None,
            tags: None,
            custom_fields: None,
        };
        
        let create_url = format!("{}/api/dcim/device-types/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&device_type)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create device type: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get device type ID".into())
        }
    }

    pub async fn get_or_create_device_role(&self, name: &str) -> Result<u32, Box<dyn Error>> {
        // First try to find existing device role
        let search_url = format!("{}/api/dcim/device-roles/?name={}", self.base_url, name);
        let response = self.client
            .get(&search_url)
            .header("Authorization", format!("Token {}", self.token))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        if let Some(results) = data["results"].as_array() {
            if !results.is_empty() {
                if let Some(id) = results[0]["id"].as_u64() {
                    return Ok(id as u32);
                }
            }
        }
        
        // Create new device role if not found
        let role = serde_json::json!({
            "name": name,
            "slug": name.to_lowercase(),
            "color": "0066cc"
        });
        
        let create_url = format!("{}/api/dcim/device-roles/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&role)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create device role: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get device role ID".into())
        }
    }

    pub async fn find_device_by_serial(&self, serial: &str) -> Result<Option<u32>, Box<dyn Error>> {
        let search_url = format!("{}/api/dcim/devices/?serial={}", self.base_url, serial);
        let response = self.client
            .get(&search_url)
            .header("Authorization", format!("Token {}", self.token))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        if let Some(results) = data["results"].as_array() {
            if !results.is_empty() {
                if let Some(id) = results[0]["id"].as_u64() {
                    return Ok(Some(id as u32));
                }
            }
        }
        
        Ok(None)
    }

    pub async fn create_or_update_device(&self, device: &NetBoxDevice) -> Result<u32, Box<dyn Error>> {
        // Check if device exists by serial number
        if let Some(device_id) = self.find_device_by_serial(&device.serial).await? {
            // Update existing device
            let update_url = format!("{}/api/dcim/devices/{}/", self.base_url, device_id);
            let response = self.client
                .patch(&update_url)
                .header("Authorization", format!("Token {}", self.token))
                .json(&device)
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(format!("Failed to update device: {}", response.status()).into());
            }
            
            Ok(device_id)
        } else {
            // Create new device
            let create_url = format!("{}/api/dcim/devices/", self.base_url);
            let response = self.client
                .post(&create_url)
                .header("Authorization", format!("Token {}", self.token))
                .json(&device)
                .send()
                .await?;
            
            if !response.status().is_success() {
                return Err(format!("Failed to create device: {}", response.status()).into());
            }
            
            let created: serde_json::Value = response.json().await?;
            if let Some(id) = created["id"].as_u64() {
                Ok(id as u32)
            } else {
                Err("Failed to get device ID".into())
            }
        }
    }

    pub async fn create_interface(&self, interface: &NetBoxInterface) -> Result<u32, Box<dyn Error>> {
        let create_url = format!("{}/api/dcim/interfaces/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&interface)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create interface: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get interface ID".into())
        }
    }

    pub async fn create_ip_address(&self, ip: &NetBoxIPAddress) -> Result<u32, Box<dyn Error>> {
        let create_url = format!("{}/api/ipam/ip-addresses/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&ip)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create IP address: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get IP address ID".into())
        }
    }

    pub async fn create_inventory_item(&self, item: &NetBoxInventoryItem) -> Result<u32, Box<dyn Error>> {
        let create_url = format!("{}/api/dcim/inventory-items/", self.base_url);
        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&item)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to create inventory item: {}", response.status()).into());
        }
        
        let created: serde_json::Value = response.json().await?;
        if let Some(id) = created["id"].as_u64() {
            Ok(id as u32)
        } else {
            Err("Failed to get inventory item ID".into())
        }
    }
}

pub async fn sync_to_netbox(
    server_info: &ServerInfo,
    netbox_url: &str,
    token: &str,
    site_name: Option<&str>,
    device_role: Option<&str>,
    skip_tls_verify: bool,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let client = NetBoxClient::new(netbox_url.to_string(), token.to_string(), skip_tls_verify)?;
    
    // Use provided site name or default to "digital-ocean"
    let site = site_name.unwrap_or("Digital Ocean");
    let site_slug = site.to_lowercase().replace(" ", "-");
    let site_id = client.get_or_create_site(site, &site_slug).await?;
    
    // Get or create manufacturer
    let manufacturer = &server_info.summary.system_info.product_manufacturer;
    let manufacturer_id = client.get_or_create_manufacturer(manufacturer).await?;
    
    // Get or create device type with 4U height for Digital Ocean nodes
    let model = &server_info.summary.system_info.product_name;
    let device_type_id = client.get_or_create_device_type(manufacturer_id, model, 4.0).await?;
    
    // Get or create device role
    let role = device_role.unwrap_or("production");
    let device_role_id = client.get_or_create_device_role(role).await?;
    
    // Create custom fields for additional hardware info including BMC
    let mut custom_fields = HashMap::new();
    custom_fields.insert("bios_version".to_string(), serde_json::Value::String(server_info.summary.bios.version.clone()));
    custom_fields.insert("bios_vendor".to_string(), serde_json::Value::String(server_info.summary.bios.vendor.clone()));
    custom_fields.insert("cpu_model".to_string(), serde_json::Value::String(server_info.summary.cpu_topology.cpu_model.clone()));
    custom_fields.insert("cpu_cores".to_string(), serde_json::Value::Number(server_info.summary.cpu_topology.total_cores.into()));
    custom_fields.insert("cpu_threads".to_string(), serde_json::Value::Number(server_info.summary.cpu_topology.total_threads.into()));
    custom_fields.insert("numa_nodes".to_string(), serde_json::Value::Number(server_info.summary.cpu_topology.numa_nodes.into()));
    custom_fields.insert("total_memory".to_string(), serde_json::Value::String(server_info.summary.total_memory.clone()));
    custom_fields.insert("total_storage".to_string(), serde_json::Value::String(server_info.summary.total_storage.clone()));
    custom_fields.insert("rack_height".to_string(), serde_json::Value::String("4U".to_string()));
    
    // Add BMC information if available
    if let Some(bmc_ip) = &server_info.bmc_ip {
        if bmc_ip != "0.0.0.0" {
            custom_fields.insert("bmc_ip".to_string(), serde_json::Value::String(bmc_ip.clone()));
        }
    }
    if let Some(bmc_mac) = &server_info.bmc_mac {
        if bmc_mac != "00:00:00:00:00:00" {
            custom_fields.insert("bmc_mac".to_string(), serde_json::Value::String(bmc_mac.clone()));
        }
    }
    
    // Create or update device - need to build this manually to include BMC fields
    let mut device_data = serde_json::json!({
        "name": server_info.fqdn,
        "device_type": device_type_id,
        "device_role": device_role_id,
        "serial": server_info.summary.chassis.serial,
        "site": site_id,
        "face": "front",
        "status": "active",
        "airflow": "front-to-rear",
        "description": format!("{} @ {}", model, site),
        "comments": format!("Auto-imported by hardware_report\nUUID: {}", server_info.summary.system_info.uuid),
        "custom_fields": custom_fields
    });
    
    // Add BMC information directly to device fields
    if let Some(bmc_ip) = &server_info.bmc_ip {
        if bmc_ip != "0.0.0.0" {
            device_data["oob_ip"] = serde_json::Value::String(bmc_ip.clone());
        }
    }
    
    // Note: NetBox typically doesn't have a direct BMC MAC field on devices
    // The MAC will be associated with the BMC interface we create
    
    let device = NetBoxDevice {
        name: server_info.fqdn.clone(),
        device_type: device_type_id,
        device_role: device_role_id,
        platform: None,
        serial: server_info.summary.chassis.serial.clone(),
        asset_tag: None,
        site: site_id,
        rack: None,
        position: None,
        face: Some("front".to_string()),
        status: "active".to_string(),
        airflow: Some("front-to-rear".to_string()),
        primary_ip4: None, // Will be set after creating IPs
        primary_ip6: None,
        oob_ip: None, // Will be set to BMC IP ID if available
        cluster: None,
        virtual_chassis: None,
        vc_position: None,
        vc_priority: None,
        description: Some(format!("{} @ {}", model, site)),
        comments: Some(format!("Auto-imported by hardware_report\nUUID: {}", server_info.summary.system_info.uuid)),
        config_template: None,
        local_context_data: None,
        tags: None,
        custom_fields: Some(custom_fields),
    };
    
    if dry_run {
        println!("DRY RUN: Would create/update device:");
        println!("{:#?}", device);
        return Ok(());
    }
    
    let device_id = client.create_or_update_device(&device).await?;
    println!("Created/updated device {} (ID: {})", device.name, device_id);
    
    // Create BMC interface first if BMC information is available
    let mut bmc_interface_id = None;
    let mut bmc_ip_id = None;
    if let (Some(bmc_ip), Some(bmc_mac)) = (&server_info.bmc_ip, &server_info.bmc_mac) {
        if bmc_ip != "0.0.0.0" && bmc_mac != "00:00:00:00:00:00" {
            let bmc_interface = NetBoxInterface {
                device: device_id,
                name: "BMC".to_string(),
                type_: "1000base-t".to_string(), // Most BMCs are 1Gb
                enabled: true,
                parent: None,
                bridge: None,
                lag: None,
                mtu: None,
                mac_address: Some(bmc_mac.clone()),
                speed: Some(1_000_000), // 1Gb in Kbps
                duplex: Some("auto".to_string()),
                wwn: None,
                mgmt_only: true, // BMC is always management only
                description: Some("Baseboard Management Controller (IPMI/BMC)".to_string()),
                mode: None,
                rf_role: None,
                rf_channel: None,
                poe_mode: None,
                poe_type: None,
                rf_channel_frequency: None,
                rf_channel_width: None,
                tx_power: None,
                untagged_vlan: None,
                tagged_vlans: None,
                mark_connected: true, // BMC should be connected
                cable: None,
                cable_end: None,
                wireless_link: None,
                link_peers: None,
                link_peers_type: None,
                wireless_lans: None,
                vrf: None,
                tags: None,
                custom_fields: {
                    let mut cf = HashMap::new();
                    cf.insert("interface_type".to_string(), serde_json::Value::String("BMC".to_string()));
                    Some(cf)
                },
            };
            
            bmc_interface_id = Some(client.create_interface(&bmc_interface).await?);
            println!("Created BMC interface (ID: {})", bmc_interface_id.unwrap());
            
            // Create BMC IP address
            let subnet_mask = if bmc_ip.starts_with("10.") {
                "/8"
            } else if bmc_ip.starts_with("172.") {
                "/12"
            } else if bmc_ip.starts_with("192.168.") {
                "/24"
            } else {
                "/24"
            };
            
            let bmc_netbox_ip = NetBoxIPAddress {
                address: format!("{}{}", bmc_ip, subnet_mask),
                vrf: None,
                tenant: None,
                status: "active".to_string(),
                role: Some("vip".to_string()), // BMC IPs are VIPs
                assigned_object_type: Some("dcim.interface".to_string()),
                assigned_object_id: bmc_interface_id,
                nat_inside: None,
                nat_outside: None,
                dns_name: Some(format!("{}-bmc.example.com", server_info.hostname)),
                description: Some("BMC/IPMI Management IP".to_string()),
                comments: None,
                tags: None,
                custom_fields: None,
            };
            
            bmc_ip_id = Some(client.create_ip_address(&bmc_netbox_ip).await?);
            println!("Created BMC IP address {} (ID: {})", bmc_netbox_ip.address, bmc_ip_id.unwrap());
        }
    }
    
    // Create interfaces and IP addresses from network interfaces
    let mut primary_ip4_id = None;
    let mut interface_count = 0;
    
    // Enhanced IP detection - collect all IPs from os_ip field for better coverage
    let mut all_interface_ips: HashMap<String, Vec<String>> = HashMap::new();
    for interface_ip in &server_info.os_ip {
        let interface_name = &interface_ip.interface;
        for ip_addr in &interface_ip.ip_addresses {
            all_interface_ips
                .entry(interface_name.clone())
                .or_insert_with(Vec::new)
                .push(ip_addr.clone());
        }
    }
    
    for nic in &server_info.network.interfaces {
        // Determine if this is a management interface (out-of-band)
        let is_mgmt = nic.name.contains("ilo") || 
                     nic.name.contains("idrac") || 
                     nic.name.contains("ipmi") ||
                     nic.name.contains("bmc") ||
                     nic.name.to_lowercase().contains("mgmt");
        
        // Enhanced interface type detection
        let interface_type = match nic.speed.as_ref().map(|s| s.as_str()) {
            Some(speed) if speed.contains("100000") || speed.contains("100Gb") => "100gbase-x-qsfp28",
            Some(speed) if speed.contains("40000") || speed.contains("40Gb") => "40gbase-x-qsfpp",
            Some(speed) if speed.contains("25000") || speed.contains("25Gb") => "25gbase-x-sfp28",
            Some(speed) if speed.contains("10000") || speed.contains("10Gb") => {
                if nic.model.to_lowercase().contains("sfp") {
                    "10gbase-x-sfpp"
                } else {
                    "10gbase-t"
                }
            },
            Some(speed) if speed.contains("1000") || speed.contains("1Gb") => "1000base-t",
            Some(speed) if speed.contains("100") => "100base-tx",
            _ => "other",
        };
        
        let interface = NetBoxInterface {
            device: device_id,
            name: nic.name.clone(),
            type_: interface_type.to_string(),
            enabled: true,
            parent: None,
            bridge: None,
            lag: None,
            mtu: None,
            mac_address: if nic.mac != "00:00:00:00:00:00" && nic.mac != "Unknown" { 
                Some(nic.mac.clone()) 
            } else { 
                None 
            },
            speed: nic.speed.as_ref().and_then(|s| {
                // Parse various speed formats
                if s.contains("Gb/s") {
                    s.trim_end_matches("Gb/s").parse::<u32>().ok().map(|v| v * 1_000_000)
                } else if s.contains("Mb/s") {
                    s.trim_end_matches("Mb/s").parse::<u32>().ok().map(|v| v * 1_000)
                } else {
                    None
                }
            }),
            duplex: Some("auto".to_string()),
            wwn: None,
            mgmt_only: is_mgmt,
            description: Some(format!("{} {} - PCI: {}", nic.vendor, nic.model, nic.pci_id)),
            mode: None,
            rf_role: None,
            rf_channel: None,
            poe_mode: None,
            poe_type: None,
            rf_channel_frequency: None,
            rf_channel_width: None,
            tx_power: None,
            untagged_vlan: None,
            tagged_vlans: None,
            mark_connected: !is_mgmt, // Assume production interfaces are connected
            cable: None,
            cable_end: None,
            wireless_link: None,
            link_peers: None,
            link_peers_type: None,
            wireless_lans: None,
            vrf: None,
            tags: None,
            custom_fields: {
                let mut cf = HashMap::new();
                if let Some(numa) = nic.numa_node {
                    cf.insert("numa_node".to_string(), serde_json::Value::Number(numa.into()));
                }
                if cf.is_empty() { None } else { Some(cf) }
            },
        };
        
        let interface_id = client.create_interface(&interface).await?;
        println!("Created interface {} (ID: {})", interface.name, interface_id);
        interface_count += 1;
        
        // Create IP addresses for this interface - use enhanced IP collection
        let interface_ips = all_interface_ips.get(&nic.name)
            .cloned()
            .unwrap_or_else(|| vec![nic.ip.clone()]); // Fallback to single IP
        
        for ip in &interface_ips {
            if ip != "127.0.0.1" && !ip.starts_with("::") && !ip.starts_with("fe80:") && ip != "Unknown" && !ip.is_empty() {
                // Detect Tailscale interfaces
                let is_tailscale = nic.name.contains("tailscale") || 
                                 nic.name.contains("ts") ||
                                 nic.name == "tailscale0" ||
                                 // Check if IP is in Tailscale CGNAT range (100.64.0.0/10)
                                 (ip.starts_with("100.") && {
                                     if let Ok(ip_parts) = ip.split('.').take(2).collect::<Vec<_>>()[1].parse::<u8>() {
                                         ip_parts >= 64 && ip_parts <= 127
                                     } else {
                                         false
                                     }
                                 });
                
                // Determine subnet mask based on IP class and common patterns
                let subnet_mask = if ip.starts_with("10.") {
                    "/8"  // Private Class A
                } else if ip.starts_with("172.") {
                    "/12" // Private Class B  
                } else if ip.starts_with("192.168.") {
                    "/24" // Private Class C
                } else if ip.starts_with("169.254.") {
                    "/16" // Link-local
                } else if is_tailscale {
                    "/32" // Tailscale IPs are typically /32
                } else {
                    "/24" // Default assumption
                };
                
                // Determine IP role and priority
                let ip_role = if is_mgmt {
                    Some("vip".to_string()) // Management/OOB IPs are VIPs
                } else if is_tailscale {
                    Some("anycast".to_string()) // Tailscale is overlay/anycast
                } else if nic.name.starts_with("eth0") || nic.name.starts_with("eno1") || 
                         nic.name.starts_with("enp") || interface_count == 1 {
                    Some("loopback".to_string()) // Primary interface
                } else {
                    Some("secondary".to_string()) // Additional interfaces
                };
                
                let description = if is_mgmt {
                    "Out-of-band Management IP".to_string()
                } else if is_tailscale {
                    "Tailscale VPN IP".to_string()
                } else {
                    "Primary Network IP".to_string()
                };
                
                let netbox_ip = NetBoxIPAddress {
                    address: format!("{}{}", ip, subnet_mask),
                    vrf: None,
                    tenant: None,
                    status: "active".to_string(),
                    role: ip_role,
                    assigned_object_type: Some("dcim.interface".to_string()),
                    assigned_object_id: Some(interface_id),
                    nat_inside: None,
                    nat_outside: None,
                    dns_name: if !is_mgmt { 
                        Some(server_info.fqdn.clone()) 
                    } else { 
                        Some(format!("{}-{}.example.com", server_info.hostname, 
                            if is_tailscale { "ts" } else { "mgmt" }))
                    },
                    description: Some(description),
                    comments: if is_tailscale { 
                        Some("Tailscale mesh VPN address".to_string()) 
                    } else { 
                        None 
                    },
                    tags: None,
                    custom_fields: {
                        let mut cf = HashMap::new();
                        if is_tailscale {
                            cf.insert("network_type".to_string(), serde_json::Value::String("Tailscale VPN".to_string()));
                        }
                        if cf.is_empty() { None } else { Some(cf) }
                    },
                };
                
                let ip_id = client.create_ip_address(&netbox_ip).await?;
                println!("Created IP address {} (ID: {}) - {}", 
                    netbox_ip.address, ip_id, 
                    if is_tailscale { "Tailscale" } else if is_mgmt { "Management" } else { "Primary" }
                );
                
                // Set as primary IP with proper priority:
                // 1. Tailscale IPs have highest priority for primary IP
                // 2. Then primary interfaces (eth0, eno1, etc.)
                // 3. Skip management interfaces for primary IP
                if !is_mgmt && (
                    (is_tailscale && primary_ip4_id.is_none()) ||
                    (primary_ip4_id.is_none() && (
                        nic.name.starts_with("eth0") || 
                        nic.name.starts_with("eno1") || 
                        nic.name.starts_with("enp") ||
                        interface_count == 1
                    ))
                ) {
                    primary_ip4_id = Some(ip_id);
                    println!("Set as primary IP: {} ({})", ip, if is_tailscale { "Tailscale" } else { "Standard" });
                }
            }
        }
    }
    
    // Update device with primary IP and BMC information if found
    let mut update_payload = serde_json::json!({});
    
    if let Some(primary_ip) = primary_ip4_id {
        update_payload["primary_ip4"] = serde_json::Value::Number(primary_ip.into());
        println!("Setting primary IPv4 to ID: {}", primary_ip);
    }
    
    // Set BMC IP as out-of-band IP if we created one
    if let Some(bmc_ip_ref) = bmc_ip_id {
        update_payload["oob_ip"] = serde_json::Value::Number(bmc_ip_ref.into());
        println!("Setting out-of-band IP to BMC IP ID: {}", bmc_ip_ref);
    }
    
    // Only update if we have changes to make
    if !update_payload.as_object().unwrap().is_empty() {
        let update_url = format!("{}/api/dcim/devices/{}/", client.base_url, device_id);
        let response = client.client
            .patch(&update_url)
            .header("Authorization", format!("Token {}", client.token))
            .json(&update_payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("Successfully updated device with IP assignments");
        } else {
            println!("Warning: Failed to update device IP assignments: {}", response.status());
        }
    }
    
    // Create inventory items for components
    
    // CPU inventory items - create one per socket
    for socket in 0..server_info.summary.cpu_topology.sockets {
        let cpu_item = NetBoxInventoryItem {
            device: device_id,
            parent: None,
            name: format!("CPU-Socket-{}", socket),
            label: Some(format!("Socket {}", socket)),
            role: None,
            manufacturer: {
                // Try to extract CPU manufacturer from model
                let cpu_model = &server_info.summary.cpu_topology.cpu_model;
                if cpu_model.to_lowercase().contains("intel") {
                    client.get_or_create_manufacturer("Intel Corporation").await.ok()
                } else if cpu_model.to_lowercase().contains("amd") {
                    client.get_or_create_manufacturer("Advanced Micro Devices").await.ok()
                } else {
                    Some(manufacturer_id)
                }
            },
            part_id: Some(server_info.summary.cpu_topology.cpu_model.clone()),
            serial: None,
            asset_tag: None,
            discovered: true,
            description: Some(format!(
                "{} - {} cores, {} threads per socket",
                server_info.summary.cpu_topology.cpu_model,
                server_info.summary.cpu_topology.cores_per_socket,
                server_info.summary.cpu_topology.cores_per_socket * server_info.summary.cpu_topology.threads_per_core
            )),
            component_type: None,
            component_id: None,
            tags: None,
            custom_fields: {
                let mut cf = HashMap::new();
                cf.insert("cores_per_socket".to_string(), serde_json::Value::Number(server_info.summary.cpu_topology.cores_per_socket.into()));
                cf.insert("threads_per_core".to_string(), serde_json::Value::Number(server_info.summary.cpu_topology.threads_per_core.into()));
                cf.insert("numa_nodes".to_string(), serde_json::Value::Number(server_info.summary.cpu_topology.numa_nodes.into()));
                Some(cf)
            },
        };
        let cpu_item_id = client.create_inventory_item(&cpu_item).await?;
        println!("Created CPU inventory item: Socket {} (ID: {})", socket, cpu_item_id);
    }
    
    // Memory inventory items - enhanced with detailed info
    for dimm in &server_info.hardware.memory.modules {
        let mem_manufacturer_id = if dimm.manufacturer != "Unknown" && !dimm.manufacturer.is_empty() {
            client.get_or_create_manufacturer(&dimm.manufacturer).await.unwrap_or(manufacturer_id)
        } else {
            manufacturer_id
        };
        
        let mem_item = NetBoxInventoryItem {
            device: device_id,
            parent: None,
            name: format!("Memory-{}", dimm.location),
            label: Some(dimm.location.clone()),
            role: None,
            manufacturer: Some(mem_manufacturer_id),
            part_id: None, // MemoryModule doesn't have part_number field
            serial: Some(dimm.serial.clone()),
            asset_tag: None,
            discovered: true,
            description: Some(format!(
                "{} {} @ {} - {}",
                dimm.size,
                dimm.type_,
                dimm.speed,
                dimm.location
            )),
            component_type: None,
            component_id: None,
            tags: None,
            custom_fields: {
                let mut cf = HashMap::new();
                cf.insert("memory_type".to_string(), serde_json::Value::String(dimm.type_.clone()));
                cf.insert("memory_speed".to_string(), serde_json::Value::String(dimm.speed.clone()));
                Some(cf)
            },
        };
        let mem_item_id = client.create_inventory_item(&mem_item).await?;
        println!("Created memory inventory item: {} (ID: {})", dimm.location, mem_item_id);
    }
    
    // Storage inventory items - enhanced with more details
    for disk in &server_info.hardware.storage.devices {
        // StorageDevice only has name, type_, size, model fields
        let storage_manufacturer_id = manufacturer_id; // Use system manufacturer as fallback
        
        let storage_item = NetBoxInventoryItem {
            device: device_id,
            parent: None,
            name: format!("Disk-{}", disk.name),
            label: Some(disk.name.clone()),
            role: None,
            manufacturer: Some(storage_manufacturer_id),
            part_id: Some(disk.model.clone()),
            serial: None, // Not available in current StorageDevice struct
            asset_tag: None,
            discovered: true,
            description: Some(format!("{} {} - {}", disk.model, disk.size, disk.type_)),
            component_type: None,
            component_id: None,
            tags: None,
            custom_fields: {
                let mut cf = HashMap::new();
                cf.insert("interface_type".to_string(), serde_json::Value::String(disk.type_.clone()));
                cf.insert("capacity".to_string(), serde_json::Value::String(disk.size.clone()));
                Some(cf)
            },
        };
        let storage_item_id = client.create_inventory_item(&storage_item).await?;
        println!("Created storage inventory item: {} (ID: {})", disk.name, storage_item_id);
    }
    
    // GPU inventory items - enhanced with detailed info
    for (i, gpu) in server_info.hardware.gpus.devices.iter().enumerate() {
        let gpu_manufacturer_id = if gpu.vendor != "Unknown" && !gpu.vendor.is_empty() {
            client.get_or_create_manufacturer(&gpu.vendor).await.unwrap_or(manufacturer_id)
        } else {
            manufacturer_id
        };
        
        let gpu_item = NetBoxInventoryItem {
            device: device_id,
            parent: None,
            name: format!("GPU-{}", i + 1),
            label: Some(gpu.name.clone()),
            role: None,
            manufacturer: Some(gpu_manufacturer_id),
            part_id: Some(gpu.pci_id.clone()),
            serial: None,
            asset_tag: None,
            discovered: true,
            description: Some(format!("{} {} - {} Memory", gpu.vendor, gpu.name, gpu.memory)),
            component_type: None,
            component_id: None,
            tags: None,
            custom_fields: {
                let mut cf = HashMap::new();
                cf.insert("memory_size".to_string(), serde_json::Value::String(gpu.memory.clone()));
                cf.insert("pci_id".to_string(), serde_json::Value::String(gpu.pci_id.clone()));
                if let Some(numa) = gpu.numa_node {
                    cf.insert("numa_node".to_string(), serde_json::Value::Number(numa.into()));
                }
                Some(cf)
            },
        };
        let gpu_item_id = client.create_inventory_item(&gpu_item).await?;
        println!("Created GPU inventory item: {} (ID: {})", gpu.name, gpu_item_id);
    }
    
    // Motherboard inventory item
    let mb_item = NetBoxInventoryItem {
        device: device_id,
        parent: None,
        name: "Motherboard".to_string(),
        label: Some("System Board".to_string()),
        role: None,
        manufacturer: Some(manufacturer_id),
        part_id: Some(server_info.summary.motherboard.product_name.clone()),
        serial: Some(server_info.summary.motherboard.serial.clone()),
        asset_tag: None,
        discovered: true,
        description: Some(format!(
            "{} {} v{} - {}",
            server_info.summary.motherboard.manufacturer,
            server_info.summary.motherboard.product_name,
            server_info.summary.motherboard.version,
            server_info.summary.motherboard.type_
        )),
        component_type: None,
        component_id: None,
        tags: None,
        custom_fields: {
            let mut cf = HashMap::new();
            cf.insert("version".to_string(), serde_json::Value::String(server_info.summary.motherboard.version.clone()));
            cf.insert("location".to_string(), serde_json::Value::String(server_info.summary.motherboard.location.clone()));
            Some(cf)
        },
    };
    let mb_item_id = client.create_inventory_item(&mb_item).await?;
    println!("Created motherboard inventory item (ID: {})", mb_item_id);
    
    Ok(())
}