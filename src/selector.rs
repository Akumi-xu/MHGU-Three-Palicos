use super::*;
use core::sync::atomic::AtomicI32;

static CHOICE: AtomicI32 = AtomicI32::new(-1);
static INSIDE: AtomicU32 = AtomicU32::new(0);
static HELD: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" {
    #[link_name = "_ZN2nn2fs12GetEntryTypeEPNS0_18DirectoryEntryTypeEPKc"]
    fn entry_type(kind: *mut u32, path: *const u8) -> u32;
    #[link_name = "_ZN2nn2fs10CreateFileEPKcx"]
    fn create_file(path: *const u8, size: i64) -> u32;
    #[link_name = "_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci"]
    fn open_file(handle: *mut u32, path: *const u8, mode: u32) -> u32;
    #[link_name = "_ZN2nn2fs11SetFileSizeENS0_10FileHandleEx"]
    fn set_size(handle: u32, size: i64) -> u32;
    #[link_name = "_ZN2nn2fs9WriteFileENS0_10FileHandleExPKvjRKNS0_11WriteOptionE"]
    fn write_file(handle: u32, offset: i64, data: *const u8, size: u32, option: *const u32) -> u32;
    #[link_name = "_ZN2nn2fs9CloseFileENS0_10FileHandleE"]
    fn close_file(handle: u32);
}

unsafe fn write_temp(path: &str, data: &[u8]) -> bool {
    let path = alloc::ffi::CString::new(path).unwrap();
    let mut kind = 0;
    let exists = entry_type(&mut kind, path.as_ptr().cast()) == 0;
    if (exists && kind != 1)
        || (!exists && create_file(path.as_ptr().cast(), data.len() as i64) != 0)
    {
        return false;
    }
    let mut handle = 0;
    if open_file(&mut handle, path.as_ptr().cast(), 2) != 0 {
        return false;
    }
    let ok = set_size(handle, data.len() as i64) == 0
        && write_file(handle, 0, data.as_ptr(), data.len() as u32, &1) == 0;
    close_file(handle);
    ok
}

// Accessed only by the board update on the game thread; the native footer copies the text.
static mut MESSAGE: [u8; 256] = [0; 256];

unsafe fn notice(message: &str) {
    let text = core::ptr::addr_of_mut!(MESSAGE).cast::<u8>();
    let len = message.len().min(255);
    core::ptr::copy_nonoverlapping(message.as_ptr(), text, len);
    text.add(len).write(0);
    info!("[threecats] selector: {}", message);
}

unsafe fn draw_notice() {
    let ui = r32(addr(0x1899458));
    let root = if ui != 0 { r32(ui + 328) } else { 0 };
    let children = if root != 0 { r32(root + 52) } else { 0 };
    let footer = if children != 0 { r32(children + 16) } else { 0 };
    if footer == 0 {
        return;
    }
    let text = core::ptr::addr_of!(MESSAGE).cast::<u8>();
    let current = r32(footer + 96);
    if current == 0
        || core::ffi::CStr::from_ptr(current as *const _) != core::ffi::CStr::from_ptr(text.cast())
    {
        call3(0x6431bc, footer, text as u32, 0);
    }
}

unsafe fn move_file(from: &str, to: &str) -> bool {
    let from = alloc::ffi::CString::new(from).unwrap();
    let to = alloc::ffi::CString::new(to).unwrap();
    call2(0x829b4c, from.as_ptr() as u32, to.as_ptr() as u32) != 0
}

