#![no_std]
#![allow(non_snake_case, unsafe_op_in_unsafe_fn)]

use core::sync::atomic::{AtomicU32, Ordering};
use forge::log::prelude::*;
extern crate alloc;
mod rules;
mod selector;
mod purple_lib;
use forge::mt::{
    fs::{MtFile, OpenMode},
    object::Object,
};
use selector::board_update;

static BASE: AtomicU32 = AtomicU32::new(0);
static EXTRA: AtomicU32 = AtomicU32::new(0);
static PARAM: AtomicU32 = AtomicU32::new(0);
static FLAGS: AtomicU32 = AtomicU32::new(0);
static LISTED: AtomicU32 = AtomicU32::new(0);
static SECOND: AtomicU32 = AtomicU32::new(0);
static CARTS: AtomicU32 = AtomicU32::new(0);
static EXP_DONE: AtomicU32 = AtomicU32::new(0);
static COUNT_DONE: AtomicU32 = AtomicU32::new(0);
static MORALE_DONE: AtomicU32 = AtomicU32::new(0);
static PLACING: AtomicU32 = AtomicU32::new(0);

unsafe fn r32(a: u32) -> u32 {
    core::ptr::read_volatile(a as *const u32)
}
unsafe fn r8(a: u32) -> u32 {
    core::ptr::read_volatile(a as *const u8) as u32
}
unsafe fn w32(a: u32, v: u32) {
    core::ptr::write_volatile(a as *mut u32, v);
}
unsafe fn w8(a: u32, v: u32) {
    core::ptr::write_volatile(a as *mut u8, v as u8);
}
unsafe fn r16(a: u32) -> u32 {
    core::ptr::read_volatile(a as *const u16) as u32
}
unsafe fn w16(a: u32, v: u32) {
    core::ptr::write_volatile(a as *mut u16, v as u16);
}
unsafe fn rf(a: u32) -> f32 {
    core::ptr::read_volatile(a as *const f32)
}
fn addr(off: u32) -> u32 {
    BASE.load(Ordering::Relaxed).wrapping_add(off)
}
unsafe fn call1(off: u32, a: u32) -> u32 {
    let f: unsafe extern "C" fn(u32) -> u32 = core::mem::transmute(addr(off));
    f(a)
}
unsafe fn call2(off: u32, a: u32, b: u32) -> u32 {
    let f: unsafe extern "C" fn(u32, u32) -> u32 = core::mem::transmute(addr(off));
    f(a, b)
}
unsafe fn call3(off: u32, a: u32, b: u32, c: u32) -> u32 {
    let f: unsafe extern "C" fn(u32, u32, u32) -> u32 = core::mem::transmute(addr(off));
    f(a, b, c)
}
unsafe fn virtual1(obj: u32, off: u32) -> u32 {
    let f: unsafe extern "C" fn(u32) -> u32 = core::mem::transmute(r32(r32(obj) + off));
    f(obj)
}
unsafe fn virtual2(obj: u32, off: u32, arg: u32) -> u32 {
    let f: unsafe extern "C" fn(u32, u32) -> u32 = core::mem::transmute(r32(r32(obj) + off));
    f(obj, arg)
}
unsafe fn roster(mgr: u32, index: u32) -> u32 {
    if index >= 84 {
        return 0;
    }
    let storage = r8(mgr + 79950 + index);
    if storage >= 84 {
        return 0;
    }
    let p = mgr + 80184 + 1376 * storage;
    if r8(p + 629) == 0 { 0 } else { p }
}
unsafe fn name<'a>(p: u32) -> &'a str {
    let buf = core::slice::from_raw_parts((p + 1300) as *const u8, 32);
    core::str::from_utf8(&buf[..buf.iter().position(|x| *x == 0).unwrap_or(32)]).unwrap_or("?")
}

fn read_config(path: &str) -> Option<rules::SelectionConfig> {
    let path = alloc::ffi::CString::new(path).ok()?;
    let file = MtFile::open(&path, OpenMode::Read).ok()?;
    let mut buf = [0u8; 4096];
    let size = file.size();
    let result = if size <= buf.len() && file.read(&mut buf[..size]) == size {
        core::str::from_utf8(&buf[..size])
            .ok()
            .map(rules::parse_config)
    } else {
        None
    };
    file.close();
    file.destroy();
    result
}

