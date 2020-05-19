use anyhow::*;
use lazy_static::*;
use log::*;
use regex::Regex;
use semver::Version as SemVer;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

lazy_static! {
    static ref PREFIX: Regex =
        Regex::new(r"^(?P<prefix>.*?)(?P<version>[0-9]+?.[0-9]+?.[0-9]+?(?:.*)$)").unwrap();
    pub static ref SEMVER_PATTERN: Regex =
        Regex::new(r#"^v?\d+?.\d+?.\d+?(\-[\w.-]+?)?(\+[\w.-]+?)?$"#).unwrap();
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(super) struct Version {
    prefix: String,
    ver: SemVer,
}

impl FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = PREFIX
            .captures(s)
            .ok_or_else(|| anyhow!("Can't find semver format. value: {}", s))?;

        let cap_pre = caps.name("prefix");
        let cap_ver = caps.name("version");

        let (prefix, version) = match (cap_pre, cap_ver) {
            (Some(p), Some(v)) => (p.as_str(), v.as_str()),
            (None, Some(v)) => ("", v.as_str()),
            _ => bail!("Can't find semver format. value: {}", s),
        };

        debug!("prefix: {}", prefix);
        debug!("version: {}", version);

        Ok(Version {
            prefix: prefix.to_string(),
            ver: SemVer::parse(version)?,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.ver)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Versions(Vec<Version>);

impl Versions {
    fn new() -> Versions {
        Versions(Vec::new())
    }

    fn add(&mut self, elem: Version) {
        self.0.push(elem);
    }

    pub fn latest_range(&mut self) -> (Option<&Version>, Option<&Version>) {
        self.0.sort();
        self.0.reverse();
        let mut it = self.0.iter();
        let latest_tag = it.next();
        let previous_tag = it.next();
        (latest_tag, previous_tag)
    }
}

impl FromIterator<Version> for Versions {
    fn from_iter<I: IntoIterator<Item = Version>>(iter: I) -> Self {
        let mut c = Versions::new();
        for i in iter {
            c.add(i);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_range_ok() -> Result<()> {
        let expected_latest = Version::from_str("0.2.0")?;
        let expected_prev = Version::from_str("0.1.0")?;
        let mut versions = vec![expected_latest.clone(), expected_prev.clone()]
            .into_iter()
            .collect::<Versions>();

        let (latest, prev) = versions.latest_range();
        assert_eq!(prev, Some(&expected_prev));
        assert_eq!(latest, Some(&expected_latest));
        Ok(())
    }

    #[test]
    fn parse_ok() -> Result<()> {
        let a = Version::from_str("0.2.0")?;
        assert!(a.prefix.is_empty());
        assert_eq!(a.to_string(), "0.2.0");

        let a = Version::from_str("v0.2.0")?;
        assert_eq!(a.prefix, "v");
        assert_eq!(a.to_string(), "v0.2.0");

        let a = Version::from_str("web-0.2.0")?;
        assert_eq!(a.prefix, "web-");
        assert_eq!(a.to_string(), "web-0.2.0");

        let a = Version::from_str("product-0.2.0")?;
        assert_eq!(a.prefix, "product-");
        assert_eq!(a.to_string(), "product-0.2.0");

        Ok(())
    }
}
