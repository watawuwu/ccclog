use anyhow::*;
use inflector::Inflector;
use lazy_static::*;
use regex::Regex;
use std::str::FromStr;
use std::string::ToString;
use strum::EnumMessage;

#[derive(Debug, PartialEq, Eq, EnumMessage, Clone, Hash, AsRefStr, PartialOrd, Ord)]
pub enum CommitType {
    Feat,
    Fix,
    Build,
    Doc,
    Chore,
    #[strum(message = "CI")]
    Ci,
    Style,
    Refactor,
    Perf,
    Test,
    Revert,
    Security,
    Custom(String),
    Others,
}

// Not available EnumString for custom type
impl FromStr for CommitType {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "feat" => Ok(CommitType::Feat),
            "fix" => Ok(CommitType::Fix),
            "build" => Ok(CommitType::Build),
            "doc" => Ok(CommitType::Doc),
            "chore" => Ok(CommitType::Chore),
            "ci" => Ok(CommitType::Ci),
            "style" => Ok(CommitType::Style),
            "refactor" => Ok(CommitType::Refactor),
            "perf" => Ok(CommitType::Perf),
            "test" => Ok(CommitType::Test),
            "revert" => Ok(CommitType::Revert),
            "security" => Ok(CommitType::Security),
            "others" => Ok(CommitType::Others),
            _ => Ok(CommitType::Custom(s.to_string())),
        }
    }
}

impl std::fmt::Display for CommitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CommitType::Custom(s) => s.to_string().to_sentence_case(),
            CommitType::Ci => self.get_message().unwrap().to_string(),
            _ => self.as_ref().to_string().to_sentence_case(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ConventionalCommits {
    break_change: bool,
    pub _type: CommitType,
    pub scope: Option<String>,
    pub description: String,
}

impl ConventionalCommits {
    #[cfg(test)]
    pub(crate) fn new(
        break_change: bool,
        _type: CommitType,
        scope: Option<String>,
        description: &str,
    ) -> Self {
        ConventionalCommits {
            break_change,
            _type,
            scope,
            description: String::from(description),
        }
    }

    fn break_change(summary: &str, body: Option<&str>) -> bool {
        summary.contains("!:") || body.map_or_else(|| false, |s| s.contains("BREAKING CHANGE: "))
    }

    pub fn raw_type(&self) -> CommitType {
        self._type.clone()
    }
}

impl FromStr for ConventionalCommits {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref CONVENTIONAL_COMMIT_PATTERN: Regex =
                Regex::new(r"^(?P<type>.+?)(?P<scope>\(.+?\))?!?: (?P<description>.+?)$").unwrap();
        }
        let lines = s.splitn(2, '\n').collect::<Vec<&str>>();
        let (summary, body) = if lines.len() == 2 {
            (lines[0], Some(lines[1]))
        } else {
            (s, None)
        };

        let cap = CONVENTIONAL_COMMIT_PATTERN
            .captures(&summary)
            .ok_or_else(|| anyhow!("Invalid conventional commits format"))?;
        let _type = cap
            .name("type")
            .context("Invalid conventional commits format")?
            .as_str()
            .to_string();
        let scope = cap.name("scope").map(|s| String::from(s.as_str()));
        let description = cap
            .name("description")
            .context("Invalid conventional commits format")?
            .as_str()
            .to_string();

        let cc = ConventionalCommits {
            break_change: Self::break_change(summary, body),
            _type: CommitType::from_str(&_type)?,
            scope,
            description,
        };

        Ok(cc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn commit_type_str_ok() -> Result<()> {
        let a = CommitType::from_str("unknown")?.to_string();
        let e = "Unknown";
        assert_eq!(a, e);

        let a = CommitType::from_str("ci")?.to_string();
        let e = "CI";
        assert_eq!(a, e);

        let a = CommitType::from_str("test")?.to_string();
        let e = "Test";
        assert_eq!(a, e);
        Ok(())
    }
}
