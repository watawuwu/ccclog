mod commit;
mod conventional_commit;
mod github_url;
mod repository;
mod version;

use std::convert::From;
use std::path::Path;

use anyhow::*;
use git2::{self, Repository};
use repository::{Findable, TagFindable};

pub use commit::*;
pub use conventional_commit::*;
pub use github_url::GithubUrl;
use version::*;

pub fn repo<P: AsRef<Path>>(path: P) -> Result<Repository> {
    Ok(Repository::open(&path).context("Not found git repository path")?)
}

pub fn gurl(repo: &Repository) -> Option<GithubUrl> {
    let url = repo.remote_url();
    url.map(|u| GithubUrl::new(u.as_str()))
}

pub fn commits(repo: &Repository, spec: Option<&str>) -> Result<Commits> {
    let range = match spec {
        Some(s) => parse_range(repo, s)?,
        None => {
            let mut versions = repo.versions()?;
            detect_range(repo, &mut versions)?
        }
    };

    let list = repo.find_by(&range)?;
    let commits = Commits::new(range.prev(), list);
    Ok(commits)
}

fn parse_range(repo: &Repository, spec: &str) -> Result<ScanRange> {
    let revspec = repo.revparse(spec).context("Invalid revspec")?;
    if !revspec.mode().contains(git2::RevparseMode::RANGE) {
        anyhow::bail!("Don't support mode. Supported mode is only range(two-dot)")
    }

    let from = revspec
        .from()
        .and_then(|o| o.peel_to_commit().ok())
        .map(Commit::from);
    let to = revspec
        .to()
        .and_then(|o| o.peel_to_commit().ok())
        .map(Commit::from);
    // revspec from..to is reversed when scanning
    let (latest, previous) = match (to, from) {
        (Some(l), Some(p)) => (Some(l), p),
        (Some(l), None) => (Some(l), Commit::empty()?),
        _ => (None, Commit::empty()?),
    };
    Ok(ScanRange::new(latest, previous))
}

fn detect_range(repo: &Repository, vs: &mut Versions) -> Result<ScanRange> {
    let (latest, previous) = match vs.latest_range() {
        (Some(l), Some(p)) => (Some(repo.find_by(l)?), repo.find_by(p)?),
        (Some(l), None) => (Some(repo.find_by(l)?), Commit::empty()?),
        _ => (None, Commit::empty()?),
    };
    Ok(ScanRange::new(latest, previous))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use std::path::PathBuf;

    use anyhow::Result;
    use chrono::{DateTime, Utc};
    use flate2::read::GzDecoder;
    use git2::Oid;
    use std::str::FromStr;
    use tar::Archive;
    use tempdir::TempDir;

    const GIT_DATA1: &[u8] = include_bytes!("../../tests/assets/git-data1.tar.gz");

    pub fn git_dir() -> Result<PathBuf> {
        let tmp_dir = TempDir::new("")?;
        let prefix = tmp_dir.into_path();

        let tar = GzDecoder::new(GIT_DATA1.as_ref());
        let mut archive = Archive::new(tar);
        archive.unpack(&prefix)?;
        Ok(prefix.join("git-data1"))
    }

    pub fn dummy_commit(
        id: &str,
        _type: &str,
        scope: Option<&str>,
        break_change: bool,
        description: &str,
        author: &str,
        datetime: &str,
        tag: Option<&str>,
    ) -> Result<Commit> {
        let cc_scope = scope.map(String::from);
        let cc = ConventionalCommits::new(
            break_change,
            CommitType::from_str(_type)?,
            cc_scope,
            description,
        );
        let _type = scope.map_or_else(|| _type.to_string(), |s| format!("{}({})", _type, s));
        let summary = format!("{}: {}", _type, description);
        let datetime = DateTime::parse_from_str(datetime, "%a %b %d %H:%M:%S %Y %z")?;
        let datetime = datetime.with_timezone(&Utc);
        let id = Oid::from_str(id)?;
        let tag = tag.map(|t| NamableObj::new(t, datetime));

        let commit = Commit::new(id, &summary, author, datetime, Some(cc), tag)?;

        Ok(commit)
    }

    pub fn dummy_invalid_commit(
        id: &str,
        summary: &str,
        author: &str,
        datetime: &str,
        tag: Option<&str>,
    ) -> Result<Commit> {
        let datetime = DateTime::parse_from_str(datetime, "%a %b %d %H:%M:%S %Y %z")?;
        let datetime = datetime.with_timezone(&Utc);
        let id = Oid::from_str(id)?;
        let tag = tag.map(|t| NamableObj::new(t, datetime));
        let commit = Commit::new(id, summary, author, datetime, None, tag)?;

        Ok(commit)
    }

    #[test]
    fn get_ok() -> Result<()> {
        let git_dir = git_dir()?;
        let repo = repo(git_dir);
        assert!(repo.is_ok());
        Ok(())
    }
}
