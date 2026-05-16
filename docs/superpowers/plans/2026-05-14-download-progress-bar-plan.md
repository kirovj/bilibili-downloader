# 下载进度条展示 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 Bilibili 下载器的视频下载、音频下载、弹幕下载环节添加 tqdm 进度条

**Architecture:** tqdm 进度条实例由 `run()` 方法创建并通过 `with` 上下文管理器管理生命周期，传递给各下载方法。各方法内部仅负责 `pbar.update()` 更新进度，不创建进度条。tqdm 参数均为可选（默认 None），确保现有测试无需改动。

**Tech Stack:** tqdm

---

### Task 1: 添加 tqdm 依赖

**Files:**
- Modify: `requirements.txt`

- [ ] **Step 1: 添加 tqdm 到依赖**

```txt
requests>=2.31
betterproto>=2.0.0b6
fake-useragent>=1.5
tqdm>=4.66
pytest>=8.0
```

- [ ] **Step 2: 安装 tqdm**

```bash
pip install tqdm>=4.66
```

- [ ] **Step 3: Commit**

```bash
git add requirements.txt
git commit -m "chore: 添加 tqdm 依赖

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: 改造 download_chunks 支持进度条

**Files:**
- Modify: `bilidown/downloader.py:165-187`

- [ ] **Step 1: 修改 download_chunks 方法签名和实现**

将 `download_chunks` 改为：

```python
def download_chunks(self, video: "Video", pbar: "tqdm | None" = None) -> int:
    """并发分块下载视频，可选择传入 tqdm 进度条"""
    from concurrent.futures import ThreadPoolExecutor, as_completed

    chunk_size = 10 * 1024 * 1024  # 10MB
    futures: dict = {}
    start = 0
    index = 0

    with ThreadPoolExecutor(max_workers=self.task_num) as executor:
        while start < video.content_len:
            end = min(start + chunk_size, video.content_len) - 1
            if end < start:
                end = start
            f = executor.submit(self.download_chunk, video, (start, end), index)
            block_size = end - start + 1
            futures[f] = block_size
            start = end + 1
            index += 1

        for f in as_completed(futures):
            f.result()  # 传播异常
            if pbar:
                pbar.update(futures[f])

    return index
```

关键变更：
- `futures` 从 `list` 改为 `dict`，存储 `{future: block_size}` 映射
- 每完成一个 chunk 后，根据映射获取实际字节数并更新进度条
- `pbar` 参数可选，默认 `None`，不影响现有测试

- [ ] **Step 2: 验证现有测试通过**

```bash
python -m pytest tests/test_downloader.py::TestDownloadChunks -v
```

预期：PASS（现有测试不传 pbar，行为不变）

- [ ] **Step 3: Commit**

```bash
git add bilidown/downloader.py
git commit -m "feat: download_chunks 支持 tqdm 进度条

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: 改造 download_audio 支持进度条

**Files:**
- Modify: `bilidown/downloader.py:189-199`

- [ ] **Step 1: 修改 download_audio 方法签名和实现**

将 `download_audio` 改为：

```python
def download_audio(self, video: "Video", pbar: "tqdm | None" = None) -> None:
    """下载音频流，可选择传入 tqdm 进度条"""
    if not video.audio_url:
        return
    r = self.session.get(video.audio_url, stream=True, timeout=30)
    r.raise_for_status()
    filepath = f"{self.dir}/audio.mp3"
    with open(filepath, "wb") as f:
        for chunk in r.iter_content(chunk_size=8192):
            if chunk:
                f.write(chunk)
                if pbar:
                    pbar.update(len(chunk))
```

- [ ] **Step 2: 验证现有测试通过**

```bash
python -m pytest tests/test_downloader.py::TestDownloadAudio -v
```

预期：PASS

- [ ] **Step 3: Commit**

```bash
git add bilidown/downloader.py
git commit -m "feat: download_audio 支持 tqdm 进度条

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: 改造 download_danmaku 支持进度条

**Files:**
- Modify: `bilidown/downloader.py:239-260`

- [ ] **Step 1: 修改 download_danmaku 方法签名和实现**

将 `download_danmaku` 改为：

```python
def download_danmaku(self, video: "Video", pbar: "tqdm | None" = None) -> None:
    """下载并解析弹幕，写入 JSON Lines 文件，可选择传入 tqdm 进度条"""
    from concurrent.futures import ThreadPoolExecutor, as_completed
    import json

    bags = (video.duration + 359) // 360  # 每 6 分钟一个 segment
    results = {}

    with ThreadPoolExecutor(max_workers=self.task_num) as executor:
        futures = {
            executor.submit(self.download_danmaku_segment, video, i + 1): i
            for i in range(bags)
        }
        for f in as_completed(futures):
            idx = futures[f]
            results[idx] = f.result()
            if pbar:
                pbar.update(1)

    with open(f"{self.dir}/danmuku.txt", "w", encoding="utf-8") as f:
        for i in range(bags):
            for elem in results[i].elems:
                d = elem.to_dict()
                f.write(json.dumps(d, ensure_ascii=False) + "\n")
```

- [ ] **Step 2: 验证现有测试通过**

```bash
python -m pytest tests/test_downloader.py::TestDownloadDanmaku -v
```

预期：PASS

- [ ] **Step 3: Commit**

```bash
git add bilidown/downloader.py
git commit -m "feat: download_danmaku 支持 tqdm 进度条

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: 改造 run 方法，集成进度条

**Files:**
- Modify: `bilidown/downloader.py:262-287`

- [ ] **Step 1: 修改 run 方法，统一管理 tqdm 进度条生命周期**

将 `run` 方法改为：

```python
def run(self, bv: str) -> None:
    """执行完整的下载流程"""
    import os
    from tqdm import tqdm

    video = self.build_video(bv)

    # 创建输出目录
    dir_name = f"{video.title}_{bv}"
    try:
        os.makedirs(dir_name, exist_ok=True)
        self.dir = dir_name
    except OSError:
        os.makedirs(bv, exist_ok=True)
        self.dir = bv

    if video.content_len == 0:
        print(f"download {video.title} fail, video size is 0")
        return

    print(f'download {video.bv} start, title: "{video.title}"')

    # 阶段 1: 下载视频流
    with tqdm(total=video.content_len, unit="B", unit_scale=True,
              unit_divisor=1024, desc="视频") as video_pbar:
        chunk_count = self.download_chunks(video, pbar=video_pbar)

    # 阶段 2: 下载音频流
    if video.audio_url:
        with tqdm(unit="B", unit_scale=True, unit_divisor=1024, desc="音频") as audio_pbar:
            self.download_audio(video, pbar=audio_pbar)
            # 从已下载的字节数推断 total（实际已在迭代中更新）
    else:
        self.download_audio(video)

    # 阶段 3: 合并混流
    print("正在合并音视频...")
    self.build_final_video(video, chunk_count)

    # 阶段 4: 下载弹幕
    bags = (video.duration + 359) // 360
    with tqdm(total=bags, desc="弹幕") as dm_pbar:
        self.download_danmaku(video, pbar=dm_pbar)

    print(f'download {video.bv} finished')
```

- [ ] **Step 2: 验证全部测试通过**

```bash
python -m pytest tests/test_downloader.py -v
```

预期：全部 PASS

- [ ] **Step 3: Commit**

```bash
git add bilidown/downloader.py
git commit -m "feat: run 方法集成 tqdm 进度条展示

Co-Authored-By: Claude <noreply@anthropic.com>"
```
