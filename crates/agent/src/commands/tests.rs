//! Unit tests for slash commands

#[cfg(test)]
mod tests {
    use crate::commands::advanced::desktop::DesktopCommand;
    use crate::commands::advanced::diff::DiffCommand;
    use crate::commands::advanced::fast::FastCommand;
    use crate::commands::advanced::hooks::HooksCommand;
    use crate::commands::advanced::mcp::McpCommand;
    use crate::commands::advanced::memory::MemoryCommand;
    use crate::commands::advanced::permissions::PermissionsCommand;
    use crate::commands::advanced::plan::PlanCommand;
    use crate::commands::advanced::review::ReviewCommand;
    use crate::commands::advanced::share::ShareCommand;
    use crate::commands::advanced::skills::SkillsCommand;
    use crate::commands::advanced::stickers::StickersCommand;
    use crate::commands::advanced::tasks::TasksCommand;
    use crate::commands::advanced::upgrade::UpgradeCommand;
    use crate::commands::advanced::voice::VoiceCommand;
    use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
    use crate::commands::config::cmd::ConfigCommand;
    use crate::commands::config::login::LoginCommand;
    use crate::commands::config::logout::LogoutCommand;
    use crate::commands::config::theme::ThemeCommand;
    use crate::commands::core::clear::ClearCommand;
    use crate::commands::core::compact::CompactCommand;
    use crate::commands::core::cost::CostCommand;
    use crate::commands::core::doctor::DoctorCommand;
    use crate::commands::core::exit::ExitCommand;
    use crate::commands::core::help::HelpCommand;
    use crate::commands::core::model::ModelCommand;
    use crate::commands::core::resume::ResumeCommand;
    use crate::commands::edit::add_dir::AddDirCommand;
    use crate::commands::edit::autofix_pr::AutofixPrCommand;
    use crate::commands::edit::bughunter::BughunterCommand;
    use crate::commands::edit::context::ContextCommand;
    use crate::commands::edit::copy::CopyCommand;
    use crate::commands::edit::effort::EffortCommand;
    use crate::commands::edit::env::EnvCommand;
    use crate::commands::edit::files::FilesCommand;
    use crate::commands::edit::ide::IdeCommand;
    use crate::commands::edit::passes::PassesCommand;
    use crate::commands::edit::rename::RenameCommand;
    use crate::commands::edit::rewind::RewindCommand;
    use crate::commands::edit::src::SrcCommand;
    use crate::commands::edit::summary::SummaryCommand;
    use crate::commands::edit::tag::TagCommand;
    use crate::commands::edit::terminal_setup::TerminalSetupCommand;
    use crate::commands::edit::thinkback::ThinkbackCommand;
    use crate::commands::edit::vim::VimCommand;

    #[tokio::test]
    async fn test_help_command() {
        let cmd = HelpCommand::new();
        assert_eq!(cmd.name(), "help");
        assert_eq!(cmd.description(), "显示帮助信息");
        assert!(!cmd.aliases().is_empty());

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
        assert!(result.output.is_some());
    }

