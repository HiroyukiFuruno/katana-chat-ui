use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandLaunchEntry {
    pub id: String,
    pub label: String,
    pub kind: CommandLaunchKind,
}

impl CommandLaunchEntry {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: CommandLaunchKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandLaunchKind {
    Prompt,
    Skill,
    Workflow,
    Command,
    Hook,
    Mcp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandLaunchIntent {
    pub entry_id: String,
    pub kind: CommandLaunchKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashLauncherRenderModel {
    pub visible: bool,
    pub query: String,
    pub entries: Vec<CommandLaunchEntry>,
}

impl SlashLauncherRenderModel {
    pub fn from_draft(text: &str, entries: &[CommandLaunchEntry]) -> Self {
        let Some(query) = text.strip_prefix('/') else {
            return Self::hidden();
        };
        Self {
            visible: true,
            query: query.to_string(),
            entries: Self::matching_entries(query, entries),
        }
    }

    fn hidden() -> Self {
        Self {
            visible: false,
            query: String::new(),
            entries: Vec::new(),
        }
    }

    fn matching_entries(query: &str, entries: &[CommandLaunchEntry]) -> Vec<CommandLaunchEntry> {
        entries
            .iter()
            .filter(|entry| query.is_empty() || entry.label.contains(query))
            .cloned()
            .collect()
    }

    pub fn intent_for(&self, entry_id: &str) -> Option<CommandLaunchIntent> {
        self.entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .map(|entry| CommandLaunchIntent {
                entry_id: entry.id.clone(),
                kind: entry.kind,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandLaunchEntry, CommandLaunchKind, SlashLauncherRenderModel};

    #[test]
    fn slash_launcher_opens_from_slash_without_changing_draft() {
        let entries = vec![CommandLaunchEntry::new(
            "review",
            "レビュー",
            CommandLaunchKind::Workflow,
        )];

        let launcher = SlashLauncherRenderModel::from_draft("/", &entries);

        assert!(launcher.visible);
        assert_eq!(launcher.query, "");
        assert_eq!(launcher.entries, entries);
    }

    #[test]
    fn slash_launcher_filters_host_entries() {
        let entries = vec![
            CommandLaunchEntry::new("review", "レビュー", CommandLaunchKind::Workflow),
            CommandLaunchEntry::new("test", "テスト", CommandLaunchKind::Command),
        ];

        let launcher = SlashLauncherRenderModel::from_draft("/テ", &entries);

        assert_eq!(launcher.entries.len(), 1);
        assert_eq!(launcher.entries[0].id, "test");
    }

    #[test]
    fn slash_launcher_returns_command_launch_intent() {
        let entries = vec![CommandLaunchEntry::new(
            "review",
            "レビュー",
            CommandLaunchKind::Workflow,
        )];
        let launcher = SlashLauncherRenderModel::from_draft("/", &entries);

        let intent = launcher.intent_for("review");

        assert_eq!(intent.map(|it| it.kind), Some(CommandLaunchKind::Workflow));
    }
}
