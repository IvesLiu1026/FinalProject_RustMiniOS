# MiniOS 功能總報告

更新日期：2026-03-16

## 專案定位

`FinalProject_RustMiniOS` 是以 `STM32F407` 為核心的復古風 MiniOS。它不是單一 demo，而是一個包含桌面 shell、媒體系統、遊戲、存檔、診斷、benchmark 與展示模式的整合式嵌入式系統。

## 系統層功能

### 1. Retro Desktop / Shell

- 復古桌面首頁
- icon-based app launcher
- 統一 top bar / taskbar / back navigation
- `Settings / About / Diagnostics / Safe Mode / Performance / Benchmark` 等系統頁

### 2. Touch Calibration

- 開機進入觸控校準流程
- 支援校正結果保存
- factory reset 後可重新校準

### 3. Settings

- 語言切換
- 主題切換
- render strategy 切換
- diagnostics / performance / showcase mode 入口
- clear saves / factory reset

### 4. Persistence / Storage

- 使用 STM32 internal flash 保存系統設定與 app 資料
- 保存項目包含：
  - theme
  - language
  - render strategy
  - touch calibration
  - Album 狀態
  - Paint 畫布
  - Station Hunter progression
  - Pseudo Racer best time

## 媒體系統

### 1. Album

`Album` 現在有三種內容路徑：

- `STILL`
  - 圖片先前處理成 `RGB565`
  - 板上直接顯示
- `MOTION`
  - GIF 先拆成 frame sequence
  - 再轉成 `RGB565 motion clips`
  - 板上播放 frame animation
- `JPEG`
  - 板上即時解碼
  - 使用 `TJpgDec`

### 2. Media Pipeline

- 原始素材放在 `assets/test_media/images` 與 `assets/test_media/gifs`
- `tools/process_test_media.sh` 會生成：
  - preview images
  - motion frames
  - firmware RGB565 assets
  - manifests
- `build.rs` 會把這些資產註冊進 firmware

## 遊戲功能

### 1. Station Hunter

這是目前最完整的 2D 主打遊戲。

特色：

- 角色站住才會自動射擊
- 5 個主關卡
- 永久角色成長
- 關卡解鎖
- 每波升級選擇
- 補血道具
- Boss wave
- 存檔保存 progression

展示重點：

- 2D 遊戲狀態機
- 關卡控制
- 存檔系統
- UI/HUD 與戰鬥節奏

### 2. Dungeon Core

這是本專案的 3D showcase。

特色：

- software raycasting
- textured walls
- pseudo-3D dungeon traversal
- HUD / minimap / overlays
- 多張地圖
- 射擊與敵人互動

展示重點：

- 3D 圖形數學
- viewport rendering
- memory optimization
- render pipeline 拆模組

### 3. Pseudo Racer

這是 pseudo-3D 賽車展示。

特色：

- pseudo-3D road rendering
- checkpoint / countdown / finish flow
- 速度感與碰撞減速
- best time 保存

展示重點：

- road projection
- fixed-size viewport buffer
- 低解析 render 再放大顯示

### 4. Graphics Lab

這是數學圖形效果展示區。

目前 mode：

- Starfield
- Plasma
- Rotozoom
- Tunnel
- Wireframe
- Fire

展示重點：

- framebuffer effects
- procedural graphics
- math-heavy rendering
- render stability optimization

### 5. Pixel Paint

- 像素畫板
- 顏色切換
- 畫布保存

## 診斷與效能工具

### 1. Diagnostics

- 系統狀態
- storage 狀態
- recent app
- media counts
- recovery actions
- touch IRQ counter
- button IRQ counter

### 2. Performance Console

- current focus app
- FPS
- render pipeline
- cadence
- flash / data / bss usage

### 3. Benchmark

目前 benchmark case：

- UI Fill
- RGB Blit
- Pseudo Racer Sample
- Graphics Lab Sample

結果頁提供：

- overall avg
- min fps
- score
- grade
- per-case summary

### 4. Showcase Mode

- 自動輪播主要展示場景
- 適合期末 demo 與展示模式

## 通訊與輸入

### 1. GPIO

- 按鍵
- LED
- touch bit-bang SPI

### 2. USART

- `Mac companion` 路線保留
- 可作為外部媒體或 host integration 基礎

### 3. EXTI

目前已實作：

- `Touch IRQ -> EXTI1`
- `WKUP -> EXTI0`
- `K1 / K0 -> EXTI9_5`

設計原則：

- ISR 只做計數與事件旗標
- UI 邏輯留在主迴圈

## 工程成果

### 1. 記憶體優化

- 早期 `.bss` 幾乎打滿 SRAM
- 後來透過：
  - `stack -> CCMRAM`
  - dungeon viewport 改成較小 render target
  - 避免 app 任意開大 framebuffer
- 成功把系統拉回穩定範圍

### 2. 顯示穩定化

- `Graphics Lab` 走低解析 framebuffer 再放大
- `Pseudo Racer` 路面改成 buffered viewport
- 局部 redraw 與 cadence 控制降低閃爍

### 3. 模組化

- shell / ui / dungeon / apps / storage / media 已明確分層
- 較大型功能已拆出子模組

## 目前可展示的技術亮點

如果作為期末 demo，可強調以下幾點：

- Rust bare-metal embedded system
- FSMC LCD graphics pipeline
- 3D raycasting
- pseudo-3D racer
- procedural graphics lab
- flash persistence
- on-board JPEG decompression
- EXTI-based touch/button events
- diagnostics / benchmark / showcase mode

## 當前資源狀態

最新驗證版大約為：

- `text`: 767684
- `data`: 16
- `bss`: 46732

這代表：

- flash 已明顯使用不少，但仍未超出目前保留的程式空間
- bss 比早期危險時期健康得多，仍在安全範圍內

## 總結

本專案目前已經不是單一作業功能，而是一個具備：

- shell
- media
- 多種遊戲
- graphics showcase
- persistence
- diagnostics
- benchmark
- showcase mode

的完整嵌入式 MiniOS。從課程專案角度來看，已經能同時展示系統整合能力、圖形能力、輸入/儲存能力，以及嵌入式優化與工程判斷。
