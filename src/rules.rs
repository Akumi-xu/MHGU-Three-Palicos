extern crate alloc;

pub const DEFAULT_COMBO: u32 = 0x404;
// Native sPad+27636 bits, not nn::hid / Forge button ordinals.
pub const COMBO_BUTTONS: [(&str, u32); 6] = [
    ("L", 0x100), ("R", 0x200), ("ZL", 0x400), ("ZR", 0x800),
    ("LSTICK", 0x2), ("RSTICK", 0x4),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionConfig {
    pub index: Option<i32>,
    pub combo: u32,
    pub combo_error: Option<&'static str>,
}

pub fn parse_combo(value: &str) -> Result<u32, &'static str> {
    let mut mask = 0;
    for token in value.split('+') {
        let bit = COMBO_BUTTONS.iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(token.trim()))
            .map(|(_, bit)| *bit)
            .ok_or("包含未知或不支持的按键")?;
        if mask & bit != 0 {
            return Err("包含重复按键");
        }
        mask |= bit;
    }
    if mask.count_ones() < 2 {
        return Err("需要至少两个不同按键");
    }
    Ok(mask)
}

pub fn parse_config(text: &str) -> SelectionConfig {
    let mut section = false;
    let mut found = None;
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.split([';', '#']).next().unwrap_or("").trim();
        if line.starts_with('[') {
            section = line == "[ThreePalicos]";
        } else if section {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "SelectCombo" {
                    if found.is_some() {
                        found = Some(Err("SelectCombo重复定义"));
                        break;
                    }
                    found = Some(parse_combo(value.trim()));
                }
            }
        }
    }
    let (combo, combo_error) = match found.unwrap_or(Ok(DEFAULT_COMBO)) {
        Ok(mask) => (mask, None),
        Err(reason) => (DEFAULT_COMBO, Some(reason)),
    };
    SelectionConfig { index: parse_index(text), combo, combo_error }
}

pub fn combo_name(mask: u32) -> alloc::string::String {
    let mut text = alloc::string::String::new();
    for (name, bit) in COMBO_BUTTONS {
        if mask & bit != 0 {
            if !text.is_empty() { text.push('+'); }
            text.push_str(name);
        }
    }
    text
}

pub fn set_combo_button(mask: u32, row: u32, enabled: u32) -> u32 {
    let Some((_, bit)) = COMBO_BUTTONS.get(row as usize) else { return mask };
    let next = match enabled {
        0 => mask & !bit,
        1 => mask | bit,
        _ => return mask,
    };
    if next.count_ones() >= 2 { next } else { mask }
}

pub fn format_selection(index: i32, combo: u32) -> alloc::string::String {
    alloc::format!("[ThreePalicos]\nThirdPalicoIndex = {}\nSelectCombo = {}\n", index, combo_name(combo))
}

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
    fn menu_toggles_keep_every_combo_valid_and_allow_row_cancel() {
        for bits in 0u32..64 {
            if bits.count_ones() < 2 { continue; }
            let mask = COMBO_BUTTONS.iter().enumerate().fold(0, |m, (i, (_, bit))|
                m | if bits & (1 << i) != 0 { *bit } else { 0 });
            assert_eq!(parse_combo(&combo_name(mask)), Ok(mask));
            for row in 0..6 {
                let bit = COMBO_BUTTONS[row as usize].1;
                let before = (mask & bit != 0) as u32;
                let after = set_combo_button(mask, row, 1 - before);
                assert!(after.count_ones() >= 2);
                assert_eq!(set_combo_button(after, row, before), mask);
            }
            assert_eq!(set_combo_button(mask, 6, 1), mask);
            assert_eq!(set_combo_button(mask, 0, 2), mask);
        }
        let mut mask = DEFAULT_COMBO;
        for (row, value) in [(0,1), (1,1), (2,0), (5,0)] {
            mask = set_combo_button(mask, row, value);
        }
        assert_eq!(mask, 0x300);
    }
    #[test]
    fn combo_defaults_keep_old_configs_compatible() {
        let old = parse_config("[ThreePalicos]\nThirdPalicoIndex=3");
        assert_eq!(old, SelectionConfig { index: Some(3), combo: 0x404, combo_error: None });
        assert_eq!(parse_config("[Other]\nSelectCombo=L+R").combo, DEFAULT_COMBO);
        assert_eq!(parse_config("[ThreePalicos]\nThirdPalicoIndex=bad\nSelectCombo=L+R").index, None);
    }
    #[test]
    fn combo_names_map_to_native_pad_bits() {
        assert_eq!(parse_combo(" l + r "), Ok(0x300));
        assert_eq!(parse_combo("zl+rstICK"), Ok(0x404));
        assert_eq!(parse_combo("ZR+LSTICK+R"), Ok(0xa02));
        assert_eq!(parse_combo("L+R+ZL+ZR+LSTICK+RSTICK"), Ok(0xf06));
    }
    #[test]
    fn malformed_combo_uses_default_without_disabling_cat() {
        for value in ["", "L", "L+L", "L+R+L", "L+A", "ZL+UP", "L+", "+L", "Ctrl+F8"] {
            let text = format!("[ThreePalicos]\nThirdPalicoIndex=9\nSelectCombo={value}");
            let config = parse_config(&text);
            assert_eq!(config.index, Some(9));
            assert_eq!(config.combo, DEFAULT_COMBO);
            assert!(config.combo_error.is_some());
        }
        assert_eq!(parse_config("[ThreePalicos]\nSelectCombo=L+R\nSelectCombo=ZL+ZR").combo_error, Some("SelectCombo重复定义"));
    }
    #[test]
    fn selection_save_roundtrip_preserves_custom_combo() {
        let config = parse_config("\u{feff}[ThreePalicos]\nThirdPalicoIndex=3\nSelectCombo= r + l ; note");
        for index in [-1, 0, 83] {
            let saved = format_selection(index, config.combo);
            assert!(saved.contains("SelectCombo = L+R\n"));
            assert_eq!(parse_config(&saved), SelectionConfig { index: Some(index), combo: 0x300, combo_error: None });
        }
    }
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

// Rotate either native +/-110-degree follow position to a third, rear position.
pub fn third_formation_offset(x: f32, z: f32, left: bool) -> (f32, f32) {
    let sin = if left { 0.9396926 } else { -0.9396926 };
    (0.34202015 * x + sin * z, -sin * x + 0.34202015 * z)
}

#[cfg(test)]
mod selector_tests {
    use super::*;
    #[test]
    fn third_formation_preserves_radius_and_separates_from_native_second() {
        for (x, z) in [(200.0, 0.0), (0.0, 200.0), (-187.93852, -68.40403)] {
            let (a, b) = third_formation_offset(x, z, false);
            assert!((a * a + b * b - x * x - z * z).abs() < 0.02);
            assert!((a - x) * (a - x) + (b - z) * (b - z) > 50000.0);
        }
        let (x, z) = third_formation_offset(-187.93852, -68.40403, false);
        assert!(x.abs() < 0.01 && (z + 200.0).abs() < 0.01);
        let (x, z) = third_formation_offset(187.93852, -68.40403, true);
        assert!(x.abs() < 0.01 && (z + 200.0).abs() < 0.01);
    }
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
