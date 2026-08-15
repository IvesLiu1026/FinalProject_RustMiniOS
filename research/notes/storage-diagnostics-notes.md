# Storage / Flash Diagnostics Note

這份文件整理 `FinalProject_RustMiniOS` 目前的 storage 設計、flash 配置、save record 結構，以及 `Diagnostics` 頁面對應到的檢查邏輯。這份內容可以直接作為 `微算機與實驗` 的專案工程紀錄，特別適合說明：

- 為什麼要保留獨立 flash sector
- 為什麼目前只用單一 save record
- `Clear Save Data` 和 `Factory Reset` 的差別
- `Diagnostics` 頁面到底在檢查什麼

## 1. 設計目標

這個 MiniOS 已經不是單一遊戲，而是同時包含：

- 桌面式 launcher
- Album
- Pixel Paint
- Auto Hunter / Tap Rush / Dungeon Core
- Touch calibration
- 系統設定與 diagnostics

所以 storage 層需要同時解決兩件事：

1. 保存系統設定
2. 保存 app 狀態與遊戲資料

目前保存的內容包括：

- `theme`
- `language`
- `render strategy`
- `touch calibration`
- 最近開啟的 app
- Album 上次瀏覽位置
- Pixel Paint 畫布
- Auto Hunter 最高擊殺
- Tap Rush 最高分

## 2. Flash 版圖

目前 linker 配置在 [memory.x](./memory.x)：

```ld
FLASH   : ORIGIN = 0x08000000, LENGTH = 896K
STORAGE : ORIGIN = 0x080E0000, LENGTH = 128K
RAM     : ORIGIN = 0x20000000, LENGTH = 128K
CCMRAM  : ORIGIN = 0x10000000, LENGTH = 64K
```

對 `STM32F407ZG` 來說，這代表：

- `0x08000000 ~ 0x080DFFFF`
  - 程式本體使用
  - 對應 flash sectors `0` 到 `10`
- `0x080E0000 ~ 0x080FFFFF`
  - 保留給 storage
  - 對應最後一個 `128 KB` sector，也就是 sector `11`

### 為什麼要保留整個 128 KB sector

因為 `STM32F4` 的 flash 擦除粒度不是「我想改 552 bytes 就只擦 552 bytes」，而是要整個 sector 擦除。  
既然 sector `11` 本來就是 `128 KB`，最穩定的做法就是：

- 讓 app 程式只使用前面 `896 KB`
- 最後一個 sector 整塊留給 storage
- 避免程式碼與存檔彼此覆蓋

這種設計很適合課程專案，因為簡單、可預測、好 debug。

## 3. 目前的 Flash 使用摘要

以目前 release build 為例：

```text
text = 511056
data = 16
bss  = 32724
```

如果只看 flash 佔用，主要是 `text + data`：

- app flash budget: `896 KB = 917,504 bytes`
- current flash use: `511,072 bytes`
- remaining app flash headroom: 約 `406,432 bytes`

storage 區本身：

- reserved sector size: `131,072 bytes`
- current record size: `552 bytes`
- record 只占保留 sector 的約 `0.42%`

### 為什麼只用了 552 bytes，卻要保留 128 KB

不是因為程式浪費，而是因為 flash controller 的擦除單位就是 sector。  
目前這個版本重視的是：

- 穩定
- 好理解
- 報告好說明
- 板上診斷容易做

如果未來要進一步優化 flash 壽命，才會考慮：

- 多 record 輪替
- wear leveling
- append-only log

## 4. Save Record 結構

目前 storage record 定義在 [src/storage.rs](./src/storage.rs)，總長度是 `552 bytes`。

### 4.1 Header 與系統設定

