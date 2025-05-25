use hardware_report::ServerInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect hardware information
    let server_info = ServerInfo::collect()?;
    
    // Example 1: Calculate total memory capacity in GB
    let total_memory_str = &server_info.summary.total_memory;
    println!("Total Memory: {}", total_memory_str);
    
    // Example 2: Count memory modules by type
    let mut memory_types = std::collections::HashMap::new();
    for module in &server_info.hardware.memory.modules {
        *memory_types.entry(&module.type_).or_insert(0) += 1;
    }
    println!("\nMemory Module Types:");
    for (mem_type, count) in memory_types {
        println!("- {}: {} modules", mem_type, count);
    }
    
    // Example 3: Analyze NUMA topology
    println!("\nNUMA Analysis:");
    for (node_id, node) in &server_info.summary.numa_topology {
        println!("Node {}: {} CPUs, Memory: {}", 
            node_id, 
            node.cpus.len(),
            node.memory
        );
        
        // Find devices attached to this NUMA node
        let gpu_count = server_info.hardware.gpus.devices.iter()
            .filter(|gpu| gpu.numa_node == Some(node.id))
            .count();
        let nic_count = server_info.network.interfaces.iter()
            .filter(|nic| nic.numa_node == Some(node.id))
            .count();
            
        if gpu_count > 0 || nic_count > 0 {
            println!("  Attached devices: {} GPUs, {} NICs", gpu_count, nic_count);
        }
    }
    
    // Example 4: Storage analysis
    println!("\nStorage Devices:");
    for device in &server_info.hardware.storage.devices {
        println!("- {} ({}): {}", device.name, device.type_, device.size);
    }
    println!("Total Storage Capacity: {:.2} TB", server_info.summary.total_storage_tb);
    
    // Example 5: Network interface speed analysis
    let mut total_bandwidth = 0u64;
    println!("\nNetwork Interface Speeds:");
    for interface in &server_info.network.interfaces {
        if let Some(speed) = &interface.speed {
            println!("- {}: {}", interface.name, speed);
            // Parse speed (assuming format like "10000Mb/s")
            if let Some(num_str) = speed.split("Mb/s").next() {
                if let Ok(mbps) = num_str.parse::<u64>() {
                    total_bandwidth += mbps;
                }
            }
        }
    }
    if total_bandwidth > 0 {
        println!("Total Network Bandwidth: {} Mb/s ({} Gb/s)", 
            total_bandwidth, 
            total_bandwidth / 1000
        );
    }
    
    Ok(())
}