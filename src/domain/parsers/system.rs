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

//! System information parsing functions

use super::common::{clean_value, extract_dmidecode_value};
use crate::domain::{BiosInfo, ChassisInfo, SystemInfo};

/// Parse system information from dmidecode output
pub fn parse_dmidecode_system_info(dmidecode_output: &str) -> Result<SystemInfo, String> {
    let uuid =
        extract_dmidecode_value(dmidecode_output, "UUID").unwrap_or_else(|_| "Unknown".to_string());
    let serial = extract_dmidecode_value(dmidecode_output, "Serial Number")
        .unwrap_or_else(|_| "Unknown".to_string());
    let product_name = extract_dmidecode_value(dmidecode_output, "Product Name")
        .unwrap_or_else(|_| "Unknown".to_string());
    let manufacturer = extract_dmidecode_value(dmidecode_output, "Manufacturer")
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(SystemInfo {
        uuid: clean_value(&uuid),
        serial: clean_value(&serial),
        product_name: clean_value(&product_name),
        product_manufacturer: clean_value(&manufacturer),
    })
}

/// Parse BIOS information from dmidecode output
pub fn parse_dmidecode_bios_info(dmidecode_output: &str) -> Result<BiosInfo, String> {
    let vendor = extract_dmidecode_value(dmidecode_output, "Vendor")
        .unwrap_or_else(|_| "Unknown".to_string());
    let version = extract_dmidecode_value(dmidecode_output, "Version")
        .unwrap_or_else(|_| "Unknown".to_string());
    let release_date = extract_dmidecode_value(dmidecode_output, "Release Date")
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(BiosInfo {
        vendor: clean_value(&vendor),
        version: clean_value(&version),
        release_date: clean_value(&release_date),
        firmware_version: version.clone(),
    })
}

/// Parse chassis information from dmidecode output
pub fn parse_dmidecode_chassis_info(dmidecode_output: &str) -> Result<ChassisInfo, String> {
    let manufacturer = extract_dmidecode_value(dmidecode_output, "Manufacturer")
        .unwrap_or_else(|_| "Unknown".to_string());
    let type_ =
        extract_dmidecode_value(dmidecode_output, "Type").unwrap_or_else(|_| "Unknown".to_string());
    let serial = extract_dmidecode_value(dmidecode_output, "Serial Number")
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(ChassisInfo {
        manufacturer: clean_value(&manufacturer),
        type_: clean_value(&type_),
        serial: clean_value(&serial),
    })
}

/// Parse hostname from hostname command output
pub fn parse_hostname_output(hostname_output: &str) -> Result<String, String> {
    Ok(clean_value(hostname_output.trim()))
}