fn load_config() -> rules::SelectionConfig {
    let unavailable = rules::SelectionConfig {
        index: None, combo: rules::DEFAULT_COMBO, combo_error: None,
    };
    if !forge::fs::savedata::mount() {
        warn!("[threecats] cannot mount savedata to read config, disabled");
        return unavailable;
    }
    let saved = alloc::format!("{}:/three_palicos.ini", forge::fs::savedata::mount_point());
    let bundled = alloc::format!(
        "{}:/nativeNX/plugins/three_palicos.ini",
        forge::fs::romfs::mount_point()
    );
    let backup = alloc::format!("{}.bak", saved);
    let config = if forge::fs::file_exists(&saved) {
        read_config(&saved)
    } else if forge::fs::file_exists(&backup) {
        // Recover a selection if interrupted between renaming the old and new files.
        read_config(&backup)
    } else {
        read_config(&bundled)
    };
    forge::fs::savedata::unmount();
    let config = config.unwrap_or(unavailable);
    if config.index.is_none() {
        warn!(
            "[threecats] unreadable/invalid config: use [ThreePalicos], one ThirdPalicoIndex=-1..83; disabled"
        );
    }
    if let Some(reason) = config.combo_error {
        warn!("[threecats] SelectCombo: {}; using ZL+RSTICK", reason);
    }
    info!("[threecats] ThirdPalicoIndex={} SelectCombo={}", config.index.unwrap_or(-1), rules::combo_name(config.combo));
    config
}

fn load_index() -> i32 {
    load_config().index.unwrap_or(-1)
}

unsafe fn original_status(p: u32) -> u32 {
    rules::restore_status(r32(p + 768), FLAGS.load(Ordering::Relaxed))
}

unsafe fn restore_standby() {
    let p = PARAM.load(Ordering::Relaxed);
    if p != 0 {
        w32(p + 768, original_status(p));
        PARAM.store(0, Ordering::Relaxed);
        EXTRA.store(0, Ordering::Relaxed);
        SECOND.store(0, Ordering::Relaxed);
        info!("[threecats] returned to town, restored standby status");
    }
}

// Native AI uOtomo construction from 0x259894, in the unused third companion slot.
unsafe fn spawn(mgr: u32, p: u32, owner: u32) -> u32 {
    let actor = call2(0x268cd4, 15952, 16);
    if actor == 0 {
        return 0;
    }
    call2(0x268d10, actor, 2);
    w32(mgr + 36, actor);
    w32(actor + 5068, r32(actor + 5068) | 10);
    w32(actor + 5064, 0);
    w8(actor + 4177, 2);
    // The map uses virtual getColor (+416) to index its four-entry icon table.
    w32(actor + 10116, 3);
    w32(actor + 5060, p);
    virtual2(actor, 988, owner);
    let player = call2(0x274ce4, r32(addr(0x1885b58)), owner);
    let brain = r32(actor + 13096);
    w32(brain + 1776, player);
    w8(brain + 1503, call1(0x27bcd0, player));
    w8(brain + 1504, 0);
    w8(brain + 961, 2);
    call1(0x288248, actor);
    call1(0x27acd8, actor);
    call2(0x26fd90, p, actor);
    let equip: unsafe extern "C" fn(u32, u32, u32) -> u32 = core::mem::transmute(addr(0x259ba8));
    equip(p, actor, p);
    if virtual1(actor, 344) == 0 {
        w8(actor + 4378, 1);
    }
    actor
}

#[forge::hook(offset = 0x11243a8)]
fn place_companion(actor: u32) -> u32 {
    let extra = actor != 0 && actor == EXTRA.load(Ordering::Relaxed);
    if extra {
        PLACING.store(actor, Ordering::Relaxed);
    }
    let result = original!(actor);
    if extra {
        PLACING.store(0, Ordering::Relaxed);
    }
    result
}

