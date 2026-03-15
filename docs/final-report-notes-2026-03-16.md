# 期末報告筆記

- 更新時間：`2026-03-16`
- 課程：`微算機與實驗`
- 專案：`FinalProject_RustMiniOS`

## 1. 專案定位

這個專案不是單一功能 demo，而是一個跑在 `STM32F407ZG` 上的 `Rust bare-metal MiniOS`。  
目標是把一塊小型開發板做得像一台「迷你復古電腦」，而不是只有：

- 點 LED
- 顯示固定畫面
- 切換簡單 menu

目前系統整合了：

- 桌面式首頁 launcher
- `Album`
- `Game Center`
- `Pixel Paint`
- `Settings`
- `Diagnostics`
- `Safe Mode`
- `Dungeon Core`
- `Auto Hunter`
- `Tap Rush`

也就是說，本專案的重點是「在資源非常有限的 MCU 上，做出有系統感、可互動、可維護的 MiniOS」。

## 2. 硬體平台

| 項目 | 規格 |
| --- | --- |
| MCU | `STM32F407ZG` |
| Flash | `1 MB` |
| SRAM | `128 KB` |
| CCM RAM | `64 KB` |
| Display | `320x240 ILI9341` |
| Touch | 單點電阻式觸控 |
| 開發語言 | `Rust no_std` |
| 目標平台 | `thumbv7em-none-eabihf` |

### 實際記憶體配置

目前 linker 配置為：

- application flash：`896 KB`
- reserved storage：`128 KB`
- 主 stack 放在 `CCM RAM`

這樣做的目的是：

- 保留 flash sector 做設定與存檔
- 避免 stack 和主 SRAM 裡的大型 buffer 互撞

## 3. 系統架構

目前 repo 已經整理成比較健康的分層：

- `src/main.rs`
  - 最小化板級初始化與主迴圈
- `src/shell/`
  - MiniOS lifecycle、切頁、render orchestration、persist hooks
- `src/app_registry.rs`
  - app metadata 與 launcher mapping
- `src/ui/`
  - Home / Settings / Diagnostics / About / Safe Mode 等系統畫面
- `src/apps/`
  - Album、Paint、Tap Rush、Auto Hunter 等 app-style 模組
- `src/dungeon/`
  - 獨立的重量級遊戲子系統
- `src/storage.rs`
  - 系統設定與 app save data
- `src/media.rs`
  - Album 媒體索引

這種結構的好處是：

- 新增 app 不會一直把 `main.rs` 撐大
- shell、UI、遊戲、持久化比較不會糾纏在一起
- 之後做期末展示或繼續擴充時，比較容易維護

## 4. 主要功能

### 4.1 MiniOS shell

- 開機 splash
- 觸控校正
- 首頁 launcher
- 主題切換
- 中英切換
- About / Diagnostics
- Safe Mode
- 統一返回與導覽

### 4.2 Album

- 靜態圖片瀏覽
- GIF 轉成 motion clip 後播放
- 保留上次瀏覽位置
- 已建立媒體前處理流程

### 4.3 Game Center

- `Dungeon Core`
- `Auto Hunter`
- `Tap Rush`

### 4.4 Pixel Paint

- 低解析像素畫布
- 調色與清空
- 畫布持久化

## 5. 最重要的工程問題與解法

### 5.1 `.bss` / RAM 幾乎滿載

早期版本曾經出現燒錄成功但開機黑屏的問題。  
後來發現：

- 不是 LCD 初始化失敗
- 不是 OpenOCD 燒錄失敗
- 不是 HardFault
- 而是 `.bss` 幾乎吃滿主 SRAM，stack 與靜態資料互撞

這次問題的根因與修法已整理在：

- [docs/bss-debugging-notes.md](/Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/docs/bss-debugging-notes.md)

核心解法有兩層：

1. 把 stack 搬到 `CCM RAM`
2. 把 dungeon 改成低解析度 render 再放大顯示，根治大 viewport buffer 問題

### 5.2 Flash 佔用過大

後來另一個瓶頸變成 flash，特別是 Album 內建圖片與動圖。  
我們做過一次完整的 flash 分析，並實驗把 Album 改走 `Mac companion` 路線：

- [docs/flash-usage-analysis-2026-03-16.md](/Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/docs/flash-usage-analysis-2026-03-16.md)

這份分析的重點是：

- 真正最吃 flash 的通常不是單純程式碼
- 而是 `.rodata` 裡的大量媒體資產
- 把大型媒體搬出 firmware，是比微調 codegen 更有效的優化方法

目前正式板上版本已先回到 embedded Album，比較適合展示與課堂測試；`Mac companion` 仍然保留成可選路線。

### 5.3 畫面閃爍問題

在 `Album`、`Pixel Paint`、`Auto Hunter` 上，曾經遇到整頁重刷造成的閃爍。  
後來採用：

- partial redraw
- dirty rect
- 局部更新 panel / arena / 媒體區

這讓畫面更穩，也比較符合復古遊戲機的手感。

## 6. 驗證方式

### Host-side checks

目前 repo 有獨立的 host 驗證 crate：

- `host_checks/`

可以驗證：

- storage codec round-trip
- checksum 壞掉是否正確拒絕
- app registry 映射
- media manifests 與 firmware 素材是否一致

### Board-side smoke test

板上驗證流程整理在：

- [docs/smoke-test-checklist.md](/Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/docs/smoke-test-checklist.md)

包含：

- 開機
- 觸控校正
- Home launcher
- Album
- Pixel Paint
- Auto Hunter
- Dungeon Core
- Diagnostics
- Safe Mode
- Factory Reset

## 7. 目前成果

目前系統已經做到：

- 有完整開機流程
- 有桌面入口
- 有多個 app 與遊戲
- 有 persistent settings 與 app save data
- 有 diagnostics / about / safe mode
- 有可維護的模組結構
- 有 host-side checks 與 smoke checklist

預設 embedded 版目前量到：

```text
text = 521,432
data = 16
bss  = 33,560
```

這代表：

- flash 還有空間
- RAM 已經回到安全區
- 系統可以穩定在板上展示

## 8. 目前限制

雖然系統已經有完整 MiniOS 的樣子，但仍然有幾個限制：

- `Settings` 頁面目前偏擁擠
- 開機與桌面視覺還可以更像復古電腦
- Album 仍然比較像媒體展示器，而不是真正檔案系統
- `Dungeon` 和 UI 還有進一步 polish 空間
- 沒有真正的桌面 icon 化操作體驗

## 9. 下一步最值得做的方向

### 9.1 開機與桌面 polish

這是最有展示效果的一步。  
如果做得好，整個專案會從「很多功能的板子」變成「真的像一台小電腦」。

### 9.2 Settings 捲動化

當 app 和設定變多時，scrollable settings 會比硬擠在同一頁健康很多。

### 9.3 復古桌面 icon 化

把首頁從卡片 launcher 再往前推，做成：

- 桌面背景
- icon + label
- 狀態列或小型 HUD
- 像復古 GUI 的視覺語言

這會非常符合專案主題。

## 10. 一句話總結

這個專案證明了：  
在 `STM32F407ZG` 這種資源有限的平台上，透過 `Rust no_std + 結構化設計 + 記憶體優化 + 局部重繪`，可以做出一個真正有系統感、有遊戲、有相簿、有設定與診斷工具的 MiniOS，而不只是單一功能 demo。
