# Graphics Lab 規格書

- 更新時間：`2026-03-16 05:35:42 CST`
- 專案：`FinalProject_RustMiniOS`
- 模組定位：`MiniOS` 技術展示型 app
- 文件用途：定義一個 `asset-light, math-heavy` 的 demo-scene / 圖學展示 app，作為後續完整實作與報告依據

## 1. 設計目標

`Graphics Lab` 的目的不是遊戲，而是純技術展示。

它要讓人一眼看到：

- 這塊 MCU 不只是在跑 UI
- 它可以做即時數學圖形效果
- 程式可以靠演算法與色彩，而不是大量素材，產生有衝擊力的畫面

它在整個 `MiniOS` 中的定位是：

- `Album`：媒體顯示能力
- `Dungeon`：3D 場景與互動
- `Station Hunter`：完整遊戲系統
- `Graphics Lab`：數學與圖學效果純展示

## 2. 核心原則

### 2.1 核心方向

- `asset-light`
- `math-heavy`
- `demo-scene`
- `high frame activity`
- `retro computer feel`

### 2.2 禁忌

不要把它做成：

- 又一個只是切換圖片的頁面
- 又一個普通小遊戲
- 高素材依賴的播放器

### 2.3 應該強調

- 固定點與 lookup table
- 幾何 / 變換 / 極座標
- scanline 或低解析內部 buffer
- palette cycling
- 程序式圖樣

## 3. 產品形式

### 3.1 App 入口

建議作為 `Graphics Lab` 獨立 app，放在：

- `Game Center`
  或
- 未來新增的 `Labs / Demos`

如果要維持桌面簡潔，我更推薦先放在 `Game Center`。

### 3.2 主流程

建議畫面流程：

1. `Graphics Lab Home`
2. `Effect Select`
3. `Effect Runtime`
4. `Mode Info / Parameter Overlay`
5. 返回選單

## 4. 最推薦的效果清單

### 4.1 Starfield

最適合作為第一個 mode。

展示重點：

- 深度投影
- 速度感
- 多層移動

優點：

- 實作簡單
- 幾乎零素材
- 很有「老電腦啟動畫面」感

### 4.2 Plasma

典型 demo scene 效果。

展示重點：

- `sin/cos LUT`
- palette cycling
- 程序式彩色場

優點：

- 視覺效果強
- 運算規律
- 很適合展示 lookup table 優化

### 4.3 Rotozoom

非常適合炫 affine transform。

展示重點：

- 旋轉
- 縮放
- 採樣
- 中心點控制

優點：

- 視覺衝擊強
- 很能展示數學變換

### 4.4 Tunnel

很有復古 demo 味道。

展示重點：

- 極座標
- 距離映射
- 流動感

優點：

- 看起來像高級技術效果
- 很少素材就能有強烈畫面

### 4.5 Wireframe 3D

展示點很學術，也很好講解。

展示重點：

- 旋轉矩陣
- 投影
- 線段裁切

優點：

- 很容易拿來報告數學
- 和 `Dungeon` 的技術點不同

### 4.6 Fire

經典像素火焰。

展示重點：

- 緩衝擴散
- 色盤映射
- 時間演化

優點：

- 很有視覺吸引力
- 很適合展示內部 buffer 操作

## 5. 為什麼是這六個

這六個效果剛好涵蓋不同數學面向：

- `Starfield`：透視與深度
- `Plasma`：週期函數與 palette
- `Rotozoom`：仿射變換
- `Tunnel`：極座標映射
- `Wireframe`：3D 投影與幾何
- `Fire`：程序式模擬與 diffusion

這樣做完之後，你可以很清楚地說：

這個 app 不是單一特效，而是一個小型圖學實驗室。

## 6. 效果完整規格

### 6.1 Starfield

#### 核心想法

維護一群星點，每個星點有：

- `x`
- `y`
- `z`

隨著時間向玩家移動。

#### 畫面公式

```text
screen_x = center_x + x / z
screen_y = center_y + y / z
```

#### 可調參數

- 星數量
- 速度
- 深度範圍
- 中心偏移

#### 額外變化

- 彩色星點
- 分層星點
- warp mode

### 6.2 Plasma

#### 核心想法

每個像素顏色由多個 `sin` 場組合決定。

範例：

```text
v = sin(x*a + t) + sin(y*b + t) + sin((x+y)*c + t)
```

#### 建議做法

- 不逐像素直接算 `libm`
- 盡量用 LUT
- 可以先在低解析 buffer 做，再放大

#### 可調參數

- 速度
- 色帶
- 波頻
- 模式數

### 6.3 Rotozoom

#### 核心想法

對一張小型程序式貼圖或棋盤圖做旋轉與縮放。

#### 資產策略

貼圖可以不是真圖，而是：

- checkerboard
- stripes
- radial pattern
- generated pattern

#### 展示重點

- affine transform
- 紋理取樣
- 定點數學優化

#### 可調參數

- rotation speed
- zoom speed
- pattern type

### 6.4 Tunnel

#### 核心想法

對畫面上的每個像素，計算：

- 到中心點距離
- 與中心點夾角

