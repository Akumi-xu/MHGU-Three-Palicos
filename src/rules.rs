pub fn parse_index(text: &str) -> Option<i32> {
    let mut section = false;
    let mut found = None;
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.split([';', '#']).next()?.trim();
        if line.starts_with('[') {
            section = line == "[ThreePalicos]";
        } else if section {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "ThirdPalicoIndex" {
                    if found.is_some() {
                        return None;
                    }
                    let value: i32 = value.trim().parse().ok()?;
                    if !(-1..84).contains(&value) {
                        return None;
                    }
                    found = Some(value);
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_is_explicit_and_bounded() {
        assert_eq!(parse_index("[ThreePalicos]\nThirdPalicoIndex=3"), Some(3));
        assert_eq!(
            parse_index("\u{feff}[ThreePalicos]\nThirdPalicoIndex=83 ; last"),
            Some(83)
        );
        assert_eq!(parse_index("[ThreePalicos]\nThirdPalicoIndex=-1"), Some(-1));
        for value in ["", "cats", "84", "-2", "2147483648"] {
            assert_eq!(
                parse_index(&format!("[ThreePalicos]\nThirdPalicoIndex={value}")),
                None
            );
        }
    }
    #[test]
    fn missing_or_ambiguous_key_never_selects_first_cat() {
        assert_eq!(parse_index("ThirdPalicoIndex=0"), None);
        assert_eq!(parse_index("[Other]\nThirdPalicoIndex=0"), None);
        assert_eq!(
            parse_index("[ThreePalicos]\nThirdPalicoIndex=0\nThirdPalicoIndex=3"),
            None
        );
    }
    #[test]
    fn saving_restores_only_the_activity_bits_owned_by_the_mod() {
        assert_eq!(restore_status(0x402, 0x20), 0x420);
        assert_eq!(restore_status(0x802, 0x420), 0x820);
        assert_eq!(restore_status(0x21, 0x02), 0x02);
    }
}
pub fn restore_status(active: u32, original: u32) -> u32 {
    (active & !0x23) | (original & 0x23)
}
