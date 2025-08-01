use codes_iso_639::part_1::ALL_CODES;
use std::collections::HashSet;
use std::sync::LazyLock;

static LANG_OPTIONS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    ALL_CODES
        .iter()
        .map(|lc| {
            let ln = lc.language_name();
            (lc.code(), ln.split_once(" ;").map(|v| v.0).unwrap_or(ln))
        })
        .collect()
});

static LANG_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| ALL_CODES.iter().map(|lc| lc.code()).collect());

pub fn is_lang(lang: &str) -> bool {
    LANG_SET.contains(lang)
}

pub fn lang_choices() -> Vec<(&'static str, &'static str)> {
    LANG_OPTIONS.clone()
}