#[forge::hook(offset = 0x1813bc)]
fn place_on_ground(
    world: u32,
    actor: u32,
    pos: u32,
    from: u32,
    area: u32,
    hit: u32,
    flags: u32,
    mask: u32,
    option: u32,
) -> u32 {
    if actor != 0 && PLACING.load(Ordering::Relaxed) == actor && pos == actor + 64 {
        unsafe {
            let owner = r32(r32(actor + 13096) + 1776);
            let (x, z) = rules::third_formation_offset(
                rf(pos) - rf(owner + 64),
                rf(pos + 8) - rf(owner + 72),
                false,
            );
            w32(pos, (rf(owner + 64) + x).to_bits());
            w32(pos + 8, (rf(owner + 72) + z).to_bits());
        }
    }
    // Keep the tail call: the native ground routine also returns height in VFP s0.
    original!(world, actor, pos, from, area, hit, flags, mask, option)
}

#[forge::hook(offset = 0x1136a08)]
fn follow_target(actor: u32, pos: u32, side: u32) -> u32 {
    let result = original!(actor, pos, side);
    if actor != 0 && actor == EXTRA.load(Ordering::Relaxed) {
        unsafe {
            let brain = r32(actor + 13096);
            let direction = r8(side);
            if r8(brain + 1505) == 0 && (direction == 2 || direction == 3) {
                let owner = r32(brain + 1776);
                let (x, z) = rules::third_formation_offset(
                    rf(pos) - rf(owner + 64),
                    rf(pos + 8) - rf(owner + 72),
                    direction == 3,
                );
                w32(pos, (rf(owner + 64) + x).to_bits());
                w32(pos + 8, (rf(owner + 72) + z).to_bits());
            }
        }
    }
    result
}

#[forge::hook(offset = 0x2596d0)]
fn spawn_selected(mgr: u32, phase: u32) -> u32 {
    if phase == 1 {
        unsafe { restore_standby() };
        LISTED.store(0, Ordering::Relaxed);
    }
    let result = original!(mgr, phase);
    info!(
        "[threecats] spawn_selected phase={} slots={:x}/{:x}/{:x}",
        phase,
        unsafe { r32(mgr + 28) },
        unsafe { r32(mgr + 32) },
        unsafe { r32(mgr + 36) }
    );
    if phase != 0 || EXTRA.load(Ordering::Relaxed) != 0 {
        return result;
    }
    unsafe {
        let index = load_index();
        if !(0..84).contains(&index) {
            return result;
        }
        let net = r32(addr(0x18843f0));
        let game = r32(addr(0x188ca48));
        let players = r32(addr(0x1885b58));
        if net == 0 || game == 0 || players == 0 {
            return result;
        }
        let owner = call1(0x3f8398, game);
        if owner >= 4 || (0..4).any(|i| i != owner && r32(players + 28 + 4 * i) != 0) {
            warn!("[threecats] more than one player, skipped");
            return result;
        }
        let player = call2(0x274ce4, players, owner);
        let p = roster(mgr, index as u32);
        if p == 0 {
            warn!("[threecats] index {} is not a hired cat, skipped", index);
            return result;
        }
        let mode = r8(players + 40028);
        if index as u32 == r8(mgr + 79945)
            || index as u32 == r8(mgr + 79946)
            || (mode == 1 && index as u32 == r8(mgr + 79944))
        {
            warn!(
                "[threecats] index {} duplicates a companion or the controlled Prowler, skipped",
                index
            );
            return result;
        }
        if r32(p + 768) & 0x20 == 0 || call2(0x25f34c, mgr, index as u32) == 0 {
            warn!(
                "[threecats] index {} is not available on standby, skipped",
                index
            );
            return result;
        }
        if r8(net + 1598) != 0
            || player == 0
            || virtual1(player, 344) != 0
            || mode > 1
            || r32(mgr + 28) == 0
            || r32(mgr + 32) == 0
            || r32(mgr + 36) != 0
        {
            warn!(
                "[threecats] requires an offline solo Hunter/Prowler quest with two native companions, skipped"
            );
            return result;
        }
        let before = r32(p + 768);
        FLAGS.store(before, Ordering::Relaxed);
        PARAM.store(p, Ordering::Relaxed);
        SECOND.store(r32(r32(mgr + 32) + 5060), Ordering::Relaxed);
        for counter in [&CARTS, &EXP_DONE, &COUNT_DONE, &MORALE_DONE] {
            counter.store(0, Ordering::Relaxed);
        }
        w32(p + 768, (before & !0x20) | 2);
        let actor = spawn(mgr, p, owner);
        if actor == 0 {
            w32(p + 768, before);
            PARAM.store(0, Ordering::Relaxed);
        }
        EXTRA.store(actor, Ordering::Relaxed);
        info!(
            "[threecats] spawned {} actor={:08x} param={:08x}",
            name(p),
            actor,
            p
        );
    }
    result
}

