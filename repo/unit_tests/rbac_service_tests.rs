#[cfg(test)]
mod tests {
    use app_core::types::UserRole;
    use app_services::rbac_service::RbacService;

    #[test]
    fn admin_is_only_role_that_can_manage_users() {
        assert!(RbacService::require_manage_users(&UserRole::Admin).is_ok());
        assert!(RbacService::require_manage_users(&UserRole::Coordinator).is_err());
        assert!(RbacService::require_manage_users(&UserRole::Proctor).is_err());
        assert!(RbacService::require_manage_users(&UserRole::Auditor).is_err());
    }

    #[test]
    fn inventory_permissions_match_role_matrix() {
        assert!(RbacService::require_manage_inventory(&UserRole::Admin).is_ok());
        assert!(RbacService::require_manage_inventory(&UserRole::Coordinator).is_ok());
        assert!(RbacService::require_manage_inventory(&UserRole::Proctor).is_err());
        assert!(RbacService::require_manage_inventory(&UserRole::Auditor).is_err());
    }

    #[test]
    fn reporting_and_print_permissions_follow_expected_roles() {
        assert!(RbacService::require_reporting(&UserRole::Admin).is_ok());
        assert!(RbacService::require_reporting(&UserRole::Coordinator).is_ok());
        assert!(RbacService::require_reporting(&UserRole::Auditor).is_ok());
        assert!(RbacService::require_reporting(&UserRole::Proctor).is_err());

        assert!(RbacService::require_print(&UserRole::Admin).is_ok());
        assert!(RbacService::require_print(&UserRole::Coordinator).is_ok());
        assert!(RbacService::require_print(&UserRole::Proctor).is_ok());
        assert!(RbacService::require_print(&UserRole::Auditor).is_err());
    }
}
