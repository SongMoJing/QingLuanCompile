use crate::_lib::io::{Log, LogType};
use crate::PROJECT_ROOT;
use std::io::ErrorKind;

pub mod toml;
pub mod qls;


pub fn start() {
	if let Some(path) = PROJECT_ROOT.get() {
		// 遍历path下的所有文件
		for res in std::fs::read_dir(format!("{}/src/", path)).unwrap_or_else(|e| match e.kind() {
			ErrorKind::NotFound => {
				Log::new(LogType::Err, format!("路径 {} 不存在。", path).as_str()).throw(21);
			}
			ErrorKind::PermissionDenied => {
				Log::new(LogType::Err, format!("路径 {} 读取失败，请检查文件权限。", path).as_str()).throw(22);
			}
			_ => {
				Log::new(LogType::Err, format!("路径 {} 产生未知错误。", path).as_str()).throw(20);
			}
		}) {
			let path = res.unwrap_or_else(|e| match e.kind() {
				ErrorKind::NotFound => {
					Log::new(LogType::Err, format!("路径 {} 不存在。", path).as_str()).throw(21);
				}
				ErrorKind::PermissionDenied => {
					Log::new(LogType::Err, format!("路径 {} 读取失败，请检查文件权限。", path).as_str()).throw(22);
				}
				_ => {
					Log::new(LogType::Err, format!("路径 {} 产生未知错误。", path).as_str()).throw(20);
				}
			});
			// 判断文件后缀
			if let Some(ext) = path.path().extension() {
				match ext.to_str() {
					Some("qls") => {
						qls::parser_qls(path);
					}
					_ => {}
				}
			}
		}
	}
}