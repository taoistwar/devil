use agent::tools::task::{Task, TaskPriority, TaskStatus, TaskStore, TaskUpdate};
use agent::tools::config::{ConfigStore, ConfigGetTool, ConfigSetTool};
use agent::tools::ask::AskUserQuestionTool;
use agent::tools::task::TaskListOptions;
use agent::Tool;

#[tokio::test]
async fn test_task_store_create() {
    let store = TaskStore::new();
    let task = Task::new("Test Task".to_string());
    let created = store.create(task).await;
    
    assert_eq!(created.title, "Test Task");
    assert_eq!(created.status, TaskStatus::Pending);
    assert_eq!(created.priority, TaskPriority::Medium);
}

#[tokio::test]
async fn test_task_store_get() {
    let store = TaskStore::new();
    let task = Task::new("Test Task".to_string());
    let created = store.create(task).await;
    
    let retrieved = store.get(&created.id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test Task");
}

#[tokio::test]
async fn test_task_store_update() {
    let store = TaskStore::new();
    let task = Task::new("Original".to_string());
    let created = store.create(task).await;
    
    let update = TaskUpdate {
        title: Some("Updated".to_string()),
        description: None,
        status: Some(TaskStatus::Completed),
        priority: None,
        tags: None,
    };
    
    let updated = store.update(&created.id, update).await;
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().title, "Updated");
}

#[tokio::test]
async fn test_task_store_list() {
    let store = TaskStore::new();
    
    store.create(Task::new("Task 1".to_string())).await;
    store.create(Task::new("Task 2".to_string())).await;
    
    let all = store.list(TaskListOptions { status: None, tags: None, limit: None }).await;
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_config_store() {
    let store = ConfigStore::new();
    
    store.set("key1", serde_json::json!("value1")).await;
    
    let value = store.get("key1").await;
    assert!(value.is_some());
    assert_eq!(value.unwrap(), serde_json::json!("value1"));
    
    let list = store.list().await;
    assert!(list.contains_key("key1"));
}

#[tokio::test]
async fn test_ask_user_question_tool() {
    let tool = AskUserQuestionTool::default();
    
    let input = agent::tools::ask::AskUserQuestionInput {
        question: "What is your name?".to_string(),
        options: None,
    };
    
    let ctx = Default::default();
    let result = tool.execute(input, &ctx, None::<fn(agent::tools::tool::ToolProgress<_>)>).await;
    
    assert!(result.is_ok());
}
