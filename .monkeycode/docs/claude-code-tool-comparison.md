# Claude Code 工具系统对比分析

## 概述

本文档对比分析了 `taoistwar/claude-code` 仓库的工具系统实现与当前 `devil` 项目中的工具系统差异，为后续优化提供指导。

---

## 一、核心架构对比

### 1.1 Tool 类型定义

| 特性 | Claude Code | Devil (当前) | 差异说明 |
|------|-------------|--------------|----------|
| **语言** | TypeScript/JavaScript | Rust | 语言差异导致实现方式不同 |
| **泛型参数** | `<Input, Output, Progress>` | `<Input, Output, Progress>` | 三者分离设计一致 |
| **Schema 定义** | Zod v4 | serde_json::Value | TS 使用 Zod 运行时验证，Rust 使用 JSON Schema |
| **默认方法** | `buildTool` + `TOOL_DEFAULTS` | `buildTool` + fail-closed | 设计理念一致 |
| **类型计算** | 复杂的 TypeScript 类型体操 | 简化版本 | TS 版本更精细 |

### 1.2 buildTool 工厂函数对比

**Claude Code 实现 (TypeScript):**
```typescript
const TOOL_DEFAULTS = {
  isEnabled: () => true,
  isConcurrencySafe: (_input?: unknown) => false,
  isReadOnly: (_input?: unknown) => false,
  isDestructive: (_input?: unknown) => false,
  checkPermissions: (input, _ctx) => 
    Promise.resolve({ behavior: 'allow', updatedInput: input }),
  toAutoClassifierInput: (_input?: unknown) => '',
  userFacingName: (_input?: unknown) => '',
}

export function buildTool<D extends AnyToolDef>(def: D): BuiltTool<D> {
  return {
    ...TOOL_DEFAULTS,
    userFacingName: () => def.name,
    ...def,
  } as BuiltTool<D>
}
```

**Devil 实现 (Rust):**
```rust
pub struct ToolBuilder<I, O> {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    // ... 更多字段
    concurrency_safe: bool,  // 默认 false - fail-closed
    read_only: bool,         // 默认 false - fail-closed
}

impl<I, O> ToolBuilder<I, O> {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            concurrency_safe: false, // fail-closed
            read_only: false,        // fail-closed
            // ...
        }
    }
    
    pub fn concurrency_safe(mut self) -> Self {
        self.concurrency_safe = true;
        self
    }
}
```

**差异分析:**
- Claude Code 使用对象展开合并默认值
- Devil 使用 Builder 模式链式调用
- 两者都遵循 fail-closed 原则

---

## 二、工具属性对比

### 2.1 核心属性对照表

| 属性/方法 | Claude Code | Devil | 实现差异 |
|-----------|-------------|-------|----------|
| `name` | ✓ | ✓ | 一致 |
| `aliases` | ✓ | ✓ | 一致 |
| `inputSchema` | Zod Schema | JSON Schema | TS 有类型推断优势 |
| `validateInput` | ✓ | ✓ | 一致 |
| `checkPermissions` | 异步，返回 `PermissionResult` | 同步/异步，返回 `PermissionCheckResult` | Claude Code 更复杂 |
| `isConcurrencySafe` | 方法 `(input) => bool` | 方法 `() -> bool` | Claude Code 可基于输入判断 |
| `isReadOnly` | 方法 `(input) => bool` | 方法 `() -> bool` | Claude Code 更灵活 |
| `isDestructive` | ✓ (可选) | ✓ | 一致 |
| `isEnabled` | ✓ | ✗ | Devil 缺失 |
| `interruptBehavior` | `'cancel' | 'block'` | ✗ | Devil 缺失 |
| `shouldDefer` | ✓ | ✗ | Devil 缺失延迟发现 |
| `alwaysLoad` | ✓ | ✓ | 一致 |
| `maxResultSizeChars` | ✓ | ✗ | Devil 缺失结果大小限制 |
| `strict` | ✓ | ✗ | Devil 缺失严格模式 |
| `getPath` | ✓ (可选) | ✗ | Devil 缺失 |
| `toAutoClassifierInput` | ✓ | ✗ | Devil 缺失分类器输入 |
| `getActivityDescription` | ✓ | ✗ | Devil 缺失活动描述 |
| `isSearchOrReadCommand` | ✓ | ✗ | Devil 缺失语义分析 |

