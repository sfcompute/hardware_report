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

use std::fmt;

/// Domain-level errors that don't expose infrastructure details
#[derive(Debug, Clone)]
pub enum DomainError {
    /// Hardware information collection failed
    HardwareCollectionFailed(String),
    /// System information unavailable
    SystemInfoUnavailable(String),
    /// Insufficient privileges to collect information
    InsufficientPrivileges(String),
    /// Invalid configuration provided
    InvalidConfiguration(String),
    /// Required system dependencies missing
    MissingDependencies(Vec<String>),
    /// Data parsing failed
    ParsingFailed(String),
    /// Operation timed out
    Timeout(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::HardwareCollectionFailed(msg) => {
                write!(f, "Hardware collection failed: {}", msg)
            }
            DomainError::SystemInfoUnavailable(msg) => {
                write!(f, "System information unavailable: {}", msg)
            }
            DomainError::InsufficientPrivileges(msg) => {
                write!(f, "Insufficient privileges: {}", msg)
            }
            DomainError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            DomainError::MissingDependencies(deps) => {
                write!(f, "Missing required dependencies: {}", deps.join(", "))
            }
            DomainError::ParsingFailed(msg) => {
                write!(f, "Data parsing failed: {}", msg)
            }
            DomainError::Timeout(msg) => {
                write!(f, "Operation timed out: {}", msg)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Errors specific to hardware reporting service
#[derive(Debug, Clone)]
pub enum ReportError {
    /// Domain operation failed
    Domain(DomainError),
    /// Report generation failed
    GenerationFailed(String),
    /// Report validation failed
    ValidationFailed(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportError::Domain(err) => write!(f, "{}", err),
            ReportError::GenerationFailed(msg) => write!(f, "Report generation failed: {}", msg),
            ReportError::ValidationFailed(msg) => write!(f, "Report validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ReportError {}

impl From<DomainError> for ReportError {
    fn from(err: DomainError) -> Self {
        ReportError::Domain(err)
    }
}

/// Errors specific to publishing reports
#[derive(Debug, Clone)]
pub enum PublishError {
    /// Domain operation failed
    Domain(DomainError),
    /// Network/HTTP operation failed
    NetworkFailed(String),
    /// Authentication failed
    AuthenticationFailed(String),
    /// Serialization failed
    SerializationFailed(String),
}

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublishError::Domain(err) => write!(f, "{}", err),
            PublishError::NetworkFailed(msg) => write!(f, "Network operation failed: {}", msg),
            PublishError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            PublishError::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
        }
    }
}

impl std::error::Error for PublishError {}

impl From<DomainError> for PublishError {
    fn from(err: DomainError) -> Self {
        PublishError::Domain(err)
    }
}

/// System-level errors for adapters (not exposed to domain)
#[derive(Debug, Clone)]
pub enum SystemError {
    /// Command execution failed
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    /// Command not found
    CommandNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// I/O operation failed
    IoError(String),
    /// Parsing error
    ParseError(String),
    /// Timeout
    Timeout(String),
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::CommandFailed { command, exit_code, stderr } => {
                write!(f, "Command '{}' failed", command)?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if !stderr.is_empty() {
                    write!(f, ": {}", stderr)?;
                }
                Ok(())
            }
            SystemError::CommandNotFound(cmd) => write!(f, "Command not found: {}", cmd),
            SystemError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            SystemError::IoError(msg) => write!(f, "I/O error: {}", msg),
            SystemError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SystemError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for SystemError {}

/// Convert system errors to domain errors (with context loss for abstraction)
impl From<SystemError> for DomainError {
    fn from(err: SystemError) -> Self {
        match err {
            SystemError::CommandFailed { command, .. } => {
                DomainError::HardwareCollectionFailed(format!("System command failed: {}", command))
            }
            SystemError::CommandNotFound(cmd) => {
                DomainError::MissingDependencies(vec![cmd])
            }
            SystemError::PermissionDenied(_) => {
                DomainError::InsufficientPrivileges("System access denied".to_string())
            }
            SystemError::IoError(msg) => {
                DomainError::SystemInfoUnavailable(format!("I/O error: {}", msg))
            }
            SystemError::ParseError(msg) => {
                DomainError::ParsingFailed(msg)
            }
            SystemError::Timeout(msg) => {
                DomainError::Timeout(msg)
            }
        }
    }
}

/// Command execution errors
#[derive(Debug, Clone)]
pub enum CommandError {
    /// System error occurred
    System(SystemError),
    /// Command execution failed
    ExecutionFailed(String),
    /// Invalid command arguments
    InvalidArguments(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::System(err) => write!(f, "{}", err),
            CommandError::ExecutionFailed(msg) => write!(f, "Command execution failed: {}", msg),
            CommandError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<SystemError> for CommandError {
    fn from(err: SystemError) -> Self {
        CommandError::System(err)
    }
}

impl From<CommandError> for DomainError {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::System(sys_err) => sys_err.into(),
            CommandError::ExecutionFailed(msg) => {
                DomainError::SystemInfoUnavailable(format!("Command execution failed: {}", msg))
            }
            CommandError::InvalidArguments(msg) => {
                DomainError::InvalidConfiguration(format!("Invalid command arguments: {}", msg))
            }
        }
    }
}