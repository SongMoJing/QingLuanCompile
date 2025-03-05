# 青鸾编译器

## 构建

### 编译命令

```
用法：
QingLuanCompile [参数] [选项]

参数：
[必填] <脚本路径> 填入用于解释的脚本路径

选项：
<-h | --help>     获取帮助
<-c | --compile>  打包方式 可选：debug|release
```

### 编译返回值

0为相关未知错误

| 返回值 | 含义 | 1   | 2    | 3    | 4 | 5 |
|-----|----|-----|------|------|---|---|
| 30  |    |     |      |      |   |   |
| 20  | 文件 | 不存在 | 不可IO | 解析失败 |   |   |
| 10  | 参数 | 未知  | 缺少   | 错误   |   |   |
| 0   | 编译 | 成功  | 警告   |      |   |   |

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
# version 版本
# include 包含 和 exclude 排除 二选一
feature0 = { version = "0.1.0", include = ["feature0-1", "feature0-2"] }
feature1 = { version = "0.1.0", exclude = ["feature0"] }
# 指定版本，包含所有内容
feature2 = "0.1.0"

# 构建配置
[build]
# 输出路径
path = "./target/"
# 输出类型： <App | Lib>
target = "App"
```



