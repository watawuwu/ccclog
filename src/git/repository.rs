use crate::git::version::{Version, Versions};
use crate::git::{Commit, ScanRange};
use anyhow::*;
use git2::Repository;
use regex::Regex;
use std::str::FromStr;

pub(super) trait Findable<T, R> {
    fn find_by(&self, v: &T) -> Result<R>;
}

impl Findable<Version, Commit> for Repository {
    // TODO chang return type to more simple type
    fn find_by(&self, version: &Version) -> Result<Commit> {
        let obj = self.revparse_single(version.to_string().as_str())?;
        let commit = Commit::from(obj.peel_to_commit()?);
        Ok(commit)
    }
}

impl Findable<ScanRange, Vec<Commit>> for Repository {
    fn find_by(&self, range: &ScanRange) -> Result<Vec<Commit>> {
        let mut rev = self.revwalk()?;
        match range.latest_id() {
            Some(id) => rev.push(id.clone())?,
            None => rev.push_head()?,
        };
        let commits = rev
            .take_while(|oid| match oid {
                Ok(id) => id != range.prev_id(),
                Err(_) => false,
            })
            .filter_map(|id| id.ok())
            .filter_map(|id| self.find_commit(id).ok())
            .map(Commit::from)
            .collect::<Vec<Commit>>();

        Ok(commits)
    }
}

pub(super) trait TagFindable {
    fn versions(&self, tag_pattern: &Regex) -> Result<Versions>;
    fn remote_url(&self) -> Option<String>;
}

impl TagFindable for Repository {
    fn versions(&self, tag_pattern: &Regex) -> Result<Versions> {
        let tags = self.tag_names(None)?;
        let versions = tags
            .into_iter()
            .filter_map(|t| t)
            .filter(|t| tag_pattern.is_match(t))
            .filter_map(|t| Version::from_str(t).ok())
            .collect::<Versions>();
        Ok(versions)
    }

    // TODO change to get from config
    fn remote_url(&self) -> Option<String> {
        self.find_remote("origin")
            .ok()
            .and_then(|r| r.url().map(String::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::*;
    use crate::git::SEMVER_PATTERN;

    #[test]
    fn semantic_tags_ok() -> Result<()> {
        let git_dir = git_dir()?;
        let repo = Repository::open(git_dir)?;

        let versions = repo.versions(&SEMVER_PATTERN)?;
        let expect = vec![Version::from_str("0.1.0")?, Version::from_str("0.2.0")?]
            .into_iter()
            .collect::<Versions>();

        assert_eq!(versions, expect);
        Ok(())
    }

    #[test]
    fn find_by_scan_range_ok() -> Result<()> {
        let git_dir = git_dir()?;
        let repo = Repository::open(git_dir)?;

        let latest = dummy_commit(
            "9cd36629bddcf2ce9cfc16fcfbd9ea48815b2dc8",
            "feat",
            None,
            false,
            "new fun",
            "Test User <test-user@test.com>",
            "Wed Apr 29 16:31:39 2020 +0900",
            1,
            None,
        )?;

        let previous = dummy_commit(
            "9fa3647bfd047ee3c4c120a492065fa6f1c97bcb",
            "chore",
            None,
            false,
            "add README",
            "Test User <test-user@test.com>",
            "Wed Apr 29 16:29:47 2020 +0900",
            1,
            None,
        )?;

        let range = ScanRange::new(Some(latest), previous);

        let commits = repo.find_by(&range)?;
        let actual = commits
            .iter()
            .map(|c| c.id.to_string())
            .collect::<Vec<String>>();
        let expected = vec![
            "9cd36629bddcf2ce9cfc16fcfbd9ea48815b2dc8",
            "6f904822757b9d40ba885d946f9e78a7b5b63ddf",
            "a673434d9fa4efc63c7026a426a36841b247f446",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();
        assert_eq!(actual, expected);

        Ok(())
    }
}
