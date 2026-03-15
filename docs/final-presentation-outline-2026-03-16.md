# 期末簡報與口頭大綱

- 更新時間：`2026-03-16`
- 專案：`FinalProject_RustMiniOS`

## Slide 1. 題目與目標

可以講：

- 我想做的不是單純 menu，而是一台小型復古電腦
- 平台是 `STM32F407ZG + 320x240 TFT`
- 用 `Rust bare-metal` 實作 MiniOS

一句話版本：

> 我這次想證明，在微算機平台上也可以做出有桌面、有 app、有遊戲、有設定與診斷工具的 MiniOS。

## Slide 2. 硬體平台

重點：

- `STM32F407ZG`
- `1 MB flash`
- `128 KB SRAM`
- `64 KB CCM RAM`
- `ILI9341 320x240`
- 電阻式觸控

可以補一句：

> 這個平台不大，所以每個 buffer、每個 asset、每個 redraw 策略都要很精打細算。

## Slide 3. 系統功能

秀目前功能：

- Home launcher
- Album
- Game Center
- Pixel Paint
- Settings / Diagnostics / Safe Mode
- Dungeon Core
- Auto Hunter
- Tap Rush

## Slide 4. 系統架構

講 repo 分層：

- `main`
- `shell`
- `ui`
- `apps`
- `dungeon`
- `storage`
- `media`

主軸：

> 我後來有花很多力氣做模組化，不然整個系統會很難維護。

## Slide 5. 最重要的技術挑戰

這頁建議直接講兩個：

1. `.bss` / RAM 滿載
2. flash 被媒體資產吃掉

## Slide 6. RAM 問題心路歷程

可以講：

- 板子一度黑屏
- 一開始以為是 LCD 壞掉或程式 crash
- debugger 發現 CPU 還活著
- 真正原因是 `.bss` 幾乎吃滿 SRAM，stack 快撞到全域資料

這頁很適合當老師會覺得你有真的 debug 過的證據。

## Slide 7. RAM 解法

講兩個關鍵：

- stack 搬去 `CCM RAM`
- dungeon 改成低解析 render + 放大顯示

重點是：

> 不是只修 bug，而是從記憶體架構上根治。

## Slide 8. Flash 問題與媒體資產

可以講：

- 分析後發現最大的 flash 大戶是 Album 媒體
- 不是純 code size 問題
- 所以後來有做 `Mac companion` 的方向研究

如果老師問最後為什麼板上版本沒走 companion：

> 因為展示穩定性優先，所以目前正式板上版還是保留 embedded Album；但 companion 路線和分析文件都已經做出來了。

## Slide 9. 遊戲與 UI 優化

這頁可以講：

- `Auto Hunter` 做了 partial redraw / dirty rect
- `Pixel Paint` 改局部重繪
- `Dungeon` 做模組拆分
- shell / settings / diagnostics / safe mode 都是統一 lifecycle

## Slide 10. 驗證方式

秀：

- host-side checks
- smoke checklist
- release build
- 實際上板測試

可以講：

> 我不是只看 compile 過沒過，而是把 host 驗證、size 量測、燒錄、板上流程都固定化。

## Slide 11. 目前成果

重點：

- 功能已完整到像一台小電腦
- 預設 embedded 版 `bss` 已回到安全範圍
- repo 結構健康
- 可以繼續擴充

## Slide 12. 未來工作

建議講這 4 個：

- 開機與桌面再 polish
- settings 做捲動
- 桌面改成 icon 風格
- 之後再考慮更完整的外部媒體路線

## Demo flow

上台實機展示時，建議順序：

1. 開機 splash
2. Home launcher
3. Album
4. Pixel Paint
5. Game Center -> Auto Hunter
6. Game Center -> Dungeon Core
7. Settings -> Diagnostics / About / Safe Mode

這個順序的好處是：

- 先讓老師看到「像電腦」
- 再看到「像產品」
- 最後再看到「真的有遊戲和系統能力」

## 問答時的核心回答模板

### 如果被問：為什麼用 Rust？

可以答：

> 因為這個專案越做越像系統，不只是單一程式，Rust 在模組化、型別安全和大型專案維護上很有幫助。

### 如果被問：最難的是什麼？

可以答：

> 最難的是記憶體問題，因為在 MCU 上沒有作業系統幫你隔離 stack 和全域資料，所以黑屏不一定是 crash，很多時候其實是記憶體互踩。

### 如果被問：這個作品的亮點是什麼？

可以答：

> 亮點不是單一遊戲，而是我把整塊板子做成一個有桌面、有 app、有遊戲、有設定、有診斷工具，而且可持續擴充的 MiniOS。
