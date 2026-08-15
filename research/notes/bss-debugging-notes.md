# BSS 滿載問題排查紀錄

這份筆記整理 `FinalProject_RustMiniOS` 在 `STM32F407ZG` 上遇到的 `.bss` / RAM 壓力問題，包含症狀、調試心路歷程、量測方法、根因，以及最後採用的解法。這份內容可以直接作為 `微算機與實驗` 的專案工程紀錄。

## 1. 專案背景

- MCU: `STM32F407ZG`
- 內部 SRAM: `128 KB`
- 額外 `CCM RAM`: `64 KB`
- 螢幕: `320x240 ILI9341`
- 專案型態: `Rust no_std` bare-metal MiniOS + dungeon + Game Center

這個專案本來就不只是 menu demo，而是同時包含：

- 桌面式 launcher
- Album
- Game Center
- Pixel Paint
- 3D dungeon
- 觸控校正與控制頁

也因為功能越來越完整，記憶體壓力開始變成真正的系統級問題。

## 2. 問題症狀

某一版加入更多功能與資產後，板子燒錄成功，但開機有時候會直接黑屏。

一開始很容易以為是：

- LCD 初始化壞掉
- OpenOCD 沒有真的燒進去
- 程式 HardFault
- 某個畫面 render 壞掉

但實際上都不是。

## 3. 第一個關鍵觀察：黑屏不等於 crash

當時用 debugger 看，CPU 並沒有掉進 HardFault，而是還在主迴圈裡正常跑，甚至停在 `wfi` 等待下一個 tick。這代表：

- 程式本身還活著
- 主迴圈沒有直接炸掉
- 問題更像是記憶體被互相踩壞，導致顯示資料或狀態異常

這是很重要的判斷點，因為它把問題從「功能 bug」導向「記憶體配置 bug」。

## 4. 怎麼量到問題

### 4.1 先看整體 size

用下面這個指令看程式大小：

```bash
cargo build --release
~/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-size \
  target/thumbv7em-none-eabihf/release/finalproject_rustminios
```

問題發生前，我們量到：

| 項目 | 數值 |
| --- | ---: |
| `text` | 約 `455,368` bytes |
| `data` | `16` bytes |
| `bss` | `129,364` bytes |

`STM32F407ZG` 的一般 SRAM 只有 `128 KB = 131,072 bytes`。  
也就是說，那時候 `.bss` 幾乎把一般 SRAM 吃光了，幾乎沒有空間留給 stack。

### 4.2 再看 stack 與 `.bss` 的相對位置

當時在 debugger 裡看到：

- `.bss` 結尾大約在 `0x2001f964`
- `MSP` 當下大約在 `0x2001f9c8`

中間只剩大約 `100` 多 bytes 的距離。

這表示：

- 全域靜態資料幾乎頂到 SRAM 上方
- 主 stack 又從 SRAM 高位址往下長
- 兩者已經非常接近，幾乎要互撞

在 bare-metal 系統上，這種事不會有作業系統幫你保護。  
只要函式呼叫深一點、區域變數多一點、某段流程暫時多吃一點 stack，就可能把 `.bss` 或畫面資料踩壞。

## 5. 根因是什麼

真正的 RAM 大戶不是 Album、不是 Auto Hunter，也不是 UI，而是 dungeon。

當時最大的靜態資料是 dungeon 的整塊 viewport buffer：

- 舊版 viewport 大小接近 `320 x 200`
- 格式是 `RGB565`
- 單獨就吃掉大約 `128,000 bytes`

如果再加上：

- `ZBUFFER`
- 其他遊戲狀態
- 觸控、UI、系統狀態
- 主 stack

就非常容易逼近上限。

## 6. 第一階段解法：先把 stack 搬走

第一個修正不是根治，但可以先把黑屏救回來。

我們在 [memory.x](./memory.x) 新增 `CCMRAM`，並把 stack 改放到 `CCM RAM`：

```ld
CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K

_stack_start = ORIGIN(CCMRAM) + LENGTH(CCMRAM);
_stack_end = ORIGIN(CCMRAM);
```

效果是：

- 一般 SRAM 留給 `.bss`
- stack 改由 `CCM RAM` 承擔
- 不再讓 stack 跟 dungeon buffer 擠同一塊記憶體

後來驗證時看到新的 stack 位址已經搬到 `0x1000f9c8` 左右，黑屏問題也立刻改善。

### 為什麼這不是根治

因為這只是把 stack 與 `.bss` 分開，並沒有減少 dungeon 自己對 RAM 的需求。  
如果後面再繼續加大型 buffer，SRAM 還是會再滿一次。

