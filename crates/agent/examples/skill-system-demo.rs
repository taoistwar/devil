//! 技能系统使用示例
//!
//! 演示如何使用 SkillLoader、SkillExecutor 和 SkillPermissionChecker

use devil_agent::skills::{
    PermissionRule, RuleSource, RuleType, SkillExecutor, SkillLoader, SkillPermissionChecker,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MonkeyCode Skill System Demo ===\n");

    // 1. 创建技能加载器
    let mut loader = SkillLoader::new();

    // 2. 从磁盘加载所有技能
    println!("Loading skills from disk...");
    match loader.load_all_from_disk() {
        Ok(count) => println!("Loaded {} skills from disk\n", count),
        Err(e) => eprintln!("Failed to load skills: {}", e),
    }

    // 3. 注册内置技能
    println!("Loading bundled skills...");
    let bundled_skills = devil_agent::skills::bundled::register_bundled_skills();
    println!("Loaded {} bundled skills\n", bundled_skills.len());

    // 4. 激活匹配路径的条件技能
    let file_path = "src/auth/validate.rs";
    println!("Activating skills for path: {}", file_path);
    let activated = loader.activate_skills_for_path(file_path);
    println!("Activated {} skills for path\n", activated.len());

    // 5. 创建权限检查器
    let checker = SkillPermissionChecker::new()
        .with_allow_rules(vec![
            PermissionRule::new("skill:safe-*", RuleType::Skill).with_source(RuleSource::User)
        ])
        .with_deny_rules(vec![PermissionRule::new(
            "skill:dangerous-*",
            RuleType::Skill,
        )
        .with_source(RuleSource::User)])
        .with_remote_canonical_allow(true);

    // 6. 获取所有可用技能
    let skills = loader.get_all_skills();
    println!("=== Available Skills ===");
    for skill in skills {
        println!("- {} (source: {:?})", skill.name, skill.source);

        // 执行权限检查
        match checker.check(skill) {
            devil_agent::skills::PermissionCheckResult::Allow => {
                println!("  ✓ Permission: Allow");
            }
            devil_agent::skills::PermissionCheckResult::Deny(reason) => {
                println!("  ✗ Permission: Deny - {}", reason);
            }
            devil_agent::skills::PermissionCheckResult::Ask {
                reason,
                suggested_rules,
            } => {
                println!("  ? Permission: Ask - {}", reason);
                println!("    Suggested rules: {:?}", suggested_rules);
            }
        }
    }

    // 7. 创建执行器并执行技能
    println!("\n=== Executing Skills ===");
    let executor = SkillExecutor::new("demo-session-001");

    // 执行一个简单技能（如果有）
    if let Some(skill) = skills.first() {
        println!("\nExecuting skill: {}", skill.name);

        match executor.execute(skill, Some("demo argument")).await {
            Ok(result) => match result {
                devil_agent::skills::SkillExecutionResult::Inline {
                    new_messages,
                    context_modifier,
                } => {
                    println!("  Mode: Inline");
                    println!("  Messages: {}", new_messages.len());
                    println!("  Allowed tools: {:?}", context_modifier.allowed_tools);
                }
                devil_agent::skills::SkillExecutionResult::Fork {
                    result,
                    agent_messages,
                } => {
                    println!("  Mode: Fork");
                    println!("  Result: {}", result);
                    println!("  Agent messages: {}", agent_messages.len());
                }
            },
            Err(e) => eprintln!("  Error executing skill: {}", e),
        }
    }

    // 8. 使用预算管理器
    println!("\n=== Budget Management ===");
    let budget_manager = devil_agent::skills::SkillBudgetManager::new(200000, 0.2);
    println!("Context window: 200000 tokens");
    println!("Total budget: {} chars", budget_manager.total_budget());
    println!(
        "Reserved budget: {} chars",
        budget_manager.reserved_budget()
    );
    println!(
        "Available budget: {} chars",
        budget_manager.available_budget()
    );

    // 9. 动态技能发现
    println!("\n=== Dynamic Skill Discovery ===");
    let test_path = std::path::Path::new("src/components/Button.tsx");
    let discovered_dirs = SkillLoader::discover_skills_for_path(test_path);
    println!(
        "Discovered {} skill directories for {:?}",
        discovered_dirs.len(),
        test_path
    );

    println!("\n=== Demo Complete ===");

    Ok(())
}
