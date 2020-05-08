use anyhow::*;
use semver::Version as SemVer;
use std::fmt;
use std::iter::FromIterator;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(super) struct Version(SemVer);

impl Version {
    pub fn parse(version: &str) -> Result<Self> {
        Ok(Version(SemVer::parse(version)?))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.0.to_string())?;
        Ok(())
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
        let expected_latest = Version::parse("0.2.0")?;
        let expected_previous = Version::parse("0.1.0")?;
        let mut versions = vec![expected_latest.clone(), expected_previous.clone()]
            .into_iter()
            .collect::<Versions>();

        let (latest, previous) = versions.latest_range();
        assert_eq!(previous, Some(&expected_previous));
        assert_eq!(latest, Some(&expected_latest));
        Ok(())
    }
}
