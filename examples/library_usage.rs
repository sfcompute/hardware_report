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

//! Example of using the hardware_report library
//! 
//! This demonstrates both the new Ports & Adapters API and the legacy compatibility.

use hardware_report::{ServerInfo, new_domain::HardwareReport, ReportConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hardware Report Library Usage Examples");
    println!("======================================");
    
    // Example 1: Legacy API (backward compatibility)
    println!("\n1. Using Legacy API:");
    match ServerInfo::collect() {
        Ok(server_info) => {
            println!("   Hostname: {}", server_info.hostname);
            println!("   CPU: {}", server_info.summary.cpu_summary);
            println!("   Memory: {}", server_info.summary.total_memory);
            println!("   Storage: {}", server_info.summary.total_storage);
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    
    // Example 2: New Domain Types (can convert between old and new)
    println!("\n2. Type Conversion:");
    match ServerInfo::collect() {
        Ok(legacy_info) => {
            // Convert legacy to new format
            let new_report: HardwareReport = legacy_info.into();
            println!("   Converted to new format:");
            println!("   System: {} ({})", new_report.hostname, new_report.summary.system_info.product_name);
            println!("   CPUs: {}", new_report.summary.total_gpus);
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    
    // Example 3: New Configuration Options
    println!("\n3. New Configuration API:");
    let config = ReportConfig {
        include_sensitive: false,
        skip_sudo: true,
        command_timeout: 10,
        verbose: false,
    };
    println!("   Created config with timeout: {} seconds", config.command_timeout);
    
    // Example 4: Pure parsing functions
    println!("\n4. Pure Parsing Functions:");
    use hardware_report::new_domain::parsers::cpu::parse_lscpu_output;
    
    let sample_lscpu = r#"Model name:                      Intel(R) Core(TM) i7-10875H CPU @ 2.30GHz
CPU(s):                          16
Thread(s) per core:              2
Core(s) per socket:              8
Socket(s):                       1
CPU MHz:                         2300.000"#;
    
    match parse_lscpu_output(sample_lscpu) {
        Ok(cpu_info) => {
            println!("   Parsed CPU: {}", cpu_info.model);
            println!("   Cores: {}, Threads: {}, Sockets: {}", cpu_info.cores, cpu_info.threads, cpu_info.sockets);
        }
        Err(e) => {
            println!("   Parse error: {}", e);
        }
    }
    
    println!("\n✅ All examples completed successfully!");
    println!("\nNote: The new service factory will be available in Phase 4");
    println!("      For now, use ServerInfo::collect() for actual hardware detection");
    
    Ok(())
}