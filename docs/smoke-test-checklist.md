# Smoke Test Checklist

這份 checklist 是 `FinalProject_RustMiniOS` 目前的固定板上驗證流程，目標是讓每次做完結構整理、存檔功能、UI 調整或遊戲更新後，都能快速確認核心路徑沒有被破壞。

## Host checks

先在 Mac 上跑不需要板子的檢查：

```bash
(cd host_checks && cargo test)
```

這一組會檢查：

- `storage codec` 的 encode / decode round-trip
- checksum 損壞時是否正確拒絕讀取
- `app_registry` 的首頁與 Game Center 映射
- `media manifests` 是否和轉好的 firmware 素材一致

## Board smoke flow

### 1. 正常開機

- 上電後應看到 boot splash
- 若已有有效校正資料，系統應直接進 `Home`
- 若是第一次開機或剛做 `Factory Reset`，應進 `Touch Calibration`

### 2. Touch calibration

- 依序點完五個校正點
- 完成後應回到上一頁或 `Home`
- reset 後不應重新要求校正

### 3. Home launcher

- `K0 / WKUP / K1` 可以正常切換與開啟
- 觸控點首頁卡片可切換或進入 app
- `Album / Game Center / Pixel Paint / Settings` 都能打開

### 4. Album

- 預設 embedded 版 firmware 應直接顯示 still / motion 內容
- 若使用 `Mac companion` 版 firmware，先在 Mac 啟動 companion host
- companion 版 `Album` 左上來源 chip 應顯示 `MAC LINK`，未連線時應顯示等待提示
- 可以切換 `Still / Motion / JPEG`
- still 圖片能正常顯示
- motion clip 能播放與暫停
- `JPEG` tab 能正常進行板上即時解碼
- 返回 `Home` 後再進入，位置應保留

### 5. Pixel Paint

- 拖曳可畫圖
- 換色與清空功能正常
- 畫完後退出再進，畫布應保留

### 6. Auto Hunter

- 進場後玩家頭上應顯示 `HP / XP` 兩條狀態列，不應再有右上角大面板擋住怪物
- 移動時不會射擊，停下來會自動射最近敵人
- 打出 best kill 後退出再進，最佳擊殺應保留
- 升級三選一畫面應可正常選擇並回戰鬥

### 7. Pseudo Racer

- 倒數、跑動與 finish 切換穩定
- 畫面不應出現明顯白閃或整頁重刷
- best time 應正確保存

### 8. Graphics Lab

- 六個 mode 都可進入與返回
- 執行中畫面應穩定
- `BACK` 先回 mode 選單，再回 `Game Center`

### 9. Dungeon Core

- `Game Center -> Map Select -> Dungeon Core` 路徑正常
- 可切換地圖並進入遊戲
- HUD、武器切換、返回 `Map Select / Game Center` 都正常
- 畫面不應黑屏，返回路徑不應卡死

### 10. Diagnostics

- 可看到 FPS、touch 狀態、storage 狀態、最近 app
- 可看到 build version、git sha、media counts、boot mode
- 可看到 `BTN IRQ` 與 `TOUCH IRQ` 計數
- `Clear Save Data` 兩次確認後，app 存檔應清空
- 系統設定與觸控校正不應被 `Clear Save Data` 清掉

### 11. Performance / Benchmark

- `Performance Console` 文字不應重疊
- `FLASH / DATA / BSS / FPS / PIPELINE` 應正常顯示
- `BENCH` 可順利跑完 `UI Fill / RGB Blit / Pseudo Racer / Graphics Lab`
- 結果頁應顯示 `AVG / MIN / SCORE / GRADE`

### 12. About

- `Settings -> About` 可以正常進入與返回
- 應顯示 `version / git sha / build profile / target`
- 應顯示 media counts 與 storage bytes
- 應顯示安全模式提示文字

### 13. Safe Mode

- 開機時按住 `K1`
- boot splash 應顯示 safe mode requested
- 系統應進 `Safe Mode`
- 可以用按鍵選 `Home / Touch Calibration / Diagnostics`
- 從 `Safe Mode -> Diagnostics` 返回時，應回到 `Safe Mode`

### 14. Showcase Mode

- 可從 `Settings` 正常進入
- 能輪播 `Desktop / Album / Station Hunter / Pseudo Racer / Graphics Lab / Diagnostics`
- `K1` 可暫停/恢復，`K0/WKUP` 可切場景

### 15. Factory Reset

- 在 `Diagnostics` 觸發 `Factory Reset`
- reset 後應回到首次開機狀態
- 系統設定、app save data、recent app 都應清空
- 開機後應再次進入 touch calibration 流程

## Release validation commands

每次要交板上版本前，建議固定跑：

```bash
(cd host_checks && cargo test)
cargo build --release
./tools/arm-size.sh target/thumbv7em-none-eabihf/release/finalproject_rustminios
cargo run --release

(cd mac_companion && cargo build --release)
```

這樣可以把 `host-side invariants`、`release compile`、`size`、`flash/upload` 串成固定流程。
