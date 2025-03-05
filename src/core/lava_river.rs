// use std::sync::Mutex;
// use crate::core::Build;

// 使用 lazy_static 延迟初始化
lazy_static::lazy_static! {
    // static ref GLOBAL_DATA: Mutex<Build>;
}

fn add_data(_s: String) {
	// GLOBAL_DATA.lock().unwrap().push(s);
}

fn main() {
	// add_data("Hello".to_string());
	// println!("Data: {:?}", GLOBAL_DATA.lock().unwrap());
}

