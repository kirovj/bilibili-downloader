# Python 重写设计文档

**日期**: 2026-05-03（更新于 2026-05-05）
**目标**: 将 B站视频与弹幕下载器从 Rust 重写为 Python，功能 1:1 照搬
**虚拟环境**: 项目根目录已存在 `.venv/`（Python 3.14.4），所有操作需在激活虚拟环境后执行

## 项目形态

CLI 工具，通过 `python -m bilidown <bv>` 使用。

## 技术选型

| 维度 | 选择 | 理由 |
|------|------|------|
| HTTP 客户端 | `requests` | 最熟悉，API 简洁 |
| 并发模型 | `concurrent.futures.ThreadPoolExecutor` | 线程模型直观，7-10 并发完全够用 |
| CLI 框架 | `argparse` | 标准库，零额外依赖 |
| 弹幕解析 | `betterproto` (v2 beta) | 生成的 dataclass 更 Pythonic，内置 `to_dict()`，纯 Python 编译器无需 protoc |
| 测试 | `pytest` | 简洁，社区标准 |
| Python 版本 | 3.14（`.venv`） | 项目已配置虚拟环境 |
| 依赖管理 | `requirements.txt` | 用户偏好 |

## 依赖

```
requests>=2.31
betterproto>=2.0.0b6
```

开发依赖: `pytest`

## 项目结构

```
bilibili-downloader/
├── requirements.txt
├── cookie.txt              # 用户自备
├── README.md
└── bilidown/
    ├── __init__.py
    ├── __main__.py         # CLI 入口
    ├── downloader.py       # 核心下载器
    ├── model.py            # 数据类
    ├── util.py             # 工具函数
    ├── danmaku.proto       # 弹幕 protobuf 定义（源文件）
    └── danmaku_pb2.py      # betterproto 预编译生成，提交到仓库
```

## 模块职责

### `__main__.py` — CLI 入口
- argparse 解析参数：`bv`（位置参数，必填）、`-t / --tasknum`（并发数，默认 7，上限 10）
- 调用 `Downloader(task_num).run(bv)`

### `downloader.py` — 核心下载器

`Downloader` 类持有 `requests.Session`（含 cookie）、并发数、输出目录。

主要方法：
1. `check_login()` — GET `/nav` 验证 cookie
2. `build_video(bv)` — 调 API 获取 video info + playurl，返回 `Video`
3. `download_chunks(video)` — 按 10MB 分块，ThreadPoolExecutor 并发下载到 `chunk_{index}`
4. `download_audio(video)` — 下载 DASH 音频流到 `audio.mp3`
5. `build_final_video(video, chunk_count)` — 合并 chunk → `video.{format}`，调用 ffmpeg 混流，清理中间文件
6. `download_danmaku(video)` — 按每 6 分钟 segment 拉取弹幕，用 betterproto 解析，调用 `to_dict()` 逐行写 JSON 到 `danmuku.txt`

涉及的 B站 API：
- `https://api.bilibili.com/x/web-interface/view?bvid=` — 视频信息
- `https://api.bilibili.com/x/player/playurl` — 播放地址（dash/durl）
- `http://api.bilibili.com/x/v2/dm/web/seg.so` — 弹幕 segment
- `https://api.bilibili.com/x/web-interface/nav` — 登录验证

### `model.py` — 数据模型
- `Video` dataclass：bv, cid, video_url, audio_url, title, format, duration, content_len
- `DanmakuElem` / `DanmakuSegment`：由 `danmaku.proto` 经 betterproto 预编译生成（`danmaku_pb2.py` 提交到仓库，用户无需安装 protoc）

### `util.py` — 工具函数
- `mix_video_audio(video_path, audio_path, output_path)` — 调用 ffmpeg 混流
- `replace_illegal_chars_in_windows(value)` — 替换 Windows 文件名非法字符
- `write_bytes_to_file(filepath, bytes, offset)` — 带偏移量的文件写入

## 下载流程

```
CLI 输入 BV
  → Downloader.__init__()         读取 cookie.txt，创建 requests.Session
  → check_login()                 验证 cookie
  → build_video(bv)               获取视频元信息 → Video
  → 创建输出目录 {title}_{bv}/
  → download_chunks(video)        ThreadPoolExecutor 分块并发下载
  → download_audio(video)         下载音频流
  → build_final_video()           合并 chunk + ffmpeg 混流
  → download_danmaku(video)       下载并解析弹幕
```

## 错误处理

自定义异常类：
- `DownloadError` — 基类
- `LoginFailError` — 登录验证失败
- `VideoInfoError` — 获取视频信息失败
- `DanmakuError` — 弹幕下载/解析失败

网络异常由 `requests.RequestException` 原生抛出。

## CLI 接口

```bash
python -m bilidown BVxxx          # 默认 7 并发
python -m bilidown BVxxx -t 5     # 指定并发数
```

与原 Rust 版参数完全一致。

## 测试

使用 `pytest`。测试范围：
- `util.py` 中的纯函数单元测试
- `model.py` 中的数据模型测试
- 下载流程集成测试（需网络和 cookie）

## 与原 Rust 版的差异

| 方面 | Rust 版 | Python 版 |
|------|---------|-----------|
| 异步 | tokio | ThreadPoolExecutor |
| HTTP | reqwest | requests |
| 错误 | thiserror 枚举 | 自定义 Exception 类 |
| Protobuf | prost 编译时派生 | betterproto 生成 dataclass |
| CLI | clap | argparse |
| 无音频文件时 | 仍尝试 ffmpeg | 跳过 ffmpeg 混流 |
