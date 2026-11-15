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

pub fn board_slot(row: u32, per_page: u32, page: u32) -> Option<u32> {
    if per_page != 6 || row >= per_page || page >= 14 {
        return None;
    }
    Some(row + per_page * page)
}

#[cfg(test)]
mod selector_tests {
    use super::*;
    #[test]
    fn board_position_is_bounded_before_reading_the_sort_map() {
        assert_eq!(board_slot(2, 6, 1), Some(8));
        assert_eq!(board_slot(5, 6, 13), Some(83));
        for args in [(6, 6, 0), (0, 0, 0), (0, 6, 14), (0, 84, 0)] {
            assert_eq!(board_slot(args.0, args.1, args.2), None);
        }
        let order = [13, 9, 8, 11, 12, 10, 0, 1, 2];
        assert_eq!(order[board_slot(2, 6, 1).unwrap() as usize], 2);
    }
}
