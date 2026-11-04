use std::env;

const LOCALE_VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

pub fn current() -> Option<String> {
    LOCALE_VARIABLES
        .iter()
        .filter_map(|variable| env::var(variable).ok())
        .find(|value| !value.is_empty() && value != "C" && value != "POSIX")
}

pub fn candidates(locale: &str) -> impl Iterator<Item = &str> {
    let without_encoding = locale.split(['.', '@']).next().unwrap_or(locale);
    let language = without_encoding
        .split('_')
        .next()
        .unwrap_or(without_encoding);
    let regional = (language != without_encoding).then_some(without_encoding);
    regional
        .into_iter()
        .chain(Some(language).filter(|language| !language.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regional_locale_falls_back_to_its_language() {
        let found: Vec<_> = candidates("de_DE.UTF-8").collect();
        assert_eq!(found, ["de_DE", "de"]);
    }

    #[test]
    fn bare_language_is_its_only_candidate() {
        let found: Vec<_> = candidates("fr").collect();
        assert_eq!(found, ["fr"]);
    }
}
