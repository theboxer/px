use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufReader, Read},
};

use super::script::{Script, ScriptExecutor};

enum ConfigType {
    PxJson,
    PxToml,
    PackageJson,
    CargoToml,
    ComposerJson,
}

#[derive(Default, Debug, Clone)]
pub struct Config {
    pub scripts: HashMap<String, Script>,

    executors: HashMap<ScriptExecutor, ScriptExecutor>,
}

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();

        config.load_configs();

        config
    }

    fn load_configs(&mut self) {
        let config_types = vec![
            ConfigType::PxJson,
            ConfigType::PxToml,
            ConfigType::PackageJson,
            ConfigType::CargoToml,
            ConfigType::ComposerJson,
        ];
        for config_type in config_types {
            self.load_config(&config_type);
        }
    }

    fn load_config(&mut self, config_type: &ConfigType) {
        match config_type {
            ConfigType::PxJson => {
                self.load_px_json();
            }
            ConfigType::PxToml => {
                self.load_px_toml();
            }
            ConfigType::PackageJson => {
                self.load_package_json();
            }
            ConfigType::CargoToml => {
                self.load_cargo_toml();
            }
            ConfigType::ComposerJson => {
                self.load_composer_json();
            }
        }
    }

    fn find_config(name: &str) -> Option<(String, BufReader<File>)> {
        let current_dir = env::current_dir().ok()?;

        for ancestor in current_dir.ancestors() {
            let config_path = ancestor.join(name);

            if config_path.exists() {
                if let Ok(file) = File::open(&config_path) {
                    let reader = BufReader::new(file);
                    return Some((ancestor.to_string_lossy().to_string(), reader));
                }
            }
        }

        None
    }

    fn load_px_json(&mut self) -> Option<()> {
        let (cwd, reader) = Self::find_config("px.json")?;

        let px: serde_json::Value = serde_json::from_reader(reader).unwrap();

        if let Some(executor) = px["executor"].as_object() {
            for (name, value) in executor.iter() {
                let name = name.parse::<ScriptExecutor>();
                if name.is_err() {
                    continue;
                }

                let name = name.unwrap();

                if self.executors.contains_key(&name) {
                    continue;
                }

                self.executors.insert(
                    name,
                    ScriptExecutor::Custom(value.as_str().unwrap().to_string()),
                );
            }
        }

        if let Some(scripts) = px["scripts"].as_object() {
            for (name, value) in scripts.iter() {
                if self.scripts.contains_key(name) {
                    continue;
                }

                if value.is_string() {
                    self.scripts.insert(
                        name.clone(),
                        Script::from_string(
                            name.clone(),
                            String::from(value.as_str().unwrap()),
                            cwd.clone(),
                            super::script::ScriptExecutor::Direct,
                        ),
                    );

                    continue;
                }

                if value.is_object() {
                    let script = Script::from_json_value(
                        name.clone(),
                        value,
                        cwd.clone(),
                        crate::config::script::ScriptExecutor::Direct,
                    );

                    if let Some(script) = script {
                        self.scripts.insert(name.clone(), script);
                    }
                }
            }
        }

        Some(())
    }

    fn load_px_toml(&mut self) -> Option<()> {
        let (cwd, mut reader) = Self::find_config("px.toml")?;

        let mut content = String::new();
        reader.read_to_string(&mut content).ok()?;

        let px: toml::Value = toml::from_str(&content).unwrap();

        if let Some(executor) = px["executor"].as_table() {
            for (name, value) in executor.iter() {
                let name = name.parse::<ScriptExecutor>();
                if name.is_err() {
                    continue;
                }

                let name = name.unwrap();

                if self.executors.contains_key(&name) {
                    continue;
                }

                self.executors.insert(
                    name,
                    ScriptExecutor::Custom(value.as_str().unwrap().to_string()),
                );
            }
        }

        if let Some(scripts) = px["scripts"].as_table() {
            for (name, value) in scripts.iter() {
                if self.scripts.contains_key(name) {
                    continue;
                }

                if value.is_str() {
                    self.scripts.insert(
                        name.clone(),
                        Script::from_string(
                            name.clone(),
                            String::from(value.as_str().unwrap()),
                            cwd.clone(),
                            super::script::ScriptExecutor::Direct,
                        ),
                    );

                    continue;
                }

                if value.is_table() {
                    let script = Script::from_toml_value(
                        name.clone(),
                        value,
                        cwd.clone(),
                        crate::config::script::ScriptExecutor::Direct,
                    );

                    if let Some(script) = script {
                        self.scripts.insert(name.clone(), script);
                    }
                }
            }
        }

        Some(())
    }

    fn load_package_json(&mut self) -> Option<()> {
        let (cwd, reader) = Self::find_config("package.json")?;

        let px: serde_json::Value = serde_json::from_reader(reader).unwrap();

        let executor = if let Some(executor) = px["px"]["executor"].as_str() {
            ScriptExecutor::Custom(executor.to_string())
        } else {
            let default_executor = px["packageManager"].as_str().into();

            self.executors
                .get(&default_executor)
                .unwrap_or(&default_executor)
                .clone()
        };

        if let Some(scripts) = px["scripts"].as_object() {
            for (name, _) in scripts.iter() {
                if self.scripts.contains_key(name) {
                    continue;
                }

                self.scripts.insert(
                    name.clone(),
                    Script::from_string(
                        name.clone(),
                        String::default(),
                        cwd.clone(),
                        executor.clone(),
                    ),
                );
            }
        }

        Some(())
    }

    fn load_cargo_toml(&mut self) -> Option<()> {
        let (cwd, mut reader) = Self::find_config("Cargo.toml")?;

        let mut content = String::new();
        reader.read_to_string(&mut content).ok()?;

        let cfg: toml::Value = toml::from_str(&content).ok()?;

        if let Some(scripts) = cfg["package"]["metadata"]["scripts"].as_table() {
            for (name, value) in scripts.iter() {
                if self.scripts.contains_key(name) {
                    continue;
                }

                if value.is_str() {
                    self.scripts.insert(
                        name.clone(),
                        Script::from_string(
                            name.clone(),
                            String::from(value.as_str().unwrap()),
                            cwd.clone(),
                            super::script::ScriptExecutor::Direct,
                        ),
                    );

                    continue;
                }

                if value.is_table() {
                    let script = Script::from_toml_value(
                        name.clone(),
                        value,
                        cwd.clone(),
                        crate::config::script::ScriptExecutor::Direct,
                    );

                    if let Some(script) = script {
                        self.scripts.insert(name.clone(), script);
                    }
                }
            }
        }

        Some(())
    }

    fn load_composer_json(&mut self) -> Option<()> {
        let (cwd, reader) = Self::find_config("composer.json")?;

        let px: serde_json::Value = serde_json::from_reader(reader).unwrap();

        if let Some(executor) = px["px"]["executor"].as_str() {
            self.executors.insert(
                ScriptExecutor::Composer,
                ScriptExecutor::Custom(executor.to_string()),
            );
        }

        if let Some(scripts) = px["scripts"].as_object() {
            for (name, _) in scripts.iter() {
                if self.scripts.contains_key(name) {
                    continue;
                }

                self.scripts.insert(
                    name.clone(),
                    Script::from_string(
                        name.clone(),
                        String::default(),
                        cwd.clone(),
                        self.executors
                            .get(&ScriptExecutor::Composer)
                            .unwrap_or(&ScriptExecutor::Composer)
                            .clone(),
                    ),
                );
            }
        }

        Some(())
    }
}
