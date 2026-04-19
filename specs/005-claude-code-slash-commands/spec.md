# Feature Specification: Claude Code Slash Commands Alignment

## Summary

移植 Claude Code 的 100+ 斜杠命令到 devil-agent (Rust)。每个命令对应一个子模块，实现相同功能但使用 Rust 编写。

## Technical Context

- **Language/Version**: Rust 1.75+
- **Primary Dependencies**: tokio, anyhow, serde
- **Storage**: 文件系统配置 (TOML)
- **Testing**: cargo test
- **Target Platform**: Linux/macOS/Windows CLI
- **Project Type**: CLI tool with slash command system

## User Scenarios & Testing

### Core Slash Commands

| 命令 | 功能 | 测试场景 |
|------|------|----------|
| `/help` | 显示帮助信息 | 输入 /help 验证输出 |
| `/compact` | 手动压缩上下文 | 触发上下文压缩 |
| `/model` | 切换模型 | 切换模型并验证 |
| `/clear` | 清除对话 | 清除历史并验证 |
| `/config` | 配置管理 | 查看/修改配置 |
| `/login` | 认证登录 | OAuth 登录流程 |
| `/logout` | 认证登出 | 清除会话 |
| `/doctor` | 系统诊断 | 运行诊断检查 |

### Advanced Slash Commands

| 命令 | 功能 |
|------|------|
| `/mcp` | MCP 服务器管理 |
| `/hooks` | Hook 管理 |
| `/skills` | 技能管理 |
| `/tasks` | 任务管理 |
| `/memory` | 记忆管理 |
| `/permissions` | 权限管理 |
| `/diff` | 查看文件差异 |
| `/review` | 代码审查 |
| `/plan` | 计划模式 |
| `/resume` | 恢复会话 |
| `/share` | 分享对话 |
| `/voice` | 语音输入 |
| `/fast` | 快速模式 |

### All Supported Commands (100+)

1. `/help` - 帮助信息
2. `/compact` - 手动压缩上下文
3. `/model` - 切换模型
4. `/clear` - 清除对话
5. `/exit` - 退出
6. `/resume` - 恢复会话
7. `/review` - 代码审查
8. `/diff` - 查看文件差异
9. `/share` - 分享对话
10. `/reset-limits` - 重置限制
11. `/output-style` - 输出样式
12. `/config` - 配置管理
13. `/login` - 认证登录
14. `/logout` - 认证登出
15. `/doctor` - 系统诊断
16. `/cost` - 查看费用
17. `/usage` - 使用统计
18. `/extra-usage` - 额外使用量
19. `/rate-limit-options` - 速率限制选项
20. `/privacy-settings` - 隐私设置
21. `/keybindings` - 快捷键
22. `/theme` - 主题切换
23. `/color` - 颜色配置
24. `/detach` - 分离会话
25. `/branch` - 分支管理
26. `/session` - 会话管理
27. `/mcp` - MCP 服务器管理
28. `/hooks` - Hook 管理
29. `/skills` - 技能管理
30. `/tasks` - 任务管理
31. `/memory` - 记忆管理
32. `/permissions` - 权限管理
33. `/agents` - 多代理管理
34. `/workflows` - 工作流管理
35. `/pipes` - 管道管理
36. `/status` - 状态查看
37. `/stats` - 统计信息
38. `/issue` - 问题管理
39. `/pr_comments` - PR 评论
40. `/btw` - 侧注
41. `/vim` - Vim 编辑模式
42. `/thinkback` - Thinkback 工具
43. `/files` - 文件管理
44. `/add-dir` - 添加目录
45. `/copy` - 复制内容
46. `/src` - 源代码相关
47. `/env` - 环境变量
48. `/terminalSetup` - 终端设置
49. `/ide` - IDE 设置
50. `/context` - 上下文管理
51. `/summary` - 摘要生成
52. `/rewind` - 回退会话
53. `/tag` - 标签管理
54. `/rename` - 重命名会话
55. `/passes` - 代码改进
56. `/autofix-pr` - 自动修复 PR
57. `/bughunter` - Bug 追踪
58. `/effort` - 估算工作量
59. `/peers` - 对等连接
60. `/send` - 发送消息
61. `/voice` - 语音输入
62. `/stickers` - 贴纸
63. `/feedback` - 反馈
64. `/release-notes` - 发布说明
65. `/onboarding` - 入门引导
66. `/attach` - 附加内容
67. `/mobile` - 移动端
68. `/desktop` - 桌面应用
69. `/chrome` - Chrome 集成
70. `/upgrade` - 自动更新
71. `/plugin` - 插件管理
72. `/reload-plugins` - 重载插件
73. `/debug-tool-call` - 调试工具调用
74. `/mock-limits` - 模拟限制
75. `/ant-trace` - Ant 追踪
76. `/backfill-sessions` - 会话填充
77. `/break-cache` - 清除缓存
78. `/claim-main` - 声明主会话
79. `/heapdump` - 堆转储
80. `/perf-issue` - 性能问题
81. `/teleport` - 远程切换
82. `/bridge` - 桥接模式
83. `/sandbox-toggle` - 沙箱切换
84. `/remote-setup` - 远程设置
85. `/remote-env` - 远程环境
86. `/oauth-refresh` - OAuth 刷新
87. `/install-github-app` - 安装 GitHub 应用
88. `/good-claude` - 反馈好的体验
89. `/poor` - 反馈差的体验
90. `/advisor` - 顾问模式
91. `/buddy` - Buddy 模式
92. `/ctx_viz` - 上下文可视化

## Functional Requirements

### FR-001: 命令注册系统
- 每个命令是一个独立的 Rust 模块
- 命令通过 `#[derive(SlashCommand)]` 属性注册
- 命令名称与 Claude Code 保持一致

