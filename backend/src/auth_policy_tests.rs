use super::*;

#[test]
fn privileged_users_require_two_factor_when_the_policy_is_enabled() {
    assert!(requires_two_factor_enrollment("system_admin", true, false));
    assert!(requires_two_factor_enrollment("jury_member", true, false));
}

#[test]
fn an_exemption_affects_only_the_targeted_account() {
    assert!(!requires_two_factor_enrollment("system_admin", true, true));
    assert!(requires_two_factor_enrollment("system_admin", true, false));
}

#[test]
fn disabled_policy_does_not_require_enrollment() {
    assert!(!requires_two_factor_enrollment(
        "system_admin",
        false,
        false
    ));
}
