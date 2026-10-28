//! Project links (Help menu, About, CLI, MCP): Oracle Layout's repository.

use serde_json::{Value, json};

/// The app's slug on GitHub.
pub const APP: &str = "oracle-layout";
/// The GitHub repository: it keeps the app's former name (Oracle Brand Guide) until it is renamed.
pub const REPO: &str = "oracle-guide";
/// Oracle Suite: the launcher of every Oracle app (Help ▸ Oracle Suite).
pub const DISCORD: &str = "https://github.com/Marvingxx/oracle-suite";
/// The project's home.
pub const WEBSITE: &str = "https://github.com/Marvingxx/oracle-guide";
/// Oracle Layout's home page.
pub const APP_PAGE: &str = "https://github.com/Marvingxx/oracle-guide";
/// Oracle Layout's source repository.
pub const GITHUB: &str = "https://github.com/Marvingxx/oracle-guide";
/// Where to report bugs and request features.
pub const ISSUES: &str = "https://github.com/Marvingxx/oracle-guide/issues";

/// All links as JSON (the `app.links` command).
pub fn all() -> Value {
    json!({
        "discord": DISCORD,
        "website": WEBSITE,
        "appPage": APP_PAGE,
        "github": GITHUB,
        "issues": ISSUES,
    })
}

/// A link by key (`discord`, `website`, `appPage`, `github`, `issues`).
pub fn get(key: &str) -> Option<&'static str> {
    Some(match key {
        "discord" => DISCORD,
        "website" => WEBSITE,
        "appPage" => APP_PAGE,
        "github" => GITHUB,
        "issues" => ISSUES,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_follow_the_app_slug() {
        assert!(GITHUB.ends_with(&format!("/Marvingxx/{REPO}")));
        assert!(ISSUES.starts_with(GITHUB));
        for k in ["discord", "website", "appPage", "github", "issues"] {
            assert!(get(k).is_some_and(|u| u.starts_with("https://")), "{k}");
            assert_eq!(all()[k], get(k).unwrap());
        }
    }
}
