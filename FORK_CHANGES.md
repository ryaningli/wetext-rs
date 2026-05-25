# Fork 本地改动说明

> 本文件记录 fork 仓库相对于上游 `SpenserCai/wetext-rs` 的所有改动，
> 以便主线更新时评估合并冲突和回归风险。

## 改动概览

共修改 3 个源码文件，新增 1 个依赖 + 1 项并发支持，使 `Normalizer` 可通过 `Arc<Normalizer>` 在多线程中无锁共享。

### 1. 新增依赖: `dashmap`

将 `FstCache` 内部的 `HashMap` 替换为 `DashMap`（并发安全 HashMap），支持多线程并发读取和惰性加载 FST 文件。

| 文件 | 改动 |
|------|------|
| `Cargo.toml` | +1 行依赖 `dashmap = "6"` |

### 2. `FstCache` 线程安全改造

**原问题：** `FstCache::get_or_load` 接收 `&mut self`，导致 `Normalizer` 的所有公开方法也需要 `&mut self`，无法在多线程中共享使用。

**改动：** 将 `HashMap` 替换为 `DashMap`，`get_or_load` 拆分为 `get_or_load(&self)` 和 `get(&self)` 两个方法，均为 `&self`。`Normalizer` 的所有方法签名从 `&mut self` 改为 `&self`。

- `get_or_load(&self, path)` — 确保 FST 已加载（利用 DashMap entry API 原子性，同一 key 只 load 一次）
- `get(&self, path)` — 获取已加载的 FST 引用（DashMap 的读操作是 lock-free 的）

**并发行为：**
- 已加载的 FST：多线程无锁并发读
- 初次加载：DashMap 对应 shard 短暂加锁，保证同一 key 只 load 一次
- load 完成后：该 key 的后续读取完全无锁

| 文件 | 改动 |
|------|------|
| `src/normalizer.rs` | `FstCache` 内部 `HashMap` → `DashMap`，`&mut self` → `&self`，`get_or_load` 拆分为两步 |
| `src/lib.rs` | 便利函数 `normalize` 去掉 `mut` |

### 3. 新增并发测试

添加 `test_concurrent_normalize` 集成测试，验证 4 个线程通过 `Arc<Normalizer>` 并发调用 `normalize` 的正确性。

| 文件 | 改动 |
|------|------|
| `tests/integration_test.rs` | 去掉所有 `let mut normalizer`，新增并发测试 |

## 使用方式

```rust
use std::sync::Arc;
use wetext_rs::{Normalizer, NormalizerConfig, Language};

// 单线程（与原 API 兼容，只是不需要 mut 了）
let normalizer = Normalizer::with_defaults("path/to/fsts");
let result = normalizer.normalize("2024年").unwrap();

// 多线程并发
let normalizer = Arc::new(Normalizer::with_defaults("path/to/fsts"));
let handles: Vec<_> = (0..4).map(|_| {
    let n = Arc::clone(&normalizer);
    std::thread::spawn(move || {
        n.normalize("2024年").unwrap()
    })
}).collect();
```

## 代码改动约束

> 所有后续改动必须遵循以下原则，以保持与上游 `SpenserCai/wetext-rs` 的可合入性。

### 原则 1：不改变公开 API 语义

- `Normalizer::new`、`Normalizer::with_defaults`、`Normalizer::normalize` 等方法的参数和返回值不变
- 唯一的签名变化是 `&mut self` → `&self`，这是向后兼容的（原来需要 `mut` 的代码现在不需要了也能编译）

### 原则 2：不新增公开 API

- `FstCache`、`get_or_load`、`get` 均保持私有
- 不新增公开 trait 或方法

### 原则 3：最小化改动行数

- 只修改 `FstCache` 内部实现和相关方法签名
- 不重构、不重命名、不调整代码顺序
- `dashmap` 是唯一的依赖变更

## 合入注意事项

### 上游合并时可能冲突的位置

1. **`src/normalizer.rs` — `FstCache` 结构体和 `get_or_load` 方法**
   - 如果上游修改了 `FstCache`（如更换缓存策略），合并时保留 DashMap 方案

2. **`src/normalizer.rs` — `Normalizer` 方法签名**
   - 如果上游添加了新方法，需要确保新方法也使用 `&self` 而非 `&mut self`

3. **`Cargo.toml` — `[dependencies]`**
   - dashmap 是新增行，通常不会冲突

4. **`tests/integration_test.rs`**
   - 去掉 `mut` 是非破坏性改动，合并时保留即可

### 合入策略

```bash
# 1. 添加上游 remote（首次）
git remote add upstream https://github.com/SpenserCai/wetext-rs.git

# 2. 拉取上游最新代码
git fetch upstream

# 3. 合并到当前分支
git merge upstream/main

# 4. 解决冲突时，参考上述"可能冲突的位置"

# 5. 合并后重新构建验证
cargo test
```
