// Purple-lib ABI: Copyright (c) 2026 PurplefinNeptuna, MIT; see THIRD_PARTY_NOTICES.txt.
use super::*;
use core::ffi::{CStr, c_char, c_void};
use core::sync::atomic::{AtomicBool, AtomicPtr};

#[repr(C)]
pub struct Api {
    version: u32,
    add_options: extern "C" fn(*const Options) -> bool,
    add_help: *const c_void,
    hold_input: *const c_void,
    options_shown: extern "C" fn(*const Options) -> bool,
}

#[repr(C)]
struct OptionRow {
    name: *const c_char,
    help: *const c_char,
    labels: *const *const c_char,
    label_count: u32,
    kind: u32,
    value_count: u32,
    first: i32,
    step: i32,
    suffix: *const c_char,
    flags: u32,
}

#[repr(C)]
struct Options {
    size: u32,
    context: *mut c_void,
    title: extern "C" fn(*mut c_void, u32) -> *const c_char,
    row_count: u32,
    row: extern "C" fn(*mut c_void, u32, u32, *mut OptionRow) -> bool,
    get: extern "C" fn(*mut c_void, u32) -> u32,
    set: extern "C" fn(*mut c_void, u32, u32),
    closed: Option<extern "C" fn(*mut c_void)>,
    value_text: *const c_void,
    action: *const c_void,
}

const _: () = assert!(core::mem::size_of::<Options>() == 40
    && core::mem::size_of::<OptionRow>() == 40);
// All pointers in these records refer to immutable, static data.
unsafe impl Sync for Options {}
struct Labels([*const c_char; 2]);
unsafe impl Sync for Labels {}
static OFF_ON: Labels = Labels([c"Off".as_ptr(), c"On".as_ptr()]);
static OFF_ON_ZH: Labels = Labels([c"不使用".as_ptr(), c"使用".as_ptr()]);
static KEYS: [&CStr; 6] = [c"L", c"R", c"ZL", c"ZR", c"LSTICK", c"RSTICK"];
static OPTIONS: Options = Options {
    size: core::mem::size_of::<Options>() as u32,
    context: core::ptr::null_mut(), title, row_count: 6, row, get, set, closed: None,
    value_text: core::ptr::null(), action: core::ptr::null(),
};
static API: AtomicPtr<Api> = AtomicPtr::new(core::ptr::null_mut());
static COMBO: AtomicU32 = AtomicU32::new(rules::DEFAULT_COMBO);
static OPEN_COMBO: AtomicU32 = AtomicU32::new(rules::DEFAULT_COMBO);
static SHOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn title(_: *mut c_void, language: u32) -> *const c_char {
    if matches!(language, 7 | 8) { c"三随从猫：选猫组合键".as_ptr() }
    else { c"Three Palicos: Select Combo".as_ptr() }
}

extern "C" fn row(_: *mut c_void, language: u32, index: u32, out: *mut OptionRow) -> bool {
    let (Some(key), Some(out)) = (KEYS.get(index as usize), unsafe { out.as_mut() }) else {
        return false;
    };
    let chinese = matches!(language, 7 | 8);
    out.name = key.as_ptr();
    out.help = if chinese { c"至少保留两键，换键先启用新键。\n离开本页保存，重开选猫列表生效。".as_ptr() }
        else { c"Keep 2+ keys; enable new keys first.\nSaved on leaving this page; reopen Palico list.".as_ptr() };
    out.labels = if chinese { OFF_ON_ZH.0.as_ptr() } else { OFF_ON.0.as_ptr() };
    out.label_count = 2;
    true
}

extern "C" fn get(_: *mut c_void, index: u32) -> u32 {
    rules::COMBO_BUTTONS.get(index as usize).map_or(0, |(_, bit)|
        (COMBO.load(Ordering::Relaxed) & bit != 0) as u32)
}

extern "C" fn set(_: *mut c_void, index: u32, value: u32) {
    let current = COMBO.load(Ordering::Relaxed);
    COMBO.store(rules::set_combo_button(current, index, value), Ordering::Relaxed);
}

fn save_combo() {
    let combo = COMBO.load(Ordering::Relaxed);
    if combo == OPEN_COMBO.load(Ordering::Relaxed) { return; }
    let current = load_config();
    if current.combo == combo { return; }
    if current.index.is_some_and(|index| selector::save_selection(index, combo)) {
        info!("[threecats] Purple-lib saved SelectCombo={}", rules::combo_name(combo));
    } else {
        COMBO.store(current.combo, Ordering::Relaxed);
        warn!("[threecats] Purple-lib save failed; keeping existing INI and selection");
    }
}

pub fn update() {
    let Some(api) = (unsafe { API.load(Ordering::Acquire).as_ref() }) else { return };
    let shown = (api.options_shown)(&OPTIONS);
    let was_shown = SHOWN.swap(shown, Ordering::Relaxed);
    if shown && !was_shown {
        let combo = load_config().combo;
        COMBO.store(combo, Ordering::Relaxed);
        OPEN_COMBO.store(combo, Ordering::Relaxed);
    } else if !shown && was_shown {
        // Purple-lib's closed callback waits until the whole window closes, including other pages.
        save_combo();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn purplelib_on_register(api: *const Api) {
    let Some(api) = (unsafe { api.as_ref() }) else { return };
    if api.version < 4 { return; }
    COMBO.store(load_config().combo, Ordering::Relaxed);
    if (api.add_options)(&OPTIONS) {
        API.store(api as *const Api as *mut Api, Ordering::Release);
        info!("[threecats] Purple-lib combo settings registered (API {})", api.version);
    } else {
        warn!("[threecats] Purple-lib refused combo settings; INI and board still available");
    }
}
