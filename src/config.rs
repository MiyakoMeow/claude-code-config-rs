//! 配置模块
//!
//! 包含 CCC 配置文件和 Claude Settings 的读写功能

pub mod ccc_config;
pub mod claude_settings;
pub mod paths;

pub use paths::{ensure_ccc_config_exists, validate_profile_name};

pub use ccc_config::{CccConfig, EnvConfig, Profile};

pub use claude_settings::{ClaudeEnv, ClaudeSettings};
