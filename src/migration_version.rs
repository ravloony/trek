use std::fmt::{self, Display};

use chrono::{self, DateTime, UTC};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "rustc-serialize", derive(RustcEncodable, RustcDecodable))]
pub struct MigrationVersion {
    version: DateTime<UTC>,
}
impl MigrationVersion {
    pub fn new() -> Self {
        MigrationVersion { version: UTC::now() }
    }
    pub fn from_datetime(datetime: DateTime<UTC>) -> Self {
        MigrationVersion { version: datetime }
    }
    pub fn from_rfc3339_string(string: &str) -> Result<Self, chrono::format::ParseError> {
        let datetime = try!(string.parse::<DateTime<UTC>>());
        Ok(MigrationVersion::from_datetime(datetime))
    }
    pub fn serialize(&self) -> String {
        self.version.to_rfc3339()
    }
}
impl Display for MigrationVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.version.format("%Y%m%d%H%M%S"))
    }
}
