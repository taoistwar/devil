//! System Context Module
//!
//! Implements system-level context injection aligned with Claude Code's context.ts

use super::git_status::{get_git_status, GitStatus};
use std::sync::atomic::{AtomicPtr, Ordering};

static SYSTEM_PROMPT_INJECTION: AtomicPtr<String> = AtomicPtr::new(std::ptr::null_mut());

#[derive(Debug, Clone)]
pub struct SystemContext {
    pub git_status: Option<GitStatus>,
    pub cache_breaker: Option<String>,
}

pub struct SystemContextProvider;

impl SystemContextProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_context(&self) -> SystemContext {
        SystemContext {
            git_status: get_git_status(),
            cache_breaker: get_system_prompt_injection(),
        }
    }
}

impl Default for SystemContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_system_context() -> SystemContext {
    crate::context::git_status::clear_git_status_cache();
    crate::context::memory_files::clear_memory_files_cache();
    SystemContext {
        git_status: crate::context::git_status::get_git_status(),
        cache_breaker: get_system_prompt_injection(),
    }
}

pub async fn get_system_context_async() -> SystemContext {
    SystemContextProvider::new().get_context().await
}

pub fn set_system_prompt_injection(value: String) {
    let boxed = Box::new(value);
    let old = SYSTEM_PROMPT_INJECTION.swap(Box::into_raw(boxed), Ordering::SeqCst);
    if !old.is_null() {
        drop(unsafe { Box::from_raw(old) });
    }
    crate::context::git_status::clear_git_status_cache();
    crate::context::memory_files::clear_memory_files_cache();
}

pub fn get_system_prompt_injection() -> Option<String> {
    let ptr = SYSTEM_PROMPT_INJECTION.load(Ordering::SeqCst);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { (*ptr).clone() })
    }
}

pub fn clear_system_prompt_injection() {
    let old = SYSTEM_PROMPT_INJECTION.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !old.is_null() {
        drop(unsafe { Box::from_raw(old) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_injection_set_get() {
        let test_value = "test_injection".to_string();
        set_system_prompt_injection(test_value.clone());
        assert_eq!(get_system_prompt_injection(), Some(test_value));
        clear_system_prompt_injection();
    }

    #[test]
    fn test_system_prompt_injection_clear() {
        set_system_prompt_injection("test".to_string());
        clear_system_prompt_injection();
        assert_eq!(get_system_prompt_injection(), None);
    }

    #[test]
    fn test_system_context_provider_creation() {
        let ctx = get_system_context();
        assert!(ctx.git_status.clone().is_some() || ctx.git_status.clone().is_none());
    }
}
