use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Types of permissions that can be requested
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionType {
    /// Read access to a specific path or pattern
    Read(String),
    /// Write access to a specific path or pattern
    Write(String),
    /// Network access to a specific host or pattern
    Net(String),
    /// Command execution for a specific command or pattern
    Run(String),
}

/// Result of a permission check
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionState {
    /// Permission was granted
    Granted,
    /// Permission was denied
    Denied,
    /// Permission needs to be requested from user
    Prompt,
}

/// Permission response from user
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionResponse {
    /// Allow once
    AllowOnce,
    /// Allow this and similar requests (e.g., all files in directory)
    AllowAll,
    /// Deny once
    DenyOnce,
    /// Deny this and similar requests
    DenyAll,
}

/// Manages permissions for the loft runtime
pub struct PermissionManager {
    /// Global allow-all flag (from --allow-all CLI flag)
    allow_all: bool,
    /// Allow all read operations (from --allow-read CLI flag)
    allow_read: bool,
    /// Allow all write operations (from --allow-write CLI flag)
    allow_write: bool,
    /// Allow all network operations (from --allow-net CLI flag)
    allow_net: bool,
    /// Allow all command execution (from --allow-run CLI flag)
    allow_run: bool,
    /// Cache of granted permissions
    granted: HashMap<PermissionType, bool>,
    /// Path to the permission cache file
    cache_path: Option<PathBuf>,
    /// Whether to prompt for permissions (false in non-interactive mode)
    interactive: bool,
}

impl PermissionManager {
    /// Create a new permission manager with all permissions denied
    pub fn new() -> Self {
        Self {
            allow_all: false,
            allow_read: false,
            allow_write: false,
            allow_net: false,
            allow_run: false,
            granted: HashMap::new(),
            cache_path: Self::get_cache_path(),
            interactive: Self::is_interactive(),
        }
    }

    /// Create a permission manager with all permissions allowed
    pub fn allow_all() -> Self {
        Self {
            allow_all: true,
            allow_read: true,
            allow_write: true,
            allow_net: true,
            allow_run: true,
            granted: HashMap::new(),
            cache_path: None,
            interactive: false,
        }
    }

    /// Create a permission manager with specific flags
    pub fn with_flags(
        allow_all: bool,
        allow_read: bool,
        allow_write: bool,
        allow_net: bool,
        allow_run: bool,
    ) -> Self {
        if allow_all {
            Self::allow_all()
        } else {
            Self {
                allow_all: false,
                allow_read,
                allow_write,
                allow_net,
                allow_run,
                granted: HashMap::new(),
                cache_path: Self::get_cache_path(),
                interactive: Self::is_interactive(),
            }
        }
    }

