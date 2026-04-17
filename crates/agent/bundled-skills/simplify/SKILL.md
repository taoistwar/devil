---
name: simplify
description: 代码简化与重构审查
when_to_use: 用户要求简化代码、重构、消除重复时
allowed-tools:
  - Read
  - Write
  - Grep
  - Glob
arguments: file_path
argument-hint: "[file-path]"
model: inherit
effort: high
user-invocable: true
---

# Simplify - 代码简化技能

审查代码并提供简化和重构建议。

## 审查要点

1. **重复代码检测** - 查找可以提取为函数的重复模式
2. **复杂度分析** - 识别过于复杂的函数和条件
3. **命名优化** - 改进变量、函数、类的命名
4. **消除死代码** - 移除未使用的导入和函数
5. **简化逻辑** - 用更简洁的方式表达相同的逻辑

## 重构原则

- 保持单一职责
- 减少嵌套层级
- 提取共用逻辑
- 使用更合适的抽象
