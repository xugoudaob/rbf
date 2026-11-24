# rbf（bf-rs）

一个用 Rust 写的 Brainfuck 解释器，小巧、直接、可读。

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
