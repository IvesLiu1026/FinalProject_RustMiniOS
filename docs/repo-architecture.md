# Repo 架構與模組邊界

這份筆記整理 `FinalProject_RustMiniOS` 目前的 repo 結構，以及為什麼現在 `dungeon` 先不放進 `src/apps/`。

## 1. 目前的分層

- `src/main.rs`
  - 最小化的板級初始化與主迴圈，只負責把控制權交給 shell
- `src/shell.rs` + `src/shell/`
  - MiniOS shell 本體，已拆成 `update / render / persistence / calibration`
- `src/app_registry.rs`
  - app metadata、首頁/遊戲中心分組，以及共用 launch registry
- `src/ui.rs` + `src/ui/`
  - Home、Settings、Diagnostics、Control Room 等 shell UI，已拆成子畫面模組
- `src/apps/`
  - 較輕量的 app-style 模組，例如 `Album`、`Paint`、`Tap Rush`、`Auto Hunter`
- `src/dungeon/`
  - 較重的獨立子系統，包含 render pipeline、碰撞、資料、武器、更新邏輯
  - 其中 `render/` 已再拆成 `viewport / floor / sprites / controls / hud / effects / weapon`
- `src/storage.rs`
  - 系統設定與 app save data 的持久化層，儲存在保留 flash sector
- `src/system_info.rs`
  - About / Diagnostics / Safe Mode 共用的 build 與 runtime metadata
- `src/media.rs`
  - 由 build script 自動生成的媒體索引

## 2. 為什麼 `dungeon` 先不放進 `src/apps/`

目前我不建議把 `dungeon` 直接搬進 `src/apps/`，原因有三個。

### 2.1 它不是單純畫面 app，而是「子系統」

`Album`、`Paint`、`Tap Rush` 這類模組的責任比較單純：

- 收輸入
- 更新自己狀態
- render 自己畫面
- 回傳要不要退出

但 `dungeon` 現在還多了一整套比較像 engine 的東西：

- 3D 軟體 raycast render
- 深度與 sprite 排序
- 碰撞檢查
- 地圖 / enemy / pickup data
- 武器與 render strategy

它的內部結構明顯比一般 app 更重，所以先保留 top-level 比較清楚。

### 2.2 現在的 `src/apps/` 是「App 層」

`src/apps/` 目前比較像是：

- 使用 MiniOS shell 提供的生命周期
- 透過 `src/app_registry.rs` 掛進首頁或遊戲中心
- 盡量不外溢太多 engine 級別邏輯
- 方便後續再繼續加小遊戲或工具型 app

如果這時候把 `dungeon` 也塞進去，會讓 `apps/` 同時代表：

- 一般 app 容器
- 重型遊戲子系統容器

語意會開始混在一起。

### 2.3 先穩定邊界，比先追求目錄一致更重要

現在 `dungeon` 才剛完成 RAM 根治與第二輪 render 模組拆分，先讓它以獨立子系統穩定下來比較重要。  
如果太早搬目錄，只是把風險集中在檔案路徑和模組引用，不會帶來真正的可維護性提升。

## 3. 什麼時候適合搬進 `src/apps/`

等到系統再往下一步整理時，如果我們做出一個更正式的 app contract，例如：

- 共用 `AppId`
- 共用 `enter / update / render / exit`
- 共用 app registry / metadata
- shell 用同一種方式啟動所有 app

那時候可以考慮把 `dungeon` 改成：

- `src/apps/dungeon/mod.rs`
- `src/apps/dungeon/render.rs`
- `src/apps/dungeon/update.rs`

也就是把它視為「重量級 app」，而不是現在這樣的「獨立子系統」。

## 4. 目前最推薦的維護策略

現階段最穩的結構是：

- `src/apps/` 放輕量 app 與 2D 遊戲
- `src/app_registry.rs` 放 app metadata 與 launch 路由表
- `src/shell/` 放系統生命周期與 redraw orchestration
- `src/ui/` 放 shell 各畫面
- `src/dungeon/` 保持獨立
- `src/storage.rs`、`src/media.rs` 這類系統服務維持 top-level
- `src/main.rs` 只做 shell orchestration，不再吸收更多細節

這樣之後要加新遊戲時，判斷標準會很清楚：

- 如果是一般 app 或 2D 小遊戲，放 `src/apps/`
- 如果是有自己資料層、render pipeline、子模組群的重量級系統，再考慮獨立 top-level

## 5. 一句話結論

現在的 `dungeon` 先不要搬進 `src/apps/`。  
它目前比較像一個獨立遊戲子系統，而不是一般 app；等未來有完整 app registry / trait 之後，再決定要不要收進 `src/apps/` 會更合理。
