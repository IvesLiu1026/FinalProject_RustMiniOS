# Pseudo-3D Racer 規格書

- 更新時間：`2026-03-16 05:35:42 CST`
- 專案：`FinalProject_RustMiniOS`
- 模組定位：`Game Center` 主打技術型遊戲
- 文件用途：定義一個 `asset-light, math-heavy` 的復古假 3D 賽車遊戲，作為後續完整實作與報告依據

## 1. 設計目標

這個遊戲的核心目的不是只做一個「好玩的賽車」，而是要作為整台 `MiniOS` 的技術 showcase。

它要展示的能力包含：

- 假 3D 透視與投影
- 曲線道路生成
- 速度感與場景流動
- 程序式道路與背景
- 低素材、高數學密度的畫面表現

它在整個專案中的技術定位應該是：

- `Dungeon`：3D raycasting / 場景數學
- `Station Hunter`：2D gameplay / state machine / progression
- `Album`：媒體前處理與顯示
- `Pseudo-3D Racer`：假 3D 透視與高速畫面更新

## 2. 高層產品定位

### 2.1 角色

玩家駕駛一台復古像素賽車，在帶有起伏與彎道的道路上衝刺，避開障礙、維持速度、完成分段賽程。

### 2.2 遊戲感受

應該走這種感覺：

- `OutRun`
- `Pole Position`
- `Road Rash` 的早期假 3D 味道
- 老電腦或老主機上的街機賽車感

### 2.3 視覺方向

- 高地平線
- 粗像素道路
- 誇張的白線節奏
- 少量 sprite 車輛與障礙物
- 強烈速度感
- 偏街機，不追求擬真

## 3. 核心玩法

### 3.1 基本玩法

- 左右控制車身位置
- 速度自動前進，玩家可控制加速與煞車
- 目標是在時間內完成賽段或抵達 checkpoint
- 撞到障礙會降速

### 3.2 玩法重點

這個遊戲最重要的不是複雜物理，而是：

- 彎道時的視覺透視
- 高速時的節奏感
- 路邊物件飛過去的速度感
- 在有限硬體下營造「很快」的錯覺

### 3.3 最小可玩 loop

1. 開始一局
2. 車輛自動往前
3. 玩家控制左右位置與速度
4. 避開障礙與其他車
5. 維持速度衝過 checkpoints
6. 完成賽段或時間耗盡

## 4. 遊戲模式規劃

### 4.1 建議第一版模式

- `Arcade Run`
  - 一條長賽道
  - checkpoint 制
  - 以距離與完成時間為主

### 4.2 第二版可加模式

- `Time Attack`
  - 固定賽段
  - 比最佳時間

- `Endless Cruise`
  - 無限道路
  - 比最遠距離

- `Traffic Trial`
  - 車流更密集
  - 比生存時間

## 5. 關卡與內容結構

### 5.1 賽道組成

賽道不建議用逐像素地圖，而是用 `segments` 組成。

每個 segment 包含：

- 長度
- 曲率
- 坡度
- 道路寬度
- 地表顏色
- 路邊裝飾類型

### 5.2 Segment-based track

推薦資料結構：

```rust
struct TrackSegment {
    length: u16,
    curve: i16,
    hill: i16,
    road_width: u16,
    roadside_theme: RoadsideTheme,
    lane_style: LaneStyle,
}
```

### 5.3 建議賽道主題

第一版可先做 `3` 條：

- `Seaside`
  - 淺藍天空、棕路肩、低難度彎道
- `Sunset Ridge`
  - 暖色天空、較多彎道與坡度
- `Night Circuit`
  - 深色背景、亮白路線與高對比障礙

## 6. 渲染架構

這個遊戲的關鍵在渲染方法。

### 6.1 核心策略

採 `scanline pseudo-3d road renderer`。

概念：

