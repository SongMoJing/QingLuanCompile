# 青鸾编译器

## 构建

### 编译命令

```
用法
  QingLuanCompile.exe [OPTIONS] [PATH]
参数
  [PATH]  项目路径
选项
  -p, --package <debug|release>  输出类型 [默认：release]
  -o, --os <os>                  目标操作系统 [默认：x86_64-pc-windows-msvc]
  -l, --language <lang>          默认控制台输出语言 [默认：zh-CN]
  -h, --help                     显示帮助
  -v, --version                  显示版本
```

### 编译返回值

0为相关未知错误

| 返回值 | 含义 | 1    | 2    | 3    | 4 | 5 |
|-----|----|------|------|------|---|---|
| 40  | 源码 | 词法错误 | 语法错误 |      |   |   |
| 30  | 通信 | 不存在  | 错误返回 | 未知返回 |   |   |
| 20  | 文件 | 不存在  | 不可IO | 解析失败 |   |   |
| 10  | 参数 | 未知   | 缺少   | 错误   |   |   |
| 0   | 编译 | 成功   | 警告   |      |   |   |

### QingLuan.toml 规范

```toml
# 项目配置
[project]
# 项目名称 输出文件名
name = "青鸾测试"
# 项目版本
version = "1.0.0"
# SDK版本
sdk_edition = "0.1.0"
# 入口文件
main = "src/main.qls"
# 作者
authors = [
    "PRC.松蓦箐 <Song_Mojing@outlook.com>"
]

# 依赖
[dependencies]
feature = { version = "0.1.0"}
```