## 7. 第二階段解法：直接砍掉 dungeon 的 RAM 根因

根治方式是把 dungeon 改成：

- 內部先用較低解析度 render
- 最後再放大顯示到 `320x240`

這版的核心改動在：

- [src/dungeon.rs](./src/dungeon.rs)
- [src/display.rs](./src/display.rs)

關鍵做法：

1. dungeon 內部 viewport 改為半解析度
2. `VIEWPORT_BUFFER` 大小大幅縮小
3. `ZBUFFER` 也跟著縮小
4. 顯示端新增 `draw_rgb565_scaled(...)`，把低解析畫面整數倍放大回螢幕

這樣做的好處是：

- RAM 立刻下降很多
- 仍然保留復古像素感
- 顯示風格反而更接近這個專案的復古主題

## 8. 修正後的實測數據

目前版本重新量到：

```bash
~/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-size \
  target/thumbv7em-none-eabihf/release/finalproject_rustminios
```

結果是：

| 項目 | 數值 |
| --- | ---: |
| `text` | `490,984` bytes |
| `data` | `16` bytes |
| `bss` | `32,724` bytes |

這代表 `.bss` 從原本的 `129,364` bytes，大幅降到 `32,724` bytes。

### 主要符號位置

用下面這個指令可以看到目前關鍵記憶體符號：

```bash
~/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-nm -n \
  target/thumbv7em-none-eabihf/release/finalproject_rustminios | \
  rg "(_stack_start|_stack_end|__sbss|__ebss|VIEWPORT_BUFFER|ZBUFFER)"
```

目前觀察到：

- `_stack_end = 0x10000000`
- `_stack_start = 0x10010000`
- `__sbss = 0x20000010`
- `VIEWPORT_BUFFER = 0x20000014`
- `ZBUFFER = 0x20007d14`
- `__ebss = 0x20007fe4`

這代表：

- stack 已經確實搬到 `CCM RAM`
- `.bss` 只占用一般 SRAM 的前段
- SRAM 後段重新有了很大的安全空間

### 最大靜態記憶體大戶

用 `nm -S` 可以看到：

- `VIEWPORT_BUFFER` 大小約 `0x7d00 = 32,000 bytes`
- `ZBUFFER` 大小約 `0x280 = 640 bytes`

相比舊版 `VIEWPORT_BUFFER` 約 `128,000 bytes`，差距非常明顯。

## 9. 這次排查的心路歷程

這次其實很有代表性，因為它不是那種「哪一行打錯」的 bug，而是典型微算機專案會遇到的系統整合問題。

大致上的思路是：

1. 先確認板子是不是根本沒燒進去  
   結果 OpenOCD 顯示 `Verified OK`，所以不是 upload 失敗。

2. 再確認是不是直接 crash  
   debugger 顯示沒有掉進 HardFault，所以不是最直觀的崩潰。

3. 轉去看記憶體  
   `arm-none-eabi-size` 一看就發現 `.bss` 幾乎貼滿 SRAM。

4. 再看 stack 位置  
   發現 `MSP` 和 `.bss` 幾乎撞在一起。

5. 先把 stack 搬到 `CCM RAM`  
   讓系統先恢復穩定畫面。

6. 最後回頭處理真正的 RAM 大戶  
   把 dungeon 的 render buffer 改成低解析放大輸出，才算真正根治。

## 10. 這次學到的事

這個案例很適合放在 `微算機與實驗` 的報告裡，因為它反映了幾個很實際的嵌入式觀念：

- 黑屏不一定是程式 crash
- `build 成功` 不代表 `runtime` 一定安全
- `no_std` bare-metal 沒有人幫你保護 stack 與全域資料
- 大型畫面 buffer 往往是 MCU RAM 的真正殺手
- 有時候「換演算法 / 換 render 策略」比單純微調語法更重要
- `CCM RAM` 這種額外記憶體區塊，若善用配置，可以救很多系統整合問題

## 11. 目前的結論

這次問題目前可以這樣總結：

- 黑屏主因不是 LCD 初始化，也不是 HardFault
- 根因是 `.bss` 幾乎吃滿一般 SRAM，stack 與靜態資料太接近
- 短期解法是把 stack 搬到 `CCM RAM`
- 長期根治是把 dungeon 改成低解析渲染再放大輸出

目前 `Dungeon` 已經能穩定運作，而且留下了足夠的 RAM 空間，之後要再擴充 `Auto Hunter`、`Album`、`Pixel Paint` 都安全很多。
