# 期末 Demo Runbook

- 更新時間：`2026-03-16`
- 專案：`FinalProject_RustMiniOS`
- 用途：實機展示時的固定流程、節奏與備援方案

## Demo 目標

這場 demo 要讓老師快速看到 4 件事：

1. 這不是單一畫面，而是一台 `MiniOS`
2. 這台板子上同時有 `系統殼層 + 媒體 + 2D + 3D + benchmark`
3. 我真的處理過 `RAM / flash / render stability`
4. 這份專案有可持續擴充的架構

## 建議總長

- 建議總長：`5-7 分鐘`

## 最推薦的展示順序

1. 開機與桌面
2. `Album`
3. `Graphics Lab`
4. `Pseudo Racer`
5. `Station Hunter`
6. `Dungeon`
7. `Performance -> Benchmark`

這條順序的好處是：

- 先看到像電腦
- 再看到像產品
- 再看到技術炫技
- 最後用 benchmark 收尾

## 每段要講什麼

### 1. 開機與桌面

建議講：

> 我做的不是單一功能 demo，而是一台跑在 `STM32F407ZG` 上的 `Rust bare-metal MiniOS`。  
> 我想把這塊板子做成一台迷你復古電腦，所以它不只有桌面，也有 app、遊戲、設定、診斷和 benchmark。

### 2. Album

建議講：

> `Album` 主要展示的是媒體前處理與小螢幕顯示管理，包含 still 和 motion clip，而不是單純把圖片塞進去。

### 3. Graphics Lab

建議講：

> `Graphics Lab` 刻意走 `asset-light, math-heavy`，用純數學和 framebuffer 來做視覺效果，這塊比較像 demo-scene / graphics showcase。

### 4. Pseudo Racer

建議講：

> `Pseudo Racer` 用來展示 pseudo-3D 透視、路面 viewport buffer 和速度感。  
> 它跟 `Dungeon` 的 raycasting 是不同方向的圖形技術展示。

### 5. Station Hunter

建議講：

> `Station Hunter` 是 2D 主打遊戲，展示關卡 progression、每波升級、補血、Boss 和永久成長系統。

### 6. Dungeon

建議講：

> `Dungeon` 這塊展示的是 3D raycasting、場景、HUD，以及我如何把 RAM 壓力真的處理掉。

### 7. Performance / Benchmark

建議講：

> 最後我會直接在板子上跑 benchmark。  
> 這樣不是只說「跑得動」，而是直接看 `FPS / render path / flash / bss`。

## 展示前檢查

1. 板子能正常開機，觸控校正可完成
2. `Graphics Lab` 與 `Pseudo Racer` 畫面穩定
3. `Station Hunter` 能正常進 `Profile / Stage Select / Battle`
4. `Dungeon` 可正常進場，不黑屏
5. `Performance -> Benchmark` 能完整跑完 4 個測項
6. 各主要畫面 `BACK` 正常

## 備援版本

### 最短版：`3 分鐘`

1. 桌面
2. `Graphics Lab`
3. `Pseudo Racer`
4. `Performance -> Benchmark`

### 穩定版：`5 分鐘`

1. 桌面
2. `Album`
3. `Station Hunter`
4. `Dungeon`
5. `Benchmark`

## 收尾一句話

> 這個專案想證明的是，在 `STM32F407ZG` 這種資源受限的平台上，透過 `Rust no_std`、模組化架構、記憶體優化與 render 策略設計，還是能做出一台真正有系統感、也有技術深度的 MiniOS。
