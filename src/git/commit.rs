use super::ConventionalCommits;

use chrono::{DateTime, NaiveDateTime, Utc};
use git2::{Commit as LibCommit, DescribeOptions, Oid as LibOid, Oid, Signature};

use std::cmp::Ordering;

use std::convert::From;
use std::hash::Hash;

use crate::git::CommitType;
use anyhow::*;
use lazy_static::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::option::Option;
use std::str::FromStr;

const EMPTY_HASH: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

#[derive(Debug, PartialEq)]
pub struct Commits {
    // TODO remove this struct
    prev: Commit,
    commits: Vec<Commit>,
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ReleaseRange {
    Release(NamableObj, NamableObj),
    UnRelease(NamableObj),
}

impl Commits {
    pub(crate) fn new(prev: Commit, commits: Vec<Commit>) -> Self {
        Commits { prev, commits }
    }

    // TODO refactor
    pub fn group_by(&self) -> Vec<(ReleaseRange, BTreeMap<CommitType, Vec<&Commit>>)> {
        let mut releases: Vec<(ReleaseRange, BTreeMap<CommitType, Vec<&Commit>>)> = Vec::new();

        let (obj, vec) =
            self.commits
                .iter()
                .fold((None, Vec::new()), |(latest, mut acc), commit| {
                    match (latest.clone(), commit.tag()) {
                        (Some(latest_obj), Some(current_obj)) => {
                            releases.push((
                                ReleaseRange::Release(current_obj.clone(), latest_obj),
                                self.group_by_commit_type(acc),
                            ));
                            (Some(current_obj.clone()), vec![commit])
                        }
                        (None, Some(current_obj)) => {
                            if !acc.is_empty() {
                                releases.push((
                                    ReleaseRange::UnRelease(current_obj.clone()),
                                    self.group_by_commit_type(acc),
                                ));
                            }
                            (Some(current_obj.clone()), vec![commit])
                        }
                        _ => {
                            acc.push(commit);
                            (latest, acc)
                        }
                    }
                });

        let bmap = self.group_by_commit_type(vec);
        let prev = self.prev_obj();
        match obj {
            Some(n) => releases.push((ReleaseRange::Release(prev, n), bmap)),
            None => releases.push((ReleaseRange::UnRelease(prev), bmap)),
        };

        releases
    }

    fn group_by_commit_type<'a>(
        &self,
        vec: Vec<&'a Commit>,
    ) -> BTreeMap<CommitType, Vec<&'a Commit>> {
        vec.into_iter()
            .map(|x| (x.raw_type(), x))
            .fold(BTreeMap::new(), |mut acc, (k, v)| {
                acc.entry(k).or_insert_with(Vec::new).push(v);
                acc
            })
    }

    fn prev_obj(&self) -> NamableObj {
        match self.prev.obj.as_ref() {
            Some(n) => n.clone(),
            None => NamableObj::new(self.prev.short_hash().as_str(), self.prev.datetime),
        }
    }
}

// commit hash or tag name(include Lightweight tag)
#[derive(Debug, Eq, Clone, PartialEq, Hash)]
pub struct NamableObj {
    name: String,
    datetime: DateTime<Utc>,
}

impl NamableObj {
    pub fn new(name: &str, datetime: DateTime<Utc>) -> Self {
        NamableObj {
            name: name.to_string(),
            datetime,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn date(&self) -> String {
        self.datetime.format("%Y-%m-%d").to_string()
    }
}

impl PartialOrd for NamableObj {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NamableObj {
    fn cmp(&self, other: &Self) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

#[derive(Debug, Eq, Clone, PartialEq, Hash, Default)]
pub struct Author {
    name: Option<String>,
    email: Option<String>,
}

impl Author {
    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or_else(|| "Unknown")
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }
}

impl<'a> From<Signature<'a>> for Author {
    fn from(sig: Signature) -> Self {
        Author {
            name: sig.name().map(String::from),
            email: sig.email().map(String::from),
        }
    }
}

impl FromStr for Author {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref NAME: Regex = Regex::new(r"^(?P<name>.+) <?(?P<email>.*?)>?$").unwrap();
        }

        let captures = NAME
            .captures(s)
            .ok_or_else(|| anyhow!("Failed commit signature capture. sig: {}", s));

        let author = match captures {
            Ok(cap) => Author {
                name: cap.name("name").map(|n| n.as_str()).map(String::from),
                email: cap.name("email").map(|n| n.as_str()).map(String::from),
            },
            _ => Author {
                ..Default::default()
            },
        };
        Ok(author)
    }
}

// TODO Separate responsibilities
#[derive(Debug, Eq, Clone, PartialEq, Hash)]
pub struct Commit {
    pub id: LibOid,
    summary: String,
    author: Author,
    datetime: DateTime<Utc>,
    cc: Option<ConventionalCommits>,
    obj: Option<NamableObj>,
}

impl Commit {
    pub(crate) fn new(
        id: LibOid,
        summary: &str,
        author: &str,
        datetime: DateTime<Utc>,
        cc: Option<ConventionalCommits>,
        obj: Option<NamableObj>,
    ) -> Result<Self> {
        Ok(Commit {
            id,
            summary: String::from(summary),
            author: Author::from_str(author)?,
            datetime,
            cc,
            obj,
        })
    }

