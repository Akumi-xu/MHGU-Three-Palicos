# MHGU 单人三随从猫 MOD

面向 **MHGU v1.4.0 的 PC MHGURecomp**。单人猎人携带三只不同的已雇佣随从，第三只使用自己的装备、技能和成长记录。**第三只不结算采集、掠夺和鉴定物，前两只照常结算。**

[下载 v0.2.1 测试包](https://github.com/Akumi-xu/MHGU-Three-Palicos/releases/tag/v0.2.1)

## 安装

1. 退出游戏并备份存档。
2. 下载 Release 中的 ZIP，把其中的 `romfs` 文件夹解压到游戏目录的 `mods/THREE_PALICOS/`。
3. 最终路径应为 `mods/THREE_PALICOS/romfs/nativeNX/plugins/three_palicos.nro`。PC 移植版自行加载，无需 Switch exefs 加载器。
4. 在游戏内选好前两只随从。**默认已启用第三只，选取纸板第 4 只（编号 3），该猫必须待机且不与前两只重复。**

## 游戏内选择第三只

随从看板 → 选择随从 → 打开完整猫列表 → 高亮待机猫 → **ZL + 按下右摇杆**。

左下角显示第三只姓名和保存结果；对同一只再按一次即可关闭。**下次出发生效**，无需编辑配置。翻页、等级排序后，仍按实际高亮的猫选择。前两只照常在原版菜单里选，原版的两个槽位保持不变。排序、留言和确认对话框内不响应快捷键。

键盘按键随移植版绑定。默认 ZL 和右摇杆按下分别为 Tab 和 Alt，会与系统切换窗口冲突；键盘用户请先在按键设置里改绑其中一键，再使用组合键。

## 配置

编辑插件同目录的 `three_palicos.ini`：

```ini
[ThreePalicos]
ThirdPalicoIndex = 3
```

编号是当前角色的逻辑编号，从 0 开始，范围 0～83；`-1` 关闭。排序后的屏幕格子号不等于此编号，手动配置请查 `[threecats] roster 编号: 名字` 日志。默认编号 3 是初始手动排列中的第 4 只。

游戏内选择会保存到当前存档目录的 `three_palicos.ini`（通常为 `save/save_0`），优先于包内配置；此前配置另存 `.bak`。主文件缺失时可读取备份；主文件存在但格式无效时关闭第三只。**同一存档目录内的角色共用此配置**，更换角色或解雇/替换猫后请重新确认。任务中修改配置不会替换当前第三只。

出发时固定第三只身份，任务中改配置到下次出发生效。无效、未雇佣、非待机或重复选择会提示原因，并保持原版人数。原版菜单仍为两个选择槽，任务 HUD 显示三只猫的姓名和血量。不改变存档格式。

## 支持范围与验证

仅支持允许两只原版随从出战的离线单人猎人任务，包含离线单人集会所。联机、猎猫模式和禁止随从任务不额外生成。已测 PC 运行时 0.16.0；修改相同随从或结算函数的插件可能冲突。

已测：三只独立对象、真实攻击伤害、换区、第三只力尽恢复、任务成功/超时失败/放弃、连续出发、经验保存重载、无效/重复配置回退，以及第三只物品隔离且前两只物品数量保留。构建、重定位检查和规则测试通过。

**当前版本仍为测试包。**尚未完整验证全部支援动作（含新解锁的其他条件）、全部任务类型，以及联机/猎猫模式的实玩回归。安卓和 Switch 未测。请先使用副本存档；反馈时附上任务名、配置和相关日志。

## 卸载

回村保存并退出游戏，移除 `mods/THREE_PALICOS/`。若在存档目录另放过 `three_palicos.ini`，也可移除。任务中不要热卸载插件。

## 开发与参考

Rust + mhgu-forge 1.5.0；参考用户提供的 3DS MHXX 三猫金手指思路，重新定位 GU 逻辑，未沿用 3DS 地址。由 Codex（GPT-6）协助实现。发布包不包含游戏文件、存档或密钥。

## 从源码构建

本仓库包含 v0.2.1 的插件源码、默认配置、链接脚本、规则测试和重定位检查工具。游戏程序、游戏资源与本机调试存档不包含在仓库内。

工程路径须使用纯 ASCII 字符。需要 Rust nightly（含 rust-src）、Arm GNU Toolchain 的 `arm-none-eabi-gcc`、Python 3，以及 `cargo-mhgu-forge` 1.5.0。原构建环境的 Rust 为 `1.101.0-nightly (12433c307 2026-10-24)`；仓库中的 nightly 配置跟随已安装工具链，不保证未来 nightly 的兼容性。

在源码根目录执行：

```powershell
cargo install cargo-mhgu-forge --version 1.5.0 --locked
cargo forge build --release
python tools/check_relocations.py three_palicos.nro
python tools/check_formation.py target/armv7a-none-eabi/release/libthree_palicos.so
```

新增站位修复会保留原生地面高度返回寄存器，`check_formation.py` 必须 PASS；它使用同一工具链的 `arm-none-eabi-objdump`，可把完整路径作为脚本第二个参数。未来编译器若改变尾调用形式，应先检查产物，不能忽略失败。

须确保 `arm-none-eabi-gcc` 在 PATH 中，或用环境变量 `CARGO_TARGET_ARMV7A_NONE_EABI_LINKER` 指向它的完整路径。构建后将 `three_palicos.nro` 和 `three_palicos.ini` 放到安装说明中的插件目录。重定位检查必须为 PASS；它不代表功能已在安卓或 Switch 上验证。

配置解析和活动标记恢复测试（Windows）：

```powershell
rustc --test src/rules.rs -o rules_tests.exe
.\rules_tests.exe
```

`src/lib.rs` 为随从运行时逻辑，`src/selector.rs` 为纸板选择、提示与配置保存，`src/rules.rs` 为可独立测试的配置、光标边界与状态恢复规则。Hook 偏移针对上述游戏版本，移植到其他游戏版本前须重新核对。重定位检查脚本改编自 Forge PR #7（Lawrence-al），并补充 NRO 解析。

## 补充验证

2026-11-15 已完成第三只爆弹、笛类增益、回复支援与卸载读档检查，未发现需要修复的运行逻辑问题；成品 NRO 保持不变。测试条件、证据摘要与剩余范围见 [VALIDATION.md](VALIDATION.md)。

2026-11-15 v0.1.1 补充验证：第三只自然发动爆弹与笛类支援，专属新支援解锁正例和条件不符反例通过。新版本只调整版本标识和说明，运行逻辑未变。源码、构建配置与测试记录均随版本提供；验证细节及剩余范围见 [VALIDATION.md](VALIDATION.md)。

## v0.2.0 更新

新增纸板内第三只选择、取消、姓名提示、配置保存与备份恢复；修复开发测试中发现的目录占用导致保存崩溃，以及同时按 A 时误改原版槽位的问题。战斗、支援与结算逻辑沿用 v0.1.1。详见 [VALIDATION.md](VALIDATION.md)。

## v0.2.1：修复第二、第三只持续重叠

旧版第三只共用第二只的出生站位和普通跟随目标。现在为第三只安排独立的后方站位，并沿用原版移动和地面碰撞处理；前两只、支援及结算逻辑保持原有实现。选择方式不变，仍默认开启。

这是持续共用站位的修复；战斗中追向同一目标、通过窄路或刚换区时仍可能短暂相交，并未给随从增加互相阻挡的实体碰撞。

本轮侧重站位回归，未重新遍历全部支援、力尽恢复、任务结局与角色组合；具体证据和限制见 VALIDATION.md。
