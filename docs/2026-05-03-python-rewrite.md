# Python 重写 B站视频下载器 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
> **更新于 2026-05-05**：使用 betterproto 替代 protobuf，所有命令使用 `.venv` 虚拟环境。

**Goal:** 将 B站视频与弹幕下载器从 Rust 1:1 重写为 Python CLI 工具

**Architecture:** requests + ThreadPoolExecutor 替代 tokio + reqwest，argparse 替代 clap，betterproto 替代 prost，其余结构照搬原 Rust 版。代码放在 `bilidown/` 包目录下。项目已配置 `.venv/`（Python 3.10+），所有命令均使用 `.venv/Scripts/python` 和 `.venv/Scripts/pip`。

**Tech Stack:** Python 3.10+ (`.venv`), requests, betterproto, argparse, pytest, ffmpeg(外部)

**项目结构：**
```
bilidown/
├── requirements.txt
├── cookie.txt
├── README.md
└── bilidown/
    ├── __init__.py
    ├── __main__.py
    ├── downloader.py
    ├── model.py
    ├── util.py
    ├── danmaku.proto
    └── danmaku_pb2.py
```

**使用方式：** `.venv/Scripts/python -m bilidown <bv>`

---

### Task 1: 项目骨架搭建

**Files:**
- Create: `bilidown/__init__.py`
- Create: `requirements.txt`

- [ ] **Step 1: 创建包和测试目录结构**

```bash
mkdir bilidown
mkdir tests
```

- [ ] **Step 2: 编写 requirements.txt**

```txt
requests>=2.31
betterproto>=2.0.0b6
pytest>=8.0
```

- [ ] **Step 3: 编写 `__init__.py`（空占位，后续 Task 填充）**

```python
__version__ = "0.1.0"
```

- [ ] **Step 4: 安装依赖**

```bash
.venv/Scripts/pip install -r requirements.txt
```

- [ ] **Step 5: Commit**

```bash
git add requirements.txt bilidown/__init__.py
git commit -m "feat: scaffold project structure and dependencies"
```

---

### Task 2: util.py — 工具函数

**Files:**
- Create: `tests/test_util.py`
- Create: `bilidown/util.py`

- [ ] **Step 1: 编写 `tests/test_util.py` — replace_illegal_chars 测试**

```python
import pytest
from bilidown.util import replace_illegal_chars_in_windows


def test_replace_illegal_chars_all():
    result = replace_illegal_chars_in_windows('a\\a/a:a*a?a"a<a>a|a')
    assert result == "a╲a╱a：a✱a？a"a《a》a│a"


def test_replace_illegal_chars_no_illegal():
    result = replace_illegal_chars_in_windows("hello world")
    assert result == "hello world"


def test_replace_illegal_chars_empty():
    result = replace_illegal_chars_in_windows("")
    assert result == ""
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_util.py -v
```
Expected: 3 FAILED (module not found)

- [ ] **Step 3: 编写 `bilidown/util.py` — replace_illegal_chars_in_windows**

```python
def replace_illegal_chars_in_windows(value: str) -> str:
    replacements = {
        "\\": "╲",
        "/": "╱",
        ":": "：",
        "*": "✱",
        "?": "？",
        '"': """,
        "<": "《",
        ">": "》",
        "|": "│",
    }
    for old, new in replacements.items():
        value = value.replace(old, new)
    return value
```

- [ ] **Step 4: 运行测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_util.py::test_replace_illegal_chars_all tests/test_util.py::test_replace_illegal_chars_no_illegal tests/test_util.py::test_replace_illegal_chars_empty -v
```
Expected: 3 PASSED

- [ ] **Step 5: 编写 `tests/test_util.py` — write_bytes_to_file 测试**

```python
import os
import tempfile
from bilidown.util import write_bytes_to_file


