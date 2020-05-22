use anyhow::*;
use itertools::Itertools;
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
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Version {
    prefix: String,
    ver: SemVer,
}

impl Version {
    pub fn starts_with(&self, pre: &str) -> bool {
        self.prefix.starts_with(pre)
    }
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
pub struct Versions(Vec<Version>);

impl Versions {
    fn new() -> Versions {
        Versions(Vec::new())
    }

    #[cfg(test)]
    pub(super) fn from(vec: Vec<Version>) -> Versions {
        Versions(vec)
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

    pub fn prefix(&self) -> Vec<&str> {
        self.0.iter().map(|x| x.prefix.as_str()).unique().collect()
    }

    pub fn select(self, prefix: Option<&str>) -> Self {
        if let Some(pre) = prefix {
            return self
                .0
                .into_iter()
                .filter(|x| x.prefix == pre)
                .collect::<Versions>();
        }

        let prefix = "";
        if self.has_prefix(prefix) {
            return self.filter(prefix);
        }

        let prefix = "v";
        if self.has_prefix(prefix) {
            return self.filter(prefix);
        }

        self
    }

    fn filter(self, prefix: &str) -> Self {
        self.0
            .into_iter()
            .filter(|x| x.prefix == prefix)
            .collect::<Versions>()
    }

    fn has_prefix(&self, prefix: &str) -> bool {
        self.0.iter().any(|x| x.prefix == prefix)
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
    fn latest_range_ng() -> Result<()> {
        let expected = Version::from_str("0.2.0")?;
        let mut versions = vec![expected.clone()].into_iter().collect::<Versions>();

        let (latest, prev) = versions.latest_range();
        assert_eq!(prev, None);
        assert_eq!(latest, Some(&expected));

        let mut versions = Vec::new().into_iter().collect::<Versions>();

        let (latest, prev) = versions.latest_range();
        assert_eq!(prev, None);
        assert_eq!(latest, None);

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

    fn dummy_versions(vs: Vec<&str>) -> Result<Versions> {
        let v = vs
            .into_iter()
            .map(Version::from_str)
            .collect::<Result<Vec<Version>>>()?;
        Ok(Versions::from(v))
    }

    #[test]
    fn prefix_count_ok() -> Result<()> {
        let a = dummy_versions(vec!["0.1.0", "v0.2.0", "prefix-0.2.0", "test-0.2.0"])?;
        assert_eq!(a.prefix(), vec!["", "v", "prefix-", "test-"]);

        let a = dummy_versions(Vec::new())?;
        assert_eq!(a.prefix().iter().count(), 0);

        Ok(())
    }

    #[test]
    fn select_ok() -> Result<()> {
        let versions = dummy_versions(vec!["0.1.0", "v0.2.0", "prefix-0.2.0", "test-0.2.0"])?;

        let a = versions.clone();
        let e = dummy_versions(vec!["0.1.0"])?;
        assert_eq!(a.select(Some("")), e);

        let a = versions.clone();
        let e = dummy_versions(vec!["v0.2.0"])?;
        assert_eq!(a.select(Some("v")), e);

        let a = versions.clone();
        let e = dummy_versions(vec!["prefix-0.2.0"])?;
        assert_eq!(a.select(Some("prefix-")), e);

        let a = versions.clone();
        let e = dummy_versions(vec!["test-0.2.0"])?;
        assert_eq!(a.select(Some("test-")), e);

        let a = versions.clone();
        let e = dummy_versions(vec!["0.1.0"])?;
        assert_eq!(a.select(None), e);

        let a = dummy_versions(vec!["v0.2.0", "prefix-0.2.0", "test-0.2.0"])?;
        let e = dummy_versions(vec!["v0.2.0"])?;
        assert_eq!(a.select(None), e);

        let a = dummy_versions(vec!["prefix-0.2.0", "test-0.2.0"])?;
        let e = dummy_versions(vec!["prefix-0.2.0", "test-0.2.0"])?;
        assert_eq!(a.select(None), e);

        Ok(())
    }
}