| Offset | Size | 欄位 | 說明 |
| --- | ---: | --- | --- |
| `0x000` | 4 | `MAGIC` | 固定為 `MOS2`，用來判斷這是不是有效紀錄 |
| `0x004` | 2 | `VERSION` | storage 格式版本 |
| `0x006` | 2 | `record_bytes` | 紀錄長度，現在是 `552` |
| `0x008` | 1 | `theme` | `Dark / Light` |
| `0x009` | 1 | `language_zh` | 是否為中文介面 |
| `0x00A` | 1 | `render_strategy` | `Quality / Balanced / Performance` |
| `0x00B` | 1 | `touch_ready` | 是否已有可用校正資料 |
| `0x00C` | 1 | `recent_app` | 最近啟動的 app |
| `0x00D` | 1 | `album_motion_tab` | 相簿上次停在 still 還是 motion |
| `0x00E` | 1 | `album_playing` | 動畫上次是否為播放狀態 |
| `0x00F` | 1 | `paint_selected_color` | 畫板目前選色 |
| `0x010` | 2 | `auto_battle_best_kills` | Auto Hunter 最佳擊殺 |
| `0x012` | 2 | `tap_rush_best_score` | Tap Rush 最佳分數 |
| `0x014` | 2 | `album_still_index` | Album still index |
| `0x016` | 2 | `album_motion_index` | Album motion index |

### 4.2 Touch calibration 區

| Offset | Size | 欄位 | 說明 |
| --- | ---: | --- | --- |
| `0x018` | 2 | `x_min` | 原始觸控最小值 |
| `0x01A` | 2 | `x_max` | 原始觸控最大值 |
| `0x01C` | 2 | `y_min` | 原始觸控最小值 |
| `0x01E` | 2 | `y_max` | 原始觸控最大值 |
| `0x020` | 1 | `swap_xy` | 是否交換 XY |
| `0x021` | 1 | `invert_x` | 是否反轉 X |
| `0x022` | 1 | `invert_y` | 是否反轉 Y |
| `0x023` | 1 | `valid` | 校正資料是否有效 |
| `0x024` | 1 | `affine` | 是否啟用 affine 校正 |
| `0x025` | 3 | reserved | 對齊保留 |
| `0x028` | 4 | `ax` | affine 係數 |
| `0x02C` | 4 | `bx` | affine 係數 |
| `0x030` | 4 | `cx` | affine 係數 |
| `0x034` | 4 | `ay` | affine 係數 |
| `0x038` | 4 | `by` | affine 係數 |
| `0x03C` | 4 | `cy` | affine 係數 |

### 4.3 Pixel Paint 畫布與尾端

| Offset | Size | 欄位 | 說明 |
| --- | ---: | --- | --- |
| `0x040` | `480` | `paint_pixels` | `24 x 20` 的低解析畫布 |
| `0x220` | 4 | reserved | 對齊保留 |
| `0x224` | 4 | checksum | FNV-1a checksum |

整份 record 結束於：

- `0x228 = 552 bytes`

## 5. 為什麼選擇單一 Record 設計

目前 storage 流程是：

1. encode 一份完整 state
2. 擦除 sector `11`
3. 重新寫入單一 record
4. 立刻 verify

這樣的優點是：

- 格式單純
- 問題容易定位
- `Diagnostics` 可以直接解讀
- 不需要在 bare-metal 專案裡再做 log compaction
- 很適合作為課程 demo 與報告內容

目前的缺點也很明確：

- 每次保存都要整個 sector erase
- 沒有 wear leveling
- 沒有雙備份 record

所以這是一個「為穩定與清楚說明而選的版本」，不是最進階的 storage 系統。

## 6. Diagnostics 頁面檢查了什麼

