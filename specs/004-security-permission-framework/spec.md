# Feature: Security Permission Framework Alignment

**Feature Directory**: `specs/004-security-permission-framework`

**Created**: 2026-04-19

**Feature Summary**: Align devil-agent with Claude Code's four-stage security permission framework: validateInput, hasPermissionsToUseTool, checkPermissions, and interactive prompts.

---

## 1. Concept & Vision

devil-agent 实现了与 Claude Code 一致的安全权限框架，通过四阶段安全检测确保工具执行的安全性。框架采用防御深度（defense-in-depth）策略，每一阶段都有独立的检查职责，工具可自定义检查逻辑，同时支持规则引擎和交互式用户确认。

## 2. User Scenarios & Testing

### 2.1 Primary User Flows

| Scenario | Trigger | Expected Behavior |
|----------|---------|------------------|
| 工具调用-允许 | 用户执行安全工具（如 `read`） | 直接执行并返回结果 |
| 工具调用-验证失败 | 用户调用工具但输入无效 | 返回 `InputValidationError` |
| 工具调用-规则拒绝 | 用户调用被拒绝的工具 | 返回 `PermissionDenied` |
| 工具调用-需要确认 | 用户调用需要确认的工具 | 显示权限提示等待用户确认 |
| 沙箱自动允许 | 在沙箱中执行安全的 bash 命令 | 自动放行无需确认 |

### 2.2 Security Check Flow

```
User Tool Call
      ↓
[Stage 1: validateInput] → 输入格式验证 (Zod Schema)
      ↓ Pass
[Stage 2: checkPermissions] → 工具特定权限检查
      ↓
[Stage 3: hasPermissionsToUseTool] → 规则匹配 (deny/ask/allow rules)
      ↓
[Stage 4: Interactive Prompt] → 用户确认 (若需要)
      ↓
Tool Execution
```

## 3. Functional Requirements

### 3.1 Stage 1: Input Validation (validateInput)

**FR-401**: 工具必须实现 `validate_input` 方法进行输入值验证

**FR-402**: 输入验证失败返回 `InputValidationResult { is_valid: bool, error_message: Option<String>, error_code: Option<ErrorCode> }`

**FR-403**: 支持 Zod-compatible 的 JSON Schema 验证

**FR-404**: 工具可自定义验证逻辑（如文件路径存在性检查）

**FR-405**: 验证性能要求：单次验证 < 1ms

### 3.2 Stage 2: Tool-specific Permissions (checkPermissions)

**FR-410**: 每个工具实现 `check_permissions` 方法

**FR-411**: 返回 `PermissionResult { behavior: Behavior, message: Option<String>, updated_input: Option<Value>, suggestions: Vec<String> }`

**FR-412**: 支持返回 `updated_input` 修改输入参数

**FR-413**: 敏感路径检查（.git/, .claude/, .vscode/ 等）

**FR-414**: 工具特定权限检查性能要求：< 2ms

### 3.3 Stage 3: Rule-based Matching (hasPermissionsToUseTool)

**FR-420**: 实现权限规则引擎匹配

**FR-421**: 支持三种规则：
- **Deny Rule**: 明确拒绝的工具
- **Ask Rule**: 需要用户确认的工具
- **Allow Rule**: 自动允许的工具

**FR-422**: 规则来源：配置文件、命令行标志、策略设置

**FR-423**: 规则可按工具名称、命令前缀、内容模式匹配

**FR-424**: 规则引擎性能要求：规则匹配 < 5ms（支持 1000+ 规则规模）

**FR-425**: 规则优先级定义（从高到低）：
1. **显式拒绝 (Explicit Deny)**: 最高优先级，匹配则直接拒绝
2. **显式允许 (Explicit Allow)**: 次高优先级，匹配则直接允许
3. **模式匹配 (Pattern Match)**: 按规则顺序匹配
4. **默认行为 (Default)**: 未匹配任何规则时的默认行为

