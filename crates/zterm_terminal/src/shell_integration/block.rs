//! 命令块数据模型
//!
//! 实现类似 Warp 的块状终端渲染

use super::json_types::JsonDataType;
use super::OscSequence;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 块唯一标识符
pub type BlockId = String;

/// 命令块
#[derive(Debug, Clone)]
pub struct CommandBlock {
    /// 块 ID
    pub id: BlockId,

    /// 提示符内容
    pub prompt: String,

    /// 命令文本
    pub command: Option<String>,

    /// 命令参数
    pub args: Vec<String>,

    /// 工作目录
    pub cwd: Option<String>,

    /// 开始时间戳 (Unix 毫秒)
    pub start_time: u64,

    /// 结束时间戳 (Unix 毫秒)
    pub end_time: Option<u64>,

    /// 执行时长 (毫秒)
    pub duration_ms: Option<u64>,

    /// 退出码
    pub exit_code: Option<i32>,

    /// 块状态
    pub state: BlockState,

    /// 输出块列表
    pub outputs: Vec<OutputBlock>,

    /// 扩展元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 块状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    /// 提示符阶段
    Prompt,
    /// 输入阶段
    Input,
    /// 执行中
    Executing,
    /// 已完成
    Finished,
}

/// 输出块
#[derive(Debug, Clone)]
pub struct OutputBlock {
    /// 输出类型
    pub stream: OutputStream,

    /// 输出内容
    pub content: String,

    /// 行号范围 (start, end)
    pub line_range: Option<(usize, usize)>,
}

/// 输出流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    /// 标准输出
    Stdout,
    /// 标准错误
    Stderr,
}

impl CommandBlock {
    /// 创建新的命令块
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            prompt: String::new(),
            command: None,
            args: Vec::new(),
            cwd: None,
            start_time: current_timestamp_ms(),
            end_time: None,
            duration_ms: None,
            exit_code: None,
            state: BlockState::Prompt,
            outputs: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 设置命令
    pub fn set_command(&mut self, command: String, args: Vec<String>) {
        self.command = Some(command);
        self.args = args;
        self.state = BlockState::Input;
    }

    /// 开始执行
    pub fn start_execution(&mut self) {
        self.state = BlockState::Executing;
    }

    /// 添加输出
    pub fn add_output(&mut self, stream: OutputStream, content: String) {
        self.outputs.push(OutputBlock {
            stream,
            content,
            line_range: None,
        });
    }