    #[tokio::test]
    async fn test_help_command_with_arg() {
        let cmd = HelpCommand::new();
        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["compact"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_help_command_unknown() {
        let cmd = HelpCommand::new();
        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["unknown_cmd"]).await;
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_compact_command() {
        let cmd = CompactCommand::new();
        assert_eq!(cmd.name(), "compact");
        assert_eq!(cmd.description(), "手动压缩上下文");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_clear_command() {
        let cmd = ClearCommand::new();
        assert_eq!(cmd.name(), "clear");
        assert_eq!(cmd.description(), "清除当前对话");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_model_command_no_args() {
        let cmd = ModelCommand::new();
        assert_eq!(cmd.name(), "model");
        assert_eq!(cmd.description(), "切换 AI 模型");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_model_command_with_args() {
        let cmd = ModelCommand::new();
        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["claude-opus"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_exit_command() {
        let cmd = ExitCommand::new();
        assert_eq!(cmd.name(), "exit");
        assert!(!cmd.aliases().is_empty());

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_resume_command() {
        let cmd = ResumeCommand::new();
        assert_eq!(cmd.name(), "resume");
        assert_eq!(cmd.description(), "恢复之前的会话");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["session-123"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_doctor_command() {
        let cmd = DoctorCommand::new();
        assert_eq!(cmd.name(), "doctor");
        assert_eq!(cmd.description(), "运行系统诊断");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_cost_command() {
        let cmd = CostCommand::new();
        assert_eq!(cmd.name(), "cost");
        assert_eq!(cmd.description(), "查看 API 使用费用");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_config_command() {
        let cmd = ConfigCommand::new();
        assert_eq!(cmd.name(), "config");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_login_command() {
        let cmd = LoginCommand::new();
        assert_eq!(cmd.name(), "login");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_logout_command() {
        let cmd = LogoutCommand::new();
        assert_eq!(cmd.name(), "logout");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_theme_command() {
        let cmd = ThemeCommand::new();
        assert_eq!(cmd.name(), "theme");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["dark"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_diff_command() {
        let cmd = DiffCommand::new();
        assert_eq!(cmd.name(), "diff");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["test.rs"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_fast_command() {
        let cmd = FastCommand::new();
        assert_eq!(cmd.name(), "fast");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_plan_command() {
        let cmd = PlanCommand::new();
        assert_eq!(cmd.name(), "plan");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_review_command() {
        let cmd = ReviewCommand::new();
        assert_eq!(cmd.name(), "review");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["test.rs"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_share_command() {
        let cmd = ShareCommand::new();
        assert_eq!(cmd.name(), "share");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_mcp_command() {
        let cmd = McpCommand::new();
        assert_eq!(cmd.name(), "mcp");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_memory_command() {
        let cmd = MemoryCommand::new();
        assert_eq!(cmd.name(), "memory");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_permissions_command() {
        let cmd = PermissionsCommand::new();
        assert_eq!(cmd.name(), "permissions");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_skills_command() {
        let cmd = SkillsCommand::new();
        assert_eq!(cmd.name(), "skills");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_tasks_command() {
        let cmd = TasksCommand::new();
        assert_eq!(cmd.name(), "tasks");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_voice_command() {
        let cmd = VoiceCommand::new();
        assert_eq!(cmd.name(), "voice");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_upgrade_command() {
        let cmd = UpgradeCommand::new();
        assert_eq!(cmd.name(), "upgrade");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_desktop_command() {
        let cmd = DesktopCommand::new();
        assert_eq!(cmd.name(), "desktop");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_stickers_command() {
        let cmd = StickersCommand::new();
        assert_eq!(cmd.name(), "stickers");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["thumbs_up"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_hooks_command() {
        let cmd = HooksCommand::new();
        assert_eq!(cmd.name(), "hooks");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_vim_command() {
        let cmd = VimCommand::new();
        assert_eq!(cmd.name(), "vim");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["test.rs"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_rewind_command() {
        let cmd = RewindCommand::new();
        assert_eq!(cmd.name(), "rewind");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["5"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_context_command() {
        let cmd = ContextCommand::new();
        assert_eq!(cmd.name(), "context");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_summary_command() {
        let cmd = SummaryCommand::new();
        assert_eq!(cmd.name(), "summary");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_tag_command() {
        let cmd = TagCommand::new();
        assert_eq!(cmd.name(), "tag");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["v1.0"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_rename_command() {
        let cmd = RenameCommand::new();
        assert_eq!(cmd.name(), "rename");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["new_name"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_env_command() {
        let cmd = EnvCommand::new();
        assert_eq!(cmd.name(), "env");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["KEY", "value"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_files_command() {
        let cmd = FilesCommand::new();
        assert_eq!(cmd.name(), "files");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_add_dir_command() {
        let cmd = AddDirCommand::new();
        assert_eq!(cmd.name(), "add-dir");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["src"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_copy_command() {
        let cmd = CopyCommand::new();
        assert_eq!(cmd.name(), "copy");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &["text to copy"]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_src_command() {
        let cmd = SrcCommand::new();
        assert_eq!(cmd.name(), "src");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_ide_command() {
        let cmd = IdeCommand::new();
        assert_eq!(cmd.name(), "ide");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_terminal_setup_command() {
        let cmd = TerminalSetupCommand::new();
        assert_eq!(cmd.name(), "terminalSetup");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_passes_command() {
        let cmd = PassesCommand::new();
        assert_eq!(cmd.name(), "passes");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_autofix_pr_command() {
        let cmd = AutofixPrCommand::new();
        assert_eq!(cmd.name(), "autofix-pr");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_bughunter_command() {
        let cmd = BughunterCommand::new();
        assert_eq!(cmd.name(), "bughunter");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_effort_command() {
        let cmd = EffortCommand::new();
        assert_eq!(cmd.name(), "effort");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_thinkback_command() {
        let cmd = ThinkbackCommand::new();
        assert_eq!(cmd.name(), "thinkback");

        let ctx = CommandContext::default();
        let result = cmd.execute(&ctx, &[]).await;
        assert!(result.success);
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("test output");
        assert!(result.success);
        assert_eq!(result.output, Some("test output".to_string()));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_command_result_error() {
        let result = CommandResult::error("error message");
        assert!(!result.success);
        assert!(result.output.is_none());
        assert_eq!(result.error, Some("error message".to_string()));
    }

    #[test]
    fn test_command_result_success_with_data() {
        let data = serde_json::json!({"key": "value"});
        let result = CommandResult::success_with_data("test", data.clone());
        assert!(result.success);
        assert!(result.data.is_some());
    }
}
