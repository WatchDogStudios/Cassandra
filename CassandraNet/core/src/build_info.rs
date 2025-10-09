use serde::Serialize;

/// Build metadata captured at compile time.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BuildInfo {
    pub package: &'static str,
    pub version: &'static str,
    pub git_sha: &'static str,
    pub git_tag: &'static str,
    pub build_timestamp: &'static str,
}

pub fn build_info() -> BuildInfo {
    BuildInfo {
        package: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        git_sha: option_env!("CORE_GIT_SHA").unwrap_or(option_env!("GIT_SHA").unwrap_or("unknown")),
        git_tag: option_env!("CORE_GIT_TAG").unwrap_or(option_env!("GIT_TAG").unwrap_or("none")),
        build_timestamp: option_env!("CORE_BUILD_TIMESTAMP")
            .unwrap_or(option_env!("BUILD_TIMESTAMP").unwrap_or("unknown")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_info_has_version() {
        let info = build_info();
        assert!(!info.version.is_empty());
        assert_eq!(info.package, "cncore");
    }
}
