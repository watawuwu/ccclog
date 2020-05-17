use anyhow::*;
use itertools::Itertools;

use crate::git::{Author, Commit, CommitType, Commits, GithubUrl, ReleaseRange};
use regex::Regex;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Config {
    pub enable_email_link: bool,
    pub reverse: bool,
    pub root_indent_level: u8,
    pub ignore_summary: Option<Regex>,
    pub ignore_types: Option<Vec<CommitType>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_email_link: false,
            reverse: false,
            root_indent_level: 2u8,
            ignore_summary: None,
            ignore_types: None,
        }
    }
}

pub struct Changelog {
    conf: Config,
}

impl Changelog {
    #[cfg(test)]
    pub fn new() -> Self {
        Changelog {
            conf: Config::default(),
        }
    }

    pub fn from(config: Config) -> Self {
        Changelog { conf: config }
    }

    pub fn markdown(&self, url: Option<&GithubUrl>, commits: &Commits) -> Result<String> {
        let mut links = Vec::new();

        let func = |(range, mut vec): (ReleaseRange, BTreeMap<CommitType, Vec<&Commit>>)| {
            let (heading, h_link) = self.heading(url, &range);
            if let Some(l) = h_link {
                links.push(l)
            };

            let (contents, c_link) = self.contents(url, &mut vec);
            if let Some(l) = c_link {
                links.push(l)
            };

            format!("{}\n{}", heading, contents)
        };

        let changelog = commits.group_by().into_iter().map(func).join("\n");

        let changelog = if links.is_empty() {
            changelog
        } else {
            format!("{}\n{}\n", changelog, links.join("\n"))
        };

        Ok(changelog)
    }

    fn heading(&self, url: Option<&GithubUrl>, range: &ReleaseRange) -> (String, Option<String>) {
        let (subject, link) = match (url, range) {
            (Some(u), ReleaseRange::Release(s, e)) => {
                let sub = format!("[{}] - {}", e.name(), e.date());
                let a = format!("[{}]: {}", e.name(), u.compare(s, Some(e)));
                (sub, Some(a))
            }
            (Some(u), ReleaseRange::UnRelease(s)) => {
                let sub = "[Unreleased]".to_string();
                let a = format!("[Unreleased]: {}", u.compare(s, None));
                (sub, Some(a))
            }
            (None, ReleaseRange::Release(_, e)) => (format!("{} - {}", e.name(), e.date()), None),
            (None, ReleaseRange::UnRelease(_)) => (String::from("Unreleased"), None),
        };
        let heading = format!("{} {}", self.heading_style(), subject);
        (heading, link)
    }

    fn sub_heading(&self, ct: &CommitType) -> String {
        format!("{} {}", self.sub_heading_style(), ct.to_string())
    }

    fn contents(
        &self,
        url: Option<&GithubUrl>,
        commits: &mut BTreeMap<CommitType, Vec<&Commit>>,
    ) -> (String, Option<String>) {
        let mut links = Vec::new();

        let contents = commits
            .iter_mut()
            .map(|(ct, vec)| {
                if self.conf.reverse {
                    vec.reverse();
                }

                let (section, link) = self.section(url, ct, vec.to_vec());
                if let Some(l) = link {
                    links.push(l)
                };

                section
            })
            .filter_map(|s| s)
            .join("\n");

        let links = links.first().map(|_| links.join("\n"));
        (contents, links)
    }

    // TODO impl breaking change expressions
    fn section(
        &self,
        url: Option<&GithubUrl>,
        ct: &CommitType,
        commits: Vec<&Commit>,
    ) -> (Option<String>, Option<String>) {
        let mut links = Vec::new();
        let aggregate = |commit: &Commit| -> String {
            let hash = commit.short_hash();
            let msg = commit.message();
            let au = self.author(commit.author());
            match url {
                Some(u) => {
                    let item = format!("- [[{}]] {} ({})", &hash, &msg, &au);
                    let link = format!("[{}]: {}", &hash, u.commit(commit));
                    links.push(link);
                    item
                }
                None => format!("- [{}] {} ({})", &hash, &msg, &au),
            }
        };

        let lines = commits
            .into_iter()
            .filter(self.ignore_summary())
            .filter(self.ignore_types())
            .map(aggregate)
            .join("\n");

        if lines.is_empty() {
            return (None, None);
        }

        let heading = self.sub_heading(ct);
        let section = format!("{}\n{}\n", heading, lines);
        let links = links.first().map(|_| links.join("\n"));

        (Some(section), links)
    }

