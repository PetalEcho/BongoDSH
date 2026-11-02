use std::process::ExitCode;

use bongodsh_core::{InputEvent, KeyId, TypingState};

mod art;
mod desktop;
mod docking;
mod input;
// Native FFI is isolated here; the application and core keep their unsafe bans.
#[allow(unsafe_code)]
mod platform;
mod positioning;

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();

    match arguments.as_slice() {
        [] => {
            if let Err(error) = desktop::run() {
                eprintln!("无法启动 BongoDSH 桌面窗口：{error}");
                return ExitCode::FAILURE;
            }
        }
        [argument] if argument == "--help" || argument == "-h" => print_help(),
        [argument] if argument == "--demo" => run_demo(),
        _ => {
            eprintln!("不支持的参数。使用 bongodsh --help 查看用法。");
            return ExitCode::from(2);
        }
    }

    ExitCode::SUCCESS
}

fn print_help() {
    println!(
        "BongoDSH {}\n\n\
         透明置顶的桌面打字挂件原型。点击角色后，在挂件窗口内按键触发动画。\n\
         拖近窗口边缘或可识别输入框后松开可吸附，全屏时默认移到右下角。\n\
         右键打开菜单。macOS 输入框定位需辅助功能权限；未接入全局键盘或 DSH。\n\n\
         用法：\n  bongodsh           启动桌面挂件\n  bongodsh --demo    运行命令行模拟事件\n  bongodsh --help    显示帮助",
        env!("CARGO_PKG_VERSION")
    );
}

fn run_demo() {
    let first = KeyId::new(1);
    let second = KeyId::new(2);
    let events = [
        (
            "第一次按下",
            InputEvent::Press {
                key: first,
                repeat: false,
            },
        ),
        (
            "长按连发",
            InputEvent::Press {
                key: first,
                repeat: true,
            },
        ),
        (
            "另一键按下",
            InputEvent::Press {
                key: second,
                repeat: false,
            },
        ),
        ("第一键松开", InputEvent::Release(first)),
        ("输入源重置", InputEvent::Reset),
    ];
    let mut state = TypingState::default();

    println!("模拟事件演示，不监听真实键盘：");
    for (label, event) in events {
        let feedback = state.apply(event);
        println!(
            "{label}: 左手={}，右手={}，新敲击={:?}",
            feedback.pose.left_pressed, feedback.pose.right_pressed, feedback.strike
        );
    }
}