    pub fn empty() -> Result<Self> {
        let id = Oid::from_str(EMPTY_HASH)?;
        Self::new(id, "", "", Utc::now(), None, None)
    }

    pub fn short_hash(&self) -> String {
        self.hash().chars().take(7).collect()
    }

    pub fn hash(&self) -> String {
        self.id.to_string()
    }

    pub fn raw_type(&self) -> CommitType {
        self.cc
            .as_ref()
            .map_or_else(|| CommitType::Others, |c| c.raw_type())
    }

    pub fn message(&self) -> String {
        self.cc
            .as_ref()
            .map_or_else(|| self.summary.clone(), |c| c.description.clone())
    }

    pub fn author(&self) -> &Author {
        &self.author
    }

    pub(crate) fn tag(&self) -> Option<&NamableObj> {
        self.obj.as_ref()
    }
}

impl PartialOrd for Commit {
    fn partial_cmp(&self, other: &Commit) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Commit {
    // TODO Should I depend on git obj sort order?
    fn cmp(&self, other: &Commit) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl<'a> From<LibCommit<'a>> for Commit {
    fn from(commit: LibCommit<'a>) -> Self {
        let id = commit.id();

        let summary = commit.summary().map(String::from).unwrap_or_default();

        let author = Author::from(commit.author());
        let datetime = DateTime::from_utc(
            NaiveDateTime::from_timestamp(commit.time().seconds(), 0),
            Utc,
        );
        let cc = ConventionalCommits::from_str(commit.message().unwrap_or_default()).ok();
        let desc = commit
            .as_object()
            .describe(
                DescribeOptions::new()
                    .describe_tags()
                    .max_candidates_tags(0),
            )
            .ok();

        let obj = desc.map(|x| {
            let name = x.format(None).unwrap_or_default();
            NamableObj { name, datetime }
        });

        Commit {
            id,
            summary,
            author,
            datetime,
            cc,
            obj,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct ScanRange {
    latest: Option<Commit>,
    prev: Commit,
}

impl ScanRange {
    pub(super) fn new(latest: Option<Commit>, prev: Commit) -> Self {
        ScanRange { latest, prev }
    }

    pub(super) fn latest_id(&self) -> Option<&LibOid> {
        self.latest.as_ref().map(|c| &c.id)
    }

    pub(super) fn prev_id(&self) -> &LibOid {
        &self.prev.id
    }

    pub(super) fn prev(&self) -> Commit {
        self.prev.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::repository::Findable;
    use crate::git::tests::{dummy_commit, git_dir};
    use crate::git::version::Version;
    use anyhow::Result;
    use git2::{Repository, Time};

    #[test]
    fn find_by_ok() -> Result<()> {
        let git_dir = git_dir()?;
        let repo = Repository::open(git_dir)?;
        let version = Version::parse("0.1.0")?;

        let commit = repo.find_by(&version)?;
        let expected = dummy_commit(
            "9fa3647bfd047ee3c4c120a492065fa6f1c97bcb",
            "chore",
            None,
            false,
            "add README",
            "Test User <test-user@test.com>",
            "Wed Apr 29 16:29:47 2020 +0900",
            Some("0.1.0"),
        )?;

        assert_eq!(commit, expected);
        Ok(())
    }

    #[test]
    fn author_from_str_ok() -> Result<()> {
        let a = Author::from_str("Test User <test-user@test.com>")?;

        let e = "Test User";
        assert_eq!(a.name(), e);

        let e = "test-user@test.com";
        assert_eq!(a.email, Some(String::from(e)));
        Ok(())
    }

    #[test]
    fn author_from_sig_ok() -> Result<()> {
        let time = Time::new(Utc::now().timestamp(), 0);
        let sig = Signature::new("Test User", "test-user@test.com", &time)?;
        let a = Author::from(sig);

        let e = "Test User";
        assert_eq!(a.name(), e);

        let e = "test-user@test.com";
        assert_eq!(a.email, Some(String::from(e)));
        Ok(())
    }
}