    fn ignore_summary<'a>(&'a self) -> impl FnMut(&&'a Commit) -> bool {
        move |commit: &&'a Commit| -> bool {
            let regex = self.conf.ignore_summary.as_ref();
            match regex {
                Some(re) => !re.is_match(commit.message().as_ref()),
                _ => true,
            }
        }
    }

    fn ignore_types<'a>(&'a self) -> impl FnMut(&&'a Commit) -> bool {
        move |commit: &&'a Commit| -> bool {
            let _types = self.conf.ignore_types.as_ref();
            match _types {
                Some(t) => !t.contains(&commit.raw_type()),
                _ => true,
            }
        }
    }

    fn author(&self, author: &Author) -> String {
        let name = author.name();
        match author.email() {
            Some(email) if self.conf.enable_email_link => format!("[{}](mailto:{})", name, email),
            _ => name.to_string(),
        }
    }

    fn heading_style(&self) -> String {
        let indent = self.conf.root_indent_level;
        "#".repeat(indent as usize)
    }

    fn sub_heading_style(&self) -> String {
        let indent = self.conf.root_indent_level + 1;
        "#".repeat(indent as usize)
    }
}
#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::git::tests::*;

    use super::*;

    fn dummy_commits() -> Result<Commits> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "test",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
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
            None,
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        Ok(Commits::new(prev, commits))
    }

    #[test]
    fn all_commit_type_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "2e185faf719f12292414c88872e3397fc5dc4e62",
            "security",
            None,
            false,
            "fix security",
            "Test User12 <test-user12@test.com>",
            "Wed Apr 01 01:01:12 2020 +0000",
            Some("0.2.0"),
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "1e185faf719f12292414c88872e3397fc5dc4e62",
            "revert",
            None,
            false,
            "add some",
            "Test User11 <test-user11@test.com>",
            "Wed Apr 01 01:01:11 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "0e185faf719f12292414c88872e3397fc5dc4e62",
            "test",
            None,
            false,
            "add test",
            "Test User10 <test-user10@test.com>",
            "Wed Apr 01 01:01:10 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "9d185faf719f12292414c88872e3397fc5dc4e62",
            "perf",
            None,
            false,
            "add perf",
            "Test User9 <test-user9@test.com>",
            "Wed Apr 01 01:01:09 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "8d185faf719f12292414c88872e3397fc5dc4e62",
            "refactor",
            None,
            false,
            "add refactor",
            "Test User8 <test-user8@test.com>",
            "Wed Apr 01 01:01:08 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "7d185faf719f12292414c88872e3397fc5dc4e62",
            "style",
            None,
            false,
            "add style",
            "Test User7 <test-user7@test.com>",
            "Wed Apr 01 01:01:07 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "6d185faf719f12292414c88872e3397fc5dc4e62",
            "ci",
            None,
            false,
            "add CI",
            "Test User6 <test-user6@test.com>",
            "Wed Apr 01 01:01:06 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "5d185faf719f12292414c88872e3397fc5dc4e62",
            "chore",
            None,
            false,
            "add chore",
            "Test User5 <test-user5@test.com>",
            "Wed Apr 01 01:01:05 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "4d185faf719f12292414c88872e3397fc5dc4e62",
            "doc",
            None,
            false,
            "add doc",
            "Test User4 <test-user4@test.com>",
            "Wed Apr 01 01:01:04 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "build",
            None,
            false,
            "add build script",
            "Test User3 <test-user3@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "fix",
            None,
            false,
            "fix typo",
            "Test User2 <test-user2@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add README",
            "Test User1 <test-user1@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let previous = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.1.0"),
        )?;
        let cms = Commits::new(previous, commits);
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let changelog = Changelog::new();
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.2.0] - 2020-04-01
### Feat
- [[1d185fa]] add README (Test User1)

### Fix
- [[2d185fa]] fix typo (Test User2)

### Build
- [[3d185fa]] add build script (Test User3)

### Doc
- [[4d185fa]] add doc (Test User4)

### Chore
- [[5d185fa]] add chore (Test User5)

### CI
- [[6d185fa]] add CI (Test User6)

### Style
- [[7d185fa]] add style (Test User7)

### Refactor
- [[8d185fa]] add refactor (Test User8)

### Perf
- [[9d185fa]] add perf (Test User9)

### Test
- [[0e185fa]] add test (Test User10)

### Revert
- [[1e185fa]] add some (Test User11)

### Security
- [[2e185fa]] fix security (Test User12)