**FR-426**: 规则冲突检测机制：
- 检测同一工具匹配多条同优先级规则
- 检测矛盾规则（如同一工具同时配置 allow 和 deny）
- 冲突报告格式：`RuleConflict { tool: String, rules: Vec<RuleName>, resolution: String }`

**FR-427**: 大规模规则匹配优化：
- 使用前缀树 (Trie) 匹配工具名称模式
- 使用 HashMap 缓存常用规则匹配结果
- 支持规则组 (Rule Group) 批量应用

### 3.4 Stage 4: Interactive Prompts

**FR-430**: 需要用户确认时显示权限提示

**FR-431**: 支持三种权限模式：
- **Default**: 询问模式
- **Auto**: AI 分类器自动决策
- **Bypass**: 跳过所有确认

**FR-432**: 权限提示显示：工具名称、操作描述、风险级别

### 3.5 Tool Permission Levels

**FR-440**: 定义权限级别枚举：
```rust
pub enum ToolPermissionLevel {
    ReadOnly,        // 只读操作，无需确认
    RequiresConfirmation,  // 需要确认
    Destructive,      // 破坏性操作，需明确确认
}
```

### 3.6 Bash Tool Security

**FR-450**: Bash 工具实现语义分析器 `BashSemanticAnalyzer`

**FR-451**: 检测危险命令（rm -rf, shutdown 等）

**FR-452**: 检测敏感路径访问（/etc, /root, ~/.ssh 等）

**FR-453**: 支持沙箱模式自动允许安全命令

### 3.7 File Tool Security

**FR-460**: FileWriteTool 检查目标路径是否在允许范围内

**FR-461**: FileEditTool 验证 old_string 是否存在

**FR-462**: 文件操作记录审计日志

## 4. Key Entities

### 4.1 Permission Types

| Entity | Fields | Description |
|--------|--------|-------------|
| `PermissionDecision` | behavior, decision_reason, message, updated_input | 权限决策结果 |
| `PermissionResult` | behavior, message, suggestions | 工具返回的权限结果 |
| `ValidationResult` | is_valid, error_message, error_code | 输入验证结果 |
| `PermissionRule` | name, tool_pattern, rule_behavior, priority, source | 权限规则 |
| `RuleConflict` | tool, rules, resolution | 规则冲突描述 |

### 4.2 Permission Behavior

| Behavior | Description |
|----------|-------------|
| `allow` | 允许执行，无需确认 |
| `deny` | 拒绝执行 |
| `ask` | 需要用户确认 |
| `passthrough` | 交由下一阶段决定 |

### 4.3 Decision Reason Types

| Reason | Description |
|--------|-------------|
| `rule` | 基于规则引擎的决策 |
| `safety_check` | 安全检查失败 |
| `mode` | 基于权限模式的决策（bypass/auto） |
| `classifier` | AI 分类器决策 |
| `hook` | Hook 钩子决策 |

### 4.4 Rule Priority

| Priority | Level | Description |
|----------|-------|-------------|
| 100 | Critical | 显式系统级拒绝 |
| 80 | High | 显式用户级拒绝 |
| 60 | Medium | 模式匹配拒绝 |
| 40 | Medium | 模式匹配允许 |
| 20 | Low | 用户级允许 |
| 0 | Default | 默认行为 |

## 5. Success Criteria

### 5.1 Functional Criteria

- [ ] **SC-401**: 所有 52 个工具都实现 `validate_input` 和 `check_permissions`
- [ ] **SC-402**: 无效输入返回明确的错误消息（包含错误位置和原因）
- [ ] **SC-403**: 权限规则正确匹配并阻止/允许工具执行
- [ ] **SC-404**: 交互式提示在需要时正确显示
- [ ] **SC-405**: 规则冲突被正确检测并报告

### 5.2 Security Criteria

- [ ] **SC-410**: 危险命令被正确检测和阻止
- [ ] **SC-411**: 敏感路径访问触发适当的权限检查
- [ ] **SC-412**: 沙箱模式正确放行安全的 bash 命令