    /// Check if running in interactive mode (has a TTY)
    fn is_interactive() -> bool {
        atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout)
    }

    /// Get the path to the permission cache file in a safe location
    fn get_cache_path() -> Option<PathBuf> {
        // Use XDG_DATA_HOME on Unix, LOCALAPPDATA on Windows
        #[cfg(target_family = "unix")]
        {
            let base_dir = std::env::var("XDG_DATA_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME")
                        .ok()
                        .map(|home| PathBuf::from(home).join(".local/share"))
                })?;
            Some(base_dir.join("loft").join("permissions.json"))
        }

        #[cfg(target_family = "windows")]
        {
            let base_dir = std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)?;
            Some(base_dir.join("loft").join("permissions.json"))
        }
    }

    /// Check if a path is within the protected permissions directory
    pub fn is_protected_path(path: &str) -> bool {
        if let Some(cache_path) = Self::get_cache_path() {
            if let Some(cache_dir) = cache_path.parent() {
                let path_obj = Path::new(path);
                // Check if the path starts with the cache directory
                if let Ok(canonical_path) = path_obj.canonicalize() {
                    if let Ok(canonical_cache_dir) = cache_dir.canonicalize() {
                        return canonical_path.starts_with(canonical_cache_dir);
                    }
                }
                // Fallback: check without canonicalization
                return path_obj.starts_with(cache_dir);
            }
        }
        false
    }

    /// Load cached permissions from disk
    pub fn load_cache(&mut self) -> io::Result<()> {
        if let Some(cache_path) = &self.cache_path {
            if cache_path.exists() {
                let contents = std::fs::read_to_string(cache_path)?;
                match serde_json::from_str::<HashMap<PermissionType, bool>>(&contents) {
                    Ok(cached) => {
                        self.granted.extend(cached);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse permission cache: {}", e);
                        eprintln!("Starting with empty permission cache.");
                    }
                }
            }
        }
        Ok(())
    }

    /// Save cached permissions to disk
    pub fn save_cache(&self) -> io::Result<()> {
        if let Some(cache_path) = &self.cache_path {
            // Create parent directory if it doesn't exist
            if let Some(parent) = cache_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let contents = serde_json::to_string_pretty(&self.granted)?;
            std::fs::write(cache_path, contents)?;
        }
        Ok(())
    }

    /// Check if a permission is granted
    pub fn check(&mut self, perm: &PermissionType) -> PermissionState {
        // If allow_all is set, grant everything
        if self.allow_all {
            return PermissionState::Granted;
        }

        // Check type-specific allow flags
        match perm {
            PermissionType::Read(_) if self.allow_read => return PermissionState::Granted,
            PermissionType::Write(_) if self.allow_write => return PermissionState::Granted,
            PermissionType::Net(_) if self.allow_net => return PermissionState::Granted,
            PermissionType::Run(_) if self.allow_run => return PermissionState::Granted,
            _ => {}
        }

        // Check if we've already made a decision for this permission
        if let Some(&granted) = self.granted.get(perm) {
            return if granted {
                PermissionState::Granted
            } else {
                PermissionState::Denied
            };
        }

        // Need to prompt user
        PermissionState::Prompt
    }

    /// Request permission from user with a prompt
    pub fn request(
        &mut self,
        perm: &PermissionType,
        context: Option<&str>,
    ) -> Result<bool, String> {
        match self.check(perm) {
            PermissionState::Granted => Ok(true),
            PermissionState::Denied => Err(format!("Permission denied: {:?}", perm)),
            PermissionState::Prompt => {
                if !self.interactive {
                    // Non-interactive mode: deny by default
                    self.granted.insert(perm.clone(), false);
                    return Err(format!("Permission denied (non-interactive): {:?}", perm));
                }

                // Prompt user
                let response = self.prompt_user(perm, context)?;

                match response {
                    PermissionResponse::AllowOnce => {
                        // Don't cache, just allow this once
                        Ok(true)
                    }
                    PermissionResponse::AllowAll => {
                        // Cache and allow
                        self.granted.insert(perm.clone(), true);
                        let _ = self.save_cache();
                        Ok(true)
                    }
                    PermissionResponse::DenyOnce => {
                        // Don't cache, just deny this once
                        Err(format!("Permission denied by user: {:?}", perm))
                    }
                    PermissionResponse::DenyAll => {
                        // Cache and deny
                        self.granted.insert(perm.clone(), false);
                        let _ = self.save_cache();
                        Err(format!("Permission denied by user: {:?}", perm))
                    }
                }
            }
        }
    }

    /// Prompt the user for permission
    fn prompt_user(
        &self,
        perm: &PermissionType,
        context: Option<&str>,
    ) -> Result<PermissionResponse, String> {
        use owo_colors::OwoColorize;

        println!();
        println!("{}", "⚠️  Permission Request".bright_yellow().bold());
        println!("{}", "─────────────────────".bright_yellow());

        // Show what permission is being requested
        match perm {
            PermissionType::Read(path) => {
                println!("{}: Read access", "Type".bright_cyan());
                println!("{}: {}", "Path".bright_cyan(), path.bright_white());
            }
            PermissionType::Write(path) => {
                println!("{}: Write access", "Type".bright_cyan());
                println!("{}: {}", "Path".bright_cyan(), path.bright_white());
            }
            PermissionType::Net(host) => {
                println!("{}: Network access", "Type".bright_cyan());
                println!("{}: {}", "Host".bright_cyan(), host.bright_white());
            }
            PermissionType::Run(cmd) => {
                println!("{}: Command execution", "Type".bright_cyan());
                println!("{}: {}", "Command".bright_cyan(), cmd.bright_white());
            }
        }

        if let Some(ctx) = context {
            println!("{}: {}", "Context".bright_cyan(), ctx.dimmed());
        }

        println!();
        println!("Allow this operation?");
        println!("  {} - Allow once", "y".bright_green());
        println!("  {} - Allow all (cached)", "a".bright_green());
        println!("  {} - Deny once", "n".bright_red());
        println!("  {} - Deny all (cached)", "d".bright_red());
        print!("\n{} ", "Choice [y/a/n/d]:".bright_cyan());
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| e.to_string())?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(PermissionResponse::AllowOnce),
            "a" | "all" => Ok(PermissionResponse::AllowAll),
            "n" | "no" => Ok(PermissionResponse::DenyOnce),
            "d" | "deny" => Ok(PermissionResponse::DenyAll),
            _ => {
                println!("{}", "Invalid choice, denying...".bright_red());
                Ok(PermissionResponse::DenyOnce)
            }
        }
    }

    /// Check if a path is under a granted parent path (for batch permissions)
    ///
    /// This method is designed for future batch permission prompting functionality,
    /// where granting permission for a directory would also grant permission for all
    /// subdirectories and files within it.
    ///
    /// Example: If permission is granted for "/home/user/projects", then access to
    /// "/home/user/projects/myapp/src/main.lf" would also be allowed.
    #[allow(dead_code)] // Reserved for future batch permission feature
    pub fn has_parent_permission(
        &self,
        path: &str,
        perm_type: fn(String) -> PermissionType,
    ) -> bool {
        // Check if we have permission for the exact path
        if let Some(&granted) = self.granted.get(&perm_type(path.to_string())) {
            return granted;
        }

        // Check if we have permission for any parent directory
        let path_obj = Path::new(path);
        for ancestor in path_obj.ancestors().skip(1) {
            if let Some(ancestor_str) = ancestor.to_str() {
                if let Some(&granted) = self.granted.get(&perm_type(ancestor_str.to_string())) {
                    if granted {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Request permission for a file path operation
    pub fn request_read(&mut self, path: &str, context: Option<&str>) -> Result<bool, String> {
        self.request(&PermissionType::Read(path.to_string()), context)
    }

    /// Request permission for a write operation
    pub fn request_write(&mut self, path: &str, context: Option<&str>) -> Result<bool, String> {
        self.request(&PermissionType::Write(path.to_string()), context)
    }

    /// Request permission for a network operation
    pub fn request_net(&mut self, host: &str, context: Option<&str>) -> Result<bool, String> {
        self.request(&PermissionType::Net(host.to_string()), context)
    }

    /// Request permission for a command execution
    pub fn request_run(&mut self, command: &str, context: Option<&str>) -> Result<bool, String> {
        self.request(&PermissionType::Run(command.to_string()), context)
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all_permits_everything() {
        let mut pm = PermissionManager::allow_all();

        assert_eq!(
            pm.check(&PermissionType::Read("/tmp/test".to_string())),
            PermissionState::Granted
        );
        assert_eq!(
            pm.check(&PermissionType::Write("/tmp/test".to_string())),
            PermissionState::Granted
        );
        assert_eq!(
            pm.check(&PermissionType::Net("example.com".to_string())),
            PermissionState::Granted
        );
        assert_eq!(
            pm.check(&PermissionType::Run("ls".to_string())),
            PermissionState::Granted
        );
    }

    #[test]
    fn test_specific_flags() {
        let mut pm = PermissionManager::with_flags(false, true, false, false, false);

        assert_eq!(
            pm.check(&PermissionType::Read("/tmp/test".to_string())),
            PermissionState::Granted
        );
        assert_eq!(
            pm.check(&PermissionType::Write("/tmp/test".to_string())),
            PermissionState::Prompt
        );
    }

    #[test]
    fn test_default_requires_prompt() {
        let mut pm = PermissionManager::new();
        pm.interactive = false; // Disable interactive mode for test

        assert_eq!(
            pm.check(&PermissionType::Read("/tmp/test".to_string())),
            PermissionState::Prompt
        );
    }

    #[test]
    fn test_cached_permission_grants() {
        let mut pm = PermissionManager::new();
        let perm = PermissionType::Read("/tmp/test".to_string());

        pm.granted.insert(perm.clone(), true);
        assert_eq!(pm.check(&perm), PermissionState::Granted);
    }

    #[test]
    fn test_cached_permission_denies() {
        let mut pm = PermissionManager::new();
        let perm = PermissionType::Read("/tmp/test".to_string());

        pm.granted.insert(perm.clone(), false);
        assert_eq!(pm.check(&perm), PermissionState::Denied);
    }

    #[test]
    fn test_protected_path_detection() {
        // Test that the permissions directory is detected as protected
        #[cfg(target_family = "unix")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let protected_path = format!("{}/.local/share/loft/permissions.json", home);
                assert!(PermissionManager::is_protected_path(&protected_path));

                let safe_path = "/tmp/test.txt";
                assert!(!PermissionManager::is_protected_path(safe_path));
            }
        }

        #[cfg(target_family = "windows")]
        {
            if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
                let protected_path = format!("{}\\loft\\permissions.json", appdata);
                assert!(PermissionManager::is_protected_path(&protected_path));

                let safe_path = "C:\\temp\\test.txt";
                assert!(!PermissionManager::is_protected_path(safe_path));
            }
        }
    }
}
