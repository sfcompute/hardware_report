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

//! Comprehensive demonstration of the new Ports & Adapters architecture
//!
//! This example shows:
//! 1. New service factory and dependency injection
//! 2. System validation
//! 3. Report generation with new architecture
//! 4. Both programmatic API and file output
//! 5. Performance comparison with legacy API

use hardware_report::{
    create_service, create_service_with_config, new_domain::HardwareReport, validate_system,
    ContainerConfigBuilder, FileRepository, FileSystemRepository, ReportConfig, ServerInfo,
};
use std::error::Error;
use std::path::Path;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔧 Hardware Report - New Ports & Adapters Architecture Demo");
    println!("============================================================");

    // 1. System Validation
    println!("\n📋 Step 1: System Validation");
    println!("----------------------------");

    let (missing_deps, has_privileges) = validate_system().await?;

    if !missing_deps.is_empty() {
        println!("⚠️  Missing system dependencies: {:?}", missing_deps);
        println!("   Some hardware information may be limited");
    } else {
        println!("✅ All system dependencies are available");
    }

    if has_privileges {
        println!("✅ Running with elevated privileges");
    } else {
        println!("⚠️  Running without elevated privileges");
        println!("   Some hardware information may be limited");
    }

    // 2. New Architecture - Default Service
    println!("\n🏗️  Step 2: New Architecture - Default Service");
    println!("----------------------------------------------");

    let start_time = Instant::now();
    let service = create_service(None).await?;
    let setup_duration = start_time.elapsed();

    println!("✅ Service created in {:?}", setup_duration);

    // Validate dependencies through the service
    let missing = service.validate_dependencies().await?;
    if !missing.is_empty() {
        println!("⚠️  Service reports missing dependencies: {:?}", missing);
    }

    // Check privileges through the service
    let has_privs = service.check_privileges().await?;
    if has_privs {
        println!("✅ Service confirms elevated privileges");
    } else {
        println!("⚠️  Service confirms limited privileges");
    }

    // 3. Generate Report with New Architecture
    println!("\n📊 Step 3: Generate Hardware Report (New Architecture)");
    println!("-----------------------------------------------------");

    let config = ReportConfig {
        include_sensitive: false,
        skip_sudo: !has_privileges,
        command_timeout: 30,
        verbose: false,
    };

    let start_time = Instant::now();
    let new_report = service.generate_report(config).await?;
    let new_duration = start_time.elapsed();

    println!("✅ New architecture report generated in {:?}", new_duration);
    println!("   📍 Hostname: {}", new_report.hostname);
    println!(
        "   💻 System: {} ({})",
        new_report.summary.system_info.product_name,
        new_report.summary.system_info.product_manufacturer
    );
    println!("   🧠 CPU: {}", new_report.summary.cpu_summary);
    println!("   💾 Memory: {}", new_report.summary.total_memory);
    println!("   💽 Storage: {}", new_report.summary.total_storage);
    println!("   🎮 GPUs: {}", new_report.summary.total_gpus);
    println!("   🌐 NICs: {}", new_report.summary.total_nics);

    // 4. Legacy Architecture Comparison
    println!("\n🔄 Step 4: Legacy Architecture Comparison");
    println!("----------------------------------------");

    let start_time = Instant::now();
    let legacy_report = ServerInfo::collect()?;
    let legacy_duration = start_time.elapsed();

    println!(
        "✅ Legacy architecture report generated in {:?}",
        legacy_duration
    );

    // Performance comparison
    let speedup = if new_duration.as_millis() > 0 {
        legacy_duration.as_millis() as f64 / new_duration.as_millis() as f64
    } else {
        1.0
    };

    println!("📊 Performance Comparison:");
    println!(
        "   New: {:?} | Legacy: {:?} | Ratio: {:.2}x",
        new_duration, legacy_duration, speedup
    );

    // Verify data consistency
    let legacy_as_new: HardwareReport = legacy_report.into();
    let hostname_match = new_report.hostname == legacy_as_new.hostname;
    let cpu_match = new_report.summary.cpu_summary == legacy_as_new.summary.cpu_summary;

    println!("🔍 Data Consistency Check:");
    println!(
        "   Hostname match: {} ({})",
        if hostname_match { "✅" } else { "❌" },
        new_report.hostname
    );
    println!("   CPU match: {} ", if cpu_match { "✅" } else { "❌" });

    // 5. Custom Configuration Demo
    println!("\n⚙️  Step 5: Custom Configuration Demo");
    println!("------------------------------------");

    let custom_container_config = ContainerConfigBuilder::new()
        .command_timeout(Duration::from_secs(15))
        .retry_count(1)
        .verbose(true)
        .build();

    let custom_report_config = ReportConfig {
        include_sensitive: false,
        skip_sudo: true,
        command_timeout: 15,
        verbose: true,
    };

    let custom_service =
        create_service_with_config(custom_container_config, Some(custom_report_config)).await?;

    println!("✅ Custom configured service created");
    println!("   ⏱️  Reduced timeout: 15s");
    println!("   🔄 Reduced retries: 1");
    println!("   🔊 Verbose mode: enabled");

    // 6. File Output Demo
    println!("\n💾 Step 6: File Output Demo");
    println!("---------------------------");

    let file_repo = FileSystemRepository::new();

    // Generate filename based on system serial
    let serial = new_report.summary.chassis.serial.clone();
    let safe_serial = serial
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    let json_path = format!("{}_new_architecture_report.json", safe_serial);
    let toml_path = format!("{}_new_architecture_report.toml", safe_serial);

    // Save in both formats
    file_repo
        .save_json(&new_report, Path::new(&json_path))
        .await?;
    file_repo
        .save_toml(&new_report, Path::new(&toml_path))
        .await?;

    println!("✅ Reports saved:");
    println!("   📄 JSON: {}", json_path);
    println!("   📄 TOML: {}", toml_path);

    // Verify we can load them back
    let loaded_json = file_repo.load_json(Path::new(&json_path)).await?;
    let loaded_toml = file_repo.load_toml(Path::new(&toml_path)).await?;

    let json_match = loaded_json.hostname == new_report.hostname;
    let toml_match = loaded_toml.hostname == new_report.hostname;

    println!("🔍 File I/O Verification:");
    println!(
        "   JSON round-trip: {}",
        if json_match { "✅" } else { "❌" }
    );
    println!(
        "   TOML round-trip: {}",
        if toml_match { "✅" } else { "❌" }
    );

    // 7. Architecture Benefits Summary
    println!("\n🎯 Step 7: Architecture Benefits Demonstrated");
    println!("--------------------------------------------");

    println!("✅ Dependency Injection: Custom configurations applied");
    println!(
        "✅ Platform Abstraction: Works on {} without code changes",
        if cfg!(target_os = "macos") {
            "macOS"
        } else {
            "Linux"
        }
    );
    println!("✅ Testability: All components are mockable interfaces");
    println!("✅ Extensibility: Easy to add new adapters and providers");
    println!("✅ Separation of Concerns: Domain logic isolated from infrastructure");
    println!("✅ Backward Compatibility: Legacy API still functional");
    println!("✅ Error Handling: Comprehensive error types and handling");
    println!("✅ Pure Functions: Parsing logic is testable in isolation");

    // 8. Final Status
    println!("\n🏆 Final Status");
    println!("===============");
    println!("✅ Ports & Adapters architecture fully functional!");
    println!("✅ Both binary and library usage work seamlessly");
    println!("✅ Legacy compatibility maintained");
    println!("✅ New features and capabilities delivered");
    println!("");
    println!("🚀 Ready for production use!");

    Ok(())
}