[0.2.0]: https://github.com/watawuwu/ccclog/compare/0.1.0...0.2.0
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
[2d185fa]: https://github.com/watawuwu/ccclog/commit/2d185faf719f12292414c88872e3397fc5dc4e62
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
[4d185fa]: https://github.com/watawuwu/ccclog/commit/4d185faf719f12292414c88872e3397fc5dc4e62
[5d185fa]: https://github.com/watawuwu/ccclog/commit/5d185faf719f12292414c88872e3397fc5dc4e62
[6d185fa]: https://github.com/watawuwu/ccclog/commit/6d185faf719f12292414c88872e3397fc5dc4e62
[7d185fa]: https://github.com/watawuwu/ccclog/commit/7d185faf719f12292414c88872e3397fc5dc4e62
[8d185fa]: https://github.com/watawuwu/ccclog/commit/8d185faf719f12292414c88872e3397fc5dc4e62
[9d185fa]: https://github.com/watawuwu/ccclog/commit/9d185faf719f12292414c88872e3397fc5dc4e62
[0e185fa]: https://github.com/watawuwu/ccclog/commit/0e185faf719f12292414c88872e3397fc5dc4e62
[1e185fa]: https://github.com/watawuwu/ccclog/commit/1e185faf719f12292414c88872e3397fc5dc4e62
[2e185fa]: https://github.com/watawuwu/ccclog/commit/2e185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn multi_item_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add feat3",
            "Test User3 <test-user3@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            Some("1.0.0"),
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add feat2",
            "Test User2 <test-user2@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add feat1",
            "Test User1 <test-user1@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.1.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [1.0.0] - 2020-04-01
### Feat
- [[3d185fa]] add feat3 (Test User3)
- [[2d185fa]] add feat2 (Test User2)
- [[1d185fa]] add feat1 (Test User1)

[1.0.0]: https://github.com/watawuwu/ccclog/compare/0.1.0...1.0.0
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
[2d185fa]: https://github.com/watawuwu/ccclog/commit/2d185faf719f12292414c88872e3397fc5dc4e62
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn sort_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "4d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 4",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:04 2020 +0000",
            Some("0.2.0"),
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 2",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:02 2020 +0000",
            Some("0.1.0"),
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
            None,
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.2.0] - 2020-04-01
### Feat
- [[4d185fa]] add 4 (Test User)
- [[3d185fa]] add 3 (Test User)

## [0.1.0] - 2020-04-01
### Feat
- [[2d185fa]] add 2 (Test User)
- [[1d185fa]] add 1 (Test User)

[0.2.0]: https://github.com/watawuwu/ccclog/compare/0.1.0...0.2.0
[4d185fa]: https://github.com/watawuwu/ccclog/commit/4d185faf719f12292414c88872e3397fc5dc4e62
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
[0.1.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.1.0
[2d185fa]: https://github.com/watawuwu/ccclog/commit/2d185faf719f12292414c88872e3397fc5dc4e62
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);

        let conf = Config {
            reverse: true,
            ..Default::default()
        };

        let changelog = Changelog::from(conf);
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.2.0] - 2020-04-01
### Feat
- [[3d185fa]] add 3 (Test User)
- [[4d185fa]] add 4 (Test User)

## [0.1.0] - 2020-04-01
### Feat
- [[1d185fa]] add 1 (Test User)
- [[2d185fa]] add 2 (Test User)

[0.2.0]: https://github.com/watawuwu/ccclog/compare/0.1.0...0.2.0
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
[4d185fa]: https://github.com/watawuwu/ccclog/commit/4d185faf719f12292414c88872e3397fc5dc4e62
[0.1.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.1.0
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
[2d185fa]: https://github.com/watawuwu/ccclog/commit/2d185faf719f12292414c88872e3397fc5dc4e62
"#;

        assert_eq!(markdown, expected);

        Ok(())
    }

    #[test]
    fn unreleased_ok() -> Result<()> {
        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.1.0"),
        )?;

        let mut commits = Vec::new();
        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add first",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let changelog = Changelog::new();

        let cms = Commits::new(prev, commits);
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [Unreleased]
### Feat
- [[1d185fa]] add first (Test User)

[Unreleased]: https://github.com/watawuwu/ccclog/compare/0.1.0...HEAD
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn tag_and_unreleased_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add second",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:02 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add first",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            Some("0.1.0"),
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [Unreleased]
### Feat
- [[2d185fa]] add second (Test User)

## [0.1.0] - 2020-04-01
### Feat
- [[1d185fa]] add first (Test User)

[Unreleased]: https://github.com/watawuwu/ccclog/compare/0.1.0...HEAD
[2d185fa]: https://github.com/watawuwu/ccclog/commit/2d185faf719f12292414c88872e3397fc5dc4e62
[0.1.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.1.0
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn scope_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            Some("test"),
            false,
            "add first",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            Some("0.1.0"),
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.1.0] - 2020-04-01
### Feat
- [[1d185fa]] add first (Test User)

