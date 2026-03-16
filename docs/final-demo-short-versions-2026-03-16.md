# 期末 Demo 精簡版本

- 更新時間：`2026-03-16`
- 專案：`FinalProject_RustMiniOS`
- 用途：依現場時間快速切換 `3 分鐘 / 5 分鐘 / 7 分鐘` 講法

## 3 分鐘版

### 流程

1. 開機與桌面
2. `Graphics Lab`
3. `Pseudo Racer`
4. `Performance -> Benchmark`

### 口條

> 我這次做的不是單一功能 demo，而是一台跑在 `STM32F407ZG` 上的 `Rust bare-metal MiniOS`。  
> 這裡先看到的是復古桌面與系統殼層，代表這不是單一遊戲，而是一台小型電腦。

> `Graphics Lab` 這塊刻意走 `asset-light, math-heavy`，主要展示純數學與 framebuffer 類圖形效果。

> `Pseudo Racer` 則是 pseudo-3D 路面與速度感的展示，和 `Dungeon` 的 raycasting 是不同方向的圖形技術。

> 最後我直接在板子上跑 benchmark，量化 `UI Fill`、`RGB Blit`、`Pseudo Racer` 和 `Graphics Lab` 的平均與最低 FPS，證明這些效果不只是看起來炫，而是可量化的。

### 核心重點

- 這是一台 MiniOS，不是單一 app
- 我有數學圖形效果
- 我有 pseudo-3D
- 我有 benchmark

## 5 分鐘版

### 流程

1. 開機與桌面
2. `Album`
3. `Graphics Lab`
4. `Pseudo Racer`
5. `Station Hunter`
6. `Performance -> Benchmark`

### 口條

> 我這次想做的是一台迷你復古電腦，所以這個系統不只有桌面，也有媒體、遊戲、設定、診斷和 benchmark。

> `Album` 展示的是 still 和 motion clip 的媒體流程，不是單純把圖片塞進去。

> `Graphics Lab` 展示 asset-light, math-heavy 的數學圖形效果。

> `Pseudo Racer` 展示 pseudo-3D 路面、viewport buffer 和速度感。

> `Station Hunter` 是 2D 主打遊戲，裡面有每波升級、補血、Boss、關卡 progression 和關外永久成長。

> 最後用 `Performance Console` 和 `Benchmark` 收尾，讓這些效果可以直接被量化，而不是只靠主觀描述。

### 核心重點

- 系統感
- 媒體流程
- 數學圖形
- 2D 遊戲系統
- benchmark

## 7 分鐘版

### 流程

1. 開機與桌面
2. `Settings / Diagnostics / Performance`
3. `Album`
4. `Graphics Lab`
5. `Pseudo Racer`
6. `Station Hunter`
7. `Dungeon`
8. `Benchmark`

### 口條

> 這個專案的目標不是單一 menu 或單一遊戲，而是一台跑在 `STM32F407ZG` 上的 `Rust bare-metal MiniOS`。  
> 所以我把它做成有桌面、有 app、有遊戲、有設定、有診斷，甚至有 benchmark 的完整系統。

> 系統頁像 `Settings`、`Diagnostics`、`Performance` 是為了讓這台板子不只是功能集合，而是真的像一台可操作的復古電腦。

> `Album` 這塊展示媒體前處理和顯示管理。

> `Graphics Lab` 這塊展示純數學效果與 framebuffer。

> `Pseudo Racer` 這塊展示 pseudo-3D 與 viewport render。

> `Station Hunter` 展示 2D 遊戲機制、關卡 progression、Boss 和永久成長。

> `Dungeon` 展示 3D raycasting，也是我之前最需要處理記憶體壓力的地方。

> 最後用 `Benchmark` 收尾，可以直接看到平均 FPS、最低 FPS，以及這些 workload 在板子上的實際表現。

### 核心重點

- MiniOS 概念完整
- 多種技術面向都能展示
- 2D / 3D / 媒體 / 系統工具都有
- 有工程深度，也有量化驗證

## 現場選擇建議

- 時間很短：用 `3 分鐘版`
- 要兼顧完整度與穩定：用 `5 分鐘版`
- 老師願意多看、你想完整發揮：用 `7 分鐘版`

## 最後一句固定收尾

> 這個專案想證明的是，在 `STM32F407ZG` 這種資源受限的平台上，透過 `Rust no_std`、模組化架構、記憶體優化與 render 策略設計，還是可以做出一台真正有系統感、也有技術深度的 MiniOS。
