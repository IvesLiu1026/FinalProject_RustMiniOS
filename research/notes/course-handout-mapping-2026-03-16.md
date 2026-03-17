# MCP2026 教材對照表

更新日期：2026-03-16

## 用途

這份文件用來對照本學期教材 PDF 與 `FinalProject_RustMiniOS` 專案目前的實作狀態，方便期末報告說明「哪些教材主題有被實作、哪些是部分涵蓋、哪些是延伸應用」。

## 專案判讀原則

- `完整涵蓋`：教材核心概念在本專案中有直接實作，且不是只有碰到表面功能。
- `部分涵蓋`：有使用相關主題，但實作型態與教材示範不完全相同。
- `延伸應用`：教材觀念有被用在更完整的系統或遊戲之中。

## PDF 對照

### 1. F4_3Dtransformation.pdf

狀態：`完整涵蓋（延伸應用）`

對應內容：

- `Dungeon Core` 使用 3D raycasting 與幾何投影概念，屬於立體場景與座標/視角變換的直接應用。
- `Pseudo Racer` 使用 pseudo-3D road projection、視角縮放與道路透視。
- `Graphics Lab` 的 `Wireframe`、`Tunnel`、`Rotozoom` 等 mode 也大量使用座標變換與數學圖形。

說明：

這份教材偏重 basis、transformation matrix、座標系轉換的數學基礎；本專案則把這些觀念做成了互動圖形與遊戲，因此屬於「教材核心概念的實作延伸」。

### 2. F4_FSMC_1.pdf

狀態：`完整涵蓋`

對應內容：

- 本專案 LCD 顯示鏈路走 `FSMC / 8080-style parallel`。
- TFT 驅動與 Rust 顯示橋接直接建立在 FSMC LCD path 上。

說明：

教材講的是 F4 板上 LCD / TFT 與 FSMC 之間的關係；本專案所有主要畫面、遊戲、相簿顯示，都是建立在這條硬體路徑之上。

### 3. F4_FlashProg_3.pdf

狀態：`完整涵蓋`

對應內容：

- 系統設定保存
- `Station Hunter` 永久成長與關卡解鎖
- `Pseudo Racer` best time
- `Paint` 畫布保存
- `Album` 狀態保存

說明：

教材中的 Flash erase / program / verify 流程，在本專案中被用來實作真正的 storage service，因此這一份屬於非常直接的對應。

### 4. F4_GIF_1-2.pdf

狀態：`部分涵蓋`

對應內容：

- `Album` 支援 GIF 類型內容展示。
- 專案有完整的 GIF 前處理流程，會先將 GIF 拆 frame，再轉成板子可用的 motion clips。

說明：

本專案目前不是在 STM32 板上即時做完整 GIF/LZW 解碼，而是採用工程上更穩定的作法：先在工具端前處理，再於板上播放 frame sequence。因此屬於「GIF 顯示應用與資產流程」，不是完整板上 GIF 解碼器。

### 5. F4_Gpio_Usart_Exti_2.pdf

狀態：`大部分涵蓋`

對應內容：

- `GPIO`：按鍵、LED、touch SPI bit-bang、控制腳位設定
- `USART`：`Mac companion` 連線架構
- `EXTI`：目前已補上 `Touch IRQ -> EXTI1` 與 `K1 / K0 / WKUP` 的 EXTI 路徑

說明：

這份教材原本比較偏 GPIO mode、alternate function、USART、外部中斷觀念。專案原本在 GPIO/USART 這塊就很完整，後來再補上 EXTI 後，這份教材的對應已經更完整。

### 6. F4_JPEG_1-1.pdf

狀態：`完整涵蓋（示範路徑）`

對應內容：

- `Album` 現在新增 `JPEG` tab。
- 使用 `TJpgDec` 進行板上即時 JPEG 解碼。

說明：

專案本來的 still image 路線是先前處理成 `RGB565`，這是為了穩定與效能；現在另外加入 `JPEG demo` 後，可以明確對應教材中的 `TJpgDec` 和 embedded JPEG decompression。

### 7. F4_nvic_SysTick_wizard_4.pdf

狀態：`完整涵蓋`

對應內容：

- `SysTick` 1 kHz system tick
- startup / reset / entry flow
- interrupt handling
- `EXTI` 與 `SysTick` 同時構成系統事件基礎

說明：

這份教材談的是 reset、vector table、NVIC、SysTick 與 boot-up sequence。本專案作為完整 MiniOS，這些基礎系統控制路徑都有實際被用到。

## 總結

若以目前專案狀態來看，本學期教材的涵蓋程度如下：

- `完整涵蓋`：3D Transformation、FSMC、Flash Programming、NVIC / SysTick、JPEG
- `大部分涵蓋`：GPIO / USART / EXTI
- `部分涵蓋`：GIF

## 報告建議說法

可以用下面這段作為口頭或書面摘要：

> 本專案不是把教材內容拆成獨立練習，而是把多數核心主題整合成一個可互動的 MiniOS。系統中直接實作了 3D 圖形數學、FSMC LCD 顯示、Flash 儲存、NVIC/SysTick、GPIO/USART/EXTI，並額外在 Album 中補上板上即時 JPEG 解碼。GIF 則採工程上更穩定的前處理加播放流程，因此屬於部分涵蓋但有完整媒體應用情境。

