# 期末簡報圖像與素材規劃

- 更新時間：`2026-03-16`
- 專案：`FinalProject_RustMiniOS`
- 用途：規劃每一頁簡報應該放什麼圖、照片、截圖、數據

## Slide 1. 題目與目標

### 建議放的圖

- 板子實拍正面
- 或桌面首頁的完整畫面

### 建議標題

- `Rust MiniOS on STM32F407`
- 或 `Mini Retro Computer on STM32F407`

### 要讓老師第一眼看到

- 這不是普通 LED / menu 作業
- 這是一台有桌面感的系統

## Slide 2. 硬體平台

### 建議放的圖

- 板子照片
- 一張簡單的硬體規格表

### 建議內容

- `STM32F407ZG`
- `1 MB Flash`
- `128 KB SRAM`
- `64 KB CCM RAM`
- `ILI9341 320x240`
- 電阻式觸控

## Slide 3. 系統總覽

### 建議放的圖

- 桌面首頁截圖
- 桌面上四個 icon 清楚可見

### 補充

- 左邊條列功能
- 右邊放首頁截圖

## Slide 4. 功能地圖

### 建議放的圖

- 一張簡單方塊圖或樹狀圖

### 建議內容

- `Desktop`
- `Album`
- `Game Center`
- `Pixel Paint`
- `Settings`
- `Diagnostics`
- `Performance / Benchmark`

### 可以怎麼畫

- 中間放 `MiniOS`
- 往外連各個 app 與系統頁

## Slide 5. Repo 架構

### 建議放的圖

- 一張 repo module diagram

### 建議內容

- `main`
- `shell`
- `ui`
- `apps`
- `dungeon`
- `storage`
- `media`

### 重點

- 要讓老師知道你有做模組化

## Slide 6. 主要技術挑戰

### 建議放的圖

- 左邊 `RAM / .bss` 問題
- 右邊 `Flash / media assets` 問題

### 建議形式

- 雙欄比較圖

## Slide 7. RAM 問題與解法

### 建議放的圖

- 一張簡單的記憶體配置圖
- `SRAM`、`CCM RAM`、`stack`、`viewport buffer`

### 要寫的重點

- stack 搬到 `CCM RAM`
- `Dungeon` 改成低解析 render + 放大

### 最好講一句

> 這不是單純修 bug，而是從記憶體架構上根治。

## Slide 8. Flash 與媒體

### 建議放的圖

- 一張 flash usage 簡圖
- 標出 `.text / .rodata / media`

### 要寫的重點

- 內建媒體吃掉大量 flash
- 因此 companion / 外部媒體是合理方向

## Slide 9. Graphics Showcase

### 建議放的圖

- `Graphics Lab`
- `Pseudo Racer`
- `Dungeon`

### 最佳形式

- 三張小截圖排成一列
- 每張下面一行標註技術點

### 建議標註

- `Graphics Lab`: math-heavy framebuffer effects
- `Pseudo Racer`: pseudo-3D viewport render
- `Dungeon`: 3D raycasting

## Slide 10. Gameplay Showcase

### 建議放的圖

- `Station Hunter` 的 `Profile`
- `Stage Select`
- `Battle`

### 重點

- 關卡 progression
- 每波升級
- 永久成長

## Slide 11. System Tools

### 建議放的圖

- `Diagnostics`
- `Performance Console`
- `Benchmark Results`

### 重點

- 這不只是展示畫面，也有工具頁
- 可以量化 app 與 render workload

## Slide 12. 驗證方式

### 建議放的圖

- 一張流程圖

### 建議內容

- host-side checks
- smoke checklist
- release build
- flash to board
- real board testing

## Slide 13. 目前成果

### 建議放的圖

- 一張功能總覽圖或拼圖

### 建議文字

- 有桌面
- 有 app
- 有媒體
- 有 2D / 3D
- 有 benchmark
- 有可維護結構

## Slide 14. 未來工作

### 建議放的圖

- 文字即可，不一定需要圖

### 建議內容

- 音效 / chiptune
- 更多 math-heavy effects
- 外部媒體 / companion
- 更完整 profiling

## Demo 頁額外素材建議

如果你還有時間補圖，我最建議補這 5 張：

1. 桌面首頁全畫面
2. `Graphics Lab` 最好看的一個 mode
3. `Pseudo Racer` 跑動中的一張
4. `Station Hunter` 的戰鬥畫面
5. `Benchmark` 結果頁

## 實拍優先順序

如果你只來得及拍少量照片，優先順序是：

1. 板子 + 桌面首頁
2. 板子 + `Graphics Lab`
3. 板子 + `Pseudo Racer`
4. 板子 + `Station Hunter`
5. 板子 + `Benchmark`