    /// 完成块
    pub fn finish(&mut self, exit_code: Option<i32>) {
        let now = current_timestamp_ms();
        self.end_time = Some(now);
        self.duration_ms = Some(now - self.start_time);
        self.exit_code = exit_code;
        self.state = BlockState::Finished;
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// 获取总输出长度
    pub fn total_output_length(&self) -> usize {
        self.outputs.iter().map(|o| o.content.len()).sum()
    }

    /// 获取输出文本
    pub fn get_output_text(&self) -> String {
        self.outputs
            .iter()
            .map(|o| o.content.as_str())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// 块管理器
pub struct BlockManager {
    /// 所有块
    blocks: Vec<CommandBlock>,

    /// 当前活跃块 ID
    current_block_id: Option<BlockId>,

    /// 块 ID 计数器
    block_counter: u64,
}

impl BlockManager {
    /// 创建新的块管理器
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            current_block_id: None,
            block_counter: 0,
        }
    }

    /// 生成新的块 ID
    fn generate_block_id(&mut self) -> BlockId {
        self.block_counter += 1;
        format!("block_{}", self.block_counter)
    }

    /// 处理 OSC 序列,更新块状态
    pub fn handle_osc_sequence(&mut self, seq: &OscSequence) {
        match seq {
            OscSequence::PromptStart { aid, json } => {
                // 开始新块
                let block_id = aid.clone().unwrap_or_else(|| self.generate_block_id());

                let mut block = CommandBlock::new(block_id.clone());

                // 从 JSON 提取元数据
                if let Some(meta) = json {
                    if let Some(cwd) = &meta.cwd {
                        block.cwd = Some(cwd.clone());
                    }
                    // 存储其他元数据
                    if let Ok(json_value) = serde_json::to_value(meta) {
                        if let Some(obj) = json_value.as_object() {
                            block.metadata.extend(obj.clone());
                        }
                    }
                }

                self.blocks.push(block);
                self.current_block_id = Some(block_id);
            }

            OscSequence::CommandStart { aid } => {
                // 进入输入阶段
                if let Some(block) = self.get_current_block_mut() {
                    block.state = BlockState::Input;

                    if let Some(aid) = aid {
                        // 关联 aid
                        block
                            .metadata
                            .insert("aid".to_string(), serde_json::json!(aid));
                    }
                }
            }

            OscSequence::CommandExecuting { aid } => {
                // 开始执行
                if let Some(block) = self.get_current_block_mut() {
                    block.start_execution();

                    if let Some(aid) = aid {
                        block
                            .metadata
                            .insert("aid".to_string(), serde_json::json!(aid));
                    }
                }
            }

            OscSequence::CommandFinished {
                exit_code,
                aid,
                json,
            } => {
                // 完成块
                if let Some(block) = self.get_current_block_mut() {
                    block.finish(*exit_code);

                    if let Some(aid) = aid {
                        block
                            .metadata
                            .insert("aid".to_string(), serde_json::json!(aid));
                    }

                    // 从 JSON 提取结果元数据
                    if let Some(result) = json {
                        if let Some(duration) = result.duration_ms {
                            block.duration_ms = Some(duration);
                        }
                        if let Ok(json_value) = serde_json::to_value(result) {
                            if let Some(obj) = json_value.as_object() {
                                block.metadata.extend(obj.clone());
                            }
                        }
                    }
                }

                // 完成当前块
                self.current_block_id = None;
            }

            OscSequence::JsonData { data_type, payload } => {
                // 处理 JSON 元数据
                match data_type {
                    JsonDataType::CommandMeta => {
                        if let Some(block) = self.get_current_block_mut() {
                            // 提取命令信息
                            if let Some(cmd) = payload.get("command").and_then(|v| v.as_str()) {
                                block.command = Some(cmd.to_string());
                            }
                            if let Some(cwd) = payload.get("cwd").and_then(|v| v.as_str()) {
                                block.cwd = Some(cwd.to_string());
                            }

                            // 存储完整元数据
                            if let Some(obj) = payload.as_object() {
                                block.metadata.extend(obj.clone());
                            }
                        }
                    }
                    JsonDataType::BlockMeta => {
                        if let Some(block) = self.get_current_block_mut() {
                            if let Some(obj) = payload.as_object() {
                                block.metadata.extend(obj.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }

            OscSequence::Osc7WorkingDirectory(cwd) => {
                // 更新工作目录
                if let Some(block) = self.get_current_block_mut() {
                    block.cwd = Some(cwd.clone());
                }
            }

            _ => {}
        }
    }

    /// 获取当前活跃块
    pub fn get_current_block(&self) -> Option<&CommandBlock> {
        self.current_block_id
            .as_ref()
            .and_then(|id| self.blocks.iter().find(|b| &b.id == id))
    }

    /// 获取当前活跃块 (可变)
    pub fn get_current_block_mut(&mut self) -> Option<&mut CommandBlock> {
        let current_id = self.current_block_id.clone()?;
        self.blocks.iter_mut().find(|b| b.id == current_id)
    }

    /// 获取所有块
    pub fn get_blocks(&self) -> &[CommandBlock] {
        &self.blocks
    }

    /// 获取块数量
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// 清空所有块
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.current_block_id = None;
    }

    /// 移除旧块 (保留最近 N 个)
    pub fn trim_blocks(&mut self, max_blocks: usize) {
        if self.blocks.len() > max_blocks {
            let remove_count = self.blocks.len() - max_blocks;
            self.blocks.drain(0..remove_count);
        }
    }
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取当前 Unix 时间戳 (毫秒)
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_block_creation() {
        let block = CommandBlock::new("test_1".to_string());
        assert_eq!(block.id, "test_1");
        assert_eq!(block.state, BlockState::Prompt);
        assert!(block.command.is_none());
        assert!(block.exit_code.is_none());
    }

    #[test]
    fn test_command_block_lifecycle() {
        let mut block = CommandBlock::new("test_1".to_string());

        // 设置命令
        block.set_command("ls".to_string(), vec!["-la".to_string()]);
        assert_eq!(block.state, BlockState::Input);
        assert_eq!(block.command, Some("ls".to_string()));

        // 开始执行
        block.start_execution();
        assert_eq!(block.state, BlockState::Executing);

        // 添加输出
        block.add_output(OutputStream::Stdout, "file1.txt\nfile2.txt\n".to_string());
        assert_eq!(block.outputs.len(), 1);

        // 完成
        block.finish(Some(0));
        assert_eq!(block.state, BlockState::Finished);
        assert_eq!(block.exit_code, Some(0));
        assert!(block.is_success());
        assert!(block.duration_ms.is_some());
    }

    #[test]
    fn test_block_manager_basic() {
        let mut manager = BlockManager::new();
        assert_eq!(manager.block_count(), 0);

        // 模拟 OSC 133;A
        manager.handle_osc_sequence(&OscSequence::PromptStart {
            aid: Some("cmd_001".to_string()),
            json: None,
        });
        assert_eq!(manager.block_count(), 1);
        assert!(manager.get_current_block().is_some());

        // 模拟 OSC 133;B
        manager.handle_osc_sequence(&OscSequence::CommandStart {
            aid: Some("cmd_001".to_string()),
        });
        assert_eq!(manager.get_current_block().unwrap().state, BlockState::Input);

        // 模拟 OSC 133;C
        manager.handle_osc_sequence(&OscSequence::CommandExecuting {
            aid: Some("cmd_001".to_string()),
        });
        assert_eq!(
            manager.get_current_block().unwrap().state,
            BlockState::Executing
        );

        // 模拟 OSC 133;D
        manager.handle_osc_sequence(&OscSequence::CommandFinished {
            exit_code: Some(0),
            aid: Some("cmd_001".to_string()),
            json: None,
        });

        let block = &manager.get_blocks()[0];
        assert_eq!(block.state, BlockState::Finished);
        assert_eq!(block.exit_code, Some(0));
        assert!(manager.get_current_block().is_none());
    }

    #[test]
    fn test_block_manager_json_metadata() {
        let mut manager = BlockManager::new();

        // 模拟带 JSON 的 OSC 133;A
        use super::super::json_types::BlockMetadata;
        let meta = BlockMetadata {
            block_id: "cmd_001".to_string(),
            start_time: Some("2026-01-26T10:00:00Z".to_string()),
            cwd: Some("/home/user".to_string()),
            env: None,
            user: Some("testuser".to_string()),
            hostname: Some("testhost".to_string()),
        };

        manager.handle_osc_sequence(&OscSequence::PromptStart {
            aid: Some("cmd_001".to_string()),
            json: Some(meta),
        });

        let block = manager.get_current_block().unwrap();
        assert_eq!(block.cwd, Some("/home/user".to_string()));
    }

    #[test]
    fn test_block_manager_trim() {
        let mut manager = BlockManager::new();

        // 创建 5 个块
        for i in 0..5 {
            manager.handle_osc_sequence(&OscSequence::PromptStart {
                aid: Some(format!("cmd_{:03}", i)),
                json: None,
            });
            manager.handle_osc_sequence(&OscSequence::CommandFinished {
                exit_code: Some(0),
                aid: Some(format!("cmd_{:03}", i)),
                json: None,
            });
        }

        assert_eq!(manager.block_count(), 5);

        // 保留最近 3 个
        manager.trim_blocks(3);
        assert_eq!(manager.block_count(), 3);

        // 确认是最近的 3 个
        assert_eq!(manager.get_blocks()[0].id, "cmd_002");
        assert_eq!(manager.get_blocks()[2].id, "cmd_004");
    }

    #[test]
    fn test_output_block() {
        let mut block = CommandBlock::new("test_1".to_string());

        block.add_output(OutputStream::Stdout, "line1\n".to_string());
        block.add_output(OutputStream::Stdout, "line2\n".to_string());
        block.add_output(OutputStream::Stderr, "error\n".to_string());

        assert_eq!(block.outputs.len(), 3);
        assert_eq!(block.get_output_text(), "line1\nline2\nerror\n");
        assert_eq!(block.total_output_length(), 18); // 6 + 6 + 6 = 18
    }
}
