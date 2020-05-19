use super::Commit;
use crate::git::NamableObj;
use lazy_static::*;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct GithubUrl {
    base_url: String,
}

// TODO GitLab, GitBucket
impl GithubUrl {
    pub(crate) fn new(url: &str) -> Self {
        let base_url = git2http(url);
        GithubUrl { base_url }
    }

    pub(crate) fn compare(&self, start: &NamableObj, end: Option<&NamableObj>) -> String {
        format!(
            "{}/compare/{}...{}",
            self.base_url,
            start.name(),
            end.map_or_else(|| "HEAD", |tag| tag.name())
        )
    }

    pub(crate) fn commit(&self, commit: &Commit) -> String {
        format!("{}/commit/{}", self.base_url, commit.hash(),)
    }
}

fn git2http(url: &str) -> String {
    lazy_static! {
        static ref GIT_PROTOCOL: Regex =
            Regex::new(r"^(?:ssh://)?git@(?P<host>.+?)(?:/|:)(?P<repo>.+?)\.git$").unwrap();
        static ref HTTP_PROTOCOL: Regex =
            Regex::new(r"^(?P<scheme>https?://)(?P<host>.+?)/(?P<repo>.+?)\.git$").unwrap();
    }

    let http = HTTP_PROTOCOL.captures(url).and_then(|c| {
        match (c.name("scheme"), c.name("host"), c.name("repo")) {
            (Some(scheme), Some(host), Some(repo)) => Some(format!(
                "{}{}/{}",
                scheme.as_str(),
                host.as_str(),
                repo.as_str()
            )),
            _ => None,
        }
    });

    let git = GIT_PROTOCOL.captures(url).and_then(|c| {
        match (c.name("host"), c.name("repo")) {
            // TODO default scheme
            (Some(host), Some(repo)) => {
                Some(format!("https://{}/{}", host.as_str(), repo.as_str()))
            }
            _ => None,
        }
    });
    http.or(git).unwrap_or_else(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::NamableObj;
    use anyhow::*;
    use chrono::Utc;
    use git2::Oid;

    #[test]
    fn ssh_ok() {
        let a = git2http("ssh://git@github.com/watawuwu/ccclog.git");
        let e = "https://github.com/watawuwu/ccclog";
        assert_eq!(a, e);
    }

    #[test]
    fn git_ok() {
        let a = git2http("git@github.com/watawuwu/ccclog.git");
        let e = "https://github.com/watawuwu/ccclog";
        assert_eq!(a, e);
    }

    #[test]
    fn http_ok() {
        let a = git2http("http://github.com/watawuwu/ccclog.git");
        let e = "http://github.com/watawuwu/ccclog";
        assert_eq!(a, e);

        let a = git2http("https://github.com/watawuwu/ccclog.git");
        let e = "https://github.com/watawuwu/ccclog";
        assert_eq!(a, e);
    }

    #[test]
    fn enterprise_ok() {
        let a = git2http("https://test.com/watawuwu/ccclog.git");
        let e = "https://test.com/watawuwu/ccclog";
        assert_eq!(a, e);
    }

    #[test]
    fn unknown_ok() {
        let a = git2http("https://test.com/watawuwu/ccclog");
        let e = "https://test.com/watawuwu/ccclog";
        assert_eq!(a, e);
    }

    #[test]
    fn compare_ok() -> Result<()> {
        let url = GithubUrl::new("https://test.com/watawuwu/ccclog.git");

        let datetime = Utc::now();
        let start = NamableObj::new("0.1.0", datetime);
        let end = NamableObj::new("0.3.0", datetime);

        let a = url.compare(&start, Some(&end));
        let e = "https://test.com/watawuwu/ccclog/compare/0.1.0...0.3.0";
        assert_eq!(a, e);

        Ok(())
    }

    #[test]
    fn commit_ok() -> Result<()> {
        let url = GithubUrl::new("https://test.com/watawuwu/ccclog.git");

        let commit = Commit::new(
            Oid::from_str("1d185faf719f12292414c88872e3397fc5dc4e62")?,
            "test summary",
            "Test User<test-user@test.com>",
            Utc::now(),
            1,
            None,
            None,
        )?;
        let a = url.commit(&commit);
        let e = "https://test.com/watawuwu/ccclog/commit/1d185faf719f12292414c88872e3397fc5dc4e62";
        assert_eq!(a, e);

        Ok(())
    }
}
