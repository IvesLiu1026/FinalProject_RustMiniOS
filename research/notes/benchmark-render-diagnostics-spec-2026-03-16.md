# Benchmark and Render Diagnostics Spec

Updated: 2026-03-16 (Asia/Taipei)

## Goal

新增一個可以直接展示「這台板子現在跑得多重、多穩」的技術頁面，讓專案不只是看起來厲害，而是有數據可講。

這個頁面應該服務兩件事：

- 期末展示
- render pipeline 調校

## Positioning

這不是一般的 `Diagnostics` 重複版，而是偏向：

- benchmark
- render profiling
- app showcase metrics

可以叫：

- `Render Monitor`
- `Benchmark Lab`
- `Performance Console`

最推薦名稱：

- `Performance Console`

## What It Should Show

### System Summary

- current app
- theme
- language
- render strategy
- safe mode / normal mode

### Memory and Binary Summary

- `.text`
- `.data`
- `.bss`
- estimated flash headroom
- storage status

### Runtime Metrics

- current FPS
- average frame time
- worst frame time in last sample window
- full redraw count
- partial redraw count
- current runtime mode

### App-Specific Metrics

#### Dungeon

- render strategy
- viewport mode
- enemy count
- map id

#### Station Hunter

- stage
- wave
- enemy count
- projectile count
- current build size

#### Pseudo Racer

- track
- speed
- distance progress
- frame target
- road viewport mode

#### Graphics Lab

- current mode
- internal framebuffer size
- frame target
- runtime effect tag

## UX Layout

### Header

- `PERFORMANCE CONSOLE`
- short subtitle like `live runtime / memory / redraw`

### Left Column

- live runtime metrics
- current app info
- app-specific counters

### Right Column

- flash / bss / storage / build profile
- redraw mode
- benchmark presets

### Footer

- key hints
- `K1 RUN TEST`
- `K0/WK SWITCH PANEL`

## Benchmark Modes

建議做 3 種預設 benchmark，而不是自由輸入。

### 1. Idle Desktop

測桌面和 shell 本身的穩定度。

### 2. Racer Stress

直接開 `Pseudo Racer` 自動駕駛，測持續動畫。

### 3. Graphics Stress

直接跑 `Graphics Lab` 的高壓 mode，建議預設：

- `Plasma`
- `Tunnel`
- `Fire`

## Measurement Model

### Frame Time Sampling

以固定長度 sample window 量測：

- `64` frames moving average
- `worst frame in current window`

### Redraw Counters

每個 app 更新時記錄：

- `full redraw +1`
- `partial redraw +1`

每次切換 benchmark scene 時重置 window。

### Static Build Metadata

透過 `build.rs` 或 compile-time constants 顯示：

- target triple
- build profile
- git short sha
- build date
- text/data/bss snapshot

## Implementation Strategy

### Phase 1

先不做真 benchmark runner，只先做 live metrics panel。

內容：

- FPS
- current app
- text/data/bss
- redraw counters
- current render mode

### Phase 2

加入 benchmark presets：

- Desktop
- Racer
- Graphics Lab

可用按鍵啟動短時間測試，例如 `8-10 秒`。

### Phase 3

加入結果摘要：

- avg fps
- worst frame ms
- redraw count
- mode label

## Data Sources

### Existing Sources Already Available

- `fps_estimate`
- current screen / app
- render strategy
- storage inspect
- build metadata

### New Data To Add

- app-local redraw stats
- frame time sample buffer
- benchmark session state
- app-specific runtime counters for racer/lab

## Recommended First Version

第一版最值得先做這些：

- current app
- FPS
- `.text / .bss`
- current render path label
- redraw counters
- benchmark launch cards for:
  - `Pseudo Racer`
  - `Graphics Lab`

這樣就已經很有展示價值了。

## Why This Matters for the Course

這個頁面會讓你的專案更像工程作品，因為你可以直接講：

- 哪些 app 是 math-heavy
- 哪些 app 是 media-heavy
- 哪些 app 需要 partial redraw
- 哪些 app 需要 framebuffer
- Flash / RAM 是怎麼被控制的

這比單純說「我做了很多功能」更有說服力。

## Success Criteria

完成後應該能做到：

- demo 時可直接切進 performance page
- 清楚顯示目前 app 與 FPS
- 可以講出 `Racer / Graphics Lab` 的 render mode 差異
- 可以搭配 `Showcase Mode` 使用
- 可以作為報告中的量化證據
