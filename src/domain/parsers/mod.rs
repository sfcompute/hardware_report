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

//! Pure parsing functions for converting raw command output to domain objects
//!
//! These functions are pure (no side effects) and can be easily tested in isolation.
//! They take string input and return domain objects or parsing errors.

pub mod common;
pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod storage;
pub mod system;

pub use common::*;
pub use cpu::*;
pub use gpu::*;
pub use memory::*;
pub use network::*;
pub use storage::*;
pub use system::*;
