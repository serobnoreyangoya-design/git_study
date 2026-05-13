use std::fmt;
use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use dialoguer::Select;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Install TicGit instructions for an AI agent.
    Skill(SkillArgs),
}

#[derive(Debug, Parser)]
pub struct SkillArgs {
    /// Installation target. If omitted in a terminal, prompts interactively.
    #[arg(long = "target", value_enum)]
    pub target: Option<SkillTarget>,

    /// Check whether the selected target is already installed and current.
    #[arg(long = "check")]
    pub check: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SkillTarget {
    AgentsMd,
    AgentsLocal,
    AgentsGlobal,
    ClaudeLocal,
    ClaudeGlobal,
    OpenCodeLocal,
    OpenCodeGlobal,
    CodexLocal,
    CodexGlobal,
    CopilotLocal,
    CopilotGlobal,
    CursorLocal,
    CursorGlobal,
    WindsurfLocal,
    WindsurfGlobal,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        None => {
            crate::agent_help::print();
            Ok(())
        }
        Some(Command::Skill(args)) => install_skill(args),
    }
}

fn install_skill(args: SkillArgs) -> Result<()> {
    let target = match args.target {
        Some(target) => target,
        None if std::io::stdin().is_terminal() => prompt_target()?,
        None => anyhow::bail!("--target is required when stdin is not interactive"),
    };
    let destination = target.destination()?;

    if args.check {
        if destination.is_current()? {
            println!("{} is installed and current.", destination.path.display());
            return Ok(());
        }
        anyhow::bail!(
            "{} is not installed or is out of date",
            destination.path.display()
        );
    }

    destination.install()?;
    println!(
        "Installed TicGit agent instructions to {}.",
        destination.path.display()
    );
    Ok(())
}

fn prompt_target() -> Result<SkillTarget> {
    let targets = [
        SkillTarget::AgentsMd,
        SkillTarget::AgentsLocal,
        SkillTarget::AgentsGlobal,
        SkillTarget::ClaudeLocal,
        SkillTarget::ClaudeGlobal,
        SkillTarget::OpenCodeLocal,
        SkillTarget::OpenCodeGlobal,
        SkillTarget::CodexLocal,
        SkillTarget::CodexGlobal,
        SkillTarget::CopilotLocal,
        SkillTarget::CopilotGlobal,
        SkillTarget::CursorLocal,
        SkillTarget::CursorGlobal,
        SkillTarget::WindsurfLocal,
        SkillTarget::WindsurfGlobal,
    ];
    let labels = targets
        .iter()
        .map(|target| target.prompt_label())
        .collect::<Vec<_>>();
    let selected = Select::new()
        .with_prompt("Install TicGit agent instructions")
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(targets[selected])
}

struct Destination {
    path: PathBuf,
    kind: DestinationKind,
}

enum DestinationKind {
    AgentsMd,
    Skill,
}

impl Destination {
    fn install(&self) -> Result<()> {
        match self.kind {
            DestinationKind::AgentsMd => self.install_agents_md(),
            DestinationKind::Skill => self.install_skill_file(),
        }
    }

    fn is_current(&self) -> Result<bool> {
        match self.kind {
            DestinationKind::AgentsMd => Ok(std::fs::read_to_string(&self.path)
                .map(|contents| contents.contains(agents_md_block().trim()))
                .unwrap_or(false)),
            DestinationKind::Skill => Ok(std::fs::read_to_string(&self.path)
                .map(|contents| contents == skill_markdown())
                .unwrap_or(false)),
        }
    }

    fn install_agents_md(&self) -> Result<()> {
        let existing = std::fs::read_to_string(&self.path).unwrap_or_default();
        let next = if existing.contains(AGENTS_START) && existing.contains(AGENTS_END) {
            replace_block(&existing, agents_md_block())
        } else if existing.trim().is_empty() {
            format!("{}\n", agents_md_block())
        } else {
            format!("{}\n\n{}\n", existing.trim_end(), agents_md_block())
        };
        std::fs::write(&self.path, next).with_context(|| format!("writing {}", self.path.display()))
    }

    fn install_skill_file(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&self.path, skill_markdown())
            .with_context(|| format!("writing {}", self.path.display()))
    }
}

impl SkillTarget {
    fn destination(self) -> Result<Destination> {
        let path = match self {
            SkillTarget::AgentsMd => repo_root()?.join("AGENTS.md"),
            SkillTarget::AgentsLocal => repo_root()?.join(".agents/skills/ticgit/SKILL.md"),
            SkillTarget::AgentsGlobal => home()?.join(".agents/skills/ticgit/SKILL.md"),
            SkillTarget::ClaudeLocal => repo_root()?.join(".claude/skills/ticgit/SKILL.md"),
            SkillTarget::ClaudeGlobal => home()?.join(".claude/skills/ticgit/SKILL.md"),
            SkillTarget::OpenCodeLocal => repo_root()?.join(".opencode/skills/ticgit/SKILL.md"),
            SkillTarget::OpenCodeGlobal => home()?.join(".config/opencode/skills/ticgit/SKILL.md"),
            SkillTarget::CodexLocal => repo_root()?.join(".codex/skills/ticgit/SKILL.md"),
            SkillTarget::CodexGlobal => home()?.join(".codex/skills/ticgit/SKILL.md"),
            SkillTarget::CopilotLocal => repo_root()?.join(".copilot/skills/ticgit/SKILL.md"),
            SkillTarget::CopilotGlobal => home()?.join(".copilot/skills/ticgit/SKILL.md"),
            SkillTarget::CursorLocal => repo_root()?.join(".cursor/skills/ticgit/SKILL.md"),
            SkillTarget::CursorGlobal => home()?.join(".cursor/skills/ticgit/SKILL.md"),
            SkillTarget::WindsurfLocal => repo_root()?.join(".windsurf/skills/ticgit/SKILL.md"),
            SkillTarget::WindsurfGlobal => home()?.join(".codeium/windsurf/skills/ticgit/SKILL.md"),
        };
        let kind = match self {
            SkillTarget::AgentsMd => DestinationKind::AgentsMd,
            _ => DestinationKind::Skill,
        };
        Ok(Destination { path, kind })
    }

