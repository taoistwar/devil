# Feature Specification: Claude Code Context Alignment

**Feature Branch**: `006-claude-code-context-alignment`
**Created**: 2026-04-19
**Status**: Draft
**Input**: User description: "位于 references/claude-code/src/context.ts 的功能，需要对齐到当前项目。"

## Summary

将 Claude Code 的上下文注入功能对齐到 devil-agent。上下文在每次对话开始时被预处理并添加到对话中，包括 Git 状态、当前日期和 Memory 文件（CLAUDE.md）内容。

## User Scenarios & Testing

### User Story 1 - Git Status Context Injection (Priority: P1)

AI Agent 自动收集并注入当前 Git 仓库状态到系统上下文，包括分支名称、主分支、状态和最近提交。

**Why this priority**: Git 状态是开发者工作流的基础信息，帮助 AI 了解当前代码库状态。

**Independent Test**: 执行 `git status` 和 `git log` 命令，验证输出包含正确的分支信息。

**Acceptance Scenarios**:

1. **Given** 当前目录是 Git 仓库，**When** AI Agent 启动对话，**Then** Git 状态信息被包含在系统上下文中
2. **Given** 当前目录不是 Git 仓库，**When** AI Agent 启动对话，**Then** Git 状态信息被跳过
3. **Given** Git status 输出超过 2000 字符，**When** AI Agent 启动对话，**Then** 输出被截断并显示提示

---

### User Story 2 - Current Date Context (Priority: P1)

AI Agent 自动注入当前日期到用户上下文，帮助 AI 了解时间背景。

**Why this priority**: 日期信息对于理解代码修改时间、项目进度和任务规划至关重要。

**Independent Test**: 验证系统上下文包含格式化的当前日期。

**Acceptance Scenarios**:

1. **Given** AI Agent 正常运行，**When** 对话开始，**Then** 当前日期被包含在上下文中

---

### User Story 3 - Memory Files (CLAUDE.md) Context (Priority: P2)

AI Agent 自动发现并加载项目中的 CLAUDE.md 文件，将内容注入到上下文。

**Why this priority**: CLAUDE.md 文件允许开发者为 AI 提供项目特定的指导和上下文。

**Independent Test**: 创建测试用的 CLAUDE.md 文件，验证其内容被正确加载。

**Acceptance Scenarios**:

1. **Given** 项目根目录存在 CLAUDE.md，**When** AI Agent 启动对话，**Then** 文件内容被包含在上下文中
2. **Given** 项目不存在 CLAUDE.md，**When** AI Agent 启动对话，**Then** Memory 文件上下文为空
3. **Given** 环境变量 CLAUDE_CODE_DISABLE_CLAUDE_MDS=1，**When** AI Agent 启动对话，**Then** Memory 文件被禁用

---

### User Story 4 - Cache Breaker / System Prompt Injection (Priority: P3)

支持系统提示注入功能，用于强制刷新 AI 的缓存上下文。

**Why this priority**: 用于调试和特殊场景，需要绕过正常缓存机制。

**Independent Test**: 设置 system prompt injection，验证缓存被正确清除。

**Acceptance Scenarios**:

1. **Given** system prompt injection 被设置，**When** 对话开始，**Then** 缓存被清除并应用新的 injection

---

### User Story 5 - Bare Mode Support (Priority: P3)

在 --bare 模式下，AI Agent 跳过自动发现行为，但仍然处理显式添加的目录。

**Why this priority**: 与 Claude Code 行为保持一致，支持高级用户控制。

**Independent Test**: 使用 --bare 模式启动，验证行为符合预期。

**Acceptance Scenarios**:

1. **Given** 使用 --bare 模式且无显式添加目录，**When** AI Agent 启动，**Then** 跳过 Memory 文件自动发现
2. **Given** 使用 --bare 模式且有显式添加目录，**When** AI Agent 启动，**Then** 仍然处理显式添加的目录

---

### Edge Cases

- Git 命令执行失败时如何处理？
- CLAUDE.md 文件不存在或为空时如何处理？
- Git status 输出被截断后，用户如何获取完整信息？

## Requirements

### Functional Requirements

- **FR-001**: 系统必须在对话开始时收集 Git 状态信息（分支、主分支、状态、最近提交）
- **FR-002**: 系统必须在上下文中包含当前日期，格式为 ISO 日期
- **FR-003**: 系统必须自动发现并加载项目中的 CLAUDE.md 文件
- **FR-004**: 系统必须支持通过环境变量 CLAUDE_CODE_DISABLE_CLAUDE_MDS 禁用 Memory 文件
- **FR-005**: Git status 输出超过 2000 字符时必须截断，并提供替代方案提示
- **FR-006**: 系统上下文必须被缓存以提高性能
- **FR-007**: 系统必须支持缓存清除机制（system prompt injection）
- **FR-008**: 在 bare 模式下，系统必须跳过自动发现但处理显式添加的目录
- **FR-009**: 非 Git 目录中启动时，系统必须优雅地跳过 Git 状态收集

### Key Entities

- **SystemContext**: 系统级上下文，包含 Git 状态和缓存断路器
- **UserContext**: 用户级上下文，包含 Memory 文件内容和当前日期
- **GitStatus**: Git 仓库状态（分支、主分支、状态、最近提交）
- **MemoryFiles**: CLAUDE.md 文件的集合及其内容

## Success Criteria

### Measurable Outcomes

- **SC-001**: Git 仓库中启动对话时，上下文中包含正确的分支名称
- **SC-002**: 所有对话中都包含当前日期信息
- **SC-003**: 存在 CLAUDE.md 的项目中，内容被正确加载到上下文
- **SC-004**: Git status 超过 2000 字符时自动截断
- **SC-005**: 上下文缓存正常工作，重复调用不触发额外文件系统/Git 操作
- **SC-006**: 设置 injection 后缓存被正确清除

## Assumptions

- 用户主要在 Git 仓库中使用 AI Agent
- CLAUDE.md 文件遵循标准格式（Markdown）
- 日期格式使用 ISO 8601 标准（YYYY-MM-DD）
- Git 命令在 PATH 中可用
