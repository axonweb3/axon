use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const DEFAULT_COMMIT_ID: &str = "unknown";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    inner: semver::Version,
}

impl FromStr for Version {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        semver::Version::from_str(s).map(|inner| Version { inner })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Version {
    pub fn new(ver: &str) -> Self {
        Version::from_str(ver)
            .unwrap_or_else(|e| panic!("Parse version error {:?}", e))
            .set_commit_id(DEFAULT_COMMIT_ID)
    }

    pub fn new_with_commit_id(ver: &str, commit_id: &str) -> Self {
        Version::new(ver).set_commit_id(commit_id)
    }

    pub fn commit_id(&self) -> String {
        self.inner.build.to_ascii_lowercase()
    }

    fn set_commit_id(mut self, commit_id: &str) -> Self {
        self.inner.build = semver::BuildMetadata::new(commit_id)
            .unwrap_or_else(|e| panic!("Parse commit id error {:?}", e));

        self
    }
}

#[cfg(test)]
mod tests {
    use semver::BuildMetadata;
    use std::str::FromStr;

    use crate::DEFAULT_COMMIT_ID;

    #[test]
    fn test_parse_default_commit_id() {
        let build = BuildMetadata::from_str(DEFAULT_COMMIT_ID);
        assert_eq!(build.unwrap().as_str(), DEFAULT_COMMIT_ID);
    }
}
