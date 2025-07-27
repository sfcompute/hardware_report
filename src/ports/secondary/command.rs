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

use crate::domain::CommandError;
use async_trait::async_trait;
use std::time::Duration;

/// Represents a system command to be executed
#[derive(Debug, Clone)]
pub struct SystemCommand {
    /// Command program name
    pub program: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory (optional)
    pub working_dir: Option<String>,
    /// Environment variables (optional)
    pub env_vars: Option<Vec<(String, String)>>,
    /// Execution timeout
    pub timeout: Option<Duration>,
    /// Whether to use sudo for privilege escalation
    pub use_sudo: bool,
}

impl SystemCommand {
    /// Create a new system command
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: Vec::new(),
            working_dir: None,
            env_vars: None,
            timeout: None,
            use_sudo: false,
        }
    }

    /// Add arguments to the command
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }

    /// Add environment variables
    pub fn env_vars(mut self, vars: Vec<(&str, &str)>) -> Self {
        self.env_vars = Some(
            vars.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        self
    }

    /// Set execution timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Enable sudo for privilege escalation
    pub fn with_sudo(mut self) -> Self {
        self.use_sudo = true;
        self
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit status code
    pub exit_code: Option<i32>,
    /// Whether command was successful
    pub success: bool,
}

/// Secondary port - Command execution abstraction
///
/// This interface abstracts system command execution, allowing for different
/// implementations (direct execution, containerized, mocked for testing, etc.)
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a system command
    ///
    /// # Arguments
    /// * `command` - The command to execute
    ///
    /// # Returns
    /// * `Ok(CommandOutput)` - Command output and status
    /// * `Err(CommandError)` - Error executing command
    async fn execute(&self, command: &SystemCommand) -> Result<CommandOutput, CommandError>;

    /// Execute a command with privilege escalation (sudo)
    ///
    /// # Arguments
    /// * `command` - The command to execute
    ///
    /// # Returns
    /// * `Ok(CommandOutput)` - Command output and status
    /// * `Err(CommandError)` - Error executing command
    async fn execute_with_privileges(
        &self,
        command: &SystemCommand,
    ) -> Result<CommandOutput, CommandError>;

    /// Check if a command is available on the system
    ///
    /// # Arguments
    /// * `command_name` - Name of the command to check
    ///
    /// # Returns
    /// * `Ok(bool)` - true if command is available
    /// * `Err(CommandError)` - Error checking command availability
    async fn is_command_available(&self, command_name: &str) -> Result<bool, CommandError>;

    /// Get the path to a command if available
    ///
    /// # Arguments
    /// * `command_name` - Name of the command
    ///
    /// # Returns
    /// * `Ok(Option<String>)` - Path to command if available
    /// * `Err(CommandError)` - Error checking command path
    async fn get_command_path(&self, command_name: &str) -> Result<Option<String>, CommandError>;

    /// Check if running with elevated privileges
    ///
    /// # Returns
    /// * `Ok(bool)` - true if running as root/admin
    /// * `Err(CommandError)` - Error checking privileges
    async fn has_elevated_privileges(&self) -> Result<bool, CommandError>;
}
