//! JSON OSC 数据类型定义
//!
//! 定义通过 OSC 序列传输的结构化数据格式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 块元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockMetadata {
    /// 块唯一标识符
    pub block_id: String,

    /// 块开始时间 (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,

    /// 当前工作目录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// 环境变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// 用户名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// 主机名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

/// 命令元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandMetadata {
    /// 所属块 ID
    pub block_id: String,

    /// 解析后的命令
    pub command: String,

    /// 命令参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    /// 原始输入字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_input: Option<String>,

    /// 命令时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// 输出元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputMetadata {
    /// 所属块 ID
    pub block_id: String,

    /// 输出流类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,

    /// 行数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<u32>,

    /// 字节数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,

    /// 格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// 编码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

/// 命令结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandResult {
    /// 所属块 ID
    pub block_id: String,

    /// 退出码
    pub exit_code: i32,

    /// 结束时间 (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,

    /// 执行时长 (毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// 信号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// JSON 数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonDataType {
    /// 块级元数据
    BlockMeta,
    /// 命令元数据
    CommandMeta,
    /// 输出元数据
    OutputMeta,
    /// 自定义数据
    Custom,
}

impl JsonDataType {
    /// 从字符串解析类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "block_meta" => Some(Self::BlockMeta),
            "command_meta" => Some(Self::CommandMeta),
            "output_meta" => Some(Self::OutputMeta),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BlockMeta => "block_meta",
            Self::CommandMeta => "command_meta",
            Self::OutputMeta => "output_meta",
            Self::Custom => "custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_metadata_serialization() {
        let meta = BlockMetadata {
            block_id: "cmd_001".to_string(),
            start_time: Some("2026-01-26T10:00:00Z".to_string()),
            cwd: Some("/home/user".to_string()),
            env: None,
            user: Some("username".to_string()),
            hostname: Some("machine".to_string()),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: BlockMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta, deserialized);
    }

    #[test]
    fn test_command_metadata_serialization() {
        let meta = CommandMetadata {
            block_id: "cmd_001".to_string(),
            command: "ls".to_string(),
            args: Some(vec!["-l".to_string(), "-a".to_string()]),
            raw_input: Some("ls -la".to_string()),
            timestamp: Some("2026-01-26T10:00:01Z".to_string()),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: CommandMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta, deserialized);
    }

    #[test]
    fn test_output_metadata_serialization() {
        let meta = OutputMetadata {
            block_id: "cmd_001".to_string(),
            stream: Some("stdout".to_string()),
            line_count: Some(42),
            byte_count: Some(2048),
            format: Some("text".to_string()),
            encoding: Some("utf-8".to_string()),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: OutputMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta, deserialized);
    }

    #[test]
    fn test_command_result_serialization() {
        let result = CommandResult {
            block_id: "cmd_001".to_string(),
            exit_code: 0,
            end_time: Some("2026-01-26T10:00:02Z".to_string()),
            duration_ms: Some(1234),
            signal: None,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: CommandResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_json_data_type_from_str() {
        assert_eq!(JsonDataType::from_str("block_meta"), Some(JsonDataType::BlockMeta));
        assert_eq!(JsonDataType::from_str("command_meta"), Some(JsonDataType::CommandMeta));
        assert_eq!(JsonDataType::from_str("output_meta"), Some(JsonDataType::OutputMeta));
        assert_eq!(JsonDataType::from_str("custom"), Some(JsonDataType::Custom));
        assert_eq!(JsonDataType::from_str("unknown"), None);
    }

    #[test]
    fn test_json_data_type_as_str() {
        assert_eq!(JsonDataType::BlockMeta.as_str(), "block_meta");
        assert_eq!(JsonDataType::CommandMeta.as_str(), "command_meta");
        assert_eq!(JsonDataType::OutputMeta.as_str(), "output_meta");
        assert_eq!(JsonDataType::Custom.as_str(), "custom");
    }
}
