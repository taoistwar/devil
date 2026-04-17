# 技能系统集成指南

本文档描述如何将技能系统集成到 Agent 主循环中。

## 1. 在 Agent 初始化时加载技能

```rust
use devil_agent::skills::{SkillLoader, SkillExecutor, SkillPermissionChecker};
use devil_agent::Agent;

pub struct AgentWithSkills {
    agent: Agent,
    skill_loader: SkillLoader,
    skill_executor: SkillExecutor,
    permission_checker: SkillPermissionChecker,
}

impl AgentWithSkills {
    pub fn new(session_id: String) -> Self {
        let mut loader = SkillLoader::new();
        
        // 加载所有技能
        loader.load_all_from_disk().unwrap();
        
        // 注册内置技能
        let bundled = devil_agent::skills::bundled::register_bundled_skills();
        
        Self {
            agent: Agent::new(),
            skill_loader: loader,
            skill_executor: SkillExecutor::new(session_id),
            permission_checker: SkillPermissionChecker::new(),
        }
    }
}
```

## 2. 在 Agent 主循环中集成技能执行

```rust
impl AgentWithSkills {
    pub async fn run_with_skills(&mut self, input: &str) -> Result<String> {
        // 检查是否调用技能（以 / 开头）
        if input.starts_with('/') {
            return self.execute_skill_command(input).await;
        }
        
        // 正常 Agent 对话
        self.agent.run(input).await
    }
    
    async fn execute_skill_command(&mut self, command: &str) -> Result<String> {
        // 解析技能名称和参数
        let parts: Vec<&str> = command[1..].split_whitespace().collect();
        let skill_name = parts[0];
        let arguments = parts[1..].join(" ");
        
        // 查找技能
        let skill = self.skill_loader.get_all_skills()
            .iter()
            .find(|s| s.name == skill_name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_name))?;
        
        // 权限检查
        match self.permission_checker.check(skill) {
            PermissionCheckResult::Allow => {},
            PermissionCheckResult::Deny(reason) => {
                return Err(anyhow::anyhow!("Permission denied: {}", reason));
            }
            PermissionCheckResult::Ask { reason, .. } => {
                // 这里可以添加用户确认逻辑
                return Err(anyhow::anyhow!("Permission required: {}", reason));
            }
        }
        
        // 执行技能
        let result = self.skill_executor.execute(skill, Some(&arguments)).await?;
        
        // 处理结果
        match result {
            SkillExecutionResult::Inline { new_messages, .. } => {
                // Inline 模式：继续 Agent 对话
                let prompt = new_messages[0].content.as_text().unwrap();
                self.agent.run(prompt).await
            }
            SkillExecutionResult::Fork { result, .. } => {
                // Fork 模式：直接返回结果
                Ok(result)
            }
        }
    }
}
```

## 3. 动态技能发现集成

```rust
use std::path::Path;

impl AgentWithSkills {
    /// 在文件操作时动态发现技能
    pub fn discover_skills_for_file(&mut self, file_path: &Path) {
        let skill_dirs = SkillLoader::discover_skills_for_path(file_path);
        
        for dir in skill_dirs {
            self.skill_loader.load_from_dir(
                &dir,
                SkillSource::Disk,
                SkillLoadSource::ProjectSettings,
            ).unwrap();
        }
    }
    
    /// 激活条件技能
    pub fn activate_conditional_skills(&mut self, file_path: &str) {
        let activated = self.skill_loader.activate_skills_for_path(file_path);
        
        if !activated.is_empty() {
            println!("Activated {} conditional skills for {}", activated.len(), file_path);
        }
    }
}
```

## 4. 预算管理集成

```rust
use devil_agent::skills::SkillBudgetManager;

impl AgentWithSkills {
    /// 在系统提示中注入技能列表
    pub fn format_skills_for_system_prompt(&self, context_window_tokens: usize) -> String {
        let budget_manager = SkillBudgetManager::new(context_window_tokens, 0.2);
        
        let skills = self.skill_loader.get_all_skills();
        let (formatted, truncated) = budget_manager.format_skills_in_budget(skills);
        
        let mut output = String::from("## Available Skills\n\n");
        for desc in formatted {
            output.push_str(&desc);
            output.push('\n');
        }
        
        if truncated {
            output.push_str("\n*Some skill descriptions truncated due to context budget*\n");
        }
        
        output
    }
}
```

## 5. 完整的 Agent Loop 集成示例

