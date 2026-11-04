const TIER_EXACT: u32 = 1000;
const TIER_PREFIX: u32 = 800;
const TIER_INITIALS: u32 = 700;
const TIER_WORD_PREFIX: u32 = 600;
const TIER_SUBSTRING: u32 = 400;
const TIER_SUBSEQUENCE: u32 = 200;
const MAX_LENGTH_BIAS: u32 = 100;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchKey {
    folded: String,
    words: Vec<String>,
    initials: String,
}

impl SearchKey {
    pub fn new(name: &str) -> Self {
        let words = split_words(name);
        Self {
            folded: name.to_lowercase(),
            initials: words
                .iter()
                .filter_map(|word| word.chars().next())
                .collect(),
            words,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    folded: String,
}

impl Query {
    pub fn new(raw: &str) -> Self {
        Self {
            folded: raw.trim().to_lowercase(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.folded.is_empty()
    }

    pub fn score(&self, key: &SearchKey) -> Option<u32> {
        if self.is_empty() {
            return None;
        }
        let query = self.folded.as_str();
        let tier = if key.folded == query {
            TIER_EXACT
        } else if key.folded.starts_with(query) {
            TIER_PREFIX
        } else if key.initials.len() > 1 && key.initials.starts_with(query) {
            TIER_INITIALS
        } else if key.words.iter().any(|word| word.starts_with(query)) {
            TIER_WORD_PREFIX
        } else if key.folded.contains(query) {
            TIER_SUBSTRING
        } else if is_subsequence(query, &key.folded) {
            TIER_SUBSEQUENCE
        } else {
            return None;
        };
        let length_bias = MAX_LENGTH_BIAS.saturating_sub(key.folded.chars().count() as u32);
        Some(tier + length_bias)
    }
}

fn split_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for character in name.chars() {
        if !character.is_alphanumeric() {
            words.extend((!current.is_empty()).then(|| std::mem::take(&mut current)));
            previous_lower = false;
            continue;
        }
        if character.is_uppercase() && previous_lower && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        previous_lower = character.is_lowercase();
        current.extend(character.to_lowercase());
    }
    words.extend((!current.is_empty()).then_some(current));
    words
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut remaining = haystack.chars();
    needle
        .chars()
        .filter(|character| !character.is_whitespace())
        .all(|wanted| remaining.any(|character| character == wanted))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(query: &str, name: &str) -> Option<u32> {
        Query::new(query).score(&SearchKey::new(name))
    }

    #[test]
    fn camel_case_and_spaces_split_words() {
        assert_eq!(
            split_words("LibreOffice Writer"),
            ["libre", "office", "writer"]
        );
        assert_eq!(split_words("GNOME-Terminal"), ["gnome", "terminal"]);
    }

    #[test]
    fn initials_find_multi_word_names() {
        assert!(score("vsc", "Visual Studio Code").is_some());
        assert!(score("as", "App Store") > score("as", "Gnome Disks Utility"));
    }

    #[test]
    fn tiers_rank_exact_over_prefix_over_word_over_substring() {
        let exact = score("files", "Files");
        let prefix = score("fil", "Files");
        let word = score("office", "LibreOffice");
        let substring = score("ffi", "LibreOffice");
        assert!(exact > prefix && prefix > word && word > substring);
    }

    #[test]
    fn subsequence_matches_last_and_misses_are_none() {
        assert!(score("frfx", "Firefox").is_some());
        assert_eq!(score("zzz", "Firefox"), None);
    }

    #[test]
    fn empty_query_matches_nothing() {
        assert_eq!(score("  ", "Files"), None);
    }
}