def test_write_bytes_to_file():
    with tempfile.TemporaryDirectory() as tmpdir:
        filepath = os.path.join(tmpdir, "test.bin")
        write_bytes_to_file(filepath, b"hello", 0)
        write_bytes_to_file(filepath, b" world", 6)
        with open(filepath, "rb") as f:
            content = f.read()
        assert content == b"hello world"
```

- [ ] **Step 6: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_util.py::test_write_bytes_to_file -v
```
Expected: FAILED (function not defined)

- [ ] **Step 7: 添加 write_bytes_to_file 到 util.py**

```python
def write_bytes_to_file(filepath: str, data: bytes, offset: int) -> None:
    with open(filepath, "ab") as f:
        f.seek(offset)
        f.write(data)
```

- [ ] **Step 8: 运行测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_util.py::test_write_bytes_to_file -v
```
Expected: PASSED

- [ ] **Step 9: 编写 `tests/test_util.py` — mix_video_audio 测试（仅验证调用不抛异常）**

```python
import os
import tempfile
from bilidown.util import mix_video_audio


def test_mix_video_audio_no_files():
    """ffmpeg not found or missing files — should raise subprocess.CalledProcessError or FileNotFoundError"""
    with tempfile.TemporaryDirectory() as tmpdir:
        with pytest.raises((FileNotFoundError, Exception)):
            mix_video_audio(
                os.path.join(tmpdir, "video.mp4"),
                os.path.join(tmpdir, "audio.mp3"),
                os.path.join(tmpdir, "output.mp4"),
            )
```

- [ ] **Step 10: 添加 mix_video_audio 到 util.py**

```python
import subprocess