// Hide the extra for the entire native loot pass, including item bags and appraisal items.
#[forge::hook(offset = 0x3c6180)]
fn reward_list(rewards: u32) -> u32 {
    let mgr = unsafe { r32(addr(0x1885468)) };
    let extra = EXTRA.load(Ordering::Relaxed);
    let held = if mgr != 0 && extra != 0 && unsafe { r32(mgr + 36) } == extra {
        extra
    } else {
        0
    };
    if held != 0 {
        unsafe { w32(mgr + 36, 0) };
    }
    let result = original!(rewards);
    if held != 0 {
        unsafe { w32(mgr + 36, held) };
    }
    result
}

#[forge::hook(offset = 0x258ac8)]
fn manager_update(mgr: u32) -> u32 {
    let result = original!(mgr);
    unsafe {
        if LISTED.load(Ordering::Relaxed) == 0 && roster(mgr, 0) != 0 {
            LISTED.store(1, Ordering::Relaxed);
            for i in 0..84 {
                let p = roster(mgr, i);
                if p != 0 {
                    info!(
                        "[threecats] roster {}: {} flags={:x}",
                        i,
                        name(p),
                        r32(p + 768)
                    );
                }
            }
        }
        let game = r32(addr(0x188ca48));
        if game != 0 && call1(0x3f8678, game) != 0 {
            restore_standby();
        }
    }
    result
}

#[forge::hook(offset = 0x2635b4)]
fn save_roster(mgr: u32, file: u32) -> u32 {
    let p = PARAM.load(Ordering::Relaxed);
    let active = if p != 0 { unsafe { r32(p + 768) } } else { 0 };
    if p != 0 {
        unsafe { w32(p + 768, original_status(p)) };
    }
    let result = original!(mgr, file);
    if p != 0 {
        unsafe { w32(p + 768, active) };
    }
    result
}

// The native quest has only two cart counters; the extra must not increment the second cat's.
#[forge::hook(offset = 0x11332a8)]
fn action_stats(actor: u32) -> u32 {
    if actor != EXTRA.load(Ordering::Relaxed) || actor == 0 {
        return original!(actor);
    }
    unsafe {
        let quest = r32(addr(0x188ad90));
        let before = r16(quest + 5750);
        let result = original!(actor);
        let after = r16(quest + 5750);
        if after != before {
            CARTS.fetch_add(after.wrapping_sub(before) & 0xffff, Ordering::Relaxed);
            w16(quest + 5750, before);
        }
        result
    }
}

#[forge::hook(offset = 0x264ed8)]
fn morale(mgr: u32, record: u32, carts: u32) -> u32 {
    let result = original!(mgr, record, carts);
    let p = PARAM.load(Ordering::Relaxed);
    if p != 0
        && record == SECOND.load(Ordering::Relaxed)
        && MORALE_DONE.swap(1, Ordering::Relaxed) == 0
    {
        original!(mgr, p, CARTS.load(Ordering::Relaxed));
        info!(
            "[threecats] morale settled, own carts={}",
            CARTS.load(Ordering::Relaxed)
        );
    }
    result
}

#[forge::hook(offset = 0x3905c8)]
fn experience(window: u32) -> u32 {
    let result = original!(window);
    let p = PARAM.load(Ordering::Relaxed);
    if p != 0 && EXP_DONE.swap(1, Ordering::Relaxed) == 0 {
        unsafe {
            let mgr = r32(addr(0x1885468));
            let coeff = call1(0x264258, mgr);
            let player = call1(0x275a30, r32(addr(0x1885b58)));
            let mut exp = call1(0x3ae3c8, r32(addr(0x188ad90)));
            if player != 0 && call2(0x26fe10, player, 52) != 0 {
                exp = (exp as f32 * rf(coeff + 8)) as u32;
            }
            let before = r32(p + 576);
            call3(0xe7b7c, p + 4, exp, 0);
            info!(
                "[threecats] experience settled {}: {} -> {} (base {})",
                name(p),
                before,
                r32(p + 576),
                exp
            );
        }
    }
    result
}