### 2.2 关键缺失功能

#### 2.2.1 isEnabled - 工具启用状态
**Claude Code:**
```typescript
isEnabled(): boolean
```
用于功能开关控制，支持编译期死代码消除。

**Devil:** 缺失，需添加。

#### 2.2.2 interruptBehavior - 中断行为
**Claude Code:**
```typescript
interruptBehavior?(): 'cancel' | 'block'
```
- `'cancel'`: 用户发送新消息时停止工具并丢弃结果
- `'block'`: 继续运行，新消息等待

**Devil:** 缺失，需添加。

#### 2.2.3 shouldDefer - 延迟发现
**Claude Code:**
```typescript
readonly shouldDefer?: boolean
```
延迟工具发现机制，减少初始 prompt token 消耗。

**Devil:** 缺失，但有 `always_load` 实现部分功能。

#### 2.2.4 isSearchOrReadCommand - 语义分析
**Claude Code:**
```typescript
isSearchOrReadCommand?(input): {
  isSearch: boolean
  isRead: boolean
  isList: boolean
}
```
用于 UI 折叠展示，识别 grep、find、cat 等命令。

**Devil:** 缺失，BashTool 需要实现 AST 解析。

---

## 三、BashTool 详细对比

### 3.1 功能对比

| 功能 | Claude Code | Devil | 差异 |
|------|-------------|-------|------|
| 命令执行 | ✓ (spawnShellTask) | ✗ (TODO) | Devil 未实现 |
| AST 解析 | ✓ (`parseForSecurity`) | ✗ | Devil 缺失 |
| 语义分析 | ✓ (`commandSemantics.ts`) | ✗ | Devil 缺失 |
| 错误传播 | ✓ (取消并行 Bash) | ✗ | Devil 缺失 |
| 中断行为 | ✓ (可配置) | ✗ | Devil 缺失 |
| 沙盒集成 | ✓ (`SandboxManager`) | ✗ | Devil 缺失 |
| 后台执行 | ✓ (`run_in_background`) | ✗ | Devil 缺失 |
| 输出持久化 | ✓ (大输出存文件) | ✗ | Devil 缺失 |
| 图片输出 | ✓ (自动 resize) | ✗ | Devil 缺失 |
| sed 编辑模拟 | ✓ (`_simulatedSedEdit`) | ✗ | Devil 缺失 |
| Git 操作追踪 | ✓ (`trackGitOperations`) | ✗ | Devil 缺失 |

### 3.2 BashTool 安全特性对比

**Claude Code 安全检查:**
```typescript
// 1. 设备文件检查
const BLOCKED_DEVICE_PATHS = new Set([
  '/dev/zero', '/dev/random', '/dev/urandom',
  '/dev/stdin', '/dev/tty', // 阻塞输入
  '/dev/stdout', '/dev/stderr', // 无意义
])

// 2. 命令语义分析
function isSearchOrReadCommand(command: string) {
  // AST 解析，识别 grep、find、cat 等
}

// 3. 静默命令检测
function isSilentBashCommand(command: string): boolean {
  // 检测 cp、mv、mkdir 等无输出命令
}

// 4. 自动后台检测
function isAutobackgroundingAllowed(command: string): boolean {
  // sleep 等命令不允许自动后台
}

// 5. 沙盒检查
const shouldUseSandbox = shouldUseSandbox(command)
```