### FR-002: 命令执行框架
- 实现 `SlashCommand` trait
- 支持参数解析 (使用 serde)
- 支持异步执行

### FR-003: 命令分类
- Core Commands: 核心命令
- Config Commands: 配置命令
- Advanced Commands: 高级功能命令
- Edit Commands: 编辑命令
- Collaboration Commands: 协作命令
- System Commands: 系统命令

### FR-004: 帮助系统
- `/help [command]` 显示指定命令帮助
- `/help` 显示所有命令列表
- 命令描述与 Claude Code 一致

### FR-005: 命令执行结果
- 返回结构化结果 (JSON)
- 支持成功/失败状态
- 支持错误消息

## Success Criteria

1. **命令覆盖率**: 100+ 命令全部实现
2. **功能对齐**: 与 Claude Code 功能一致
3. **编译通过**: cargo build 成功
4. **测试通过**: 单元测试全部通过
5. **CLI 集成**: 与现有 CLI 系统集成

## Architecture

```
src/commands/
├── mod.rs                 # 模块导出
├── help.rs                # /help 命令
├── compact.rs             # /compact 命令
├── model.rs               # /model 命令
├── clear.rs               # /clear 命令
├── config.rs              # /config 命令
├── login.rs              # /login 命令
├── logout.rs             # /logout 命令
├── mcp.rs                 # /mcp 命令
├── hooks.rs               # /hooks 命令
├── skills.rs              # /skills 命令
├── tasks.rs               # /tasks 命令
├── diff.rs                # /diff 命令
├── review.rs              # /review 命令
├── plan.rs                # /plan 命令
├── resume.rs              # /resume 命令
├── share.rs               # /share 命令
├── voice.rs               # /voice 命令
├── vim.rs                 # /vim 命令
├── fast.rs                # /fast 命令
├── cost.rs                # /cost 命令
├── doctor.rs              # /doctor 命令
├── memory.rs              # /memory 命令
├── upgrade.rs             # /upgrade 命令
├── desktop.rs             # /desktop 命令
├── theme.rs               # /theme 命令
├── permissions.rs         # /permissions 命令
├── stickers.rs            # /stickers 命令
├── exit.rs                # /exit 命令
├── status.rs              # /status 命令
├── stats.rs               # /stats 命令
├── history.rs             # /history 命令
├── branch.rs              # /branch 命令
├── session.rs             # /session 命令
├── attach.rs              # /attach 命令
├── detach.rs              # /detach 命令
├── agents.rs              # /agents 命令
├── workflows.rs           # /workflows 命令
├── pipes.rs               # /pipes 命令
├── rename.rs              # /rename 命令
├── tag.rs                 # /tag 命令
├── env.rs                 # /env 命令
├── files.rs               # /files 命令
├── src.rs                 # /src 命令
├── context.rs             # /context 命令
├── summary.rs             # /summary 命令
├── rewind.rs              # /rewind 命令
├── passes.rs              # /passes 命令
├── effort.rs              # /effort 命令
├── issue.rs               # /issue 命令
├── feedback.rs            # /feedback 命令
├── release_notes.rs       # /release-notes 命令
├── onboarding.rs          # /onboarding 命令
├── mobile.rs              # /mobile 命令
├── chrome.rs              # /chrome 命令
├── plugin.rs              # /plugin 命令
├── reload_plugins.rs      # /reload-plugins 命令
├── debug_tool_call.rs     # /debug-tool-call 命令
├── mock_limits.rs         # /mock-limits 命令
├── heapdump.rs            # /heapdump 命令
├── perf_issue.rs          # /perf-issue 命令
├── teleport.rs            # /teleport 命令
├── bridge.rs              # /bridge 命令
├── sandbox_toggle.rs      # /sandbox-toggle 命令
├── remote_setup.rs        # /remote-setup 命令
├── remote_env.rs          # /remote-env 命令
├── install_github_app.rs  # /install-github-app 命令
├── keybindings.rs         # /keybindings 命令
├── color.rs               # /color 命令
├── privacy_settings.rs    # /privacy-settings 命令
├── rate_limit_options.rs  # /rate-limit-options 命令
├── extra_usage.rs         # /extra-usage 命令
├── usage.rs               # /usage 命令
├── reset_limits.rs        # /reset-limits 命令
├── output_style.rs        # /output-style 命令
├── peers.rs               # /peers 命令
├── send.rs                # /send 命令
├── btw.rs                 # /btw 命令
├── pr_comments.rs         # /pr_comments 命令
├── autofix_pr.rs          # /autofix-pr 命令
├── bughunter.rs           # /bughunter 命令
├── thinkback.rs           # /thinkback 命令
├── add_dir.rs             # /add-dir 命令
├── copy.rs                # /copy 命令
├── terminal_setup.rs      # /terminalSetup 命令
├── ide.rs                 # /ide 命令
├── good_claude.rs         # /good-claude 命令
├── poor.rs                # /poor 命令
├── advisor.rs             # /advisor 命令
├── buddy.rs               # /buddy 命令
├── ctx_viz.rs             # /ctx_viz 命令
├── ant_trace.rs           # /ant-trace 命令
├── backfill_sessions.rs   # /backfill-sessions 命令
├── break_cache.rs         # /break-cache 命令
├── claim_main.rs          # /claim-main 命令
├── oauth_refresh.rs       # /oauth-refresh 命令
└── trait.rs               # SlashCommand trait 定义
```

## Dependencies

- 依赖 `specs/003-claude-code-tools-alignment` 的工具系统
- 依赖 `specs/004-security-permission-framework` 的权限系统
