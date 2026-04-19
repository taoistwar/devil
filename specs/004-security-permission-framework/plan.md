# Implementation Plan: Security Permission Framework

**Feature**: `specs/004-security-permission-framework`
**Created**: 2026-04-19

## Overview

实现与 Claude Code 一致的四阶段安全权限框架：
1. validateInput（输入验证）
2. checkPermissions（工具特定权限检查）
3. hasPermissionsToUseTool（规则匹配）
4. Interactive Prompts（交互式提示）

## Phase 1: Core Types & Interfaces

### P1.1 Permission Types Module

- [x] 创建 `crates/agent/src/permissions/types.rs`
  - 定义 `PermissionDecision`, `PermissionResult`, `InputValidationResult`
  - 定义 `PermissionBehavior` 枚举 (allow/deny/ask/passthrough)
  - 定义 `DecisionReason` 类型

### P1.2 Tool Trait Updates

- [x] 更新 `crates/agent/src/tools/tool.rs` 中的 `Tool` trait
  - 添加 `validate_input` 方法
  - 添加 `check_permissions` 方法
  - 添加 `permission_level` 方法

### P1.3 Permission Context

- [x] 创建 `crates/agent/src/permissions/context.rs`
  - 定义 `ToolUseContext` 结构
  - 定义 `PermissionContext` 包含当前模式

## Phase 2: Rule Engine

### P2.1 Rule Types

- [x] 创建 `crates/agent/src/permissions/rules.rs`
  - 定义 `PermissionRule` 结构
  - 定义 `RuleMatch` 特性
  - 实现工具名称匹配、命令前缀匹配

### P2.2 Rule Store

- [ ] 创建 `crates/agent/src/permissions/store.rs`
  - 实现规则存储（内存 + 文件持久化）
  - 支持加载/保存 TOML 配置
  - 实现规则优先级排序

### P2.3 Rule Matching

- [x] 实现 `has_permissions_to_use_tool` 函数
  - 1a: 检查 deny rule
  - 1b: 检查 ask rule
  - 1c: 调用工具的 check_permissions
  - 2a/2b: 检查模式允许和 allow rule
  - 3: 转换 passthrough 为 ask

## Phase 3: Tool Implementation

### P3.1 Bash Tool Security

- [x] 增强 `BashTool` 的 `check_permissions`
  - 危险命令检测
  - 敏感路径检测
  - 沙箱自动放行

### P3.2 File Tools Security

- [ ] 增强 `FileWriteTool` 的 `validate_input`
  - 目标路径验证
  - 目录创建权限检查

- [ ] 增强 `FileEditTool` 的 `validate_input`
  - old_string 存在性验证

### P3.3 Web Tools Security

- [ ] 增强 `WebFetchTool` 的 `check_permissions`
  - URL 安全检查
  - 允许/阻止域名列表

## Phase 4: Permission Modes & UI

### P4.1 Permission Modes

- [x] 实现权限模式枚举
  - Default: 询问模式
  - Auto: AI 分类器自动决策
  - Bypass: 跳过所有确认

### P4.2 Interactive Prompts

- [x] 创建 `crates/agent/src/permissions/prompts.rs`
  - 定义权限提示消息格式
  - 实现等待用户确认的机制

## Phase 5: Integration

### P5.1 Tool Executor Integration

- [x] 更新 `crates/agent/src/tools/executor.rs`
  - 在工具执行前调用四阶段检查
  - 处理权限拒绝和询问

### P5.2 Agent Core Integration

- [ ] 更新 `crates/agent/src/core.rs`
  - 注册权限管理器
  - 配置权限模式

## Phase 6: Testing

### P6.1 Unit Tests

- [x] 测试权限规则匹配
- [x] 测试危险命令检测
- [x] 测试敏感路径检测

### P6.2 Integration Tests

- [ ] 测试四阶段完整流程
- [ ] 测试权限模式切换

## Dependencies

- 依赖 Phase 1-4 完成的工具定义
- 需要 tokio 用于异步处理和 channel
- 需要 serde_toml 用于 TOML 配置序列化

## Risks

- R1: ~~规则配置格式尚未确定~~ → 使用 TOML
- R2: ~~交互式提示集成方式尚未确定~~ → 使用 channel 异步通知