**Devil 当前实现:**
```rust
// 仅基础权限检查
fn has_permission(&self, input: &BashInput, _ctx: &ToolContext) -> bool {
    let dangerous_patterns = ["rm -rf", "dd if=", "mkfs", ":(){:|:&};:"];
    for pattern in &dangerous_patterns {
        if input.command.contains(pattern) {
            return false;
        }
    }
    true
}
```

**差异:** Devil 的安全检查过于简单，需添加 AST 解析和语义分析。

---

## 四、文件工具对比

### 4.1 FileReadTool

| 功能 | Claude Code | Devil |
|------|-------------|-------|
| 文件缓存 | ✓ (`FileStateCache`) | ✓ (简化版) |
| PDF 支持 | ✓ (提取/压缩) | ✗ |
| 图片处理 | ✓ (resize/downsample) | ✗ |
| Notebook 支持 | ✓ (Jupyter) | ✗ |
| 二进制文件检测 | ✓ | ✗ |
| 编码检测 | ✓ | ✗ |
| 设备文件检查 | ✓ | ✗ |
| 路径阻塞检测 | ✓ | ✗ |
| 大文件截断 | ✓ (`max_lines`) | 部分 |
| Token 估算 | ✓ (`countTokensWithAPI`) | 简化估算 |

**Claude Code 特色功能:**
- PDF 提取：超过阈值时调用 `extractPDFPages`
- 图片处理：压缩到 token 限制内
- 设备文件阻塞：防止 `/dev/zero` 等导致进程挂起
- macOS 截图路径解析：处理不同的空格字符

### 4.2 FileEditTool

| 功能 | Claude Code | Devil |
|------|-------------|-------|
| 精确字符串替换 | ✓ (`old_string -> new_string`) | ✓ |
| 行号范围 | ✗ (有意为之) | ✗ |
| sed 模拟编辑 | ✓ (`_simulatedSedEdit`) | ✗ |
| 文件历史追踪 | ✓ (`fileHistoryTrackEdit`) | ✗ |
| VS Code 通知 | ✓ (`notifyVscodeFileUpdated`) | ✗ |
| 破坏性判断 | ✓ (删除大量代码) | ✓ |
| 多处出现处理 | ✓ (`insert_index`) | ✓ |

### 4.3 FileWriteTool

| 功能 | Claude Code | Devil |
|------|-------------|-------|
| 追加模式 | ✓ | ✓ |
| 编码检测 | ✓ | ✗ |
| 行尾检测 | ✓ | ✗ |
| 文件历史 | ✓ | ✗ |
| 权限检查 | 严格 | 简化 |

---

## 五、搜索工具对比

### 5.1 GlobTool

| 功能 | Claude Code | Devil |
|------|-------------|-------|
| 底层库 | `fast-glob` (TS) | 待实现 |
| ignore 模式 | ✓ | 部分 |
| 最大结果数 | ✓ | ✓ |
| 并发安全 | ✓ | ✓ |

### 5.2 GrepTool

| 功能 | Claude Code | Devil |
|------|-------------|-------|
| 底层库 | `ripgrep` (rust) | 待实现 |
| 文件类型过滤 | ✓ | ✗ |
| 输出模式 | 多种 | 简化 |
| 大小写控制 | ✓ | ✗ |

---

## 六、UI 渲染对比

### 6.1 渲染方法

**Claude Code (React 组件):**
```typescript
renderToolUseMessage(input): React.ReactNode
renderToolUseProgressMessage(progress): React.ReactNode
renderToolResultMessage(content): React.ReactNode
renderToolUseRejectedMessage(reason): React.ReactNode
renderToolUseErrorMessage(error): React.ReactNode
renderGroupedToolUse(tools): React.ReactNode
```

**Devil (字符串模板):**
```rust
fn render_use_message(&self, input: &Input) -> String
fn render_progress_message(&self, progress: &Progress) -> Option<String>
fn render_result_message(&self, result: &Result) -> String
```

