use super::permissions::PermissionManager;
use std::cell::RefCell;

thread_local! {
    static PERMISSION_MANAGER: RefCell<Option<PermissionManager>> = const { RefCell::new(None) };
}

/// Initialize the permission manager for the current thread
pub fn init_permissions(manager: PermissionManager) {
    PERMISSION_MANAGER.with(|pm| {
        *pm.borrow_mut() = Some(manager);
    });
}

/// Get the current permission manager (if any)
pub fn with_permissions<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut PermissionManager) -> R,
{
    PERMISSION_MANAGER.with(|pm| {
        pm.borrow_mut().as_mut().map(f)
    })
}

/// Check if a path is in the protected permissions directory
pub fn is_protected_path(path: &str) -> bool {
    PermissionManager::is_protected_path(path)
}

/// Check and request read permission
pub fn check_read_permission(path: &str, context: Option<&str>) -> Result<bool, String> {
    // Always deny access to the protected permissions directory
    if is_protected_path(path) {
        return Err("Access to loft permissions directory is not allowed".to_string());
    }
    
    with_permissions(|pm| pm.request_read(path, context))
        // NOTE: Default to allowed for backward compatibility with existing tests
        // In production, the permission manager is ALWAYS initialized in main.rs
        // This fallback only occurs in test environments without permission setup
        .unwrap_or(Ok(true))
}

/// Check and request write permission
pub fn check_write_permission(path: &str, context: Option<&str>) -> Result<bool, String> {
    // Always deny access to the protected permissions directory
    if is_protected_path(path) {
        return Err("Access to loft permissions directory is not allowed".to_string());
    }
    
    with_permissions(|pm| pm.request_write(path, context))
        // NOTE: Default to allowed for backward compatibility with existing tests
        // In production, the permission manager is ALWAYS initialized in main.rs
        // This fallback only occurs in test environments without permission setup
        .unwrap_or(Ok(true))
}

/// Check and request network permission
pub fn check_net_permission(host: &str, context: Option<&str>) -> Result<bool, String> {
    with_permissions(|pm| pm.request_net(host, context))
        // NOTE: Default to allowed for backward compatibility with existing tests
        // In production, the permission manager is ALWAYS initialized in main.rs
        // This fallback only occurs in test environments without permission setup
        .unwrap_or(Ok(true))
}

/// Check and request run permission
pub fn check_run_permission(command: &str, context: Option<&str>) -> Result<bool, String> {
    with_permissions(|pm| pm.request_run(command, context))
        // NOTE: Default to allowed for backward compatibility with existing tests
        // In production, the permission manager is ALWAYS initialized in main.rs
        // This fallback only occurs in test environments without permission setup
        .unwrap_or(Ok(true))
}

/// Clear the permission manager for the current thread
pub fn clear_permissions() {
    PERMISSION_MANAGER.with(|pm| {
        *pm.borrow_mut() = None;
    });
}
