# rbf（bf-rs）

一个用 AI 写的 Brainfuck 解释器 Rust 实现，小巧、直接、可读。

>骗你的这玩意的目的是测试我自己写的终端 git(TUI) ，以节省查询`git help`或切软件打开搜索引擎抑或 LLM 的时间（而且这个 git 只自用不发布）

---
这玩意功能差不多了，我就不开issue了。
---

## 特性

- 预扫描括号，建立 `[` / `]` 跳转表，运行时 O(1) 跳转
- 纸带默认 30000 格，向右越界自动扩展
- 字节运算使用 `wrapping_add` / `wrapping_sub`，符合 BF 的 0–255 回绕语义
- 支持从文件读取源码，也支持从 stdin 读取
- 输出实时 flush，适合交互式程序
- 输入 EOF 时读入 `0`

## 构建

```bash
cargo build --release