**差异:** 
- Claude Code 使用 React 组件，支持丰富的 UI 效果
- Devil 使用字符串模板，功能简化

### 6.2 活动描述

**Claude Code:**
```typescript
getActivityDescription?(input): string | null
// 示例："Reading src/foo.ts", "Running bun test"
```

用于 spinner 显示，提供实时活动状态。

**Devil:** 缺失。

---

## 七、权限系统对比

### 7.1 权限检查流程

**Claude Code (三层管线):**
```
1. validateInput() - 输入验证
   ↓
2. checkPermissions() - 权限检查 (异步)
   ↓
3. canUseTool() / hasPermission() - 运行时判断
```

**PermissionResult 类型:**
```typescript
type PermissionResult = 
  | { behavior: 'allow', updatedInput?: object }
  | { behavior: 'deny', reason: string }
  | { behavior: 'ask', prompt: PermissionPrompt }
```

**Devil (简化版):**
```rust
fn validate_input_permissions(&self, input: &Input) -> InputValidationResult
fn has_permission(&self, input: &Input, ctx: &ToolContext) -> bool
fn check_permissions(&self, input: &Input, ctx: &ToolContext) -> PermissionCheckResult
```

**PermissionCheckResult:**
```rust
pub struct PermissionCheckResult {
    pub has_permission: bool,
    pub denial_reason: Option<String>,
    pub requires_confirmation: bool,
}
```

### 7.2 Bash 权限规则

**Claude Code:**
```typescript
// bashPermissions.ts

// 通配符匹配
function matchWildcardPattern(pattern: string, command: string): boolean

// 前缀提取
function permissionRuleExtractPrefix(rule: string): string

// 权限检查
function bashToolHasPermission(command: string, rules: Rules): boolean
```

**Devil:** 简化的字符串包含检查。

---

## 八、工具注册与发现对比

### 8.1 getAllBaseTools()

**Claude Code:**
```typescript
// packages/builtin-tools/src/tools/src/index.ts
export function getAllBaseTools(): Tools {
  return [
    bashTool,
    readTool,
    editTool,
    writeTool,
    globTool,
    grepTool,
    // ... 45+ tools
  ]
}
```

**死代码消除:**
```typescript
// 条件导入
if (FEATURE_FLAG_internal_tools) {
  tools.push(internalTool)
}
```

**Devil:** 
```rust
pub fn get_all_base_tools(&self) -> Vec<ToolMetadata>
```

### 8.2 工具过滤管线

**Claude Code:**
```
getAllBaseTools()
  ↓
filterByMode() - 简单/普通模式
  ↓
filterByDenyRules() - blanket deny
  ↓
filterByEnabled() - 启用状态
  ↓
pool - 合并内建 + MCP，排序去重
  ↓
API
```

**Devil:** `ToolFilter::filter_all()` 实现类似流程。

---

## 九、StreamingToolExecutor 对比

### 9.1 四阶段状态机

**Claude Code:**
```typescript
// 工具状态
{
  queued:     // 已入队，等待执行条件
  executing:  // 正在执行
  completed:  // 执行完成，等待顺序输出
  yielded:    // 结果已产出
}
```

**Devil:** `ToolExecutionState` 枚举完全对齐。

### 9.2 并发分区

**Claude Code:**
```typescript
function partitionToolCalls(toolCalls, isConcurrencySafe) {
  // 分区算法实现
}
```

**Devil:** `ConcurrentPartitioner::partition()` 实现相同逻辑。

### 9.3 差异点

| 特性 | Claude Code | Devil |
|------|-------------|-------|
| 顺序保证 | ✓ | ✓ |
| 错误传播 | ✓ (Bash 失败取消兄弟) | ✓ (计划中) |
| 进度即时产出 | ✓ | ✓ |
| 丢弃机制 | ✓ (流式回退) | ✓ |
| 信号传播 | ✓ (层级化取消) | 待实现 |
| 取消控制器 | ✓ (独立子控制器) | 待实现 |

