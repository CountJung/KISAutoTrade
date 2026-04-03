// Windows release 빌드에서 추가 콘솔 창 방지 - 제거 금지
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    auto_condition_trade_lib::run();
}
