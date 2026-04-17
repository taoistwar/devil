---
name: debug
description: 调试辅助，提供诊断思路
when_to_use: 用户遇到 bug、错误、异常行为需要帮助诊断时
allowed-tools:
  - Read
  - Grep
  - Glob
arguments: error_description
argument-hint: "[error-description]"
model: inherit
effort: high
user-invocable: true
---

# Debug - 调试辅助技能

帮助分析错误和提供调试思路。

## 诊断流程

1. 收集错误信息（错误消息、堆栈跟踪、环境信息）
2. 分析错误原因
3. 提供可能的解决方案
4. 指导用户逐一验证

## 常用调试命令

- `git log -n 5` - 查看最近的提交
- `git diff HEAD` - 查看未提交的变更
- `$error_description` - 运行用户描述的错误场景
