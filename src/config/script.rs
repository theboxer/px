use std::{
    env,
    process::{Command, Stdio},
    str::FromStr,
};

use serde_json::Value;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ScriptExecutor {
    Direct,
    Npm,
    Yarn,
    Pnpm,
    Composer,
    Custom(String),
}

impl From<Option<&str>> for ScriptExecutor {
    fn from(value: Option<&str>) -> Self {
        if value.is_none() {
            return Self::Npm;
        }

        let value = value.unwrap();

        if value.starts_with("pnpm") {
            return Self::Pnpm;
        }

        if value.starts_with("yarn") {
            return Self::Yarn;
        }

        if value.starts_with("composer") {
            return Self::Composer;
        }

        Self::Npm
    }
}

impl FromStr for ScriptExecutor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "composer" => Ok(ScriptExecutor::Composer),
            "npm" => Ok(ScriptExecutor::Npm),
            "yarn" => Ok(ScriptExecutor::Yarn),
            "pnpm" => Ok(ScriptExecutor::Pnpm),
            _ => Err(format!("Invalid Executor: {}", s)), // Handle invalid strings
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub cmd: String,
    pub description: Option<String>,
    cwd: String,
    executor: ScriptExecutor,
}

impl Script {
    pub fn from_string(name: String, cmd: String, cwd: String, executor: ScriptExecutor) -> Self {
        Self {
            name,
            cmd,
            description: None,
            cwd,
            executor,
        }
    }

    pub fn from_json_value(
        name: String,
        value: &Value,
        cwd: String,
        executor: ScriptExecutor,
    ) -> Option<Self> {
        Some(Self {
            name,
            executor,
            cwd,
            description: value
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            cmd: String::from(value["cmd"].as_str()?),
        })
    }

    pub fn from_toml_value(
        name: String,
        value: &toml::Value,
        cwd: String,
        executor: ScriptExecutor,
    ) -> Option<Self> {
        Some(Self {
            name,
            executor,
            cwd,
            description: value
                .get("description")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            cmd: String::from(value["cmd"].as_str()?),
        })
    }

    pub fn execute(&self, raw_args: &[&str]) {
        let cmd = match &self.executor {
            ScriptExecutor::Direct => {
                format!("{} {}", self.cmd, raw_args.join(" "))
            }
            ScriptExecutor::Npm => {
                format!("npm run {} -- {}", self.name, raw_args.join(" "))
            }
            ScriptExecutor::Yarn => {
                format!("yarn run {} -- {}", self.name, raw_args.join(" "))
            }
            ScriptExecutor::Pnpm => {
                format!("pnpm run {} -- {}", self.name, raw_args.join(" "))
            }
            ScriptExecutor::Composer => {
                format!(
                    "composer run-script {} -- {}",
                    self.name,
                    raw_args.join(" ")
                )
            }
            ScriptExecutor::Custom(exec) => {
                format!("{} {} -- {}", &exec, self.name, raw_args.join(" "))
            }
        };

        let cmd = cmd.trim_end();

        let is_windows = env::consts::OS == "windows";
        let shell = if is_windows { "cmd" } else { "sh" };
        let flag = if is_windows { "/C" } else { "-c" };

        let mut child = Command::new(shell)
            .arg(flag)
            .arg(cmd)
            .current_dir(&self.cwd)
            .stdout(Stdio::inherit()) // Inherit stdout
            .stderr(Stdio::inherit()) // Inherit stderr
            .spawn()
            .unwrap();

        let status = child.wait().unwrap();
        if !status.success() {
            eprintln!("Command exited with status: {}", status);
        }
    }
}