#[forge::hook(offset = 0x262bd0)]
fn quest_count(mgr: u32, data: u32) -> u32 {
    let result = original!(mgr, data);
    let p = PARAM.load(Ordering::Relaxed);
    if p != 0
        && data == SECOND.load(Ordering::Relaxed) + 4
        && COUNT_DONE.swap(1, Ordering::Relaxed) == 0
    {
        original!(mgr, p + 4);
        info!("[threecats] quest progression settled {}", unsafe {
            name(p)
        });
    }
    result
}

#[forge::hook(offset = 0x2625cc)]
fn support_unlocks(mgr: u32) -> u32 {
    let result = original!(mgr);
    let p = PARAM.load(Ordering::Relaxed);
    if p == 0 {
        return result;
    }
    unsafe {
        let save = r32(addr(0x1897f78));
        let quest = r32(addr(0x188ad90));
        for i in 0..16 {
            let skill = call2(0xe925c, p + 4, i);
            if skill == 57 || call2(0x52462c, save, skill) != 0 {
                continue;
            }
            let lookup = match skill {
                254 => 57,
                255 => 0,
                other => other,
            };
            let row = call2(0x25608c, r32(mgr + 11068), lookup);
            let group = if row != 0 { r32(row + 28) } else { 0 };
            if group != 0 && call2(0x3a3930, quest, r16(addr(0x1627ea0) + 4 * group)) == 1 {
                let field = save + 3460 + 4 * (skill >> 5);
                w32(field, r32(field) | (1 << (skill & 31)));
            }
        }
    }
    result
}

#[forge::entry]
fn main() {
    BASE.store(forge::mem::text_addr(), Ordering::Relaxed);
    unsafe {
        for (off, want) in [
            (0x2596d0, 0xe92d48f0),
            (0x3c6180, 0xe92d41f0),
            (0x258ac8, 0xe92d4ff0),
            (0x2635b4, 0xe92d4ff0),
            (0x11332a8, 0xe92d4010),
            (0x264ed8, 0xe92d4830),
            (0x3905c8, 0xe92d47f0),
            (0x262bd0, 0xe92d48f0),
            (0x2625cc, 0xe92d4ff0),
            (0x748174, 0xe92d41f0),
            (0x6431bc, 0xe92d41f0),
            (0x829b4c, 0xe92d47f0),
            (0x829a60, 0xe92d41f0),
            (0x11243a8, 0xe92d47f0),
            (0x1813bc, 0xe92d41f0),
            (0x1136a08, 0xe92d4ff0),
        ] {
            let got = r32(addr(off));
            if got != want {
                error!("[threecats] unsupported code {:x}: {:x}, skipped", off, got);
                return;
            }
        }
    }
    forge::install_hook!(forge::mem::text_addr(), spawn_selected);
    forge::install_hook!(forge::mem::text_addr(), reward_list);
    forge::install_hook!(forge::mem::text_addr(), manager_update);
    forge::install_hook!(forge::mem::text_addr(), save_roster);
    forge::install_hook!(forge::mem::text_addr(), action_stats);
    forge::install_hook!(forge::mem::text_addr(), morale);
    forge::install_hook!(forge::mem::text_addr(), experience);
    forge::install_hook!(forge::mem::text_addr(), quest_count);
    forge::install_hook!(forge::mem::text_addr(), support_unlocks);
    forge::install_hook!(forge::mem::text_addr(), board_update);
    forge::install_hook!(forge::mem::text_addr(), place_companion);
    forge::install_hook!(forge::mem::text_addr(), place_on_ground);
    forge::install_hook!(forge::mem::text_addr(), follow_target);
    info!("[threecats] v0.4.2 PC preview: initialize third companion map icon color, third companion has no loot");
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(_delta: f32) {
    purple_lib::update();
}
