use std::error::Error;
use std::process::Command;
use std::str;
use std::sync::Arc;
use std::time::Duration;

use futures::FutureExt;
use futures_timer::Delay;
use regex::Regex;
use tokio::sync::mpsc::Sender;

use crate::{ManagerMessage, Options};

#[derive(Debug, Clone, Copy, Default)]
pub struct GitChanges {
    pub insertions: i32,
    pub deletions: i32,
    pub total: i32,
}

impl GitChanges {
    pub fn compare(&self, other: &GitChanges) -> bool {
        self.insertions == other.insertions && self.deletions == other.deletions
    }
}

#[derive(Debug, Clone)]
pub struct GitState {
    pub git_changes: GitChanges,
    pub current_commit: String,
    pub current_commit_short: String,
    pub last_commit: Option<String>,
    pub last_commit_short: Option<String>,
    pub threshold: i32,
}

impl Default for GitState {
    fn default() -> Self {
        Self {
            git_changes: count_changes().unwrap_or_default(),
            current_commit: get_current_commit().unwrap(),
            current_commit_short: get_current_commit_short().unwrap(),
            last_commit: None,
            last_commit_short: None,
            threshold: 100,
        }
    }
}

impl GitState {
    pub fn new(threshold: i32) -> Self {
        Self {
            threshold,
            ..Self::default()
        }
    }

    pub fn update(&mut self) {
        self.last_commit = Some(self.current_commit.clone());
        self.last_commit_short = Some(self.current_commit_short.clone());
        self.current_commit_short = get_current_commit_short().unwrap();
        self.current_commit = get_current_commit().unwrap();
        self.git_changes = count_changes().unwrap();
    }

    pub fn is_above_threshold(&self) -> bool {
        self.git_changes.total > self.threshold
    }

    pub fn compare(&self, other: &Self) -> bool {
        if self.current_commit != other.current_commit {
            return false;
        }
        if self.last_commit != other.last_commit {
            return false;
        }

        self.git_changes.compare(&other.git_changes)
    }

    pub fn compare_with_prev(&self, prev: Arc<Option<GitState>>) -> bool {
        if let Some(prev) = prev.as_ref() {
            self.compare(prev)
        } else {
            false
        }
    }
}

pub async fn git_loop(tx: Sender<ManagerMessage>, options: Arc<Options>) {
    let threshold = options.threshold;
    let loop_time = options.git_update_time;
    loop {
        let mut git_state = GitState::new(threshold);
        git_state.update();

        let message = ManagerMessage::Git { git_state };

        tx.send(message).await.unwrap();
        Delay::new(Duration::from_millis(loop_time)).fuse().await;
    }
}

pub fn get_current_commit() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;
    let output = String::from_utf8(output)?;
    Ok(output.trim().to_string())
}

pub fn get_current_commit_short() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()?
        .stdout;
    let output = String::from_utf8(output)?;
    Ok(output.trim().to_string())
}

pub fn count_changes() -> Result<GitChanges, Box<(dyn Error + 'static)>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--shortstat")
        .output()?;

    let stdout = str::from_utf8(&output.stdout)?;
    let re = Regex::new(r"((\d+)\D+)((\d+)\D+)?((\d+)?\D+)?")?;
    let captures = re.captures(stdout).ok_or("No match");

    match captures {
        Ok(captures) => {
            let insertions = captures
                .get(4)
                .map_or("0", |m| m.as_str())
                .parse::<i32>()
                .unwrap();
            let deletions = captures
                .get(6)
                .map_or("0", |m| m.as_str())
                .parse::<i32>()
                .unwrap();

            let total: i32 = insertions + deletions;

            Ok(GitChanges {
                insertions,
                deletions,
                total,
            })
        }

        Err(_) => Ok(GitChanges {
            insertions: 0,
            deletions: 0,
            total: 0,
        }),
    }
}