def mix_video_audio(video_path: str, audio_path: str, output_path: str) -> None:
    subprocess.run(
        [
            "ffmpeg",
            "-i", video_path,
            "-i", audio_path,
            "-c:v", "copy",
            "-c:a", "aac",
            "-strict", "experimental",
            output_path,
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
```

- [ ] **Step 11: 运行全量 util 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_util.py -v
```
Expected: 5 PASSED

- [ ] **Step 12: Commit**

```bash
git add tests/test_util.py bilidown/util.py
git commit -m "feat: add util.py with filename sanitization, file writing, and ffmpeg mixing"
```

---

### Task 3: model.py — 数据模型与 betterproto 弹幕定义

**Files:**
- Create: `tests/test_model.py`
- Create: `bilidown/model.py`
- Create: `bilidown/danmaku.proto`
- Create: `bilidown/danmaku_pb2.py` (预编译生成，提交到仓库)

- [ ] **Step 1: 编写 `tests/test_model.py`**

```python
from bilidown.model import Video


def test_video_dataclass():
    video = Video(
        bv="BV1xx",
        cid=12345,
        video_url="https://example.com/video",
        audio_url="https://example.com/audio",
        title="测试视频",
        format="mp4",
        duration=120,
        content_len=1024000,
    )
    assert video.bv == "BV1xx"
    assert video.cid == 12345
    assert video.title == "测试视频"
    assert video.format == "mp4"
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_model.py -v
```
Expected: FAILED (module not found or Video not defined)

- [ ] **Step 3: 编写 `bilidown/model.py`**

```python
from dataclasses import dataclass


@dataclass
class Video:
    bv: str
    cid: int
    video_url: str
    audio_url: str
    title: str
    format: str
    duration: int
    content_len: int
```

- [ ] **Step 4: 运行测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_model.py -v
```
Expected: 1 PASSED

- [ ] **Step 5: 编写 `bilidown/danmaku.proto`**

```proto
syntax = "proto3";

message DanmakuElem {
    int64 id = 1;
    int32 progress = 2;
    int32 mode = 3;
    int32 fontsize = 4;
    uint32 color = 5;
    string mid_hash = 6;
    string content = 7;
    int64 ctime = 8;
    int32 weight = 9;
    string action = 10;
    int32 pool = 11;
    string dmid_str = 12;
    int32 attr = 13;
}

message DanmakuSegment {
    repeated DanmakuElem elems = 1;
}
```

- [ ] **Step 6: 用 betterproto 预编译 `.proto`，生成 `danmaku_pb2.py`**

```bash
.venv/Scripts/python -m betterproto --output_dir=bilidown bilidown/danmaku.proto
```

betterproto v2 内置纯 Python 编译器，无需安装 protoc。生成的文件提交到仓库，用户只需安装 betterproto 即可使用。

- [ ] **Step 7: 验证生成的模块可导入（betterproto 生成的是 dataclass）**

```bash
.venv/Scripts/python -c "from bilidown.danmaku_pb2 import DanmakuSegment; print(type(DanmakuSegment))"
```
Expected: `<class 'type'>` 且无 import 错误

- [ ] **Step 8: Commit**

```bash
git add tests/test_model.py bilidown/model.py bilidown/danmaku.proto bilidown/danmaku_pb2.py
git commit -m "feat: add Video dataclass and danmaku betterproto definition"
```

---

### Task 4: downloader.py — 初始化与登录验证

**Files:**
- Create: `tests/test_downloader.py`
- Create: `bilidown/downloader.py`

- [ ] **Step 1: 编写 `tests/test_downloader.py` — 初始化测试**

```python
import os
import tempfile
from unittest.mock import patch, MagicMock
from bilidown.downloader import Downloader, DownloadError, LoginFailError


class TestDownloaderInit:
    def test_default_task_num(self):
        d = Downloader()
        assert d.task_num == 7

    def test_custom_task_num(self):
        d = Downloader(task_num=3)
        assert d.task_num == 3

    def test_task_num_capped_at_10(self):
        d = Downloader(task_num=15)
        assert d.task_num == 10

    def test_empty_dir_initially(self):
        d = Downloader()
        assert d.dir == ""
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloaderInit -v
```
Expected: FAILED (module not found)

- [ ] **Step 3: 编写 `bilidown/downloader.py` — 异常类 + Downloader.__init__**

```python
import os

import requests


UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36"


class DownloadError(Exception):
    """下载器异常基类"""
    pass


class LoginFailError(DownloadError):
    """Cookie 登录验证失败"""
    pass


class VideoInfoError(DownloadError):
    """获取视频信息失败"""
    pass


class DanmakuError(DownloadError):
    """弹幕下载/解析失败"""
    pass


class Downloader:
    def __init__(self, task_num: int = 7):
        self.task_num = min(task_num, 10)
        self.dir = ""
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": UA,
            "Referer": "https://www.bilibili.com/",
        })
        if os.path.exists("cookie.txt"):
            with open("cookie.txt") as f:
                self.session.headers["Cookie"] = f.read().strip()
```

- [ ] **Step 4: 运行初始化测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloaderInit -v
```
Expected: 4 PASSED

- [ ] **Step 5: 编写 check_login 测试（mock 网络请求）**

```python
class TestCheckLogin:
    @patch.object(requests.Session, "get")
    def test_login_success(self, mock_get):
        mock_response = MagicMock()
        mock_response.json.return_value = {"data": {"isLogin": True}}
        mock_get.return_value = mock_response

        d = Downloader()
        d.check_login()  # 不抛异常即通过

    @patch.object(requests.Session, "get")
    def test_login_fail(self, mock_get):
        mock_response = MagicMock()
        mock_response.json.return_value = {"data": {"isLogin": False}}
        mock_get.return_value = mock_response

        d = Downloader()
        with pytest.raises(LoginFailError):
            d.check_login()

    @patch.object(requests.Session, "get")
    def test_login_no_data(self, mock_get):
        mock_response = MagicMock()
        mock_response.json.return_value = {}
        mock_get.return_value = mock_response

        d = Downloader()
        with pytest.raises(LoginFailError):
            d.check_login()
```

