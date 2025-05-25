use hardware_report::ServerInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect hardware information
    println!("Collecting hardware information...");
    let server_info = ServerInfo::collect()?;

    // Display basic information
    println!("\n=== System Information ===");
    println!("Hostname: {}", server_info.hostname);
    println!("FQDN: {}", server_info.fqdn);
    println!("System UUID: {}", server_info.summary.system_info.uuid);
    println!("CPU Model: {}", server_info.summary.cpu_topology.cpu_model);
    println!("Total Memory: {}", server_info.summary.total_memory);
    println!(
        "Total Storage: {} ({:.2} TB)",
        server_info.summary.total_storage, server_info.summary.total_storage_tb
    );

    // Example: Access detailed hardware information
    println!("\n=== Network Interfaces ===");
    for interface in &server_info.network.interfaces {
        println!("- {} ({}): {}", interface.name, interface.mac, interface.ip);
    }

    // Example: Access GPU information
    if !server_info.hardware.gpus.devices.is_empty() {
        println!("\n=== GPUs ===");
        for gpu in &server_info.hardware.gpus.devices {
            println!("- {}: {}", gpu.name, gpu.memory);
        }
    }

    // Example: Save to JSON
    let json_output = serde_json::to_string_pretty(&server_info)?;
    std::fs::write("hardware_report.json", json_output)?;
    println!("\nSaved report to hardware_report.json");

    // Example: Post to remote server (commented out by default)
    /*
    let labels = HashMap::from([
        ("environment".to_string(), "production".to_string()),
        ("datacenter".to_string(), "us-east-1".to_string()),
    ]);

    post_data(
        server_info,
        labels,
        "https://api.example.com/hardware",
        Some("your-auth-token"),
        None,
        false,
    ).await?;
    */

    Ok(())
}
