use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub folder: String,
    pub aliases: Vec<String>,
    pub commands: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: String,
    pub modules: HashMap<String, Module>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            modules: HashMap::new(),
        }
    }
}

impl Registry {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }

    pub fn remove_module(&mut self, name: &str) -> Option<Module> {
        self.modules.remove(name)
    }

    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn module_names(&self) -> Vec<&String> {
        let mut names: Vec<_> = self.modules.keys().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_default() {
        let reg = Registry::default();
        assert!(reg.modules.is_empty());
    }

    #[test]
    fn test_registry_add_remove() {
        let mut reg = Registry::default();
        let module = Module {
            name: "test".to_string(),
            folder: "test".to_string(),
            aliases: vec!["t".to_string()],
            commands: HashMap::new(),
        };
        reg.add_module(module);
        assert!(reg.get_module("test").is_some());
        reg.remove_module("test");
        assert!(reg.get_module("test").is_none());
    }
}