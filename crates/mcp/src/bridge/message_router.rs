//! 消息路由器
//!
//! 负责请求 - 响应匹配和通知广播

use super::BridgeMessage;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};

/// 消息路由器
pub struct MessageRouter {
    /// 请求 ID -> 响应通道
    pending_requests: RwLock<HashMap<String, mpsc::Sender<BridgeMessage>>>,
    /// 通知订阅者
    notification_subscribers: RwLock<Vec<mpsc::Sender<BridgeMessage>>>,
}

impl MessageRouter {
    /// 创建新的路由器
    pub fn new() -> Self {
        Self {
            pending_requests: RwLock::new(HashMap::new()),
            notification_subscribers: RwLock::new(Vec::new()),
        }
    }

    /// 注册请求
    pub async fn register_request(&self, id: &str, response_tx: mpsc::Sender<BridgeMessage>) {
        let mut requests = self.pending_requests.write().await;
        requests.insert(id.to_string(), response_tx);
        debug!("Registered pending request: {}", id);
    }

    /// 路由响应
    pub async fn route_response(&self, message: BridgeMessage) -> bool {
        if let BridgeMessage::Response { ref id, .. } = message {
            if let Some(id_str) = id.as_str() {
                let mut requests = self.pending_requests.write().await;

                if let Some(tx) = requests.remove(id_str) {
                    debug!("Routing response to request: {}", id_str);
                    return tx.send(message).await.is_ok();
                } else {
                    warn!("Received response for unknown request: {}", id_str);
                    return false;
                }
            }
        }

        false
    }

    /// 路由通知
    pub async fn route_notification(&self, message: BridgeMessage) {
        let subscribers = self.notification_subscribers.read().await;

        for subscriber in subscribers.iter() {
            subscriber.send(message.clone()).await.ok();
        }
    }

    /// 订阅通知
    pub async fn subscribe_notifications(&self) -> mpsc::Receiver<BridgeMessage> {
        let (tx, rx) = mpsc::channel(100);
        let mut subscribers = self.notification_subscribers.write().await;
        subscribers.push(tx);
        rx
    }

    /// 清除所有待处理请求
    pub async fn clear(&self) {
        let mut requests = self.pending_requests.write().await;
        requests.clear();

        let mut subscribers = self.notification_subscribers.write().await;
        subscribers.clear();

        debug!("MessageRouter cleared");
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_register_and_route_request() {
        let router = MessageRouter::new();
        let (tx, mut rx) = mpsc::channel::<BridgeMessage>(1);

        // 注册请求
        router.register_request("req-123", tx).await;

        // 路由响应
        let response = BridgeMessage::Response {
            id: json!("req-123"),
            result: Some(json!({"success": true})),
            error: None,
        };

        let success = router.route_response(response).await;
        assert!(success);

        // 接收响应
        let received = rx.recv().await.unwrap();
        match received {
            BridgeMessage::Response { result, .. } => {
                assert_eq!(result, Some(json!({"success": true})));
            }
            _ => panic!("Expected Response"),
        }
    }

    #[tokio::test]
    async fn test_route_to_unknown_request() {
        let router = MessageRouter::new();

        // 路由到不存在的请求
        let response = BridgeMessage::Response {
            id: json!("unknown-id"),
            result: None,
            error: None,
        };

        let success = router.route_response(response).await;
        assert!(!success);
    }

    #[tokio::test]
    async fn test_notification_broadcast() {
        let router = MessageRouter::new();

        // 订阅通知
        let rx1 = router.subscribe_notifications().await;
        let rx2 = router.subscribe_notifications().await;

        // 路由通知
        let notification = BridgeMessage::Notification {
            method: "test_method".to_string(),
            params: json!({"key": "value"}),
        };

        router.route_notification(notification.clone()).await;

        // 两个订阅者都应该收到
        let mut rx1 = rx1;
        let mut rx2 = rx2;

        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        match msg1 {
            BridgeMessage::Notification { method, .. } => {
                assert_eq!(method, "test_method");
            }
            _ => panic!("Expected Notification"),
        }

        match msg2 {
            BridgeMessage::Notification { method, .. } => {
                assert_eq!(method, "test_method");
            }
            _ => panic!("Expected Notification"),
        }
    }

    #[tokio::test]
    async fn test_clear() {
        let router = MessageRouter::new();
        let (tx, _rx) = mpsc::channel::<BridgeMessage>(1);

        router.register_request("req-1", tx).await;

        router.clear().await;

        // 清除后路由应该失败
        let response = BridgeMessage::Response {
            id: json!("req-1"),
            result: None,
            error: None,
        };

        let success = router.route_response(response).await;
        assert!(!success);
    }
}