- [ ] **Step 6: 运行 check_login 测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestCheckLogin -v
```
Expected: FAILED (check_login not defined)

- [ ] **Step 7: 添加 check_login 方法到 Downloader**

```python
    API_USERINFO = "https://api.bilibili.com/x/web-interface/nav"

    def check_login(self) -> None:
        r = self.session.get(self.API_USERINFO).json()
        if not r.get("data", {}).get("isLogin"):
            raise LoginFailError("Cookie 登录验证失败")
```

- [ ] **Step 8: 运行 check_login 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestCheckLogin -v
```
Expected: 3 PASSED

- [ ] **Step 9: Commit**

```bash
git add tests/test_downloader.py bilidown/downloader.py
git commit -m "feat: add Downloader init, exception classes, and login check"
```

---

### Task 5: downloader.py — build_video（获取视频信息）

**Files:**
- Modify: `tests/test_downloader.py`
- Modify: `bilidown/downloader.py`

- [ ] **Step 1: 编写 build_video 测试（mock API 响应）**

```python
class TestBuildVideo:
    @patch.object(requests.Session, "get")
    def test_build_video_dash(self, mock_get):
        from bilidown.model import Video

        d = Downloader()
        # Mock responses for check_login and two API calls
        mock_get.return_value.json.side_effect = [
            {"data": {"isLogin": True}},
            {
                "data": {
                    "title": "测试视频/标题",
                    "cid": 12345,
                    "duration": 360,
                }
            },
            {
                "data": {
                    "dash": {
                        "video": [{"baseUrl": "https://example.com/v.m4s", "mimeType": "video/mp4"}],
                        "audio": [{"baseUrl": "https://example.com/a.m4s"}],
                    }
                }
            },
        ]
        # Mock the range GET request for content length
        mock_head_response = MagicMock()
        mock_head_response.headers = {"Content-Range": "bytes 0-1024/104857600"}
        mock_get.return_value = mock_head_response

        d = Downloader()
        video = d.build_video("BV1xx")

        assert video.bv == "BV1xx"
        assert video.cid == 12345
        assert video.title == "测试视频╱标题"
        assert video.format == "mp4"
        assert video.duration == 360
        assert video.content_len == 104857600
        assert video.audio_url == "https://example.com/a.m4s"
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestBuildVideo -v
```
Expected: FAILED (build_video not defined)

- [ ] **Step 3: 添加 API 常量和 build_video 到 Downloader**

```python
    API_INFO = "https://api.bilibili.com/x/web-interface/view?bvid="
    API_PLAY = "https://api.bilibili.com/x/player/playurl"

    def _extract_format(self, content_type: str) -> str:
        mapping = {
            "video/mp4": "mp4", "video/x-flv": "flv",
            "application/x-mpegURL": "m3u8", "video/MP2T": "ts",
            "video/3gpp": "3gpp", "video/quicktime": "mov",
            "video/x-msvideo": "avi", "video/x-ms-wmv": "wmv",
            "audio/x-wav": "wav", "audio/x-mp3": "mp3",
            "audio/mp4": "mp4", "application/ogg": "ogg",
            "image/jpeg": "jpeg", "image/png": "png",
            "image/tiff": "tiff", "image/gif": "gif",
            "image/svg+xml": "svg",
        }
        return mapping.get(content_type, "mp4")

    def build_video(self, bv: str):
        from .model import Video
        from .util import replace_illegal_chars_in_windows

        self.check_login()

        # Step 1: 获取视频基本信息
        info_url = f"{self.API_INFO}{bv}"
        info = self.session.get(info_url).json()["data"]
        title = replace_illegal_chars_in_windows(info.get("title", bv))
        cid = info["cid"]
        duration = info["duration"]

        # Step 2: 获取播放地址
        play = self.session.get(
            self.API_PLAY,
            params={"bvid": bv, "cid": cid, "fnval": "2000"},
        ).json()["data"]

        # Step 3: 解析 DASH 或 FLV
        if "dash" in play and play["dash"] is not None:
            video_data = play["dash"]["video"][0]
            video_url = video_data["baseUrl"]
            audio_url = play["dash"]["audio"][0]["baseUrl"]
            fmt = self._extract_format(video_data.get("mimeType", ""))

            r = self.session.get(video_url, headers={"Range": "bytes=0-1024"})
            content_range = r.headers.get("Content-Range", "")
            content_len = int(content_range.split("/")[-1]) if "/" in content_range else 0
        else:
            durl = play["durl"][0]
            video_url = durl["url"]
            audio_url = ""
            r = self.session.head(video_url)
            fmt = self._extract_format(r.headers.get("Content-Type", ""))
            content_len = int(r.headers.get("Content-Length", 0))

        return Video(
            bv=bv,
            cid=cid,
            video_url=video_url,
            audio_url=audio_url,
            title=title,
            format=fmt,
            duration=duration,
            content_len=content_len,
        )
```