- 螢幕上每一條水平線代表道路上的不同深度
- 越靠近畫面下方的 scanline，代表距離玩家越近
- 每一條 scanline 根據透視、曲率、坡度，算出：
  - 道路中心點
  - 道路寬度
  - 路肩寬度
  - 車道線位置

### 6.2 分區

畫面分成：

- `Sky`
- `Horizon`
- `Road`
- `Roadside`
- `Sprites`
- `HUD`

### 6.3 為什麼適合 MCU

原因：

- 不需要真的建 3D mesh
- 不需要貼圖大量取樣
- 可以依 scanline 做遞推
- 可用固定點數學加速
- 幾乎不用大素材

## 7. 數學模型

### 7.1 深度與投影

每一條 scanline 對應一個假想的 `z` 距離。

基礎概念：

- `screen_y` 越低，`z` 越小，物件越大
- `screen_y` 越高，`z` 越大，物件越小

建議使用預先計算表：

- `z_for_scanline[y]`
- `road_half_width_for_scanline[y]`

這樣每 frame 不用重新做太多除法。

### 7.2 彎道

曲線可以由 segment 的 `curve` 累積偏移量來決定。

推薦做法：

- 每個 segment 給一個曲率值
- 對前方可視 segment 做累積
- 近處曲率影響較大，遠處較小

結果會形成：

- 地平線附近只有微小偏移
- 畫面下方會明顯彎曲

### 7.3 坡度

坡度可以影響：

- 地平線高度
- 各 scanline 的垂直對齊

可先用簡化版：

- hill 只調整道路上下位移
- 不做真正 3D elevation mesh

### 7.4 玩家車輛位置

玩家車本身不用真的進入 3D 空間，只要有：

- `lane_offset`
- `speed`
- `steering`

即可。

實際看起來的左右移動，是：

- 道路整體相對玩家平移
- 玩家 sprite 只做少量橫向擺動

## 8. 資產策略

### 8.1 原則

這個 app 必須維持 `asset-light`。

### 8.2 可接受的資產

- 玩家車 sprite：`1-3` 張
- 對手車 sprite：`2-4` 張
- 路邊裝飾 sprite：`tree / sign / cone / post`
- HUD icon：極少量

### 8.3 不建議

- 大尺寸背景圖
- 真實貼圖道路
- 多幀大型動畫
- 每條賽道都用大量獨立素材

### 8.4 建議方式

- 背景靠漸層與程序式天空
- 道路靠 scanline 直接畫
- 路邊 decor 用少量 sprite 重複利用

## 9. 道路與物件資料結構

### 9.1 建議資料結構

```rust
struct RacerState {
    speed: u16,
    track_pos: u32,
    lateral_offset: i16,
    timer_ms: u32,
    checkpoint_index: u8,
}

struct RoadObject {
    kind: RoadObjectKind,
    segment_index: u16,
    side: i8,
    lateral_offset: i16,
}
```

### 9.2 道路物件

建議第一版先做：

- 路標
- 樹
- 錐筒
- 其他車輛
- checkpoint gate

### 9.3 其他車輛

其他車輛只要有：

- 前方距離
- 左右 lane offset
- 速度

不需要複雜 AI。

## 10. 控制設計

### 10.1 板上按鍵

建議：

- `K0`：向左
- `WKUP`：向右
- `K1`：加速或確認
- `K0 + WKUP`：返回

### 10.2 觸控

建議：

- 左半邊：向左
- 右半邊：向右
- 底部中間：加速
- 可選小按鍵：煞車

### 10.3 車感

建議不是完全即時硬切，而是有：

- 轉向慣性
- 速度變化有平滑
- 撞擊時有瞬間偏移與減速

## 11. HUD 規格

畫面上建議顯示：

- 速度
- 剩餘時間
- 當前 checkpoint
- 賽道名
- 最佳紀錄

HUD 要走復古儀表板風格：

- 大字數位速度
- 小條狀 timer
- 角落小指示燈

## 12. 視覺回饋

### 12.1 速度感