### 5.3 Performance Criteria

- [ ] **SC-420**: 单次权限检查总耗时 < 10ms
- [ ] **SC-421**: validateInput 阶段 < 1ms
- [ ] **SC-422**: checkPermissions 阶段 < 2ms
- [ ] **SC-423**: hasPermissionsToUseTool 阶段 < 5ms
- [ ] **SC-424**: 规则引擎支持 1000+ 规则规模下匹配 < 5ms
- [ ] **SC-425**: 规则匹配使用高效数据结构（前缀树/HashMap）

## 6. Implementation Notes

### 6.1 Rust Type Mapping

| TypeScript | Rust |
|------------|------|
| `validateInput` | `validate_input` |
| `checkPermissions` | `check_permissions` |
| `hasPermissionsToUseTool` | `has_permissions_to_use_tool` |
| `PermissionDecision` | `PermissionDecision` |
| `PermissionResult` | `PermissionResult` |
| `ValidationResult` | `InputValidationResult` |

### 6.2 Async Trait Pattern

所有工具方法使用 `#[async_trait]` 宏：

```rust
#[async_trait]
impl Tool for BashTool {
    async fn validate_input(&self, input: &Self::Input, ctx: &ToolContext) -> InputValidationResult {
        // 实现
    }
    
    async fn check_permissions(&self, input: &Self::Input, ctx: &ToolContext) -> PermissionResult {
        // 实现
    }
}
```

### 6.3 Error Code Mapping

遵循 SPEC_DEPENDENCIES.md 中的错误码规范：

| ErrorCode | Value | Description |
|-----------|-------|-------------|
| `InvalidInput` | 1001 | 输入格式错误 |
| `MissingRequiredField` | 1002 | 缺少必填字段 |
| `InvalidFormat` | 1003 | 格式不正确 |
| `PermissionDenied` | 2001 | 权限不足 |
| `OperationNotAllowed` | 2003 | 操作不允许 |

### 6.4 Rule Engine Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Rule Engine                               │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Rule Loader │→ │ Rule Parser │→ │ Conflict Detector   │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│         │                                    │               │
│         ▼                                    ▼               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Rule Index (Trie + HashMap)                │ │
│  └─────────────────────────────────────────────────────────┘ │
│                            │                                 │
│                            ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Match Engine (< 5ms @ 1000 rules)          │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 6.5 Rule Configuration Format (TOML)

```toml
[[rules]]
name = "deny-destructive"
tool_pattern = "bash:*rm*"
behavior = "deny"
priority = 80
source = "user_config"

[[rules]]
name = "allow-readonly"
tool_pattern = "read"
behavior = "allow"
priority = 20
source = "default"
```

## 7. Dependencies

- 规则配置格式：**TOML** (Rust 生态标准)
- 交互式提示集成：**使用 channel 异步通知** (最佳用户体验)

## 8. Assumptions

- 权限模式通过环境变量或配置文件设置
- 规则配置存储在 `~/.config/devil-agent/rules.toml`
- 审计日志写入 `~/.local/share/devil-agent/audit.log`

---

## Clarifications

### Session 2026-04-19

- Q: 规则配置格式 → A: **TOML** - Rust 生态标准，易于编辑和 serde 序列化
- Q: 交互式提示集成 → A: **Channel 异步通知** - 最佳用户体验，非阻塞式权限确认

### Session 2026-04-21

- Q: 规则引擎性能要求 → A: 总耗时 < 10ms，各阶段分别为 < 1ms / < 2ms / < 5ms
- Q: 大规模规则匹配效率 → A: 使用 Trie + HashMap，支持 1000+ 规则 < 5ms
- Q: 规则冲突检测 → A: 新增 RuleConflict 实体，检测同优先级矛盾规则
- Q: 规则优先级定义 → A: 0-100 优先级，显式拒绝(80) > 显式允许(20) > 默认(0)