然後映射到一個環狀紋理座標。

#### 核心數學

- `r = sqrt(dx^2 + dy^2)`
- `theta = atan2(dy, dx)`

#### 展示重點

- 極座標轉換
- lookup
- 時間驅動位移

#### 可調參數

- 旋轉方向
- 吸入速度
- 紋理模式

### 6.5 Wireframe 3D

#### 核心想法

準備幾組 3D 點與線段關係，做：

- 旋轉
- 平移
- 投影
- 畫線

#### 幾何物件建議

- cube
- pyramid
- simple ship
- tunnel frame

#### 展示重點

- rotation matrix
- perspective projection
- line drawing

#### 可調參數

- object type
- auto rotate axis
- zoom

### 6.6 Fire

#### 核心想法

用一塊低解析 buffer，底部持續灌入熱量，再向上擴散。

#### 核心流程

1. 底部隨機高亮
2. 向上做平均或衰減
3. 用色盤映射成火焰顏色

#### 展示重點

- 程序式動畫
- 小 buffer 演化
- 調色盤映射

#### 可調參數

- fire intensity
- cooling
- palette theme

## 7. UI 規格

### 7.1 Home / Select 畫面

建議做成復古實驗室控制台：

- 左邊 mode list
- 右邊 preview / 說明
- 下方顯示數學關鍵字

### 7.2 Runtime 畫面

畫面幾乎全屏效果。

上方小條顯示：

- mode 名稱
- 參數摘要
- fps 或 frame time

### 7.3 Overlay

可切換顯示：

- mode 說明
- 使用的數學
- 參數值

## 8. 互動設計

### 8.1 建議按鍵

- `K0`：上一個 mode
- `WKUP`：下一個 mode
- `K1`：切換參數組或 overlay
- `K0 + WKUP`：返回

### 8.2 觸控

可做簡化：

- 左邊切 mode
- 右邊切參數
- 中間點一下切 overlay

## 9. 技術架構

建議模組：

- `src/apps/graphics_lab.rs`
- `src/apps/graphics_lab/modes.rs`
- `src/apps/graphics_lab/render.rs`
- `src/apps/graphics_lab/math.rs`
- `src/apps/graphics_lab/lut.rs`
- `src/apps/graphics_lab/buffer.rs`

## 10. 共用基礎設計

### 10.1 LUT

建議共用：

- `sin_lut`
- `cos_lut`
- angle wrapping helper

### 10.2 低解析 buffer

適合的 mode：

- plasma
- fire
- tunnel

可先在低解析 buffer 算，再用現有顯示放大。

### 10.3 直接渲染

適合的 mode：

- starfield
- wireframe

## 11. 效能策略

### 11.1 重要原則

- 不新增 full-screen framebuffer
- 多用低解析內部 buffer
- 多用 LUT
- 避免大量即時浮點除法

### 11.2 模式分級

可做 render strategy：

- `Quality`
- `Balanced`
- `Performance`

例如：

- Starfield：改星數
- Plasma：改內部解析度
- Wireframe：改線段數
- Fire：改 buffer 尺寸

## 12. RAM / Flash 預算

### 12.1 Flash

應盡量避免大素材。

這個 app 的 Flash 主要應該來自：

- 程式碼
- LUT
- 小型 palette

### 12.2 RAM

建議單一 mode 使用的小 buffer 限制在可控範圍，例如：

- `64x48`
- `80x60`
- `96x72`

而不是直接為每個 mode 建一個全屏 buffer。

## 13. 報告價值

這個 app 很適合用在課堂報告，因為每個 mode 都可以對應一種圖學概念。

### 可講解的主題

- 透視投影
- 三角函數與 LUT
- 仿射變換
- 極座標
- 程序式動畫
- palette cycling

## 14. 驗證與測試

### 功能測試

- 各 mode 能正確切換
- 返回不會卡住
- overlay 可切換

### 視覺測試

- 沒有明顯破圖
- 顏色層次可辨識
- 高速更新時不過度閃爍

### 效能測試

- 長時間運行不黑屏
- RAM 不暴增
- 切 mode 不出現殘影

## 15. 推薦實作順序

### Phase A

- 架好 `Graphics Lab` app shell
- mode select / runtime / back flow

### Phase B

- 先做 `Starfield`
- 再做 `Plasma`

### Phase C

- 加 `Rotozoom`
- 加 `Wireframe`

### Phase D

- 加 `Tunnel`
- 加 `Fire`

### Phase E

- 補 overlay、參數切換、效能模式

## 16. 完成標準

完整的 `Graphics Lab` 至少要做到：

- 有 `6` 個可切換 mode
- 每個 mode 都能展示不同數學概念
- 盡量不依賴大素材
- 可在板子上穩定運行
- 可直接作為課程 demo 與技術報告素材

## 17. 名稱建議

如果之後要正式命名，我推薦：

- `Graphics Lab`
- `Pixel Lab`
- `Math FX`
- `Demo Core`
- `Signal Room`

我最推薦的是：

`Graphics Lab`

原因：

- 直觀
- 很像復古電腦裡的內建工具
- 與 `MiniOS` 的系統感很搭
