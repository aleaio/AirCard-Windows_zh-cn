# AirCard (Windows) 🎴

[简体中文](README_zh.md) | [English](README.md)

> **适用于 iOS 18+ 的 Apple 钱包卡面美化与锁屏密码键盘主题工具（免越狱）**  
> 使用 Rust 编写的 Windows 原生客户端，基于 `airlift` 的 AirTraffic 同步通道技术。

> [!NOTE]
> **关于本仓库：**  
> 本项目 Fork 自 [Lumid-Off/AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows)。本版本**未对原项目进行任何功能或逻辑修改，仅添加了中文字体内嵌与中英双语国际化汉化支持**，确保国内 Windows 用户开箱即用无乱码。

---

## 核心特性
- 🎨 **自定义钱包卡面：** 为 Apple Pay 银行卡和 Apple Cash 卡片替换个性化卡面图案、纹理或自定义银行 Logo。
- 🔢 **锁屏密码按键主题 (.passthm)：** 支持直接将热门工具 Cowabunga 与 Nugget 的 `.passthm` 锁屏按键素材写入 iOS 锁屏拨号键盘。
- ⚡ **100% 原生轻量：** 单一独立运行的 `aircard.exe`（内嵌中文字体后仅约 7.5 MB），无 Python、无 Flet、无 WebView、无臃肿运行环境。
- 🪟 **Material Design 3 风格界面：** 基于 `egui` 与 `eframe` 构建的现代清爽暗色主题界面。
- 📱 **零门槛识别卡片：** iPhone 连接电脑时，只需在手机“钱包”App 中点开对应卡片，即可通过系统日志（`syslog_relay`）秒级自动抓取并填入卡片 Hash。
- 🔄 **安全且可还原：** 具备完整的 Books 状态快照与自动恢复机制，保护设备原有状态不受损坏。
- 🚀 **完全免越狱：** 仅利用 Apple 内置的 AirTraffic 同步协议通道，不修改系统只读分区，不破坏系统安全沙盒。

---

## 环境要求
- **Windows 10 / 11 (64 位系统)**
- **Apple Mobile Device Support / 64 位 iTunes**（用于提供 Apple USB 设备通信驱动程序服务）。
- 标准 Lightning 或 USB-C 苹果数据线。

---

## ⚠️ 驱动修复与疑难排解（连接不上设备时）

> [!TIP]
> **未检测到 iPhone、AirTraffic 同步卡住或写入操作失败？**  
> Windows 系统中 Apple USB 驱动损坏或版本冲突是最主要的根本原因。
> 1. 下载并安装 **爱思助手 / [3uTools](https://www.3u.com/)**。
> 2. **断开 iPhone 与电脑的数据线连接**。
> 3. 打开爱思助手，进入 **工具箱 ➔ 修复驱动 (Repair Driver)**。
> 4. 点击 **立即修复** 并耐心等待 Apple 官方底层驱动重装完成。
> 5. 重新将解锁状态的 iPhone 插入电脑，屏幕弹出提示时轻触 **信任此电脑**，然后重新启动 **AirCard**。

---

## 下载与运行

### 预编译成品下载
1. 从 [Releases 页面](https://github.com/aleaio/AirCard-Windows_zh-cn/releases) 或 GitHub Actions 的 Artifacts 下载 **`aircard.exe`**；
2. 用数据线将 iPhone 连接至电脑，解锁屏幕并在手机上轻点 **“信任此电脑”**；
3. 双击运行 **`aircard.exe`**。

---

## 如何自定义 Apple 钱包卡面
1. 用数据线将已解锁的 iPhone 连接到电脑；
2. 打开 AirCard，停留在 **“钱包卡面 (Wallet)”** 标签页，点击 **“扫描卡片 (Scan)”**；
3. 在 iPhone 手机上操作：
   - 打开 **“钱包 (Wallet)”** App（或双击侧边电源键调出钱包）；
   - 点击进入你想要自定义卡面的卡片；
   - AirCard 会自动捕获并保存该卡片的凭据 Hash，识别完成后点击 **“停止 (Stop)”**；
4. 点击 **“选择图片... (Choose Image...)”** 选取你喜欢的图片（支持 PNG、JPG、WebP 格式，会自动居中裁切并缩放到 `1536 × 969`）；
5. 点击 **“写入卡面皮肤 (Apply Card Skin)”**；
6. 写入完成后，在 iPhone 屏幕底部向上轻扫进入多任务后台，向上滑动彻底关闭“钱包”App，重新打开钱包即可看到全新卡面！

---

## 如何应用锁屏密码按键主题 (.passthm)
1. 在 AirCard 中切换至 **“密码按键 (Passcode)”** 标签页；
2. 点击 **“选择 .passthm 文件... (Choose .passthm...)”** 导入你下载的 `.passthm` 主题包（支持来自 Cowabunga 或 Nugget 的格式）；
3. 根据当前已连接设备的 iOS 系统版本选择缓存格式：
   - **自动检测 (TelephonyUI-10)** — 适用于 iOS 18+（默认推荐）
   - **TelephonyUI-9** — 适用于 iOS 16 - 17
   - **TelephonyUI-8** — 适用于更早期的 iOS 版本
4. 点击 **“写入密码主题 (Apply Passcode Theme)”**；
5. 锁定 iPhone 屏幕并再次点亮，即可看到个性化密码拨号按键！

> [!IMPORTANT]
> **请务必关闭“粗体文本”功能：**  
> 在 iPhone 上进入 **“设置 ➔ 显示与亮度”**（或辅助功能），确保 **“粗体文本 (Bold Text)”** 保持处于 **关闭** 状态。如果开启了粗体文本，iOS 系统会忽略缓存的自定义按键图片，强制渲染系统内置矢量字体。

---

## 从源代码编译构建

前置条件：需安装 [Rust 工具链](https://rustup.rs/)（`stable-x86_64-pc-windows-msvc`）。

```powershell
# 克隆本仓库
git clone https://github.com/aleaio/AirCard-Windows.git
cd AirCard-Windows

# 运行测试
cargo test

# 编译发布版本（自动打包内嵌中文字体）
cargo build --release
```

编译出的可执行文件位于 `target\release\aircard.exe`。

---

## 贡献者与维护者
- **[@aleaio](https://github.com/aleaio)**（本汉化双语分支维护者）
- **[@Lumid-Off](https://github.com/Lumid-Off)**（Windows 原生 Rust 移植与维护者）— [GitHub](https://github.com/Lumid-Off) · [Twitter / X](https://x.com/LumidOff)
- **[@mak5er](https://github.com/mak5er)**（原始 macOS 应用与漏洞研究）— [GitHub](https://github.com/mak5er) · [Twitter / X](https://x.com/mak5er)
- **[AirLift](https://github.com/0xjohnnydev/airlift)** 作者 **[0xjohnny (@0xjohnnydev)](https://github.com/0xjohnnydev)**：底层 `AirliftFFI` 所依赖的原始 AirTraffic / ATAirlock 沙盒逃逸与概念验证代码。

## 鸣谢
- 核心 Exploit 机制基于 `airlift`（AirTraffic 同步逃逸通道）；
- 主题格式设计灵感源自 [Cowabunga](https://github.com/leminlimez/Cowabunga) 与 [Nugget](https://github.com/leminlimez/Nugget)；
- 内嵌中文字体采用开源字体 **[霞鹜新晰黑 (LXGW Neo XiHei)](https://github.com/lxgw/LxgwNeoXiHei)**。
