# Auto Hunter 關卡與養成規格草案

- 更新時間：`2026-03-16 04:55:52 CST`
- 專案：`FinalProject_RustMiniOS`
- 模組：`Auto Hunter`
- 文件用途：整理下一階段 `Auto Hunter` 的關卡、養成、存檔與 UI 規格，作為後續實作依據

## 1. 設計目標

把目前的 `Auto Hunter` 從「單局小遊戲」提升成 `MiniOS` 裡的主打內容。

希望達成的體驗：

- 玩家不只是打一局，而是有長期進度
- 過關會真的解鎖新內容，而不是只有分數
- 關內有臨時 build，關外有永久成長
- 關卡節奏清楚，適合在 `STM32F407` 上穩定執行

## 2. 核心玩法定位

遊戲核心規則維持不變：

- 移動時不能攻擊
- 停下來時會自動射擊最近敵人
- 觸碰敵人會受傷
- 遠程怪物另外會用投射物施壓

這個規則是 `Auto Hunter` 最有辨識度的地方，不建議在下一階段把它改掉。

## 3. 總體結構

### 3.1 主關卡結構

- 總共 `5` 個主關卡
- 每一關包含 `30` 個 waves
- 每 `10` 個 waves` 出現一個 boss
- 也就是每關固定有：
  - `Wave 10 Boss`
  - `Wave 20 Boss`
  - `Wave 30 Boss`

### 3.2 解鎖規則

- `Stage 1` 一開始就開放
- 通過 `Stage N` 才能解鎖 `Stage N + 1`
- 已解鎖的舊關卡可以反覆挑戰

### 3.3 推薦遊玩時長

為了兼顧展示與實際體驗，建議每 wave 以 `15-25 秒` 為主。

這樣一關大約會落在：

- 普通熟練玩家：`6-9 分鐘`
- 新玩家：`8-12 分鐘`

## 4. 關卡節奏建議

每一關的內部節奏建議固定為：

1. `Wave 1-9`
   - 一般敵人波
   - 教玩家本關的主要威脅
2. `Wave 10`
   - Boss 1
3. `Wave 11-19`
   - 壓力波與混合怪
4. `Wave 20`
   - Boss 2
5. `Wave 21-29`
   - 高壓波，加入精英敵人或更密集組合
6. `Wave 30`
   - 最終 Boss

## 5. 五個主關卡方向

### Stage 1: 教學關

目標：

- 教玩家理解「移動保命，停下輸出」

主要敵人：

- `Runner`
- `Bruiser`

特性：

- 怪物數量適中
- 幾乎沒有遠程壓力
- Boss 可以做成單純直線衝撞型

### Stage 2: 遠程壓力關

目標：

- 讓玩家感受到「停下來輸出也有風險」

主要敵人：

- `Runner`
- `Shooter`
- `Bruiser`

特性：

- 開始有遠程彈幕
- Boss 可以做成固定節奏的扇形射擊

### Stage 3: 節奏破壞關

目標：

- 開始逼玩家不能長時間站樁

主要敵人：

- `Dasher`
- `Runner`
- `Shooter`

特性：

- 快速突進怪變多
- Boss 可以做成「蓄力後衝刺」型

### Stage 4: 場控關

目標：

- 讓場面逐漸變亂

主要敵人：

- `Summoner`
- `Shooter`
- `Bruiser`

特性：

- 召喚型敵人會拉高場面壓力
- Boss 可以做成會產生小怪或環形彈

### Stage 5: 綜合試煉關

目標：

- 當作整體 build 檢驗

主要敵人：

- 所有已出現敵人混合

特性：

- 波次切換更快
- Boss 可以是前面機制的混合體

## 6. 成長系統拆分

這一部分是本設計最重要的原則：

### 6.1 關內成長

關內成長只影響這一局。

觸發時機建議：

- 每次打完 boss 後跳出 `3 選 1`

代表類型：

- `多彈`
- `穿透`
- `攻速提升`
- `站樁增傷`
- `擊殺回血`
- `移動速度提升`

### 6.2 關外成長

關外成長是永久有效，會保留到下一次進關。

觸發時機建議：

- 通關一個主關卡後給永久經驗或成長點數
- 或者完成 boss 檢查點後給少量永久資源

代表類型：

- `基礎攻擊`
- `最大生命`
- `攻速`
- `移速`
- `初始護盾`

### 6.3 分離原則

一定要讓玩家一眼能分清楚：

- 這次升級是這一局有效
- 還是角色永久變強

如果兩者混在一起，玩家很容易搞不清楚進度感。

## 7. 主角色頁面規格

新增一個 `Hunter Profile` 或 `角色頁面`，作為 `Auto Hunter` 的關外入口。

建議顯示：

- 角色名稱
- 永久等級
- 基礎攻擊
- 最大生命
- 攻速
- 移速
- 已解鎖關卡
- 各關最佳紀錄
- 總通關次數
- 可用升級點數或永久經驗

建議用途：

- 玩家進關前先看自己的永久能力
- 玩家通關後在這裡做真正的升級
- 後續如果要加角色造型或徽章，也可以放在這裡

## 8. UI 流程建議

建議把目前的 `Auto Hunter` 入口流程改成：

1. `Game Center`
2. `Hunter Profile`
3. `Stage Select`
4. `Battle`
5. `Boss Reward`
6. `Stage Result`
7. 返回 `Hunter Profile` 或 `Stage Select`

這樣比直接從 `Game Center -> Battle` 更像完整遊戲。

## 9. 存檔資料建議

下一版 `Auto Hunter` 建議新增以下持久化資料：

- `player_level`
- `player_xp`
- `upgrade_points`
- `base_attack`
- `base_hp`
- `base_attack_speed`
- `base_move_speed`
- `unlocked_stage`
- `best_stage_cleared`
- `stage_best_kills[5]`
- `stage_best_wave[5]`
- `stage_clear_count[5]`

關內暫時資料則不需要永久保存，例如：

- 當局 buff
- 當局子彈數
- 當局血量
- 當局 wave

## 10. 關卡控制器建議

目前的 `Auto Hunter` 比較接近單一 arena 生怪模式。

下一版建議補一層 `Stage Controller`：

- 當前關卡編號
- 當前 wave 編號
- 當前 wave 類型
- boss 是否已登場
- 本 wave 剩餘敵人數
- 何時進入下一個 wave

建議 wave 類型：

- `Standard`
- `Pressure`
- `Elite`
- `Boss`
- `Reward`

這樣之後你要加新敵人、新 boss、新獎勵，不會把 `update` 流程越寫越亂。

## 11. Boss 建議

每個 stage 其實不一定需要完全不同的三隻 boss，也可以共用基底行為再換攻擊型態。

建議先做這幾種：

- `Ram Boss`
  - 慢速追蹤
  - 蓄力後直線衝撞
- `Burst Boss`
  - 定時朝玩家發射扇形彈
- `Nest Boss`
  - 會召喚小怪
- `Ring Boss`
  - 定時放環形彈

早期版本可以先做到：

- 每個 stage 1 隻最終 boss
- `Wave 10 / 20` 先用精英 mini-boss 替代

這樣會比一次做 15 種 boss 更實際。

## 12. 難度與硬體限制

為了配合目前板子的效能與畫面穩定性，建議控制：

- 同時敵人上限：`8-12`
- 同時投射物上限：`12-18`
- 不做太多大面積粒子特效
- 不新增 full-screen framebuffer
- 優先維持 dirty-rect 與 partial redraw

重點是用：

- 波次節奏
- 敵人組合
- boss 行為

來提升難度，而不是單純暴增怪物數量。

## 13. 推薦實作順序

### Phase A

- 擴充 `storage`，加入永久角色資料
- 定義 `HunterProfile` 存檔結構

### Phase B

- 做 `Hunter Profile` 頁面
- 做 `Stage Select` 頁面
- 加入關卡解鎖規則

### Phase C

- 實作 `Stage Controller`
- 把現有單局模式改成 `5 stage / 30 waves`

### Phase D

- 加入 `Boss wave`
- Boss 後跳出 `3 選 1` 關內 buff

### Phase E

- 做通關獎勵與永久升級
- 補更完整的戰鬥視覺回饋

## 14. 命名方向建議

如果你想替 `Auto Hunter` 改名字，我會建議從下面三個方向選：

### 方向 A：強調玩法機制

- `Station Hunter`
- `Standby Hunter`
- `定點獵手`
- `待機獵手`

這類名字的優點是能直接暗示：

- 移動時不能射
- 停下來才有輸出

### 方向 B：強調主打遊戲感

- `Hunter Core`
- `Hunter Breaker`
- `獵手核心`
- `獵手突圍`

這類名字比較像正式商品名，聽起來比較像主打遊戲。

### 方向 C：強調 MiniOS / 復古電腦風格

- `HUNTER.EXE`
- `PIXEL HUNTER`
- `獵手程式`
- `像素獵兵`

這類名字最適合你現在整台 `MiniOS` 的世界觀。

## 15. 名字推薦結論

如果你想要：

- 最像正式主打遊戲：
  - `Hunter Core / 獵手核心`
- 最符合玩法特色：
  - `Station Hunter / 定點獵手`
- 最符合這台復古 MiniOS 的世界觀：
  - `HUNTER.EXE / 獵手程式`

我個人目前最推薦的是：

`HUNTER.EXE`

原因：

- 很符合你現在整個 `MiniOS` 的復古桌面感
- 放在 `Game Center` 裡很有辨識度
- 未來做 `Hunter Profile`、`Stage Select`、`Save Data` 都很搭
- 比 `Auto Hunter` 更像正式作品名

如果你希望名字更直接表達玩法，那第二推薦是：

`Station Hunter / 定點獵手`
