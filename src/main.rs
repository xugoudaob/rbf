use std::io::{self, IsTerminal, Read, Write};
use std::process::exit;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHIFT: usize = 1024; // 左扩展时一次补多少格

fn print_help() {
    println!(
        "\
{NAME} {VERSION}
一个用 Rust 写的 Brainfuck 解释器

用法:
    {NAME} [选项] [文件]
    {NAME} [选项] < 程序.bf
    {NAME} - [选项] < 程序.bf

参数:
    <文件>              要执行的 Brainfuck 源文件
                        省略或为 - 时从标准输入读取
                        省略且标准输入是终端时显示本帮助

选项:
    -h, --help          打印此帮助并退出
    -V, --version       打印版本并退出

示例:
    {NAME} hello.bf
    {NAME} - < hello.bf
    echo ',+.' | {NAME}

指令:
    >   指针右移
    <   指针左移
    +   当前格加 1（0-255 回绕）
    -   当前格减 1（0-255 回绕）
    .   输出当前格字节
    ,   读入一个字节到当前格，EOF 读入 0
    [   当前格为 0 时跳到匹配的 ]
    ]   当前格非 0 时跳回匹配的 [
    其他字符被忽略

退出码:
    0   正常结束
    1   运行时错误（括号不匹配、指针越界等）
    2   命令行用法错误
"
    );
}

fn usage_error(msg: &str) -> ! {
    eprintln!("错误: {msg}\n");
    eprintln!("尝试 '{NAME} --help' 获取更多信息。");
    exit(2);
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "未知运行时错误".to_string()
        };
        eprintln!("错误: {msg}");
        exit(1);
    }));
}

fn main() {
    install_panic_hook();

    let mut file: Option<String> = None;
    let mut use_stdin = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            "-V" | "--version" => {
                println!("{NAME} {VERSION}");
                exit(0);
            }
            "-" => {
                if use_stdin || file.is_some() {
                    usage_error("只能指定一个输入源");
                }
                use_stdin = true;
            }
            s if s.starts_with('-') => {
                usage_error(&format!("未知选项 '{s}'"));
            }
            _ => {
                if use_stdin || file.is_some() {
                    usage_error("只能指定一个输入源");
                }
                file = Some(arg);
            }
        }
    }

    let src = match file {
        Some(path) => {
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读取 {path} 失败: {e}"))
        }
        None => {
            // 没给文件、没写 '-'，且标准输入是终端：直接显示帮助并正常退出
            if !use_stdin && io::stdin().is_terminal() {
                print_help();
                exit(0);
            }
            let mut s = String::new();
            io::stdin()
                .read_to_string(&mut s)
                .expect("从标准输入读取失败");
            s
        }
    };

    run(&src);
}

fn run(src: &str) {
    let code: Vec<u8> = src.bytes().filter(|b| b"><+-.,[]".contains(b)).collect();

    let mut jump = vec![0usize; code.len()];
    let mut stack = Vec::new();
    for (i, &b) in code.iter().enumerate() {
        match b {
            b'[' => stack.push(i),
            b']' => {
                let j = stack
                    .pop()
                    .unwrap_or_else(|| panic!("第 {i} 个指令处有不匹配的 ']'"));
                jump[i] = j;
                jump[j] = i;
            }
            _ => {}
        }
    }
    if let Some(i) = stack.pop() {
        panic!("第 {i} 个指令处有不匹配的 '['");
    }

    const INITIAL: usize = 30_000;
    let mut tape: Vec<u8> = vec![0u8; INITIAL];
    let mut offset: isize = (INITIAL / 2) as isize; // 逻辑 0 放中间，左右各留一半
    let mut ptr: isize = 0; // 逻辑指针，可为负
    let mut pc = 0usize;
    let mut stdout = io::stdout();
    let mut stdin = io::stdin().bytes();

    while pc < code.len() {
        match code[pc] {
            b'>' => {
                ptr += 1;
                if offset + ptr >= tape.len() as isize {
                    tape.push(0);
                }
            }
            b'<' => {
                ptr -= 1;
                if offset + ptr < 0 {
                    // 向左扩展：前面插 SHIFT 个 0，逻辑 0 的物理位置后移 SHIFT
                    let mut new_tape = vec![0u8; SHIFT];
                    new_tape.extend_from_slice(&tape);
                    tape = new_tape;
                    offset += SHIFT as isize;
                }
            }
            b'+' => {
                let i = (offset + ptr) as usize;
                tape[i] = tape[i].wrapping_add(1);
            }
            b'-' => {
                let i = (offset + ptr) as usize;
                tape[i] = tape[i].wrapping_sub(1);
            }
            b'.' => {
                let i = (offset + ptr) as usize;
                stdout.write_all(&[tape[i]]).unwrap();
                stdout.flush().unwrap();
            }
            b',' => {
                let i = (offset + ptr) as usize;
                tape[i] = match stdin.next() {
                    Some(Ok(b)) => b,
                    _ => 0,
                };
            }
            b'[' => {
                if tape[(offset + ptr) as usize] == 0 {
                    pc = jump[pc];
                }
            }
            b']' => {
                if tape[(offset + ptr) as usize] != 0 {
                    pc = jump[pc];
                }
            }
            _ => unreachable!(),
        }
        pc += 1;
    }
}
