use crate::secure_compare_support::constant_time_eq_str;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequiredAccountRole {
    User,
    Admin,
    SuperAdmin,
}

pub(crate) fn account_has_role(
    is_admin: i8,
    is_super_admin: i8,
    required: RequiredAccountRole,
) -> bool {
    match required {
        RequiredAccountRole::User => true,
        RequiredAccountRole::Admin => is_admin != 0 || is_super_admin != 0,
        RequiredAccountRole::SuperAdmin => is_super_admin != 0,
    }
}

pub(crate) fn token_has_full_access_ability(raw: Option<&str>) -> bool {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    serde_json::from_str::<Vec<String>>(raw)
        .ok()
        .is_some_and(|abilities| abilities.iter().any(|ability| ability == "*"))
}

pub(crate) fn is_valid_secure_admin_path(value: &str) -> bool {
    const RESERVED: &[&str] = &[
        "api",
        "app",
        "assets",
        "bootstrap",
        "healthz",
        "login",
        "monitor",
        "robots.txt",
        "tcping-agent-src",
        "theme",
    ];

    if value != value.trim() || !(8..=64).contains(&value.len()) {
        return false;
    }
    let mut chars = value.chars();
    if !chars.next().is_some_and(|ch| ch.is_ascii_alphanumeric()) {
        return false;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')) {
        return false;
    }
    !RESERVED
        .iter()
        .any(|reserved| value.eq_ignore_ascii_case(reserved))
}

pub(crate) fn secure_admin_path_matches(configured: &str, candidate: &str) -> bool {
    is_valid_secure_admin_path(configured)
        && is_valid_secure_admin_path(candidate)
        && constant_time_eq_str(configured, candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_roles_form_an_explicit_lattice() {
        assert!(account_has_role(0, 0, RequiredAccountRole::User));
        assert!(!account_has_role(0, 0, RequiredAccountRole::Admin));
        assert!(account_has_role(1, 0, RequiredAccountRole::Admin));
        assert!(!account_has_role(1, 0, RequiredAccountRole::SuperAdmin));
        assert!(account_has_role(0, 1, RequiredAccountRole::Admin));
        assert!(account_has_role(0, 1, RequiredAccountRole::SuperAdmin));
    }

    #[test]
    fn scoped_or_malformed_tokens_are_not_promoted_to_full_access() {
        assert!(token_has_full_access_ability(Some(r#"["*"]"#)));
        assert!(token_has_full_access_ability(Some(r#"["read","*"]"#)));
        for abilities in [None, Some(""), Some("null"), Some(r#"["read"]"#), Some("not-json")] {
            assert!(!token_has_full_access_ability(abilities));
        }
    }

    #[test]
    fn secure_admin_paths_are_single_safe_non_reserved_segments() {
        for value in ["a1b2c3d4", "Admin_2026", "private-entry-01"] {
            assert!(is_valid_secure_admin_path(value), "expected valid path: {value}");
        }
        for value in [
            "short",
            "bootstrap",
            "../private",
            "private/entry",
            "private%2fentry",
            " private01",
            "private01 ",
            "private.entry",
            "private entry",
        ] {
            assert!(!is_valid_secure_admin_path(value), "expected invalid path: {value}");
        }
    }

    #[test]
    fn secure_admin_path_matching_fails_closed() {
        assert!(secure_admin_path_matches("private01", "private01"));
        assert!(!secure_admin_path_matches("", ""));
        assert!(!secure_admin_path_matches("private01", "PRIVATE01"));
        assert!(!secure_admin_path_matches("private01", "private01/"));
    }
}
