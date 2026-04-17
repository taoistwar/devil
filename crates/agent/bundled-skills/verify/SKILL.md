---
name: verify
description: 验证代码变更的正确性
when_to_use: 用户要求验证代码、运行测试、检查变更是否正确时
allowed-tools:
  - Bash
  - Read
  - Glob
arguments: test_command
argument-hint: "[test-command]"
model: inherit
effort: high
user-invocable: true
---

# Verify - 验证技能

执行用户指定的验证命令来确认代码变更的正确性。

## 执行步骤

1. 使用 `$test_command` 运行验证（如果没有提供命令，询问用户或建议常见的测试命令）
2. 检查输出结果
3. 如果发现错误，提供修复建议
4. 验证通过后，总结验证结果

## 示例

- `npm test` - 运行单元测试
- `cargo test` - 运行 Rust 测试
- `make check` - 运行代码检查
- `pytest` - 运行 Python 测试
