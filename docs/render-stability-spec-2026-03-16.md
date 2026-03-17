# Render Stability Spec

Updated: 2026-03-16 (Asia/Taipei)

## Goal

根治 `Pseudo Racer` 與 `Graphics Lab` 的可見閃爍，讓這兩個 app 從「效果有做出來」提升到「展示時看起來穩、像成品」。

這一份 spec 的核心原則是：

- 不再依賴大量小型 `fill_rect` 直接刷 LCD
- 把高變動畫面改成「先組好一幀，再大區塊送出」
- 不把大型工作緩衝放在 stack，也不把危險的大陣列塞進 app struct
- 優先維持現有 `bss` 安全區間，大約以 `40 KB` 內為 guardrail

## Problem Statement

目前兩個 app 的閃爍不是單一 bug，而是顯示策略造成的：

- `Pseudo Racer`
  - 路面是 scanline 逐條算、逐條畫
  - 畫面內有大量小區塊 HUD/提示/車輛繪製
  - LCD 在同一幀內看到的是「一邊清、一邊補」，人眼就會看到掃描感或白閃
- `Graphics Lab`
  - 低解析畫面雖然有先算 frame，但放大顯示仍然是逐列複製與多次送出
  - mode runtime 每幀仍有大量文字框和資訊區跟著更新
  - 效果本身又是全畫面持續變動，所以閃爍更明顯

## Root Cause Summary

真正要處理的根因有 4 個：

1. LCD 傳輸策略不夠大塊
2. 動態視窗沒有真正獨立成 render target
3. frame pacing 還不夠穩
4. 一些 UI 與 runtime 區域仍然混在同一個重繪路徑

## Non-Goals

這一輪不做：

- 新增玩法
- 大改 UI 視覺風格
- 追求 30 FPS 以上
- 對所有 app 一次做全面 framebuffer 化

這一輪只專注在：

- `Graphics Lab` 先徹底穩定
- `Pseudo Racer` 先穩定最重的視窗
- 為後續 benchmark 頁提供可量測的 render model

## Design Direction

### Graphics Lab

`Graphics Lab` 是最適合先根治的一個，因為它天然就適合低解析 framebuffer。

建議固定改成：

- 內部 framebuffer: `64 x 48`
- 顏色格式: `RGB565`
- 顯示方式: `x5` 放大到 `320 x 240`
- framebuffer 大小: `64 * 48 * 2 = 6144 bytes`

這樣的好處：

- 正好整屏
- 不需要複雜裁切
- 每個 mode 共用同一塊 buffer
- 每幀只需要做一次「完整 frame -> batched blit」

### Pseudo Racer

`Pseudo Racer` 不需要整個 app 全畫面 framebuffer，先處理「賽道路面視窗」就夠了。

建議把道路主視窗改成：

- viewport 內部 framebuffer: `71 x 37`
- 顏色格式: `RGB565`
- 顯示方式: `x4`
- 對應實際 viewport: `284 x 148`
- framebuffer 大小: `71 * 37 * 2 = 5254 bytes`

這樣的好處：

- 剛好對齊現在的 `VIEW_W=284`、`VIEW_H=148`
- 不需要再做奇怪的插值或縮放裁切
- 路面 scanline、道路中心、rumble strip、lane stripe 都先畫到小 buffer
- LCD 最後只吃一次 `284 x 148` 的放大輸出

HUD、標題列、返回鍵、finish overlay 仍可以保留直接畫，但它們要和 road viewport 分開更新。

## Memory Budget

### Static Work Buffers

建議使用模組層級的靜態工作緩衝，不放進 `MiniOs` 或 app instance：

- `GRAPHICS_LAB_FB`: `6144 bytes`
- `PSEUDO_RACER_FB`: `5254 bytes`
- optional row/block helper buffer: reuse `Display` local static or module static

合計約：

- `11.4 KB`

這個量在目前 `bss ~ 33.5 KB` 的情況下是可以接受的，但仍要重新量一次 `arm-none-eabi-size`，確保沒有逼近危險區。

### Safety Rule

未來任何 framebuffer：

- 不放進 app struct
- 不放在 `main()` 的 stack frame
- 不在高頻函式裡開大陣列 local

## Phase Plan

### Phase A: Graphics Lab Root Fix

目標：先把 `Graphics Lab` 做到幾乎不閃。

步驟：

1. 新增 module-level `64x48 RGB565` framebuffer
2. 每個 mode 的輸出都改成寫 framebuffer，而不是混合直接畫 LCD
3. 保留 mode selection screen 現況
4. runtime screen 改成：
   - 靜態 shell chrome 只在 full redraw 時畫
   - framebuffer 每幀 batched upscale blit
   - bottom info bar 只在文字變動時重畫
   - info overlay 只在 toggle 時重畫
