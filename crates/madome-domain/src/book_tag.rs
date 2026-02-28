//! Book tag domain types.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Category of a book tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookTagKind {
    Artist,
    Group,
    Series,
    Character,
    Female,
    Male,
    Misc,
}

impl fmt::Display for BookTagKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Artist => "artist",
            Self::Group => "group",
            Self::Series => "series",
            Self::Character => "character",
            Self::Female => "female",
            Self::Male => "male",
            Self::Misc => "misc",
        };
        f.write_str(s)
    }
}

/// Error returned when a string cannot be parsed as a [`BookTagKind`].
#[derive(Debug, Error)]
#[error("unknown book tag kind: {0:?}")]
pub struct UnknownTagKind(pub String);

impl FromStr for BookTagKind {
    type Err = UnknownTagKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "artist" => Ok(Self::Artist),
            "group" => Ok(Self::Group),
            "series" => Ok(Self::Series),
            "character" => Ok(Self::Character),
            "female" => Ok(Self::Female),
            "male" => Ok(Self::Male),
            "misc" => Ok(Self::Misc),
            other => Err(UnknownTagKind(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_book_tag_kind_as_kebab_case() {
        assert_eq!(
            serde_json::to_string(&BookTagKind::Artist).unwrap(),
            "\"artist\""
        );
        assert_eq!(
            serde_json::to_string(&BookTagKind::Female).unwrap(),
            "\"female\""
        );
        assert_eq!(
            serde_json::to_string(&BookTagKind::Misc).unwrap(),
            "\"misc\""
        );
    }

    #[test]
    fn should_deserialize_book_tag_kind_from_kebab_case() {
        let all = [
            "artist",
            "group",
            "series",
            "character",
            "female",
            "male",
            "misc",
        ];
        for s in all {
            let json = format!("\"{}\"", s);
            let kind: BookTagKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind.to_string(), s);
        }
    }

    #[test]
    fn should_round_trip_book_tag_kind_via_display_and_from_str() {
        for kind in [
            BookTagKind::Artist,
            BookTagKind::Group,
            BookTagKind::Series,
            BookTagKind::Character,
            BookTagKind::Female,
            BookTagKind::Male,
            BookTagKind::Misc,
        ] {
            let s = kind.to_string();
            let parsed: BookTagKind = s.parse().unwrap();
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn should_return_error_for_unknown_tag_kind() {
        assert!("unknown".parse::<BookTagKind>().is_err());
    }
}
