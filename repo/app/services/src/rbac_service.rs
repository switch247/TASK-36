use anyhow::Result;

use app_core::errors::CoreError;
use app_core::types::UserRole;

pub struct RbacService;

impl RbacService {
    pub fn require_manage_users(role: &UserRole) -> Result<()> {
        if role.can_manage_users() {
            Ok(())
        } else {
            Err(CoreError::AuthorizationDenied.into())
        }
    }

    pub fn require_manage_inventory(role: &UserRole) -> Result<()> {
        if role.can_manage_inventory() {
            Ok(())
        } else {
            Err(CoreError::AuthorizationDenied.into())
        }
    }

    pub fn require_reporting(role: &UserRole) -> Result<()> {
        if role.can_view_reporting() {
            Ok(())
        } else {
            Err(CoreError::AuthorizationDenied.into())
        }
    }

    pub fn require_print(role: &UserRole) -> Result<()> {
        if role.can_run_prints() {
            Ok(())
        } else {
            Err(CoreError::AuthorizationDenied.into())
        }
    }
}