    fn prompt_label(self) -> String {
        match self {
            SkillTarget::AgentsMd => format!(
                "AGENTS.md - project instructions ({})",
                display_path(&self.destination_path_hint())
            ),
            SkillTarget::AgentsLocal => {
                "Agent Skills - shared local .agents/skills format (./.agents/skills/ticgit)"
                    .to_string()
            }
            SkillTarget::AgentsGlobal => {
                "Agent Skills - shared global .agents/skills format (~/.agents/skills/ticgit)"
                    .to_string()
            }
            SkillTarget::ClaudeLocal => {
                "Claude Code - local skill (./.claude/skills/ticgit)".to_string()
            }
            SkillTarget::ClaudeGlobal => {
                "Claude Code - global skill (~/.claude/skills/ticgit)".to_string()
            }
            SkillTarget::OpenCodeLocal => {
                "OpenCode - local skill (./.opencode/skills/ticgit)".to_string()
            }
            SkillTarget::OpenCodeGlobal => {
                "OpenCode - global skill (~/.config/opencode/skills/ticgit)".to_string()
            }
            SkillTarget::CodexLocal => "Codex - local skill (./.codex/skills/ticgit)".to_string(),
            SkillTarget::CodexGlobal => "Codex - global skill (~/.codex/skills/ticgit)".to_string(),
            SkillTarget::CopilotLocal => {
                "GitHub Copilot - local skill (./.copilot/skills/ticgit)".to_string()
            }
            SkillTarget::CopilotGlobal => {
                "GitHub Copilot - global skill (~/.copilot/skills/ticgit)".to_string()
            }
            SkillTarget::CursorLocal => {
                "Cursor - local skill (./.cursor/skills/ticgit)".to_string()
            }
            SkillTarget::CursorGlobal => {
                "Cursor - global skill (~/.cursor/skills/ticgit)".to_string()
            }
            SkillTarget::WindsurfLocal => {
                "Windsurf - local skill (./.windsurf/skills/ticgit)".to_string()
            }
            SkillTarget::WindsurfGlobal => {
                "Windsurf - global skill (~/.codeium/windsurf/skills/ticgit)".to_string()
            }
        }
    }

    fn destination_path_hint(self) -> PathBuf {
        match self {
            SkillTarget::AgentsMd => PathBuf::from("./AGENTS.md"),
            _ => PathBuf::from("."),
        }
    }
}

impl fmt::Display for SkillTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn home() -> Result<PathBuf> {
    dirs::home_dir().context("could not determine home directory")
}

fn repo_root() -> Result<PathBuf> {
    let repo = gix::discover(".").context("finding git repository")?;
    repo.workdir()
        .map(PathBuf::from)
        .or_else(|| repo.git_dir().parent().map(PathBuf::from))
        .context("could not determine repository root")
}

fn skill_markdown() -> &'static str {
    crate::agent_help::MARKDOWN
}

const AGENTS_START: &str = "<!-- ticgit-agent-start -->";
const AGENTS_END: &str = "<!-- ticgit-agent-end -->";

fn agents_md_block() -> &'static str {
    concat!(
        "<!-- ticgit-agent-start -->\n",
        "## TicGit\n\n",
        "This project uses TicGit (`ti`) for Git-native ticket tracking.\n\n",
        "- Install TicGit from the project README or by using the published release/install instructions.\n",
        "- Run `ti agent` to learn the TicGit workflow, command examples, and agent practices.\n",
        "- Prefer `ti list --markdown` and `ti show <id> --markdown` when reading ticket data.\n",
        "- Use `ti comment`, `ti state`, and `ti close` to record progress and resolution.\n",
        "<!-- ticgit-agent-end -->",
    )
}

fn replace_block(existing: &str, replacement: &str) -> String {
    let Some(start) = existing.find(AGENTS_START) else {
        return existing.to_string();
    };
    let Some(end_start) = existing.find(AGENTS_END) else {
        return existing.to_string();
    };
    let end = end_start + AGENTS_END.len();
    format!("{}{}{}", &existing[..start], replacement, &existing[end..])
}

fn display_path(path: &std::path::Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_agents_md_block() {
        let existing =
            "Intro\n\n<!-- ticgit-agent-start -->\nold\n<!-- ticgit-agent-end -->\nTail\n";
        let replaced = replace_block(existing, agents_md_block());
        assert!(replaced.contains("This project uses TicGit"));
        assert!(!replaced.contains("\nold\n"));
        assert!(replaced.starts_with("Intro\n\n"));
        assert!(replaced.ends_with("\nTail\n"));
    }
}
