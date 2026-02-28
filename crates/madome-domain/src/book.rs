//! Book domain types.

use serde::{Deserialize, Serialize};

use crate::pagination::Sort;

/// Genre/format category of a book.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookKind {
    Doujinshi,
    Manga,
    GameCg,
    ArtistCg,
    ImageSet,
}

/// Sort order for the `GET /books` listing endpoint.
///
/// Requires a custom `Deserialize` impl because the wire format is a single
/// hyphenated string (e.g. `"id-desc"`) rather than a nested enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookSortBy {
    Id(Sort),
    PublishedAt(Sort),
    CheckedAt(Sort),
    UpdatedAt(Sort),
    Random,
}

impl Default for BookSortBy {
    fn default() -> Self {
        Self::Id(Sort::Desc)
    }
}

impl<'de> Deserialize<'de> for BookSortBy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "id-desc" => Ok(Self::Id(Sort::Desc)),
            "id-asc" => Ok(Self::Id(Sort::Asc)),
            "published-at-desc" => Ok(Self::PublishedAt(Sort::Desc)),
            "published-at-asc" => Ok(Self::PublishedAt(Sort::Asc)),
            "checked-at-desc" => Ok(Self::CheckedAt(Sort::Desc)),
            "checked-at-asc" => Ok(Self::CheckedAt(Sort::Asc)),
            "updated-at-desc" => Ok(Self::UpdatedAt(Sort::Desc)),
            "updated-at-asc" => Ok(Self::UpdatedAt(Sort::Asc)),
            "random" => Ok(Self::Random),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &[
                    "id-desc",
                    "id-asc",
                    "published-at-desc",
                    "published-at-asc",
                    "checked-at-desc",
                    "checked-at-asc",
                    "updated-at-desc",
                    "updated-at-asc",
                    "random",
                ],
            )),
        }
    }
}

impl Serialize for BookSortBy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::Id(Sort::Desc) => "id-desc",
            Self::Id(Sort::Asc) => "id-asc",
            Self::PublishedAt(Sort::Desc) => "published-at-desc",
            Self::PublishedAt(Sort::Asc) => "published-at-asc",
            Self::CheckedAt(Sort::Desc) => "checked-at-desc",
            Self::CheckedAt(Sort::Asc) => "checked-at-asc",
            Self::UpdatedAt(Sort::Desc) => "updated-at-desc",
            Self::UpdatedAt(Sort::Asc) => "updated-at-asc",
            Self::Random => "random",
        };
        serializer.serialize_str(s)
    }
}

/// Sort order for the `GET /books/search` Meilisearch endpoint.
///
/// Includes `RankDesc` (relevance) which is not available in the general listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchBookSortBy {
    /// Meilisearch relevance score descending (default).
    #[default]
    RankDesc,
    Id(Sort),
}

impl<'de> Deserialize<'de> for SearchBookSortBy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "rank-desc" => Ok(Self::RankDesc),
            "id-desc" => Ok(Self::Id(Sort::Desc)),
            "id-asc" => Ok(Self::Id(Sort::Asc)),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["rank-desc", "id-desc", "id-asc"],
            )),
        }
    }
}

impl Serialize for SearchBookSortBy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::RankDesc => "rank-desc",
            Self::Id(Sort::Desc) => "id-desc",
            Self::Id(Sort::Asc) => "id-asc",
        };
        serializer.serialize_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> T {
        serde_json::from_str(s).unwrap()
    }

    fn to_str<T: Serialize>(v: &T) -> String {
        serde_json::to_string(v).unwrap()
    }

    // --- BookKind ---

    #[test]
    fn should_serialize_book_kind_as_kebab_case() {
        assert_eq!(to_str(&BookKind::Doujinshi), "\"doujinshi\"");
        assert_eq!(to_str(&BookKind::GameCg), "\"game-cg\"");
        assert_eq!(to_str(&BookKind::ArtistCg), "\"artist-cg\"");
        assert_eq!(to_str(&BookKind::ImageSet), "\"image-set\"");
    }

    #[test]
    fn should_deserialize_book_kind_from_kebab_case() {
        assert_eq!(from_str::<BookKind>("\"doujinshi\""), BookKind::Doujinshi);
        assert_eq!(from_str::<BookKind>("\"manga\""), BookKind::Manga);
        assert_eq!(from_str::<BookKind>("\"game-cg\""), BookKind::GameCg);
        assert_eq!(from_str::<BookKind>("\"artist-cg\""), BookKind::ArtistCg);
        assert_eq!(from_str::<BookKind>("\"image-set\""), BookKind::ImageSet);
    }

    // --- BookSortBy ---

    #[test]
    fn should_deserialize_all_book_sort_by_variants() {
        assert_eq!(
            from_str::<BookSortBy>("\"id-desc\""),
            BookSortBy::Id(Sort::Desc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"id-asc\""),
            BookSortBy::Id(Sort::Asc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"published-at-desc\""),
            BookSortBy::PublishedAt(Sort::Desc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"published-at-asc\""),
            BookSortBy::PublishedAt(Sort::Asc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"checked-at-desc\""),
            BookSortBy::CheckedAt(Sort::Desc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"checked-at-asc\""),
            BookSortBy::CheckedAt(Sort::Asc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"updated-at-desc\""),
            BookSortBy::UpdatedAt(Sort::Desc)
        );
        assert_eq!(
            from_str::<BookSortBy>("\"updated-at-asc\""),
            BookSortBy::UpdatedAt(Sort::Asc)
        );
        assert_eq!(from_str::<BookSortBy>("\"random\""), BookSortBy::Random);
    }

    #[test]
    fn should_serialize_book_sort_by_variants() {
        assert_eq!(to_str(&BookSortBy::Id(Sort::Desc)), "\"id-desc\"");
        assert_eq!(to_str(&BookSortBy::Random), "\"random\"");
        assert_eq!(
            to_str(&BookSortBy::CheckedAt(Sort::Asc)),
            "\"checked-at-asc\""
        );
    }

    #[test]
    fn should_default_book_sort_by_to_id_desc() {
        assert_eq!(BookSortBy::default(), BookSortBy::Id(Sort::Desc));
    }

    #[test]
    fn should_reject_unknown_book_sort_by_variant() {
        assert!(serde_json::from_str::<BookSortBy>("\"name-asc\"").is_err());
    }

    // --- SearchBookSortBy ---

    #[test]
    fn should_deserialize_all_search_book_sort_by_variants() {
        assert_eq!(
            from_str::<SearchBookSortBy>("\"rank-desc\""),
            SearchBookSortBy::RankDesc
        );
        assert_eq!(
            from_str::<SearchBookSortBy>("\"id-desc\""),
            SearchBookSortBy::Id(Sort::Desc)
        );
        assert_eq!(
            from_str::<SearchBookSortBy>("\"id-asc\""),
            SearchBookSortBy::Id(Sort::Asc)
        );
    }

    #[test]
    fn should_default_search_book_sort_by_to_rank_desc() {
        assert_eq!(SearchBookSortBy::default(), SearchBookSortBy::RankDesc);
    }
}