- [ ] **Step 4: 运行 build_video 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestBuildVideo -v
```
Expected: 1 PASSED

- [ ] **Step 5: Commit**

```bash
git add tests/test_downloader.py bilidown/downloader.py
git commit -m "feat: add build_video method with DASH and FLV support"
```

---

### Task 6: downloader.py — 分块下载与音频下载

**Files:**
- Modify: `tests/test_downloader.py`
- Modify: `bilidown/downloader.py`

- [ ] **Step 1: 编写 download_chunks 测试**

```python
from concurrent.futures import ThreadPoolExecutor

class TestDownloadChunks:
    @patch.object(requests.Session, "get")
    def test_download_chunks(self, mock_get, tmp_path):
        from bilidown.model import Video

        # Mock response for chunk download
        mock_response = MagicMock()
        mock_response.iter_content.return_value = [b"a" * 1024, b"b" * 1024]
        mock_response.headers = {}
        mock_get.return_value = mock_response

        video = Video(
            bv="BV1xx", cid=1,
            video_url="https://example.com/v",
            audio_url="", title="test", format="mp4",
            duration=60, content_len=10 * 1024 * 1024 + 512,  # just over 10MB
        )

        d = Downloader(task_num=2)
        d.dir = str(tmp_path)
        chunk_count = d.download_chunks(video)

        assert chunk_count == 2
        # Verify files were created
        assert (tmp_path / "chunk_0").exists()
        assert (tmp_path / "chunk_1").exists()
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloadChunks -v
```
Expected: FAILED (download_chunks not defined)

- [ ] **Step 3: 添加 download_chunk 和 download_chunks 到 Downloader**

```python
    def download_chunk(self, video, range_tuple: tuple, index: int) -> None:
        start, end = range_tuple
        r = self.session.get(
            video.video_url,
            headers={"Range": f"bytes={start}-{end}"},
            stream=True,
        )
        from .util import write_bytes_to_file
        filepath = f"{self.dir}/chunk_{index}"
        offset = 0
        for chunk in r.iter_content(chunk_size=8192):
            if chunk:
                write_bytes_to_file(filepath, chunk, offset)
                offset += len(chunk)

    def download_chunks(self, video) -> int:
        from concurrent.futures import ThreadPoolExecutor, as_completed

        chunk_size = 10 * 1024 * 1024  # 10MB
        futures = []
        start = 0
        index = 0

        with ThreadPoolExecutor(max_workers=self.task_num) as executor:
            while start < video.content_len:
                end = min(start + chunk_size, video.content_len) - 1
                if end < start:
                    end = start
                f = executor.submit(self.download_chunk, video, (start, end), index)
                futures.append(f)
                start = end + 1
                index += 1

            for f in as_completed(futures):
                f.result()  # 传播异常

        return index
```

- [ ] **Step 4: 运行 download_chunks 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloadChunks -v
```
Expected: 1 PASSED

- [ ] **Step 5: 编写 download_audio 测试**

