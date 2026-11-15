# 单人三随从猫 v0.3.0（测试版）

MHGU v1.4.0 / PC MHGURecomp 0.16.0。猎人或主控猎喵可携带三只不同的AI随从；猎喵模式共四只猫。第三只使用自己的装备、支援和成长，不结算采集、掠夺或鉴定物。前两只及主控猎喵由原版管理，不改变存档格式。

[下载v0.3.0测试版](https://github.com/Akumi-xu/MHGU-Three-Palicos/releases/tag/v0.3.0)。源码与安装包对应同一版本；后续更新仅在用户确认后发布。安装、默认配置、游戏内选猫及卸载见同目录纯文本“说明.txt”。默认ThirdPalicoIndex=3，-1关闭。第三只必须已雇佣、待机，不能重复前两只或当前主控猎喵。仅允许原版两只随从的离线单人任务启用；联机和禁止随从任务不启用。

游戏内：随从看板→选择随从→完整列表，高亮待机猫，按ZL＋按下右摇杆设置或取消，左下角提示，下次出发生效。前两只和主控猫继续使用原版菜单。同一存档目录角色共用配置，换角色或解雇猫后重新确认。默认键盘Tab＋Alt冲突，请先改绑。

## 自行构建

路径须为纯ASCII。需要Rust nightly及rust-src、Arm GNU Toolchain（arm-none-eabi-gcc及objdump）、Python 3和cargo-mhgu-forge 1.5.0。原构建Rust为1.101.0-nightly (12433c307 2026-10-24)；工具链文件跟随已安装nightly，不保证未来编译器兼容。依赖版本由Cargo.lock保留。

```powershell
cargo install cargo-mhgu-forge --version 1.5.0 --locked
cargo forge build --release
python tools/check_relocations.py three_palicos.nro
python tools/check_formation.py target/armv7a-none-eabi/release/libthree_palicos.so
rustc --test src/rules.rs -o rules_tests.exe
.\rules_tests.exe
```

确保arm-none-eabi-gcc及objdump在PATH，或设置CARGO_TARGET_ARMV7A_NONE_EABI_LINKER为gcc完整路径，并把objdump完整路径作为check_formation.py第二参数。两项产物检查必须PASS；重定位通过不代表实机功能已验证。地面函数还通过VFP寄存器返回高度，尾调用检查失败不可忽略。

将生成的three_palicos.nro和默认INI放到游戏mods/THREE_PALICOS/romfs/nativeNX/plugins/，同时安装说明.txt。覆盖前退出游戏并备份存档及旧MOD，避免同时加载两份插件。

## 源码与验证

src/lib.rs为生成、站位、成长和奖励处理；src/selector.rs为看板选择及配置保存；src/rules.rs为独立规则与测试。包含目标配置、链接脚本、依赖锁定、默认配置与检查工具。check_relocations.py改编自Forge PR #7（Lawrence-al），补充NRO解析。

测试范围与限制见VALIDATION.md。窄路、出入口及集火时模型仍可能相交；没有实体互斥。全部支援/任务结局、主控猎喵力尽流程、安卓与Switch尚未完整验证。

参考用户提供的3DS MHXX金手指思路，重新定位GU逻辑，未套用3DS地址。Rust / mhgu-forge 1.5.0，Codex协助实现。源码包不含游戏程序、资源、存档、密钥或反编译源码。