---

## 十、缺失工具清单

Claude Code 有但 Devil 缺失的工具：

| 工具类别 | 工具名称 | 说明 |
|---------|----------|------|
| **Agent** | AgentTool | 子智能体入口 |
| **任务** | TodoWriteTool | TODO 列表管理 |
| **任务** | TaskCreateTool | 任务创建 |
| **任务** | TaskOutputTool | 任务结果输出 |
| **计划** | EnterPlanModeTool | 进入计划模式 |
| **计划** | ExitPlanModeTool | 退出计划模式 |
| **MCP** | ListMcpResourcesTool | MCP 资源列表 |
| **MCP** | ReadMcpResourceTool | MCP 资源读取 |
| **网络** | WebFetchTool | URL 内容获取 |
| **网络** | WebSearchTool | 网络搜索 |
| **笔记本** | NotebookEditTool | Jupyter 编辑 |
| **技能** | SkillTool | 调用 slash command |
| **交互** | AskUserQuestionTool | 向用户提问 |
| **其他** | BriefTool | 消息发送 |
| **其他** | ConfigTool | 配置修改 |

---

## 十一、优化建议

### 11.1 高优先级

1. **添加工具启用状态 (`isEnabled`)**
   - 支持功能开关
   - 实现编译期死代码消除

2. **中断行为 (`interruptBehavior`)**
   - 支持 `'cancel'` 和 `'block'` 模式
   - 用于长时间运行任务

3. **BashTool AST 解析**
   - 语义分析 (`isSearchOrReadCommand`)
   - 错误传播机制
   - 沙盒集成

4. **延迟发现机制**
   - 工具数量超过 50 时启用
   - ToolSearchTool 按需加载

5. **结果大小限制**
   - `maxResultSizeChars` 属性
   - 大输出持久化到文件

### 11.2 中优先级

6. **文件工具增强**
   - PDF 处理支持
   - 图片 resize
   - Notebook 支持
   - 设备文件检查

7. **语义分析**
   - Bash 命令分类
   - UI 折叠支持

8. **活动描述**
   - `getActivityDescription` 方法
   - Spinner 状态显示

9. **信号传播优化**
   - 层级化取消控制器
   - 兄弟工具取消

### 11.3 低优先级

10. **UI 渲染增强**
    - 从字符串模板升级到模板引擎
    - 支持进度条、折叠面板

11. **新增工具**
    - TodoWriteTool
    - WebFetchTool
    - SkillTool

---

## 十二、总结

### 核心理念一致性

| 设计理念 | Claude Code | Devil | 状态 |
|---------|-------------|-------|------|
| 五要素协议 | ✓ | ✓ | 对齐 |
| buildTool 工厂 | ✓ | ✓ | 对齐 |
| fail-closed 原则 | ✓ | ✓ | 对齐 |
| 并发分区策略 | ✓ | ✓ | 对齐 |
| StreamingToolExecutor | ✓ | ✓ | 对齐 |
| 三层权限检查 | ✓ | ✓ (简化) | 基本对齐 |

### 主要差距

1. **安全性**: BashTool 的 AST 解析和语义分析
2. **鲁棒性**: 错误传播、取消信号链
3. **性能**: 延迟发现、结果持久化
4. **功能**: 缺失 15+ 个内建工具
5. **UI**: 字符串模板 vs React 组件

### 行动计划

```
Phase 1 (1-2 周):
- 实现 isEnabled 和 interruptBehavior
- 添加 BashTool AST 解析
- 实现错误传播机制

Phase 2 (2-3 周):
- 实现延迟发现
- 添加结果大小限制和持久化
- 增强文件工具功能

Phase 3 (3-4 周):
- 补充缺失的内建工具
- 优化信号传播
- UI 渲染增强
```

---

*文档生成时间：2026-04-16*
*对比版本：Claude Code (main) vs Devil (260416-feat-refactor-multi-crate-architecture)*
