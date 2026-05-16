# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在本项目中工作时提供指导。

## 首要原则
### 1. 默认语言：中文
- 所有代码注释、提交信息、文档编写均使用中文
- 与用户交流使用中文
- 思考过程使用中文
- 代码中的字符串提示信息使用中文
- 禁止任何unicode表情

### 2. 一致性
- 严格遵守CLAUDE.md中的规范

### 3. 实用性
- 优先选择稳定、文档齐全的技术方案，而非盲目追求最新技术
- 尽量使用项目中已经使用的第三方包，避免重复造轮子

### 4. 编码规范
- 每个方法包括单元测试都需要增加注释，说明该方法的作用，方法的参数和返回值如果作用显而易见不需要说明，否则也需要增加说明
- 所有方法中的参数都应增加类型

### 5. 单元测试

范围
- 只为核心逻辑类添加单元测试
- 异常类、参数配置类、简单工具类不需要添加单元测试
- 新增或修改代码后，先确认是否需要创建或修改对应的单元测试

### 6. 运行环境
- 运行时需 ffmpeg 在 PATH 中，或放在可执行文件同一目录下
- 运行时需 cookie.txt 与可执行文件在同一目录下
- Python 命令需使用项目虚拟环境 `.venv/Scripts/python.exe`（Windows）或 `.venv/bin/python`（Linux/Mac）


## 项目详情
### 项目简介
Bilibili（B站）视频与弹幕下载器。通过 BV 号下载视频（支持 DASH 流和传统单流格式），同时下载弹幕（protobuf 格式）并保存为 JSON Lines 文件。支持将弹幕转换为 ASS 字幕并通过 ffmpeg 硬字幕方式集成到视频中。

### 技术栈
| 类别 | 技术 |
|---|---|
| 语言 | Python 3.11+ |
| HTTP 客户端 | requests 2.31+ |
| UA 伪装 | fake-useragent 1.5+ |
| CLI 参数 | argparse（标准库） |
| 数据模型 | dataclass（标准库） |
| 序列化 | json（标准库） |
| 错误处理 | 自定义 Exception 子类 |
| Protobuf | betterproto 2.0.0b6+ |
| 并发 | concurrent.futures.ThreadPoolExecutor |
| 进度条 | tqdm 4.66+ |
| 测试 | pytest 8.0+ + unittest.mock |
| 外部依赖 | ffmpeg（运行时，用于合并音视频） |

### 项目结构
```
bilidown/
├── __init__.py          # 包入口，导出公共 API（Downloader、Video、异常类）
├── __main__.py          # CLI 入口（argparse 参数解析）
├── downloader.py        # 核心下载逻辑（Downloader 类、异常类、HTTP 会话管理）
├── model.py             # 数据模型（Video dataclass）
├── util.py              # 工具函数（文件名清理、带偏移写文件、ffmpeg 混流）
├── danmaku_pb2.py       # betterproto 生成的弹幕 protobuf 数据类
└── danmaku_ass.py       # 弹幕 JSON Lines 转 ASS 字幕格式（DanmakuAssError、convert_danmaku_to_ass()）
tests/
├── test_downloader.py   # 下载器集成测试（mock HTTP 请求）
├── test_danmaku_ass.py  # 弹幕 ASS 转换单元测试
├── test_model.py        # Video 数据模型测试
└── test_util.py         # 工具函数测试
```

### 核心流程
1. 读取 `cookie.txt` 进行 B 站登录校验
2. 根据 BV 号调用 B 站 API 获取视频元数据（标题、cid、视频/音频 URL、格式）
3. 创建输出目录 `{标题}_{bv}/`
4. 并发分片下载视频（每片 10MB，并发数由 `-t` 参数控制，最大 10）
5. 下载音频流（DASH 格式时）
6. 合并视频分片 -> 调用 ffmpeg 混合音视频 -> 输出最终视频文件
7. 清理临时文件（分片、独立音频文件）
8. 按 6 分钟一段下载弹幕（protobuf 格式），解码后写入 `danmuku.txt`（JSON Lines）
9. （可选，通过 `-d` / `--danmaku-ass` 开启）将 `danmuku.txt` 转换为 ASS 字幕格式，通过 ffmpeg 硬字幕方式烧录到视频中

### 构建与运行
```bash
# 安装依赖（在虚拟环境中）
.venv/Scripts/python.exe -m pip install -r requirements.txt

# 运行
.venv/Scripts/python.exe -m bilidown <BV_ID>
.venv/Scripts/python.exe -m bilidown <BV_ID> -t 5        # 指定并发数（最大 10）
.venv/Scripts/python.exe -m bilidown <BV_ID> -d           # 下载并嵌入弹幕字幕
.venv/Scripts/python.exe -m bilidown <BV_ID> --danmaku-ass  # 同上（长参数）

# 测试
.venv/Scripts/python.exe -m pytest tests/ -v
```

### 注意事项
- `cookie.txt` 包含敏感信息（SESSDATA），已在 `.gitignore` 中忽略，切勿提交到仓库
- `Downloader` 是最核心的类，已有完整的集成测试覆盖（含 mock HTTP 请求）
- 项目已从 Rust 重写为 Python（见 `docs/` 下的设计文档）
- `danmaku_pb2.py` 由 betterproto 预编译生成，无需安装 protoc 编译器
- `danmaku_ass.py` 负责将弹幕 JSON Lines 转换为 ASS 字幕格式，支持滚动、顶部固定、底部固定三种弹幕模式
- 弹幕硬字幕嵌入依赖 ffmpeg 重新编码视频（`-c:v libx264 -crf 18`），会增加处理时间
- 详细设计文档见 `docs/superpowers/specs/`，实现计划见 `docs/superpowers/plans/`
