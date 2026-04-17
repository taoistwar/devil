//! 子代理注册表
//! 
//! 管理内置子代理和自定义子代理的注册

use crate::subagent::types::{SubagentDefinition, SubagentType, SubagentSource, ModelConfig, PermissionMode};
use std::collections::HashMap;

/// 子代理注册表
pub struct SubagentRegistry {
    /// 内置子代理
    builtin_agents: HashMap<String, SubagentDefinition>,
    /// 自定义子代理
    custom_agents: HashMap<String, SubagentDefinition>,
    /// Fork 子代理配置
    fork_config: crate::subagent::types::ForkSubagentConfig,
}

impl SubagentRegistry {
    /// 创建注册表
    pub fn new() -> Self {
        let mut registry = Self {
            builtin_agents: HashMap::new(),
            custom_agents: HashMap::new(),
            fork_config: crate::subagent::types::ForkSubagentConfig::default(),
        };
        
        // 注册内置子代理
        registry.register_builtin();
        
        registry
    }
    
    /// 注册内置子代理
    fn register_builtin(&mut self) {
        // 注册通用目的子代理
        self.builtin_agents.insert(
            "general-purpose".to_string(),
            SubagentDefinition {
                agent_type: "general-purpose".to_string(),
                when_to_use: "通用任务，无需特殊上下文".to_string(),
                tools: vec!["*".to_string()],
                max_turns: 200,
                model: ModelConfig::ByPurpose(crate::subagent::types::ModelPurpose::Sonnet),
                permission_mode: PermissionMode::Independent,
                source: SubagentSource::Builtin,
                system_prompt_fn: None,
            },
        );
        
        // 注册 Fork 子代理（仅在配置启用时可用）
        self.builtin_agents.insert(
            "fork".to_string(),
            crate::subagent::types::FORK_AGENT,
        );
        
        // 注册其他内置子代理
        self.builtin_agents.insert(
            "research".to_string(),
            SubagentDefinition {
                agent_type: "research".to_string(),
                when_to_use: "研究性任务，需要深入分析".to_string(),
                tools: vec!["Read".to_string(), "Bash".to_string()],
                max_turns: 100,
                model: ModelConfig::ByPurpose(crate::subagent::types::ModelPurpose::Opus),
                permission_mode: PermissionMode::Independent,
                source: SubagentSource::Builtin,
                system_prompt_fn: None,
            },
        );
        
        self.builtin_agents.insert(
            "test".to_string(),
            SubagentDefinition {
                agent_type: "test".to_string(),
                when_to_use: "编写和运行测试".to_string(),
                tools: vec!["Bash".to_string(), "Read".to_string(), "Write".to_string()],
                max_turns: 50,
                model: ModelConfig::ByPurpose(crate::subagent::types::ModelPurpose::Sonnet),
                permission_mode: PermissionMode::Independent,
                source: SubagentSource::Builtin,
                system_prompt_fn: None,
            },
        );
    }
    
    /// 注册自定义子代理
    pub fn register_custom(&mut self, agent: SubagentDefinition) {
        self.custom_agents.insert(agent.agent_type.clone(), agent);
    }
    
    /// 获取子代理定义
    pub fn get_agent(&self, agent_type: &SubagentType) -> Option<&SubagentDefinition> {
        match agent_type {
            SubagentType::Fork => {
                if self.fork_config.enabled {
                    self.builtin_agents.get("fork")
                } else {
                    None
                }
            }
            SubagentType::GeneralPurpose => {
                self.builtin_agents.get("general-purpose")
            }
            SubagentType::Custom(name) => {
                self.custom_agents.get(name)
                    .or_else(|| self.builtin_agents.get(name))
            }
        }
    }
    
    /// 列出所有可用的子代理
    pub fn list_agents(&self) -> Vec<&SubagentDefinition> {
        let mut agents = Vec::new();
        
        for agent in self.builtin_agents.values() {
            // 如果是 Fork 且未启用，跳过
            if agent.agent_type == "fork" && !self.fork_config.enabled {
                continue;
            }
            agents.push(agent);
        }
        
        for agent in self.custom_agents.values() {
            agents.push(agent);
        }
        
        agents
    }
    
    /// 设置 Fork 配置
    pub fn set_fork_config(&mut self, config: crate::subagent::types::ForkSubagentConfig) {
        self.fork_config = config;
    }
    
    /// 获取 Fork 配置
    pub fn get_fork_config(&self) -> &crate::subagent::types::ForkSubagentConfig {
        &self.fork_config
    }
    
    /// 检查子代理类型是否有效
    pub fn is_valid_agent_type(&self, agent_type: &str) -> bool {
        self.builtin_agents.contains_key(agent_type)
            || self.custom_agents.contains_key(agent_type)
    }
    
    /// 获取子代理数量
    pub fn len(&self) -> usize {
        self.builtin_agents.len() + self.custom_agents.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.builtin_agents.is_empty() && self.custom_agents.is_empty()
    }
}

impl Default for SubagentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析子代理类型字符串
pub fn parse_subagent_type(s: &str) -> SubagentType {
    match s {
        "fork" => SubagentType::Fork,
        "general-purpose" | "general" => SubagentType::GeneralPurpose,
        _ => SubagentType::Custom(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_agents_registered() {
        let registry = SubagentRegistry::new();
        assert!(registry.is_valid_agent_type("general-purpose"));
        assert!(registry.is_valid_agent_type("research"));
        assert!(registry.is_valid_agent_type("test"));
    }
    
    #[test]
    fn test_parse_subagent_type() {
        assert_eq!(parse_subagent_type("fork"), SubagentType::Fork);
        assert_eq!(parse_subagent_type("general-purpose"), SubagentType::GeneralPurpose);
        assert_eq!(parse_subagent_type("custom-agent"), SubagentType::Custom("custom-agent".to_string()));
    }
}
