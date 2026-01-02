//! 配置模块
//!
//! 包含 CCCRS 配置文件和 Claude Settings 的读写功能

pub mod cccrs_config;
pub mod claude_settings;
pub mod paths;

pub use paths::{ensure_ccc_config_exists, validate_profile_name};

pub use cccrs_config::{CccConfig, EnvConfig, Profile};

pub use claude_settings::{ClaudeEnv, ClaudeSettings};
