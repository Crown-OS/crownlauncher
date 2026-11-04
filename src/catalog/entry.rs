use std::path::PathBuf;

use freedesktop_entry_parser::{AttrSelector, Entry};

use super::DesktopId;
use crate::{search::SearchKey, util::locale};

const SECTION: &str = "Desktop Entry";
const APPLICATION: &str = "Application";
const CALENDAR_CATEGORY: &str = "Calendar";

/// An installed application, as its `.desktop` file describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEntry {
    pub id: DesktopId,
    pub path: PathBuf,
    pub name: String,
    pub comment: Option<String>,
    pub icon: Option<String>,
    pub exec: Option<String>,
    pub terminal: bool,
    pub categories: Vec<String>,
    pub keys: Vec<SearchKey>,
}

pub struct Environment<'a> {
    pub locale: Option<&'a str>,
    pub desktops: &'a [String],
}

impl AppEntry {
    pub fn parse(
        id: DesktopId,
        path: PathBuf,
        bytes: Vec<u8>,
        environment: &Environment<'_>,
    ) -> Option<Self> {
        let entry = Entry::parse(bytes).ok()?;
        let section = entry.section(SECTION);
        let shown = section.attr("Type") == Some(APPLICATION)
            && !flag(&section, "NoDisplay")
            && !flag(&section, "Hidden")
            && shown_in(&section, environment.desktops);
        if !shown {
            return None;
        }
        let text = |key| localized(&section, key, environment.locale).map(str::to_owned);
        let name = text("Name")?;
        let keys = std::iter::once(name.as_str())
            .chain(text("GenericName").as_deref())
            .map(SearchKey::new)
            .chain(
                list(localized(&section, "Keywords", environment.locale))
                    .iter()
                    .map(|keyword| SearchKey::new(keyword)),
            )
            .collect();
        Some(Self {
            comment: text("Comment").or_else(|| text("GenericName")),
            icon: text("Icon"),
            exec: section.attr("Exec").map(str::to_owned),
            terminal: flag(&section, "Terminal"),
            categories: list(section.attr("Categories")),
            keys,
            name,
            id,
            path,
        })
    }

    /// Whether the launcher draws today's date as this app's icon.
    pub fn is_calendar(&self) -> bool {
        self.categories
            .iter()
            .any(|category| category == CALENDAR_CATEGORY)
    }
}

fn flag(section: &AttrSelector<'_, &str>, key: &str) -> bool {
    section.attr(key) == Some("true")
}

fn list(value: Option<&str>) -> Vec<String> {
    value
        .into_iter()
        .flat_map(|value| value.split(';'))
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn shown_in(section: &AttrSelector<'_, &str>, desktops: &[String]) -> bool {
    let listed = |key| list(section.attr(key));
    let only = listed("OnlyShowIn");
    let not = listed("NotShowIn");
    let current = |items: &[String]| items.iter().any(|item| desktops.contains(item));
    (only.is_empty() || current(&only)) && !current(&not)
}

fn localized<'s>(
    section: &'s AttrSelector<'_, &str>,
    key: &str,
    locale: Option<&str>,
) -> Option<&'s str> {
    locale
        .into_iter()
        .flat_map(locale::candidates)
        .find_map(|candidate| section.attr_with_param(key, candidate))
        .or_else(|| section.attr(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::Query;

    fn parse(text: &str, locale: Option<&str>, desktops: &[String]) -> Option<AppEntry> {
        AppEntry::parse(
            DesktopId::new("org.example.App.desktop"),
            PathBuf::from("/apps/org.example.App.desktop"),
            text.as_bytes().to_vec(),
            &Environment { locale, desktops },
        )
    }

    const APP: &str = "[Desktop Entry]\nType=Application\nName=Files\nName[de]=Dateien\nGenericName=File Manager\nKeywords=folder;explorer;\nExec=files %U\nIcon=files\nCategories=System;Utility;\n";

    #[test]
    fn localized_name_wins_over_the_default() {
        let entry = parse(APP, Some("de_DE.UTF-8"), &[]);
        assert_eq!(entry.map(|entry| entry.name), Some("Dateien".to_owned()));
    }

    #[test]
    fn generic_names_and_keywords_are_searchable() {
        let Some(entry) = parse(APP, None, &[]) else {
            panic!("the entry parses");
        };
        let best = |query: &str| {
            let query = Query::new(query);
            entry.keys.iter().filter_map(|key| query.score(key)).max()
        };
        assert!(best("manager").is_some());
        assert!(best("explorer").is_some());
        assert!(best("spreadsheet").is_none());
        assert_eq!(entry.comment.as_deref(), Some("File Manager"));
    }

    #[test]
    fn hidden_and_foreign_entries_are_skipped() {
        let hidden = format!("{APP}NoDisplay=true\n");
        assert!(parse(&hidden, None, &[]).is_none());
        let kde_only = format!("{APP}OnlyShowIn=KDE;\n");
        assert!(parse(&kde_only, None, &["CrownOS".to_owned()]).is_none());
        assert!(parse(&kde_only, None, &["KDE".to_owned()]).is_some());
    }

    #[test]
    fn calendars_are_known_by_their_category() {
        let calendar = APP.replace("System;Utility;", "Office;Calendar;");
        assert_eq!(
            parse(&calendar, None, &[]).map(|entry| entry.is_calendar()),
            Some(true)
        );
        assert_eq!(
            parse(APP, None, &[]).map(|entry| entry.is_calendar()),
            Some(false)
        );
    }
}
