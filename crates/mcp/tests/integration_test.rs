//! MCP 模块集成测试

use devil_mcp::{
    BoundedUuidSet, ClientInfo, ControlProtocol, ControlRequest, ControlResponse, EnterprisePolicy,
    IdeWhitelist, MappedTool, PermissionChecker, PermissionResult, ToolDiscoverer, UserPermissions,
};

/// 测试控制协议端到端流程
#[tokio::test]
async fn test_control_protocol_e2e() {
    // 1. 初始化连接
    let init_request = ControlRequest::Initialize {
        protocol_version: "2024-11-05".to_string(),
        capabilities: serde_json::json!({}),
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let response = ControlProtocol::handle_initialize(init_request)
        .await
        .unwrap();

    match response {
        ControlResponse::InitializeSuccess {
            protocol_version, ..
        } => {
            assert_eq!(protocol_version, "2024-11-05");
        }
        _ => panic!("Expected InitializeSuccess"),
    }

    // 2. 设置模型
    let model_response = ControlProtocol::handle_set_model("gpt-4".to_string())
        .await
        .unwrap();

    match model_response {
        ControlResponse::ModelSet { model_id } => {
            assert_eq!(model_id, "gpt-4");
        }
        _ => panic!("Expected ModelSet"),
    }

    // 3. Ping/Pong
    let ping_response = ControlProtocol::handle_ping().await.unwrap();
    assert!(matches!(ping_response, ControlResponse::Pong));
}

/// 测试四层权限检查
#[tokio::test]
async fn test_permission_checker_layers() {
    // 创建权限检查器
    let enterprise = EnterprisePolicy {
        enabled: true,
        blocked_servers: vec!["blocked-*".to_string()],
        allowed_servers: vec![],
        blocked_tools: vec!["dangerous_tool".to_string()],
        allowed_tools: vec![],
        require_admin_approval: false,
    };

    let ide = IdeWhitelist {
        allowed_servers: vec!["allowed-server".to_string()],
        allowed_tools: vec!["safe_*".to_string()],
    };

    let user = UserPermissions {
        enabled_servers: vec!["user-server".to_string()],
        disabled_servers: vec![],
        authorized_tools: vec!["user-tool".to_string()],
        blocked_tools: vec![],
    };

    let checker = PermissionChecker::new(enterprise, ide, user);

    // 测试服务器权限

    // 被企业策略阻止的服务器
    let result = checker.check_server("blocked-server").await;
    assert!(matches!(result, PermissionResult::Denied(_)));

    // 在 IDE 白名单中的服务器
    let result = checker.check_server("allowed-server").await;
    assert!(matches!(
        result,
        PermissionResult::Allowed | PermissionResult::NeedsConfirmation
    ));

    // 测试工具权限

    // 被企业策略阻止的工具
    let result = checker.check_tool("mcp__server__dangerous_tool").await;
    assert!(matches!(result, PermissionResult::Denied(_)));
}

/// 测试工具发现和映射
#[tokio::test]
async fn test_tool_discovery() {
    let discoverer = ToolDiscoverer::new();

    let raw_tools = vec![
        devil_mcp::McpTool {
            name: "bash".to_string(),
            description: Some("Execute bash commands".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                }
            }),
        },
        devil_mcp::McpTool {
            name: "read_file".to_string(),
            description: Some("Read file contents".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        },
    ];

    let mapped_tools = discoverer
        .discover_tools("test-server", raw_tools, "ServerId")
        .await
        .unwrap();

    assert_eq!(mapped_tools.len(), 2);

    // 验证名称映射
    assert_eq!(mapped_tools[0].global_name, "mcp__test-server__bash");
    assert_eq!(mapped_tools[0].original_name, "bash");
    assert_eq!(mapped_tools[0].server_id, "test-server");

    assert_eq!(mapped_tools[1].global_name, "mcp__test-server__read_file");

    // 验证工具查找
    let tool = discoverer.find_tool("mcp__test-server__bash").await;
    assert!(tool.is_some());
}

/// 测试 BoundedUUIDSet 去重
#[test]
fn test_bounded_uuid_set() {
    let mut set = BoundedUuidSet::new(5);

    // 插入 5 个 UUID
    for i in 0..5 {
        set.insert(format!("uuid-{}", i));
    }

    assert_eq!(set.len(), 5);
    assert!(set.contains("uuid-0"));
    assert!(set.contains("uuid-4"));

    // 插入第 6 个，应该移除最旧的（uuid-0）
    set.insert("uuid-5".to_string());

    assert_eq!(set.len(), 5);
    assert!(!set.contains("uuid-0"));
    assert!(set.contains("uuid-1"));
    assert!(set.contains("uuid-5"));

    // 重复插入应该被忽略
    let before_len = set.len();
    set.insert("uuid-3".to_string());
    assert_eq!(set.len(), before_len);
}

/// 测试 Unicode 清理
#[test]
fn test_clean_unicode() {
    // 包含控制字符
    let dirty = "hello\u{0000}world\u{001F}test";
    let clean = devil_mcp::clean_unicode(dirty);
    assert_eq!(clean, "helloworldtest");

    // 正常字符串
    let normal = "hello-world_123";
    assert_eq!(devil_mcp::clean_unicode(normal), normal);

    // 中文字符
    let chinese = "工具_测试";
    assert_eq!(devil_mcp::clean_unicode(chinese), chinese);
}

/// 测试工具名称解析
#[test]
fn test_tool_name_parsing() {
    use devil_mcp::{format_mcp_tool_name, parse_mcp_tool_name};

    // 格式化
    let global = format_mcp_tool_name("filesystem", "read_file");
    assert_eq!(global, "mcp__filesystem__read_file");

    // 解析
    let (server, tool) = parse_mcp_tool_name("mcp__filesystem__read_file").unwrap();
    assert_eq!(server, "filesystem");
    assert_eq!(tool, "read_file");

    // 无效格式
    assert!(parse_mcp_tool_name("invalid").is_none());
    assert!(parse_mcp_tool_name("mcp__only").is_none());
}

/// 测试 Glob 模式匹配
#[test]
fn test_glob_matching() {
    // 使用 permissions 模块的内部函数（需要导出）
    // 这里测试公共 API

    let enterprise = EnterprisePolicy {
        enabled: true,
        blocked_servers: vec!["*blocked*".to_string()],
        allowed_servers: vec!["allowed-*".to_string()],
        blocked_tools: vec![],
        allowed_tools: vec![],
        require_admin_approval: false,
    };

    let checker = PermissionChecker::new(
        enterprise,
        IdeWhitelist::default(),
        UserPermissions::default(),
    );

    // 未来添加更多 glob 测试
}

/// 测试连接状态机
#[tokio::test]
async fn test_connection_state_machine() {
    use devil_mcp::{BridgeState, McpBridge};

    let bridge = McpBridge::new("test-server", 100, 30000);

    // 初始状态
    let state = bridge.get_state().await;
    assert!(matches!(state, BridgeState::Disconnected));

    // 启动后应该变为 Connecting
    bridge.start().await.unwrap();

    // 未来添加完整的状态转换测试
}