靠這幾個元素做：

- 路線節奏加快
- 路肩條紋快速流動
- 對手車和路邊物件縮放加快
- 車體左右晃動

### 12.2 撞擊感

- 短暫白閃或紅閃
- 車速掉一截
- 車體小幅偏移

### 12.3 Checkpoint

- 大型門架
- HUD 閃字
- 短暫時間增加提示

## 13. 效能與記憶體策略

### 13.1 原則

不要新增 full-screen framebuffer。

### 13.2 推薦方法

- scanline 直接畫
- 或用低解析 line buffer
- 少量 sprite 疊加
- 少用浮點除法
- 多用 lookup table

### 13.3 最值得優化的點

- `z -> road width` lookup
- `curve accumulation` 預先展開
- `sin/cos LUT`
- sprite scaling 限制級數，而不是逐像素精細縮放

### 13.4 預期資源消耗

這個 app 應該：

- Flash 增加中等
- RAM 增加低到中等
- CPU 負載高，但可接受

## 14. 建議的技術分層

建議模組：

- `src/apps/racer.rs`
- `src/apps/racer/state.rs`
- `src/apps/racer/update.rs`
- `src/apps/racer/render.rs`
- `src/apps/racer/track.rs`
- `src/apps/racer/math.rs`

### 14.1 `track.rs`

負責：

- segment 定義
- 賽道資料
- 關卡主題

### 14.2 `math.rs`

負責：

- 投影
- lookup tables
- curve accumulation
- fixed-point helper

### 14.3 `render.rs`

負責：

- sky
- road scanlines
- sprites
- HUD

### 14.4 `update.rs`

負責：

- 玩家輸入
- 速度
- 撞擊
- checkpoint
- traffic 更新

## 15. 遊戲狀態流程

建議狀態：

- `Title`
- `Track Select`
- `Countdown`
- `Race`
- `Checkpoint`
- `Finish`
- `Game Over`

## 16. 第一版內容範圍

### 必做

- 一條可玩的假 3D 賽道
- 左右控制
- 自動前進與速度感
- checkpoint 與 timer
- 基本障礙物
- 結算畫面

### 第二優先

- 多賽道
- 對手車輛
- 坡度
- 夜間主題

### 後續加值

- 追逐模式
- 難度模式
- 最佳時間存檔

## 17. 驗證與測試計畫

### 功能測試

- 能正常啟動、返回、重新開始
- 左右控制穩定
- checkpoint 計時正確
- 結算條件正確

### 畫面測試

- 路線不抖動
- 彎道連續自然
- 速度高時仍可辨識
- HUD 不遮擋關鍵視覺

### 效能測試

- 不新增危險 `.bss`
- 長時間跑不黑屏
- 不因為 sprite 或彎道複雜度導致明顯卡頓

## 18. 推薦實作順序

### Phase A

- 建立 app skeleton
- 做 title / track select / basic state flow

### Phase B

- 做 road scanline renderer
- 先讓道路能動起來

### Phase C

- 加玩家左右控制與速度
- 加 checkpoint / timer

### Phase D

- 加路邊物件與障礙
- 加撞擊與減速

### Phase E

- 加對手車與賽道主題
- 補 HUD / polish / best time 存檔

## 19. 最終完成標準

完成版的 `Pseudo-3D Racer` 至少應該做到：

- 一眼就能感受到假 3D 速度感
- 幾乎不靠大素材
- 透視與彎道效果穩定
- 玩法可完整開始到結束
- 可作為課程展示時的「數學與渲染炫技模組」

## 20. 名稱建議

如果之後要正式命名，我推薦：

- `Roadline 95`
- `Turbo Vector`
- `Pixel Highway`
- `Retro Glide`
- `Lane Runner`

最推薦的方向是：

`Turbo Vector`

理由：

- 很有數學與速度感
- 符合復古電腦世界觀
- 跟 `Station Hunter` 並列時也很像正式作品名
