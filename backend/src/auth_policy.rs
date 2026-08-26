pub fn requires_two_factor_enrollment(role: &str, policy_enabled: bool, exempt: bool) -> bool {
    policy_enabled
        && !exempt
        && matches!(
            role,
            "system_admin"
                | "competition_manager"
                | "chief_judge"
                | "evaluation_manager"
                | "jury_member"
        )
}

#[cfg(test)]
#[path = "auth_policy_tests.rs"]
mod tests;