fn save_index(index: i32) -> bool {
    if !forge::fs::savedata::mount() {
        return false;
    }
    let path = alloc::format!("{}:/three_palicos.ini", forge::fs::savedata::mount_point());
    let temp = alloc::format!("{}.tmp", path);
    let backup = alloc::format!("{}.bak", path);
    // Rename/delete expect files; reject directory collisions before touching either selection.
    for target in [&path, &backup] {
        let target = alloc::ffi::CString::new(target.as_str()).unwrap();
        let mut kind = 0;
        let result = unsafe { entry_type(&mut kind, target.as_ptr().cast()) };
        if (result == 0 && kind != 1) || (result != 0 && result != 0x202) {
            forge::fs::savedata::unmount();
            return false;
        }
    }
    let data = alloc::format!("[ThreePalicos]\nThirdPalicoIndex = {}\n", index);
    // MtFile's write path can abort on filesystem errors; use the SDK's checked results.
    let written = unsafe { write_temp(&temp, data.as_bytes()) } && read_index(&temp) == Some(index);
    // Commit the temporary file before RenameFile: the port journals new file contents.
    forge::fs::savedata::unmount();
    if !written || !forge::fs::savedata::mount() {
        return false;
    }
    let old = forge::fs::file_exists(&path);
    let clean_backup = !old
        || !forge::fs::file_exists(&backup)
        || unsafe {
            let b = alloc::ffi::CString::new(backup.as_str()).unwrap();
            call1(0x829a60, b.as_ptr() as u32) != 0
        };
    forge::fs::savedata::unmount();
    if !clean_backup || !forge::fs::savedata::mount() {
        return false;
    }
    let result = unsafe {
        if old && !move_file(&path, &backup) {
            false
        } else if move_file(&temp, &path) && read_index(&path) == Some(index) {
            true
        } else {
            if old && !forge::fs::file_exists(&path) {
                move_file(&backup, &path);
            }
            false
        }
    };
    forge::fs::savedata::unmount();
    result
}

#[forge::hook(offset = 0x748174)]
pub fn board_update(board: u32) -> u32 {
    unsafe {
        // Use the board's input sample: Forge's frame snapshot can lag a simultaneous A press.
        let pad = r32(addr(0x188c670));
        let combo = pad != 0 && r32(pad + 27636) & 0x404 == 0x404;
        let pressed = HELD.swap(combo as u32, Ordering::Relaxed) == 0 && combo;
        let game = r32(addr(0x188ca48));
        let net = r32(addr(0x18843f0));
        let players = r32(addr(0x1885b58));
        let mgr = r32(addr(0x1885468));
        let player = if players != 0 {
            call1(0x276278, players)
        } else {
            0
        };
        let allowed = game != 0
            && net != 0
            && mgr != 0
            && player != 0
            && call1(0x3f8678, game) != 0
            && r8(net + 1598) == 0
            && virtual1(player, 344) == 0
            && r8(players + 40028) <= 1
            && r32(board + 60) == 4
            && r32(board + 72) == 3
            && r32(board + 76) == 4;
        if !allowed {
            INSIDE.store(0, Ordering::Relaxed);
            return original!(board);
        }
        if INSIDE.swap(1, Ordering::Relaxed) == 0 {
            let index = load_index();
            CHOICE.store(index, Ordering::Relaxed);
            let p = if index >= 0 {
                roster(mgr, index as u32)
            } else {
                0
            };
            let label = if index < 0 {
                "已关闭"
            } else if p != 0 {
                name(p)
            } else {
                "不可用，请重新选择"
            };
            notice(&alloc::format!(
                "第三只：{}\nZL+按下右摇杆：设置或取消",
                label
            ));
        }
        if pressed {
            if let Some(slot) =
                rules::board_slot(r8(board + 189), r8(board + 190), r16(board + 192))
            {
                let index = r8(mgr + 80034 + slot);
                let p = roster(mgr, index);
                let next = if index as i32 == CHOICE.load(Ordering::Relaxed) {
                    -1
                } else {
                    index as i32
                };
                if next >= 0
                    && (p == 0 || r32(p + 768) & 0x20 == 0 || call2(0x25f34c, mgr, index) == 0)
                {
                    notice("不可选择：需已雇佣且待机\n不得重复前两只或主控猎喵（原选择保留）");
                } else if save_index(next) {
                    CHOICE.store(next, Ordering::Relaxed);
                    if next < 0 {
                        notice("已保存：第三只已关闭\n下次出发生效");
                    } else {
                        notice(&alloc::format!("第三只已保存：{}\n下次出发生效", name(p)));
                    }
                } else {
                    notice("保存失败，保留原选择\n请检查存档目录权限或可用空间");
                }
            }
        }
        // Consume this frame inside the list so A/other simultaneous keys cannot change native slots.
        if combo {
            draw_notice();
            return 1;
        }
        let result = original!(board);
        if r32(board + 76) == 4 {
            draw_notice();
        }
        result
    }
}
