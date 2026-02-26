use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

impl Manifest {
    /// Load a manifest from a file path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let content =
            fs::read_to_string(path.as_ref()).map_err(|e| ManifestError::IoError(e.to_string()))?;

        let manifest: Manifest =
            serde_json::from_str(&content).map_err(|e| ManifestError::ParseError(e.to_string()))?;

        Ok(manifest)
    }

    /// Find and load manifest.json in the current directory or parent directories
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_and_load<P: AsRef<Path>>(start_dir: P) -> Result<Self, ManifestError> {
        let mut current = start_dir.as_ref().to_path_buf();

        loop {
            let manifest_path = current.join("manifest.json");
            if manifest_path.exists() {
                return Self::load(manifest_path);
            }

            if !current.pop() {
                return Err(ManifestError::NotFound);
            }
        }
    }

    /// Resolve an import path to a file path
    #[cfg(not(target_arch = "wasm32"))]
    pub fn resolve_import(&self, import_path: &[String]) -> Result<String, ManifestError> {
        if import_path.is_empty() {
            return Err(ManifestError::InvalidPath("Empty import path".to_string()));
        }

        let project_name = &import_path[0];

        // If importing from this project, use entrypoint
        if project_name == &self.name {
            return Ok(self.entrypoint.clone());
        }

        // Check .lflibs folder first (installed dependencies)
        if let Ok(current_dir) = std::env::current_dir() {
            let lflibs_path = current_dir.join(".lflibs");
            if lflibs_path.exists() {
                // Look for versioned directory (e.g., package-name@1.0.0)
                if let Ok(entries) = fs::read_dir(&lflibs_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if directory name starts with the project name
                            if dir_name.starts_with(&format!("{}@", project_name))
                                || dir_name == project_name
                            {
                                let manifest_path = path.join("manifest.json");
                                if manifest_path.exists() {
                                    if let Ok(dep_manifest) = Self::load(&manifest_path) {
                                        // Always use the entrypoint - exports are now managed by the `teach` keyword in files
                                        return Ok(path
                                            .join(&dep_manifest.entrypoint)
                                            .to_string_lossy()
                                            .to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check dependencies
        if let Some(dep_path) = self.dependencies.get(project_name) {
            // In the future, this would resolve through a package manager
            // For now, treat as a relative path
            return Ok(dep_path.clone());
        }

        Err(ManifestError::UnresolvedImport(import_path.join("::")))
    }
}

#[derive(Debug)]
pub enum ManifestError {
    IoError(String),
    ParseError(String),
    NotFound,
    InvalidPath(String),
    UnresolvedImport(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::IoError(e) => write!(f, "IO error: {}", e),
            ManifestError::ParseError(e) => write!(f, "Parse error: {}", e),
            ManifestError::NotFound => write!(f, "manifest.json not found"),
            ManifestError::InvalidPath(e) => write!(f, "Invalid path: {}", e),
            ManifestError::UnresolvedImport(path) => write!(f, "Unresolved import: {}", path),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse() {
        let manifest_json = r#"{
            "name": "myproject",
            "version": "1.0.0",
            "entrypoint": "src/main.lf"
        }"#;

        let manifest: Manifest = serde_json::from_str(manifest_json).unwrap();
        assert_eq!(manifest.name, "myproject");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.entrypoint, "src/main.lf");
    }

    #[test]
    fn test_resolve_import_entrypoint() {
        let manifest = Manifest {
            name: "myproject".to_string(),
            version: "1.0.0".to_string(),
            entrypoint: "src/main.lf".to_string(),
            dependencies: HashMap::new(),
        };

        let result = manifest
            .resolve_import(&vec!["myproject".to_string()])
            .unwrap();
        assert_eq!(result, "src/main.lf");
    }

    #[test]
    fn test_resolve_import_dependency() {
        let mut dependencies = HashMap::new();
        dependencies.insert("utils".to_string(), "./deps/utils".to_string());

        let manifest = Manifest {
            name: "myproject".to_string(),
            version: "1.0.0".to_string(),
            entrypoint: "src/main.lf".to_string(),
            dependencies,
        };

        let result = manifest.resolve_import(&vec!["utils".to_string()]).unwrap();
        assert_eq!(result, "./deps/utils");
    }
}
