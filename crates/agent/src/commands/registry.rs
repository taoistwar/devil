//! 命令注册表
//!
//! 集中管理所有斜杠命令的注册和查询

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use std::collections::HashMap;
use std::sync::Arc;

/// 命令注册表
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn SlashCommand>>,
    aliases: HashMap<String, String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    pub fn register<C: SlashCommand + 'static>(&mut self, command: C) {
        let name = command.name().to_string();
        let arc: Arc<dyn SlashCommand> = Arc::new(command);
        self.commands.insert(name.clone(), arc.clone());
        for alias in arc.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
        }
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn SlashCommand>> {
        if let Some(cmd) = self.commands.get(name) {
            return Some(cmd.clone());
        }
        if let Some(real_name) = self.aliases.get(name) {
            return self.commands.get(real_name).cloned();
        }
        None
    }

    pub fn all_commands(&self) -> Vec<Arc<dyn SlashCommand>> {
        self.commands.values().cloned().collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        ctx: &CommandContext,
        args: &[&str],
    ) -> Option<CommandResult> {
        if let Some(cmd) = self.get(name) {
            Some(cmd.execute(ctx, args).await)
        } else {
            None
        }
    }

    fn register_defaults(&mut self) {
        // 核心命令
        self.register(crate::commands::core::help::HelpCommand::new());
        self.register(crate::commands::core::exit::ExitCommand::new());
        self.register(crate::commands::core::clear::ClearCommand::new());
        self.register(crate::commands::core::compact::CompactCommand::new());
        self.register(crate::commands::core::model::ModelCommand::new());
        self.register(crate::commands::core::resume::ResumeCommand::new());
        self.register(crate::commands::core::doctor::DoctorCommand::new());
        self.register(crate::commands::core::cost::CostCommand::new());

        // 配置命令
        self.register(crate::commands::config::cmd::ConfigCommand::new());
        self.register(crate::commands::config::login::LoginCommand::new());
        self.register(crate::commands::config::logout::LogoutCommand::new());
        self.register(crate::commands::config::theme::ThemeCommand::new());

        // 高级命令
        self.register(crate::commands::advanced::mcp::McpCommand::new());
        self.register(crate::commands::advanced::hooks::HooksCommand::new());
        self.register(crate::commands::advanced::skills::SkillsCommand::new());
        self.register(crate::commands::advanced::tasks::TasksCommand::new());
        self.register(crate::commands::advanced::memory::MemoryCommand::new());
        self.register(crate::commands::advanced::permissions::PermissionsCommand::new());
        self.register(crate::commands::advanced::diff::DiffCommand::new());
        self.register(crate::commands::advanced::review::ReviewCommand::new());
        self.register(crate::commands::advanced::plan::PlanCommand::new());
        self.register(crate::commands::advanced::share::ShareCommand::new());
        self.register(crate::commands::advanced::voice::VoiceCommand::new());
        self.register(crate::commands::advanced::fast::FastCommand::new());
        self.register(crate::commands::advanced::upgrade::UpgradeCommand::new());
        self.register(crate::commands::advanced::desktop::DesktopCommand::new());
        self.register(crate::commands::advanced::stickers::StickersCommand::new());
        self.register(crate::commands::advanced::coordinator::CoordinatorCommand::new());

        // 编辑命令
        self.register(crate::commands::edit::vim::VimCommand::new());
        self.register(crate::commands::edit::rewind::RewindCommand::new());
        self.register(crate::commands::edit::context::ContextCommand::new());
        self.register(crate::commands::edit::summary::SummaryCommand::new());
        self.register(crate::commands::edit::tag::TagCommand::new());
        self.register(crate::commands::edit::rename::RenameCommand::new());
        self.register(crate::commands::edit::env::EnvCommand::new());
        self.register(crate::commands::edit::files::FilesCommand::new());
        self.register(crate::commands::edit::add_dir::AddDirCommand::new());
        self.register(crate::commands::edit::copy::CopyCommand::new());
        self.register(crate::commands::edit::src::SrcCommand::new());
        self.register(crate::commands::edit::ide::IdeCommand::new());
        self.register(crate::commands::edit::terminal_setup::TerminalSetupCommand::new());
        self.register(crate::commands::edit::passes::PassesCommand::new());
        self.register(crate::commands::edit::autofix_pr::AutofixPrCommand::new());
        self.register(crate::commands::edit::bughunter::BughunterCommand::new());
        self.register(crate::commands::edit::effort::EffortCommand::new());
        self.register(crate::commands::edit::thinkback::ThinkbackCommand::new());

        // 协作命令
        self.register(crate::commands::collaboration::peers::PeersCommand::new());
        self.register(crate::commands::collaboration::send::SendCommand::new());
        self.register(crate::commands::collaboration::feedback::FeedbackCommand::new());
        self.register(crate::commands::collaboration::release_notes::ReleaseNotesCommand::new());
        self.register(crate::commands::collaboration::onboarding::OnboardingCommand::new());
        self.register(crate::commands::collaboration::attach::AttachCommand::new());
        self.register(crate::commands::collaboration::mobile::MobileCommand::new());
        self.register(crate::commands::collaboration::chrome::ChromeCommand::new());
        self.register(crate::commands::collaboration::agents::AgentsCommand::new());
        self.register(crate::commands::collaboration::workflows::WorkflowsCommand::new());
        self.register(crate::commands::collaboration::pipes::PipesCommand::new());
        self.register(crate::commands::collaboration::status::StatusCommand::new());
        self.register(crate::commands::collaboration::stats::StatsCommand::new());
        self.register(crate::commands::collaboration::issue::IssueCommand::new());
        self.register(crate::commands::collaboration::pr_comments::PrCommentsCommand::new());
        self.register(crate::commands::collaboration::btw::BtwCommand::new());
        self.register(crate::commands::collaboration::good_claude::GoodClaudeCommand::new());
        self.register(crate::commands::collaboration::poor::PoorCommand::new());
        self.register(crate::commands::collaboration::advisor::AdvisorCommand::new());
        self.register(crate::commands::collaboration::buddy::BuddyCommand::new());
        self.register(crate::commands::collaboration::ctx_viz::CtxVizCommand::new());

        // 系统命令
        self.register(crate::commands::system::plugin::PluginCommand::new());
        self.register(crate::commands::system::reload_plugins::ReloadPluginsCommand::new());
        self.register(crate::commands::system::debug_tool_call::DebugToolCallCommand::new());
        self.register(crate::commands::system::mock_limits::MockLimitsCommand::new());
        self.register(crate::commands::system::ant_trace::AntTraceCommand::new());
        self.register(crate::commands::system::backfill_sessions::BackfillSessionsCommand::new());
        self.register(crate::commands::system::break_cache::BreakCacheCommand::new());
        self.register(crate::commands::system::claim_main::ClaimMainCommand::new());
        self.register(crate::commands::system::heapdump::HeapdumpCommand::new());
        self.register(crate::commands::system::perf_issue::PerfIssueCommand::new());
        self.register(crate::commands::system::teleport::TeleportCommand::new());
        self.register(crate::commands::system::bridge::BridgeCommand::new());
        self.register(crate::commands::system::sandbox_toggle::SandboxToggleCommand::new());
        self.register(crate::commands::system::remote_setup::RemoteSetupCommand::new());
        self.register(crate::commands::system::remote_env::RemoteEnvCommand::new());
        self.register(crate::commands::system::oauth_refresh::OauthRefreshCommand::new());
        self.register(crate::commands::system::install_github_app::InstallGithubAppCommand::new());
        self.register(crate::commands::system::keybindings::KeybindingsCommand::new());
        self.register(crate::commands::system::color::ColorCommand::new());
        self.register(crate::commands::system::privacy_settings::PrivacySettingsCommand::new());
        self.register(crate::commands::system::rate_limit_options::RateLimitOptionsCommand::new());
        self.register(crate::commands::system::extra_usage::ExtraUsageCommand::new());
        self.register(crate::commands::system::usage::UsageCommand::new());
        self.register(crate::commands::system::reset_limits::ResetLimitsCommand::new());
        self.register(crate::commands::system::output_style::OutputStyleCommand::new());
        self.register(crate::commands::system::detach::DetachCommand::new());
        self.register(crate::commands::system::branch::BranchCommand::new());
        self.register(crate::commands::system::session::SessionCommand::new());
        self.register(crate::commands::system::history::HistoryCommand::new());
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn global_registry() -> &'static CommandRegistry {
    static REGISTRY: std::sync::OnceLock<CommandRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(CommandRegistry::new)
}