```python
class TestDownloadAudio:
    @patch.object(requests.Session, "get")
    def test_download_audio(self, mock_get, tmp_path):
        from bilidown.model import Video

        mock_response = MagicMock()
        mock_response.content = b"fake_audio_data"
        mock_get.return_value = mock_response

        video = Video(
            bv="BV1xx", cid=1,
            video_url="", audio_url="https://example.com/a",
            title="test", format="mp4", duration=60, content_len=1000,
        )

        d = Downloader()
        d.dir = str(tmp_path)
        d.download_audio(video)

        assert (tmp_path / "audio.mp3").exists()
        assert (tmp_path / "audio.mp3").read_bytes() == b"fake_audio_data"

    def test_download_audio_skip_when_empty(self, tmp_path):
        from bilidown.model import Video

        video = Video(
            bv="BV1xx", cid=1,
            video_url="", audio_url="",
            title="test", format="mp4", duration=60, content_len=1000,
        )

        d = Downloader()
        d.dir = str(tmp_path)
        d.download_audio(video)  # 不抛异常，不创建文件

        assert not (tmp_path / "audio.mp3").exists()
```

- [ ] **Step 6: 运行 download_audio 测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloadAudio -v
```
Expected: FAILED (download_audio not defined)

- [ ] **Step 7: 添加 download_audio 到 Downloader**

```python
    def download_audio(self, video) -> None:
        if not video.audio_url:
            return
        r = self.session.get(video.audio_url)
        filepath = f"{self.dir}/audio.mp3"
        with open(filepath, "wb") as f:
            f.write(r.content)
```

- [ ] **Step 8: 运行 download_audio 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloadAudio -v
```
Expected: 2 PASSED

- [ ] **Step 9: Commit**

```bash
git add tests/test_downloader.py bilidown/downloader.py
git commit -m "feat: add chunked download and audio download methods"
```

---

### Task 7: downloader.py — 合并视频、ffmpeg 混流、弹幕下载

**Files:**
- Modify: `tests/test_downloader.py`
- Modify: `bilidown/downloader.py`

- [ ] **Step 1: 编写 build_final_video 测试**

```python
class TestBuildFinalVideo:
    @patch("subprocess.run")
    def test_build_final_video(self, mock_run, tmp_path):
        from bilidown.model import Video

        # 创建模拟 chunk 文件
        (tmp_path / "chunk_0").write_bytes(b"aaaa")
        (tmp_path / "chunk_1").write_bytes(b"bbbb")
        # 创建音频文件（download_audio 会创建它）
        (tmp_path / "audio.mp3").write_bytes(b"audio")

        video = Video(
            bv="BV1xx", cid=1,
            video_url="", audio_url="https://example.com/a",
            title="test", format="mp4", duration=60, content_len=1000,
        )

        d = Downloader()
        d.dir = str(tmp_path)
        d.build_final_video(video, 2)

        # 验证 ffmpeg 被调用
        assert mock_run.called
        # 验证临时文件被清理
        assert not (tmp_path / "chunk_0").exists()
        assert not (tmp_path / "chunk_1").exists()
        assert not (tmp_path / "video.mp4").exists()
        assert not (tmp_path / "audio.mp3").exists()
        # 验证输出文件存在
        assert (tmp_path / "test.mp4").exists()
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestBuildFinalVideo -v
```
Expected: FAILED (build_final_video not defined)

- [ ] **Step 3: 添加 build_final_video 到 Downloader**

```python
    def build_final_video(self, video, chunk_count: int) -> None:
        import os
        from .util import mix_video_audio

        video_path = f"{self.dir}/video.{video.format}"

        # 合并所有 chunk
        with open(video_path, "wb") as out:
            for i in range(chunk_count):
                chunk_path = f"{self.dir}/chunk_{i}"
                with open(chunk_path, "rb") as f:
                    out.write(f.read())
                os.remove(chunk_path)

        audio_path = f"{self.dir}/audio.mp3"
        output_path = f"{self.dir}/{video.title}.{video.format}"

        if os.path.exists(audio_path) and os.path.getsize(audio_path) > 0:
            mix_video_audio(video_path, audio_path, output_path)
            os.remove(video_path)
            os.remove(audio_path)
        else:
            os.rename(video_path, output_path)
```

