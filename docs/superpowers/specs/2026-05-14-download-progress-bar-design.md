# 下载进度条展示 - 设计文档

## 目标

为 Bilibili 下载器的各下载环节添加进度条，让用户直观感知下载进度。

## 进度条库选型

选择 **tqdm**，原因：
- 轻量，与项目 minimal 风格一致
- `update()` 方法线程安全，适配 ThreadPoolExecutor 并发下载
- 社区最流行的进度条方案，文档完善

## 进度展示规划

| 阶段 | 展示方式 | 进度来源 |
|---|---|---|
| 获取视频信息 | 无进度条 | API 请求很快 |
| 下载视频流 | tqdm 总进度条（byte 单位） | `video.content_len` |
| 下载音频流 | tqdm 进度条（byte 单位） | HEAD 获取 audio content-length |
| 合并混流 | 文字提示 `正在合并音视频...` | ffmpeg 保持不变 |
| 下载弹幕 | tqdm 计数进度条 | segment 数量 |

## 实现细节

### download_chunks 改造

- `download_chunks` 接收 `tqdm` 进度条实例作为参数
- 在 `as_completed` 循环中，每完成一个 chunk 后调用 `pbar.update(block_size)`
- `block_size = end - start + 1`，精确反映已完成字节数

### download_audio 改造

- 先发 HEAD 请求获取 `Content-Length`
- 创建 tqdm 进度条，total 为音频文件大小
- 流式下载时 `pbar.update(len(chunk))`

### download_danmaku 改造

- 创建 tqdm 计数进度条，`total=segment_count`，`desc='弹幕'`
- 在 `as_completed` 循环中 `pbar.update(1)`

### run 方法改造

- 统一管理 tqdm 实例的生命周期
- 每个阶段完成后 `pbar.close()` 或使用 `with` 上下文管理器
- 保持现有 print 语句的输出层次感

## 修改文件

| 文件 | 修改 |
|---|---|
| `bilidown/downloader.py` | `download_chunks`、`download_audio`、`download_danmaku`、`run` 方法改造 |
| `requirements.txt` | 添加 `tqdm` |

## 不涉及

- ffmpeg 混流阶段不添加进度条
- 不修改 Video 数据模型
- 不修改 CLI 参数
- 不修改弹幕解析逻辑