`Diagnostics` 的 storage 狀態來自 [src/storage.rs](./src/storage.rs#L119) 的 `inspect()`。

它目前會檢查：

- `found_magic`
  - flash 開頭是否為 `MOS2`
- `valid_record`
  - 是否能完整 decode 成目前版本的 record
- `checksum_ok`
  - checksum 是否正確
- `version`
  - 目前 record 版本
- `record_bytes`
  - record 長度
- `recent_app`
  - 最近開啟的 app
- `paint_pixels_used`
  - 畫板目前有多少格不是空白
- `has_app_saves`
  - app data 是否與預設值不同

所以頁面上的三種主要狀態可以這樣理解：

- `RECORD OK`
  - magic、version、length、checksum 都正常，而且可以 decode
- `CORRUPT DATA`
  - 看起來像是 storage record，但格式或 checksum 壞了
- `EMPTY`
  - 目前 flash sector 沒有有效 record，通常出現在 factory reset 後

## 7. Clear Save Data 與 Factory Reset 的差別

這兩個操作在 [src/main.rs](./src/main.rs#L744)。

### `Clear Save Data`

它會：

- 清空 app save data
- 重設 Album / Paint / Auto Hunter / Tap Rush 的執行期狀態
- 清空 recent app
- 重新寫回一份新的 storage record

它不會清掉：

- 主題設定
- 語言
- render strategy
- touch calibration

所以這個功能比較像：

- 清掉使用者內容
- 但保留系統環境

### `Factory Reset`

它會：

- 直接 erase 整個 storage sector
- 清掉系統設定
- 清掉 touch calibration
- 清掉 app save data
- 讓系統回到首次開機狀態

所以這個功能比較像：

- 整台 MiniOS 回到全新狀態

## 8. 為什麼要做二次確認

因為這兩個操作都是不可逆的 flash 動作。  
尤其 `Factory Reset` 之後，連觸控校正都會消失，如果只做單次點擊確認，太容易誤操作。

因此 Diagnostics 採用：

1. 第一次按下 `K1` 或點選按鈕
   - 進入 armed 狀態
   - 顯示 warning 文案
2. 第二次再按
   - 才真正執行 flash 動作

這樣也比較符合實際產品的 UX。

## 9. 寫入策略與 Flash 壽命考量

這版專案沒有採取「每一幀都自動存」，而是只在有意義的時機保存，例如：

- 切換 app
- 離開 Album
- 離開 Paint
- 分數刷新
- 完成 touch calibration
- 套用 settings

這樣做是因為：

- `STM32F4` 內建 flash 有擦寫壽命
- 畫板如果每畫一筆就寫 flash，壽命會掉很快
- 課程專案應該以可靠、易理解的策略為主

如果未來要升級 storage，可以考慮：

- dirty flag + 延遲提交
- 雙 record 備援
- append log
- sector rotation

## 10. 建議的實機驗證流程

### 驗證 1: Clear Save Data

1. 先進 `Album / Paint / Auto Hunter / Tap Rush` 留一些資料
2. 到 `Diagnostics`
3. 選 `Clear Save Data`
4. 按兩次確認
5. 檢查：
   - app 資料是否被清掉
   - theme / language / calibration 是否仍保留

### 驗證 2: Factory Reset

1. 到 `Diagnostics`
2. 選 `Factory Reset`
3. 按兩次確認
4. 檢查：
   - storage 狀態是否變成 `EMPTY`
   - 重新開機後是否回到 touch calibration

### 驗證 3: Build 與 size

```bash
cd ./
cargo build --release
~/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-size \
  target/thumbv7em-none-eabihf/release/finalproject_rustminios
```

## 11. 這個設計在課程報告裡怎麼說

如果要簡短說明，可以用這段：

> 我們把 MCU 最後一個 flash sector 獨立保留給 MiniOS storage，程式本體和存檔區分開。storage 採用單一 record 設計，保存系統設定、觸控校正、以及 app 狀態，並用 magic、version 和 checksum 做完整性檢查。雖然這種做法沒有 wear leveling，但架構簡單、除錯容易，非常適合裸機嵌入式課程專案的實作與驗證。

## 12. 後續可以怎麼擴充

如果之後要把 storage 做得更像正式產品，最適合的方向是：

- 做雙 record 備援
- 引入 sequence number
- 用 multiple slots 避免每次都擦整個 sector
- 把更多 app 的 save data 分欄位版本化
- 在 `Diagnostics` 顯示更多 flash metadata，例如最近寫入次數或 slot index
