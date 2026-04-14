//! Skill discovery, parsing, and prerequisite checking.
//!
//! Skills are Markdown files with YAML frontmatter containing instructions
//! injected into AI system prompts when active.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Parsed skill metadata from YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub prerequisites: Vec<Prerequisite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    pub kind: PrerequisiteKind,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrerequisiteKind {
    Binary,
    EnvironmentVariable,
    ConfigKey,
}

/// A loaded skill ready for use.
#[derive(Debug, Clone)]
pub struct Skill {
    pub meta: SkillMeta,
    pub body: String,
    pub source: SkillSource,
    pub enabled: bool,
    pub prerequisites_met: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillSource {
    Bundled,
    Managed,
    Workspace,
}

/// Discover skills from all sources.
pub fn discover_skills(managed_dir: &Path, workspace_dir: Option<&Path>) -> Vec<Skill> {
    let mut skills = Vec::new();

    // Managed skills directory.
    if managed_dir.is_dir() {
        for entry in std::fs::read_dir(managed_dir).into_iter().flatten().flatten() {
            if let Some(skill) = load_skill_file(&entry.path(), SkillSource::Managed) {
                skills.push(skill);
            }
        }
    }

    // Workspace skills.
    if let Some(ws) = workspace_dir {
        let ws_skills = ws.join(".conductor").join("skills");
        if ws_skills.is_dir() {
            for entry in std::fs::read_dir(ws_skills).into_iter().flatten().flatten() {
                if let Some(skill) = load_skill_file(&entry.path(), SkillSource::Workspace) {
                    skills.push(skill);
                }
            }
        }
    }

    // Check prerequisites for each skill.
    for skill in &mut skills {
        skill.prerequisites_met = check_prerequisites(&skill.meta.prerequisites);
    }

    skills
}

fn load_skill_file(path: &Path, source: SkillSource) -> Option<Skill> {
    if path.extension().map_or(true, |ext| ext != "md") {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;

    // Split YAML frontmatter from Markdown body.
    let (meta, body) = parse_frontmatter(&content)?;

    Some(Skill {
        meta,
        body,
        source,
        enabled: true,
        prerequisites_met: false,
    })
}

fn parse_frontmatter(content: &str) -> Option<(SkillMeta, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end = after_first.find("---")?;
    let yaml = &after_first[..end];
    let body = after_first[end + 3..].trim().to_string();
    let meta: SkillMeta = serde_json::from_str(yaml)
        .or_else(|_| {
            // Try a simple key-value parse as fallback.
            let mut map = HashMap::new();
            for line in yaml.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    map.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
            Ok::<SkillMeta, String>(SkillMeta {
                name: map.get("name").cloned().unwrap_or_default(),
                description: map.get("description").cloned().unwrap_or_default(),
                icon: map.get("icon").cloned(),
                prerequisites: Vec::new(),
            })
        })
        .ok()?;
    Some((meta, body))
}

/// Check whether all prerequisites for a skill are satisfied.
pub fn check_prerequisites(prereqs: &[Prerequisite]) -> bool {
    prereqs.iter().all(|p| match p.kind {
        PrerequisiteKind::Binary => which_binary(&p.value).is_some(),
        PrerequisiteKind::EnvironmentVariable => std::env::var(&p.value).is_ok(),
        PrerequisiteKind::ConfigKey => true, // assume met for now
    })
}

fn which_binary(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
