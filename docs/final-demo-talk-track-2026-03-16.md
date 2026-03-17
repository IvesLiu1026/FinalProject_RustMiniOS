# 期末 Demo 口條稿

- 更新時間：`2026-03-16`
- 專案：`FinalProject_RustMiniOS`
- 用途：上台 demo 時可直接照著講的精簡講稿

## 開場

> 大家好，我這次做的不是單一功能 demo，而是一台跑在 `STM32F407ZG` 上的 `Rust bare-metal MiniOS`。  
> 我想把這塊板子做成一台迷你復古電腦，所以它不只有桌面和 app，也有媒體、遊戲、設定、診斷與 benchmark。

## 桌面與系統頁

> 這裡是系統桌面，我把它做成復古 GUI 風格。  
> 後面還有 `Settings`、`Diagnostics`、`Performance Console`、`Safe Mode` 等系統功能。

## Album

> `Album` 這塊主要展示的是 still / motion clip 的媒體流程，以及怎麼在小螢幕上穩定顯示。

## Graphics Lab

> `Graphics Lab` 刻意走 `asset-light, math-heavy`。  
> 裡面像 `Plasma`、`Tunnel`、`Wireframe` 這些效果，重點是數學、framebuffer 和 render pipeline。

## Pseudo Racer

> `Pseudo Racer` 的目標不是只做賽車，而是展示 pseudo-3D 路面、viewport buffer 和速度感。

## Station Hunter

> `Station Hunter` 是 2D 主打遊戲。  
> 它有關卡、每波升級、補血、Boss，還有關外的永久成長系統。

## Dungeon

> `Dungeon` 這塊展示 3D raycasting、場景與 HUD。  
> 這也是我之前最需要處理記憶體壓力的地方。

## Performance Console

> 這一頁會把目前 app、FPS、render path、flash、bss 直接顯示出來，所以不只是效果展示，也是技術監看。

## Benchmark

> 最後我會直接在板子上跑 benchmark。  
> 這裡有 `UI Fill`、`RGB Blit`、`Pseudo Racer Sample`、`Graphics Lab Sample`，所以可以直接量化這些 workload。

## 收尾

> 這個專案最想證明的是，在 `STM32F407ZG` 這種資源受限的平台上，透過 `Rust no_std`、模組化設計、記憶體優化和 render 策略，還是可以做出一台有桌面、有 app、有遊戲、有媒體、也有診斷與 benchmark 的 MiniOS。