- [ ] **Step 4: 运行 build_final_video 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestBuildFinalVideo -v
```
Expected: 1 PASSED

- [ ] **Step 5: 编写 download_danmaku_segment 测试**

```python
class TestDownloadDanmaku:
    @patch.object(requests.Session, "get")
    def test_download_danmaku_segment(self, mock_get, tmp_path):
        from bilidown.model import Video
        from bilidown.danmaku_pb2 import DanmakuSegment, DanmakuElem

        # betterproto 生成的 dataclass，直接通过构造函数赋值
        elem = DanmakuElem(
            id=1,
            progress=1000,
            mode=1,
            fontsize=25,
            color=16777215,
            mid_hash="abc123",
            content="hello danmaku",
            ctime=1600000000,
        )
        seg = DanmakuSegment(elems=[elem])
        mock_response = MagicMock()
        mock_response.content = bytes(seg)
        mock_get.return_value = mock_response

        video = Video(
            bv="BV1xx", cid=1,
            video_url="", audio_url="",
            title="test", format="mp4", duration=360, content_len=1000,
        )

        d = Downloader()
        d.dir = str(tmp_path)
        d.download_danmaku(video)

        danmaku_file = tmp_path / "danmuku.txt"
        assert danmaku_file.exists()
        content = danmaku_file.read_text().strip().split("\n")
        assert len(content) == 1
        assert "hello danmaku" in content[0]
```

- [ ] **Step 6: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloadDanmaku -v
```
Expected: FAILED (download_danmaku not defined)

- [ ] **Step 7: 添加 download_danmaku_segment 和 download_danmaku 到 Downloader**

```python
    API_BULLET = "http://api.bilibili.com/x/v2/dm/web/seg.so"

    def download_danmaku_segment(self, video, seg_index: int):
        from .danmaku_pb2 import DanmakuSegment
        r = self.session.get(
            self.API_BULLET,
            params={"oid": video.cid, "segment_index": seg_index, "type": 1},
        )
        segment = DanmakuSegment().parse(r.content)
        return segment

    def download_danmaku(self, video) -> None:
        from concurrent.futures import ThreadPoolExecutor, as_completed
        import json

        bags = (video.duration + 359) // 360  # 每 6 分钟一个 segment
        segments = []

        with ThreadPoolExecutor(max_workers=self.task_num) as executor:
            futures = {
                executor.submit(self.download_danmaku_segment, video, i + 1): i
                for i in range(bags)
            }
            for f in as_completed(futures):
                segments.append(f.result())

        with open(f"{self.dir}/danmuku.txt", "w", encoding="utf-8") as f:
            for seg in segments:
                for elem in seg.elems:
                    d = elem.to_dict()
                    f.write(json.dumps(d, ensure_ascii=False) + "\n")
```

