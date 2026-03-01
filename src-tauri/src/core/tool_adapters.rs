use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct ToolAdapter {
    pub key: String,
    pub display_name: String,
    pub relative_skills_dir: String,
    pub relative_detect_dir: String,
}

impl ToolAdapter {
    fn home() -> PathBuf {
        dirs::home_dir().expect("Cannot determine home directory")
    }

    pub fn skills_dir(&self) -> PathBuf {
        Self::home().join(&self.relative_skills_dir)
    }

    pub fn detect_dir(&self) -> PathBuf {
        Self::home().join(&self.relative_detect_dir)
    }

    pub fn is_installed(&self) -> bool {
        self.detect_dir().exists()
    }
}

pub fn default_tool_adapters() -> Vec<ToolAdapter> {
    vec![
        ToolAdapter {
            key: "cursor".into(),
            display_name: "Cursor".into(),
            relative_skills_dir: ".cursor/skills".into(),
            relative_detect_dir: ".cursor".into(),
        },
        ToolAdapter {
            key: "claude_code".into(),
            display_name: "Claude Code".into(),
            relative_skills_dir: ".claude/skills".into(),
            relative_detect_dir: ".claude".into(),
        },
        ToolAdapter {
            key: "codex".into(),
            display_name: "Codex".into(),
            relative_skills_dir: ".codex/skills".into(),
            relative_detect_dir: ".codex".into(),
        },
        ToolAdapter {
            key: "opencode".into(),
            display_name: "OpenCode".into(),
            relative_skills_dir: ".config/opencode/skills".into(),
            relative_detect_dir: ".config/opencode".into(),
        },
        ToolAdapter {
            key: "antigravity".into(),
            display_name: "Antigravity".into(),
            relative_skills_dir: ".gemini/antigravity/global_skills".into(),
            relative_detect_dir: ".gemini/antigravity".into(),
        },
        ToolAdapter {
            key: "amp".into(),
            display_name: "Amp".into(),
            relative_skills_dir: ".config/agents/skills".into(),
            relative_detect_dir: ".config/agents".into(),
        },
        ToolAdapter {
            key: "kilo_code".into(),
            display_name: "Kilo Code".into(),
            relative_skills_dir: ".kilocode/skills".into(),
            relative_detect_dir: ".kilocode".into(),
        },
        ToolAdapter {
            key: "roo_code".into(),
            display_name: "Roo Code".into(),
            relative_skills_dir: ".roo/skills".into(),
            relative_detect_dir: ".roo".into(),
        },
        ToolAdapter {
            key: "goose".into(),
            display_name: "Goose".into(),
            relative_skills_dir: ".config/goose/skills".into(),
            relative_detect_dir: ".config/goose".into(),
        },
        ToolAdapter {
            key: "gemini_cli".into(),
            display_name: "Gemini CLI".into(),
            relative_skills_dir: ".gemini/skills".into(),
            relative_detect_dir: ".gemini".into(),
        },
        ToolAdapter {
            key: "github_copilot".into(),
            display_name: "GitHub Copilot".into(),
            relative_skills_dir: ".copilot/skills".into(),
            relative_detect_dir: ".copilot".into(),
        },
        ToolAdapter {
            key: "clawdbot".into(),
            display_name: "Clawdbot".into(),
            relative_skills_dir: ".clawdbot/skills".into(),
            relative_detect_dir: ".clawdbot".into(),
        },
        ToolAdapter {
            key: "droid".into(),
            display_name: "Droid".into(),
            relative_skills_dir: ".factory/skills".into(),
            relative_detect_dir: ".factory".into(),
        },
        ToolAdapter {
            key: "windsurf".into(),
            display_name: "Windsurf".into(),
            relative_skills_dir: ".codeium/windsurf/skills".into(),
            relative_detect_dir: ".codeium/windsurf".into(),
        },
        ToolAdapter {
            key: "trae".into(),
            display_name: "TRAE IDE".into(),
            relative_skills_dir: ".trae/skills".into(),
            relative_detect_dir: ".trae".into(),
        },
    ]
}

pub fn find_adapter(key: &str) -> Option<ToolAdapter> {
    default_tool_adapters().into_iter().find(|a| a.key == key)
}