```rust
use devil_agent::message::Message;
use devil_agent::state::State;

impl AgentWithSkills {
    pub async fn agent_loop(&mut self, initial_messages: Vec<Message>) -> Result<State> {
        let mut state = State::initial(initial_messages);
        
        // 检查文件操作，动态发现技能
        for msg in &state.messages {
            if let Some(text) = msg.content.as_text() {
                // 提取文件路径（简化实现）
                if let Some(path) = self.extract_file_path(text) {
                    self.discover_skills_for_file(&path);
                    self.activate_conditional_skills(path.to_str().unwrap());
                }
            }
        }
        
        // 正常的 Agent 循环
        while !state.is_terminal() {
            // 检查技能调用
            if let Some(skill_command) = self.extract_skill_command(&state.messages) {
                match self.execute_skill_command(&skill_command).await {
                    Ok(response) => {
                        state.messages.push(Message::assistant(response));
                        continue;
                    }
                    Err(e) => {
                        state.messages.push(Message::user(format!("Error: {}", e)));
                        continue;
                    }
                }
            }
            
            // 正常对话流
            let response = self.agent.step(&state).await?;
            state = state.next(ContinueReason::NextTurn, response.messages);
        }
        
        Ok(state)
    }
    
    fn extract_file_path(&self, text: &str) -> Option<PathBuf> {
        // 简化实现：查找类似 "src/file.rs" 的路径
        let path_pattern = regex::Regex::new(r"([a-zA-Z0-9_/]+\.[a-z]+)").unwrap();
        path_pattern.find(text)
            .map(|m| PathBuf::from(m.as_str()))
    }
    
    fn extract_skill_command(&self, messages: &[Message]) -> Option<String> {
        // 查找最后一条用户消息是否以 / 开头
        messages.iter()
            .rev()
            .find(|m| m.role == "user")
            .and_then(|m| m.content.as_text())
            .filter(|t| t.starts_with('/'))
            .map(|t| t.to_string())
    }
}
```

## 6. 权限配置管理

```rust
use std::path::PathBuf;
use serde_json;

impl AgentWithSkills {
    /// 从配置文件加载权限规则
    pub fn load_permission_config(&mut self, config_path: &Path) -> Result<()> {
        let config = std::fs::read_to_string(config_path)?;
        let rules: Vec<PermissionRule> = serde_json::from_str(&config)?;
        
        for rule in rules {
            match rule.source {
                RuleSource::User | RuleSource::Auto => {
                    if matches!(rule.rule_type, RuleType::Skill) && rule.pattern.starts_with("skill:") {
                        // 根据规则内容判断是 Allow 还是 Deny
                        // 这里应该根据配置文件的具体格式来判断
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

## 7. 使用频率统计

```rust
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref SKILL_USAGE: Mutex<HashMap<String, SkillUsage>> = Mutex::new(HashMap::new());
}

impl AgentWithSkills {
    /// 记录技能使用
    pub fn record_skill_usage(&self, skill_name: &str) {
        let mut usage_map = SKILL_USAGE.lock().unwrap();
        
        let usage = usage_map.entry(skill_name.to_string()).or_insert(SkillUsage::default());
        usage.record_usage();
    }
    
    /// 获取技能排名（按使用频率排序）
    pub fn get_skill_ranking(&self) -> Vec<(String, f64)> {
        let usage_map = SKILL_USAGE.lock().unwrap();
        
        let mut rankings: Vec<_> = usage_map.iter()
            .map(|(name, usage)| (name.clone(), usage.ranking_score))
            .collect();
        
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }
}
```

## 8. 配置文件示例

权限配置文件 `~/.claude/skill-permissions.json`：

```json
[
  {
    "pattern": "skill:verify",
    "rule_type": "skill",
    "source": "user"
  },
  {
    "pattern": "skill:code-review:*",
    "rule_type": "skill",
    "source": "user"
  },
  {
    "pattern": "skill:dangerous-*",
    "rule_type": "skill",
    "source": "project"
  }
]
```

## 9. 完整的 Cargo 依赖

```toml
[dependencies]
devil-agent = { path = "crates/agent" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
lazy_static = "1"
```

## 10. 测试集成

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_skill_execution() {
        let mut agent = AgentWithSkills::new("test-session".to_string());
        
        // 执行技能
        let result = agent.execute_skill_command("/verify cargo test").await;
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_permission_checking() {
        let mut agent = AgentWithSkills::new("test-session".to_string());
        
        // 设置权限规则
        // ...
        
        // 测试权限检查
        // ...
    }
}
```

## 总结

集成技能系统到 Agent 主循环的关键步骤：

1. **初始化时加载**：创建 SkillLoader 并加载所有来源的技能
2. **解析技能调用**：检测 `/skill-name` 格式的命令
3. **权限检查**：执行五层权限检查
4. **执行技能**：根据 context 选择 Inline 或 Fork 模式
5. **动态发现**：在文件操作时动态加载技能
6. **预算管理**：格式化技能列表在系统提示中
7. **使用统计**：记录技能使用频率
