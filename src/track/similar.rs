//! A system to determine if two apparently different tracks are in fact the same.

/// Determine if two given track titles are similar. This is not a perfect system.
pub fn similar(a: &str, b: &str) -> bool {
    canonical(a) == canonical(b)
}

/// Get the canonical form of a track title. This is the title with as much
/// additional information removed as possible.
fn canonical(title: &str) -> String {
    let chars = title.to_lowercase().chars().collect::<Vec<_>>();
    remove_modifiers(&chars)
        .iter()
        // remove any remaining punctuation
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

/// Remove common modifiers from the end of a title (should be lowercase).
fn remove_modifiers(title: &[char]) -> &[char] {
    for offset in 3..(title.len() - 1) {
        if title[offset] != ' ' {
            continue;
        }
        let remaining = &title[offset + 1..];
        for modifier in &MODIFIERS {
            if remaining.starts_with(&modifier.chars().collect::<Vec<_>>()) {
                return &title[..offset];
            }
        }
    }
    title
}

/// Common modifiers that may be appended to track titles. These words and any
/// following them will be removed when comparing track titles.
const MODIFIERS: [&str; 36] = [
    "mix",
    "remix",
    "edition",
    "version",
    "acoustic",
    "live",
    "movie",
    "soundtrack",
    "original",
    "extended",
    "edit",
    "full",
    "official",
    "radio",
    "featuring",
    "feat",
    "ft",
    "session",
    "demo",
    "preview",
    "bonus",
    "track",
    "intro",
    "outro",
    "prelude",
    "official",
    "remastered",
    "sped",
    "(",
    "[",
    "- ",
    "+ ",
    "| ",
    "|| ",
    ": ",
    ":: ",
];
