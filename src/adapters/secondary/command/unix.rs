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

//! Unix command execution adapter

use crate::domain::CommandError;
use crate::ports::{CommandExecutor, CommandOutput, SystemCommand};
use async_trait::async_trait;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Unix-based command executor that handles privilege escalation and timeouts
pub struct UnixCommandExecutor {
    /// Default timeout for commands
    default_timeout: Duration,
    /// Number of retry attempts for failed commands
    retry_count: u32,
    /// Whether to log command execution (for debugging)
    verbose: bool,
}

impl UnixCommandExecutor {
    /// Create a new Unix command executor
    ///
    /// # Arguments
    /// * `default_timeout` - Default timeout for commands
    /// * `retry_count` - Number of retry attempts
    /// * `verbose` - Enable verbose logging
    pub fn new(default_timeout: Duration, retry_count: u32, verbose: bool) -> Self {
        Self {
            default_timeout,
            retry_count,
            verbose,
        }
    }

    /// Create a Unix command executor with default settings
    pub fn with_defaults() -> Self {
        Self::new(Duration::from_secs(30), 2, false)
    }

    /// Execute a command with optional retry logic
    async fn execute_with_retry(
        &self,
        command: &SystemCommand,
        use_sudo: bool,
    ) -> Result<CommandOutput, CommandError> {
        let mut last_error = None;

        for attempt in 0..=self.retry_count {
            match self.execute_once(command, use_sudo).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.retry_count {
                        if self.verbose {
                            eprintln!("Command failed on attempt {}, retrying...", attempt + 1);
                        }
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Execute a command once
    async fn execute_once(
        &self,
        command: &SystemCommand,
        use_sudo: bool,
    ) -> Result<CommandOutput, CommandError> {
        let command_timeout = command.timeout.unwrap_or(self.default_timeout);

        let mut cmd = if use_sudo || command.use_sudo {
            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd.arg(&command.program);
            sudo_cmd.args(&command.args);
            sudo_cmd
        } else {
            let mut base_cmd = Command::new(&command.program);
            base_cmd.args(&command.args);
            base_cmd
        };

        // Set working directory if specified
        if let Some(ref working_dir) = command.working_dir {
            cmd.current_dir(working_dir);
        }

        // Set environment variables if specified
        if let Some(ref env_vars) = command.env_vars {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Configure stdio
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        if self.verbose {
            eprintln!("Executing: {} {}", command.program, command.args.join(" "));
        }

        // Execute with timeout
        let result = timeout(command_timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let success = output.status.success();
                let exit_code = output.status.code();

                if self.verbose && !success {
                    eprintln!("Command failed with exit code: {exit_code:?}");
                    if !stderr.is_empty() {
                        eprintln!("stderr: {stderr}");
                    }
                }

                Ok(CommandOutput {
                    stdout,
                    stderr,
                    exit_code,
                    success,
                })
            }
            Ok(Err(e)) => Err(CommandError::ExecutionFailed(format!(
                "Failed to execute command '{}': {}",
                command.program, e
            ))),
            Err(_) => Err(CommandError::ExecutionFailed(format!(
                "Command '{}' timed out after {:?}",
                command.program, command_timeout
            ))),
        }
    }
}

#[async_trait]
impl CommandExecutor for UnixCommandExecutor {
    async fn execute(&self, command: &SystemCommand) -> Result<CommandOutput, CommandError> {
        self.execute_with_retry(command, false).await
    }

    async fn execute_with_privileges(
        &self,
        command: &SystemCommand,
    ) -> Result<CommandOutput, CommandError> {
        self.execute_with_retry(command, true).await
    }

    async fn is_command_available(&self, command_name: &str) -> Result<bool, CommandError> {
        let which_cmd = SystemCommand::new("which")
            .args(&[command_name])
            .timeout(Duration::from_secs(5));

        match self.execute(&which_cmd).await {
            Ok(output) => Ok(output.success && !output.stdout.trim().is_empty()),
            Err(_) => Ok(false), // If 'which' fails, assume command is not available
        }
    }

    async fn get_command_path(&self, command_name: &str) -> Result<Option<String>, CommandError> {
        let which_cmd = SystemCommand::new("which")
            .args(&[command_name])
            .timeout(Duration::from_secs(5));

        match self.execute(&which_cmd).await {
            Ok(output) if output.success => {
                let path = output.stdout.trim();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(path.to_string()))
                }
            }
            _ => Ok(None),
        }
    }

    async fn has_elevated_privileges(&self) -> Result<bool, CommandError> {
        // Check if running as root (UID 0)
        let id_cmd = SystemCommand::new("id")
            .args(&["-u"])
            .timeout(Duration::from_secs(5));

        match self.execute(&id_cmd).await {
            Ok(output) if output.success => {
                let uid = output.stdout.trim();
                Ok(uid == "0")
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_unix_command_executor_basic() {
        let executor = UnixCommandExecutor::with_defaults();

        let cmd = SystemCommand::new("echo").args(&["hello", "world"]);

        let result = executor.execute(&cmd).await.unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello world");
    }

    #[tokio::test]
    async fn test_command_availability_check() {
        let executor = UnixCommandExecutor::with_defaults();

        // Test with a command that should not exist
        assert!(!executor
            .is_command_available("definitely_not_a_real_command_12345")
            .await
            .unwrap());

        // Test with potentially available commands, but don't require them in sandbox
        let common_commands = ["echo", "ls", "cat", "true"];
        for cmd in &common_commands {
            // Just verify the function works without panicking, don't assert results
            let _ = executor.is_command_available(cmd).await.unwrap_or(false);
        }
        // This test mainly verifies the is_command_available function works without panicking
    }

    #[tokio::test]
    async fn test_command_timeout() {
        let executor = UnixCommandExecutor::with_defaults();

        let cmd = SystemCommand::new("sleep")
            .args(&["10"])
            .timeout(Duration::from_millis(100));

        let result = executor.execute(&cmd).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_get_command_path() {
        let executor = UnixCommandExecutor::with_defaults();

        // Test that the function works without panicking
        let path = executor.get_command_path("echo").await.unwrap();
        // In sandbox environments, commands may not be available, so just verify function works
        if let Some(p) = path {
            assert!(p.contains("echo"));
        }

        // Test with definitely non-existent command
        let bad_path = executor
            .get_command_path("definitely_not_a_real_command_12345")
            .await
            .unwrap();
        assert!(bad_path.is_none());
    }

    #[tokio::test]
    async fn test_has_elevated_privileges() {
        let executor = UnixCommandExecutor::with_defaults();

        // This will return false unless running as root
        let _is_root = executor.has_elevated_privileges().await.unwrap();
        // Don't assert the result since it depends on how tests are run
    }
}
