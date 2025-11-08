// Git integration for package manager

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git repository information
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub url: String,
    pub version: String,
    pub path: PathBuf,
}

/// Clone a git repository to the cache directory
pub fn clone_repository(url: &str, cache_path: &Path) -> Result<PathBuf> {
    // Extract repo name from URL
    let repo_name = extract_repo_name(url)?;
    let repo_path = cache_path.join(&repo_name);

    // Skip if already cloned
    if repo_path.exists() {
        return Ok(repo_path);
    }

    // Clone the repository
    let output = Command::new("git")
        .args(&["clone", url, repo_path.to_str().unwrap()])
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git clone failed: {}", stderr);
    }

    Ok(repo_path)
}

/// Checkout a specific tag or commit
pub fn checkout_tag(repo_path: &Path, tag: &str) -> Result<()> {
    let output = Command::new("git")
        .args(&["-C", repo_path.to_str().unwrap(), "checkout", tag])
        .output()
        .context("Failed to execute git checkout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git checkout failed: {}", stderr);
    }

    Ok(())
}

/// Fetch latest tags from remote
pub fn fetch_tags(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(&["-C", repo_path.to_str().unwrap(), "fetch", "--tags"])
        .output()
        .context("Failed to execute git fetch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git fetch failed: {}", stderr);
    }

    Ok(())
}

/// Get all tags from repository
pub fn get_tags(repo_path: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["-C", repo_path.to_str().unwrap(), "tag", "-l"])
        .output()
        .context("Failed to execute git tag")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Git tag failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(tags)
}

/// Get latest tag matching semver pattern
pub fn get_latest_tag(repo_path: &Path) -> Result<String> {
    let tags = get_tags(repo_path)?;

    if tags.is_empty() {
        bail!("No tags found in repository");
    }

    // Filter semver tags (vX.Y.Z)
    let semver_tags: Vec<&String> = tags
        .iter()
        .filter(|t| t.starts_with('v') && t.matches('.').count() == 2)
        .collect();

    if semver_tags.is_empty() {
        bail!("No semver tags found in repository");
    }

    // Return last tag (assumes git tag -l returns sorted)
    Ok(semver_tags.last().unwrap().to_string())
}

/// Extract repository name from URL
fn extract_repo_name(url: &str) -> Result<String> {
    // Handle different URL formats:
    // - https://github.com/user/repo.git
    // - git@github.com:user/repo.git
    // - github.com/user/repo

    let url = url.trim_end_matches(".git");

    let name = if url.contains("://") {
        // HTTPS URL
        url.split('/').last()
    } else if url.contains(':') {
        // SSH URL
        url.split(':').last().and_then(|s| s.split('/').last())
    } else {
        // Plain format (github.com/user/repo)
        url.split('/').last()
    };

    name.map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid repository URL: {}", url))
}

/// Convert package URL to git URL
pub fn package_url_to_git_url(package_url: &str) -> String {
    // github.com/user/repo -> https://github.com/user/repo.git
    // gitlab:company/repo -> https://gitlab.com/company/repo.git
    // bitbucket:team/proj -> https://bitbucket.org/team/proj.git

    if package_url.starts_with("http://") || package_url.starts_with("https://") {
        return package_url.to_string();
    }

    if let Some(gitlab_path) = package_url.strip_prefix("gitlab:") {
        return format!("https://gitlab.com/{}.git", gitlab_path);
    }

    if let Some(bitbucket_path) = package_url.strip_prefix("bitbucket:") {
        return format!("https://bitbucket.org/{}.git", bitbucket_path);
    }

    // Default to GitHub
    if package_url.starts_with("github.com/") || package_url.starts_with("github:") {
        let path = package_url
            .strip_prefix("github.com/")
            .or_else(|| package_url.strip_prefix("github:"))
            .unwrap();
        return format!("https://github.com/{}.git", path);
    }

    // Assume GitHub if no prefix
    format!("https://github.com/{}.git", package_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(extract_repo_name("github.com/user/repo").unwrap(), "repo");
    }

    #[test]
    fn test_package_url_to_git_url() {
        assert_eq!(
            package_url_to_git_url("github.com/user/repo"),
            "https://github.com/user/repo.git"
        );
        assert_eq!(
            package_url_to_git_url("gitlab:company/repo"),
            "https://gitlab.com/company/repo.git"
        );
        assert_eq!(
            package_url_to_git_url("bitbucket:team/proj"),
            "https://bitbucket.org/team/proj.git"
        );
    }
}
