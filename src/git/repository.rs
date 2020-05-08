use crate::git::version::{Version, Versions};
use crate::git::{Commit, ScanRange};
use anyhow::*;
use git2::Repository;

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
            .filter_map(|id| id.ok())
            .filter_map(|id| self.find_commit(id).ok())
            // This is exactly the same as --no-merge
            .filter(|c| c.parent_count() == 1)
            .take_while(|c| range.prev_id() != &c.id())
            .map(Commit::from)
            .collect::<Vec<Commit>>();

        Ok(commits)
    }
}

pub(super) trait TagFindable {
    /*
    fn tag_names_hmap(&self) -> Result<HashMap<Oid, String>>;
    */
    fn versions(&self) -> Result<Versions>;
    fn remote_url(&self) -> Option<String>;
}

impl TagFindable for Repository {
    /*
    fn tag_names_hmap(&self) -> Result<HashMap<Oid, String>> {
        let hmap = self
            .tag_names(None)?
            .iter()
            .filter_map(|n| n)
            .flat_map(|n| {
                let obj = self.revparse_single(n).ok()?;
                if let Some(_tag) = obj.as_tag() {
                    let tag_commit = obj.peel_to_commit().ok()?;
                    Some((tag_commit.id(), n.to_string()))
                } else if let Some(commit) = obj.as_commit() {
                    Some((commit.id(), n.to_string()))
                } else {
                    None
                }
            })
            .collect::<HashMap<Oid, String>>();
        Ok(hmap)
    }
    */

    fn versions(&self) -> Result<Versions> {
        let tags = self.tag_names(None)?;
        let versions = tags
            .into_iter()
            .filter_map(|t| t)
            .filter_map(|t| Version::parse(t).ok())
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

    #[test]
    fn semantic_tags_ok() -> Result<()> {
        let git_dir = git_dir()?;
        let repo = Repository::open(git_dir)?;

        let versions = repo.versions()?;
        let expect = vec![Version::parse("0.1.0")?, Version::parse("0.2.0")?]
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