- [ ] **Step 8: 运行 download_danmaku 测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestDownloadDanmaku -v
```
Expected: 1 PASSED

- [ ] **Step 9: Commit**

```bash
git add tests/test_downloader.py bilidown/downloader.py
git commit -m "feat: add video merge, ffmpeg mixing, and danmaku download"
```

---

### Task 8: downloader.py — run 编排方法

**Files:**
- Modify: `tests/test_downloader.py`
- Modify: `bilidown/downloader.py`

- [ ] **Step 1: 编写 run 方法集成测试**

```python
class TestRun:
    @patch.object(requests.Session, "get")
    @patch("subprocess.run")
    def test_run_full_flow(self, mock_ffmpeg, mock_get, tmp_path):
        from bilidown.danmaku_pb2 import DanmakuSegment, DanmakuElem
        from bilidown.model import Video

        # 构造完整 mock 响应链
        mock_response = MagicMock()

        # check_login 响应
        mock_response.json.side_effect = None
        mock_response.json.return_value = {"data": {"isLogin": True}}

        def mock_response_for_range(*args, **kwargs):
            r = MagicMock()
            if kwargs.get("stream"):
                r.iter_content.return_value = [b"x" * 100]
            if kwargs.get("headers", {}).get("Range") == "bytes=0-1024":
                r.headers = {"Content-Range": "bytes 0-1024/5000000"}
            return r

        mock_get.side_effect = mock_response_for_range

        # 简短测试流：手动创建 chunk 和 audio
        (tmp_path / "chunk_0").write_bytes(b"video_data")
        (tmp_path / "audio.mp3").write_bytes(b"audio_data")

        mock_ffmpeg.return_value = MagicMock(returncode=0)

        # 直接测试 build_final_video 单独步骤（最终组合在 run 中）
        video = Video(
            bv="BV1xx", cid=1,
            video_url="https://example.com/v",
            audio_url="https://example.com/a",
            title="测试", format="mp4", duration=360, content_len=5000000,
        )

        d = Downloader(task_num=2)
        d.dir = str(tmp_path)
        d.build_final_video(video, 1)

        assert (tmp_path / "测试.mp4").exists()
```

- [ ] **Step 2: 运行测试确认失败**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestRun -v
```
Expected: FAILED (run not defined)

- [ ] **Step 3: 添加 run 方法到 Downloader**

```python
    def run(self, bv: str) -> None:
        import os

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

        chunk_count = self.download_chunks(video)
        self.download_audio(video)
        self.build_final_video(video, chunk_count)
        self.download_danmaku(video)
        print(f'download {video.bv} finished')
```

- [ ] **Step 4: 运行测试确认通过**

```bash
.venv/Scripts/python -m pytest tests/test_downloader.py::TestRun -v
```
Expected: 1 PASSED

- [ ] **Step 5: Commit**

```bash
git add tests/test_downloader.py bilidown/downloader.py
git commit -m "feat: add run orchestrator method"
```

---

### Task 9: __main__.py — CLI 入口

**Files:**
- Create: `bilidown/__main__.py`

- [ ] **Step 1: 编写 `bilidown/__main__.py`**

```python
import argparse
import sys

from .downloader import Downloader


def main():
    parser = argparse.ArgumentParser(
        description="Bilibili Video Downloader",
    )
    parser.add_argument(
        "bv",
        help="Bilibili video BV id",
    )
    parser.add_argument(
        "-t", "--tasknum",
        type=int,
        default=7,
        help="Async task num for downloader (max 10)",
    )
    args = parser.parse_args()

    if args.tasknum > 10:
        print("task num over 10, please use 1 ~ 10 instead")
        sys.exit(1)

    if not args.bv:
        print("bv id is empty!")
        sys.exit(1)

    try:
        downloader = Downloader(task_num=args.tasknum)
        downloader.run(args.bv)
    except Exception as e:
        print(f"error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: 验证 CLI --help 输出**

```bash
.venv/Scripts/python -m bilidown --help
```
Expected: usage 信息

- [ ] **Step 3: 验证无效参数**

```bash
.venv/Scripts/python -m bilidown -t 20 BVxxx
```
Expected: "task num over 10, please use 1 ~ 10 instead"

- [ ] **Step 4: Commit**

```bash
git add bilidown/__main__.py
git commit -m "feat: add CLI entry point with argparse"
```

---

### Task 10: 更新 `__init__.py` 导出和最终验证

**Files:**
- Modify: `bilidown/__init__.py`

- [ ] **Step 1: 更新 `__init__.py`**

```python
from .downloader import Downloader, DownloadError, LoginFailError, VideoInfoError, DanmakuError
from .model import Video

__version__ = "0.1.0"
```

- [ ] **Step 2: 运行全量测试**

```bash
.venv/Scripts/python -m pytest tests/ -v
```
Expected: ALL TESTS PASS

- [ ] **Step 3: Commit**

```bash
git add bilidown/__init__.py
git commit -m "chore: update __init__.py exports"
```