[0.1.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.1.0
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn no_conventional_commits_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_invalid_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "add first",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:01 2020 +0000",
            Some("0.1.0"),
        )?;
        commits.push(commit);
        let prev = dummy_invalid_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "add zero",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.1.0] - 2020-04-01
### Others
- [[1d185fa]] add first (Test User)

[0.1.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.1.0
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn multi_release_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            Some("0.3.0"),
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 2",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:02 2020 +0000",
            Some("0.2.0"),
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
            Some("0.1.0"),
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.3.0] - 2020-04-01
### Feat
- [[3d185fa]] add 3 (Test User)

## [0.2.0] - 2020-04-01
### Feat
- [[2d185fa]] add 2 (Test User)

## [0.1.0] - 2020-04-01
### Feat
- [[1d185fa]] add 1 (Test User)

[0.3.0]: https://github.com/watawuwu/ccclog/compare/0.2.0...0.3.0
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
[0.2.0]: https://github.com/watawuwu/ccclog/compare/0.1.0...0.2.0
[2d185fa]: https://github.com/watawuwu/ccclog/commit/2d185faf719f12292414c88872e3397fc5dc4e62
[0.1.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.1.0
[1d185fa]: https://github.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn enable_email_link_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            Some("0.3.0"),
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let conf = Config {
            enable_email_link: true,
            ..Default::default()
        };

        let changelog = Changelog::from(conf);
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"## [0.3.0] - 2020-04-01
### Feat
- [[3d185fa]] add 3 ([Test User](mailto:test-user@test.com))

[0.3.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.3.0
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn root_indent_level_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            Some("0.3.0"),
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let conf = Config {
            root_indent_level: 1,
            ..Default::default()
        };
        let changelog = Changelog::from(conf);
        let gurl = GithubUrl::new("https://github.com/watawuwu/ccclog.git");
        let markdown = changelog.markdown(Some(&gurl), &cms)?;
        let expected = r#"# [0.3.0] - 2020-04-01
## Feat
- [[3d185fa]] add 3 (Test User)

[0.3.0]: https://github.com/watawuwu/ccclog/compare/0.0.0...0.3.0
[3d185fa]: https://github.com/watawuwu/ccclog/commit/3d185faf719f12292414c88872e3397fc5dc4e62
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn no_remote_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 1",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            Some("0.3.0"),
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let conf = Config {
            root_indent_level: 1,
            ..Default::default()
        };
        let changelog = Changelog::from(conf);
        let markdown = changelog.markdown(None, &cms)?;
        let expected = r#"# 0.3.0 - 2020-04-01
## Feat
- [1d185fa] add 1 (Test User)
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn custom_ok() -> Result<()> {
        let mut commits = Vec::new();
        let commit = dummy_commit(
            "4d185faf719f12292414c88872e3397fc5dc4e62",
            "custom2",
            None,
            false,
            "add 4",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            Some("0.3.0"),
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "3d185faf719f12292414c88872e3397fc5dc4e62",
            "custom2",
            None,
            false,
            "add 3",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let commit = dummy_commit(
            "2d185faf719f12292414c88872e3397fc5dc4e62",
            "custom1",
            None,
            false,
            "add 2",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);
        let commit = dummy_commit(
            "1d185faf719f12292414c88872e3397fc5dc4e62",
            "custom1",
            None,
            false,
            "add 1",
            "Test User <test-user@test.com>",
            "Wed Apr 01 01:01:03 2020 +0000",
            None,
        )?;
        commits.push(commit);

        let prev = dummy_commit(
            "0d185faf719f12292414c88872e3397fc5dc4e62",
            "feat",
            None,
            false,
            "add 0",
            "Test User0 <test-user0@test.com>",
            "Wed Apr 01 01:01:00 2020 +0000",
            Some("0.0.0"),
        )?;

        let cms = Commits::new(prev, commits);
        let changelog = Changelog::new();
        let markdown = changelog.markdown(None, &cms)?;
        let expected = r#"## 0.3.0 - 2020-04-01
### Custom1
- [2d185fa] add 2 (Test User)
- [1d185fa] add 1 (Test User)

### Custom2
- [4d185fa] add 4 (Test User)
- [3d185fa] add 3 (Test User)
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }

    #[test]
    fn filter_ok() -> Result<()> {
        let cms = dummy_commits()?;
        let conf = Config {
            ignore_summary: Some(Regex::new(r#"^add 3$"#)?),
            ..Default::default()
        };
        let changelog = Changelog::from(conf);
        let markdown = changelog.markdown(None, &cms)?;
        let expected = r#"## 0.1.0 - 2020-04-01
### Feat
- [1d185fa] add 1 (Test User)

### Fix
- [2d185fa] add 2 (Test User)
"#;
        assert_eq!(markdown, expected);
        Ok(())
    }
}