5. 固定 runtime target 在 `15 FPS`

完成標準：

- 任一 mode 進去後不會整面掃描閃
- mode 切換時只在切換瞬間 full redraw
- 長時間跑 `Plasma / Tunnel / Fire` 不白閃

### Phase B: Pseudo Racer Root Fix

目標：先穩住最重的道路視窗。

步驟：

1. 新增 module-level `71x37 RGB565` viewport buffer
2. road background、curves、lane stripe、ground stripe 全部先畫到 buffer
3. player car、traffic、checkpoint banner 有兩個選項：
   - v1: 先仍直接畫到 LCD，但只畫在 viewport blit 後
   - v2: 一起合成到小 viewport buffer
4. HUD 與 bottom prompt 保持 direct draw，但只在低頻事件更新
5. runtime target 先固定 `15 FPS`

完成標準：

- 賽道跑動時不再有整塊路面刷新的掃描感
- countdown、checkpoint、finish overlay 只在事件切換時 full redraw
- 轉向與速度感仍保持流暢，不因降幀嚴重變卡

### Phase C: Common Blit Layer Cleanup

目標：把 `Display` 的縮放輸出路徑穩定化。

步驟：

1. 保留 batched row/block write
2. 明確區分：
   - `draw_rgb565()`
   - `draw_rgb565_scaled()`
   - `draw_rgb565_scaled_bytes()`
3. 為放大輸出加上「每次送幾列 source rows」的固定策略
4. 避免在高頻呼叫中重複配置大 local buffer

## Render Pipeline Targets

### Graphics Lab Runtime Pipeline

1. update mode state
2. render into `64x48` framebuffer
3. upscale `x5` to LCD in batched blocks
4. redraw info strip only if changed
5. redraw overlay only if toggled

### Pseudo Racer Runtime Pipeline

1. update physics / countdown / checkpoint state
2. render road frame into `71x37` framebuffer
3. upscale `x4` into `284x148` road viewport
4. overlay sprites and HUD with minimal direct draw
5. only full redraw on:
   - track select -> run
   - run -> finish
   - finish -> retry
   - finish -> back to track select

## Frame Pacing

建議值：

- `Graphics Lab`: `15 FPS` target
- `Pseudo Racer`: `15 FPS` target

理由：

- 這塊板子與 LCD 之間，穩定低頻大塊輸出比高頻小塊輸出更像成品
- 展示時人眼更在意「穩」而不是絕對 fps

## Validation Plan

### Graphics Lab

- `Starfield`: 看星點是否平穩，不整面白閃
- `Plasma`: 看色場是否只是在動，不是整屏掃描
- `Rotozoom/Tunnel`: 看高速變化模式是否仍穩
- `Wireframe`: 看線條模式是否有殘影
- `Fire`: 看底部熱源區是否不會一直白刷

### Pseudo Racer

- 倒數 `3 2 1`
- 一般直路
- 高曲率路段
- checkpoint 閃提示
- off-road warning
- finish overlay

### Regression

- `bss` 仍在 guardrail 內
- 開機不白屏
- `Game Center` 少掉 `Tap Rush` 後版面仍正常
- `Showcase Mode` 切到 `Racer / Graphics Lab` 仍可展示

## Risks

### Risk 1: framebuffer 放錯位置

如果放進 app struct 或 stack，可能直接造成白屏或不穩。

應對：

- 只用 module-level static work buffer

### Risk 2: `bss` 回升太多

應對：

- 先加 `Graphics Lab` buffer
- 再加 `Racer` buffer
- 每一步都重新量 size

### Risk 3: overlay 又把閃爍帶回來

應對：

- runtime chrome 只畫一次
- event overlay 才 full redraw

## Recommended Implementation Order

1. `Graphics Lab` 先做 `64x48` framebuffer 版
2. 驗證 `Graphics Lab` 不閃
3. `Pseudo Racer` 做 `71x37` road viewport buffer
4. 驗證 `Pseudo Racer` 不閃
5. 最後再回頭微調 `Display` blit batching

## Success Criteria

完成後應該達到：

- `Graphics Lab` 進入 mode 後沒有瘋狂閃爍
- `Pseudo Racer` 跑動時沒有明顯整面掃描感
- `bss` 仍安全
- 板子能穩定重開、進 `Showcase Mode`
- 可以在報告裡清楚講出：
  - 為什麼一開始會閃
  - 為什麼 framebuffer 化是根治方向
  - 為什麼不能把大 buffer 亂放進 stack / struct
