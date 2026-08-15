# 期末前最後執行清單

- 更新時間：`2026-03-16`
- 專案：`FinalProject_RustMiniOS`
- 用途：上台前最後一次檢查、拍攝素材、整理簡報順序

## 1. 最終 QA 流程

### A. Host 端先確認

先跑：

```bash
cd ./host_checks
cargo test

cd ./
cargo build --release
~/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-size target/thumbv7em-none-eabihf/release/finalproject_rustminios
```

要確認：

- host checks 全過
- release build 成功
- `.text / .data / .bss` 數字可正常記錄到報告

### B. 板上核心路徑

上板後依序跑：

1. 開機、校準、回桌面
2. `Album -> Still / Motion / JPEG`
3. `Game Center -> Graphics Lab`
4. `Game Center -> Pseudo Racer`
5. `Game Center -> Station Hunter`
6. `Game Center -> Dungeon`
7. `Settings -> Diagnostics`
8. `Settings -> Performance`
9. `Settings -> Performance -> BENCH`
10. `Settings -> Showcase Mode`

### C. 必須特別檢查的重點

- `BACK` 在所有系統頁都能正常返回
- `Graphics Lab` 運行時畫面穩定
- `Pseudo Racer` 跑動時路面穩定
- `Album -> JPEG` 能正常板上即時解碼
- `Diagnostics` 的 `BTN IRQ / TOUCH IRQ` 計數正常變化
- `Benchmark` 能跑完四個 case，結果頁正常顯示

## 2. Demo 拍攝清單

### A. 必拍的 8 張圖

1. 板子 + 桌面首頁全畫面
2. `Album` 的 `JPEG` 畫面
3. `Graphics Lab` 最漂亮的一個 mode
4. `Pseudo Racer` 跑動中的畫面
5. `Station Hunter` 戰鬥畫面
6. `Dungeon` 3D 畫面
7. `Performance Console`
8. `Benchmark Results`

### B. 最值得錄成短影片的 4 段

1. 開機到桌面
2. `Graphics Lab` mode 切換
3. `Pseudo Racer` 倒數到跑動
4. `Station Hunter` 打怪、升級、Boss wave

### C. 拍攝小提醒

- 盡量正面拍，不要斜角太多
- 避免環境反光壓掉 LCD 畫面
- `Graphics Lab / Pseudo Racer` 建議在較暗環境拍
- 每段短影片控制在 `5-10 秒`

## 3. 簡報頁面順序

### 建議 12 頁版本

1. 題目與目標
2. 硬體平台
3. MiniOS 桌面與功能總覽
4. 系統架構與 repo 模組
5. 教材對照表
6. RAM / BSS 問題與根治
7. Flash / Media Pipeline / JPEG
8. Graphics Showcase
9. Gameplay Showcase
10. Diagnostics / Performance / Benchmark
11. 驗證方式與測試流程
12. 結論與未來工作

### 每頁最適合放的重點

#### 1. 題目與目標

- 板子實拍
- 桌面首頁

#### 2. 硬體平台

- `STM32F407ZG`
- `1 MB Flash / 128 KB SRAM / 64 KB CCM`
- `ILI9341 320x240`

#### 3. MiniOS 桌面與功能總覽

- icon desktop
- `Album / Game Center / Paint / Settings`

#### 4. 系統架構與 repo 模組

- `main / shell / ui / apps / dungeon / storage / media`

#### 5. 教材對照表

- `3D / FSMC / Flash / NVIC/SysTick / JPEG / EXTI`

#### 6. RAM / BSS 問題與根治

- stack 移到 `CCMRAM`
- dungeon render target 優化
- `Graphics Lab / Racer` render stability

#### 7. Flash / Media Pipeline / JPEG

- `RGB565 still`
- `GIF -> motion clips`
- `JPEG on-board decode`

#### 8. Graphics Showcase

- `Graphics Lab`
- `Pseudo Racer`
- `Dungeon`

#### 9. Gameplay Showcase

- `Station Hunter`
- progression / boss / upgrades

#### 10. Diagnostics / Performance / Benchmark

- `Performance Console`
- `Benchmark score / grade`
- `BTN IRQ / TOUCH IRQ`

#### 11. 驗證方式與測試流程

- host checks
- smoke test
- flash to board
- real hardware verification

#### 12. 結論與未來工作

- `MiniOS` 已整合系統、媒體、2D、3D、benchmark
- 未來可延伸音效、更多 demo-scene effects、外部媒體

## 4. 上台前 10 分鐘檢查

- 板子有正常供電
- 觸控校準可完成
- `Graphics Lab` 和 `Pseudo Racer` 畫面穩定
- `Album -> JPEG` 能正常展示
- `Benchmark` 跑得完
- 簡報裡的截圖與實際板上版本一致

## 5. 最推薦的展示順序

如果你只展示一輪，我最推薦：

1. 開機與桌面
2. `Album -> JPEG`
3. `Graphics Lab`
4. `Pseudo Racer`
5. `Station Hunter`
6. `Dungeon`
7. `Performance -> Benchmark`

這條順序最好講，因為：

- 先證明這是一台 MiniOS
- 再證明有媒體能力
- 再證明有數學圖形與 pseudo-3D
- 再展示 2D / 3D 遊戲
- 最後用 benchmark 收尾
