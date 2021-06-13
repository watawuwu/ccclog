mod commit;
mod conventional_commit;
mod github_url;
mod repository;
mod version;

use std::convert::From;
use std::path::Path;

use anyhow::*;
use git2::{self, Repository};
use log::*;
use repository::{Findable, TagFindable};

pub use commit::*;
pub use conventional_commit::*;
pub use github_url::GithubUrl;

use version::*;

pub fn repo<P: AsRef<Path>>(path: P) -> Result<Repository> {
    Repository::open(&path).context("Not found git repository path")
}

pub fn gurl(repo: &Repository) -> Option<GithubUrl> {
    let url = repo.remote_url();
    url.map(|u| GithubUrl::new(u.as_str()))
}

pub fn commits(repo: &Repository, spec: Option<&str>, tag_prefix: Option<&str>) -> Result<Commits> {
    let range = match spec {
        Some(s) => parse_range(repo, s)?,
        None => {
            let mut versions = repo.versions(tag_prefix)?;
            detect_range(repo, &mut versions)?
        }
    };
    debug!("scan range: {:?}", &range);

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
    use tempfile::tempdir;

    const GIT_DATA1: &[u8] = include_bytes!("../../tests/assets/git-data1.tar.gz");
    const GIT_DATA2: &[u8] = include_bytes!("../../tests/assets/git-data2.tar.gz");
    const GIT_DATA3: &[u8] = include_bytes!("../../tests/assets/git-data3.tar.gz");
    const GIT_DATA4: &[u8] = include_bytes!("../../tests/assets/git-data4.tar.gz");

    pub fn git_dir(num: u8) -> Result<PathBuf> {
        let buf = match num {
            1 => GIT_DATA1.as_ref(),
            2 => GIT_DATA2.as_ref(),
            3 => GIT_DATA3.as_ref(),
            4 => GIT_DATA4.as_ref(),
            _ => bail!("Not found test git data"),
        };
        let tmp_dir = tempdir()?;
        let prefix = tmp_dir.into_path();

        let tar = GzDecoder::new(buf);
        let mut archive = Archive::new(tar);
        archive.unpack(&prefix)?;
        Ok(prefix.join(format!("git-data{}", num)))
    }

    pub fn dummy_commit(
        id: &str,
        _type: &str,
        scope: Option<&str>,
        break_change: bool,
        description: &str,
        author: &str,
        datetime: &str,
        parent_count: usize,
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
        let tag = tag.map(|x| NamableObj::Tag {
            version: Version::from_str(x).unwrap(),
            datetime,
        });

        let commit = Commit::new(id, &summary, author, datetime, parent_count, Some(cc), tag)?;

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
        let tag = tag.map(|x| NamableObj::Tag {
            version: Version::from_str(x).unwrap(),
            datetime,
        });
        let commit = Commit::new(id, summary, author, datetime, 1, None, tag)?;

        Ok(commit)
    }

    pub fn dummy_commits() -> Result<Commits> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "test",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            1,
            Some("0.1.0"),
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "fix",
            None,
            false,
            "add 2",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:02 2020 +0000",
            1,
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 1",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            1,
            None,
        )?;
        commits.push(commit);

        let prev = prev()?;
        Ok(Commits::new(prev, commits))
    }

    pub fn prev() -> Result<Commit> {
        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            1,
            Some("0.0.0"),
        )?;

        Ok(prev)
    }

    #[test]
    fn get_ok() -> Result<()> {
        let git_dir = git_dir(1)?;
        let repo = repo(git_dir);
        assert!(repo.is_ok());
        Ok(())
    }

    #[test]
    fn detect_range_ok() -> Result<()> {
        let git_dir = git_dir(3)?;
        let repo = repo(git_dir)?;

        let mut versions = Versions::from(vec![
            Version::from_str("1.0.0")?,
            Version::from_str("1.1.0")?,
        ]);

        let a = detect_range(&repo, &mut versions)?;
        let latest = dummy_commit(
            "cd3354bedd0c7b66a899d27a2e66ff41594df0b1",
            "feat",
            None,
            false,
            "8",
            "Test User <test-user@test.com>",
            "Thu May 21 21:54:57 2020 +0900",
            1,
            Some("1.1.0"),
        )?;
        let prev = dummy_commit(
            "9a5e72a6ade1f3b6975711f3bf05a82f1793c0b4",
            "feat",
            None,
            false,
            "7",
            "Test User <test-user@test.com>",
            "Thu May 21 21:54:46 2020 +0900",
            1,
            Some("1.0.0"),
        )?;
        let e = ScanRange::new(Some(latest), prev);

        assert_eq!(a, e);
        Ok(())
    }
}
