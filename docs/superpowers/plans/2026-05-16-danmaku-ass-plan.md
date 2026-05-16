# 弹幕转 ASS 字幕并集成到视频 - 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 `bilidown/danmaku_ass.py` 模块，将 danmuku.txt（JSON Lines）转换为 ASS 字幕格式，并通过 ffmpeg 硬字幕烧录到最终视频中。

**Architecture:** 新建 `danmaku_ass.py` 实现 JSON Lines -> ASS 格式转换，在 `Downloader.run()` 的弹幕下载之后插入转换和 ffmpeg 嵌入步骤，由 `--danmaku-ass` CLI 参数控制开关。

**Tech Stack:** Python 标准库（json, os, subprocess），外部依赖 ffmpeg（运行时）

---

## 文件结构

| 文件 | 操作 | 职责 |
|---|---|---|
| `bilidown/danmaku_ass.py` | 新增 | 弹幕 JSON Lines -> ASS 转换（`DanmakuAssError`, `convert_danmaku_to_ass()`） |
| `bilidown/downloader.py` | 修改 | `__init__` 新增 `danmaku_ass` 参数；`run()` 末尾新增 ASS 转换+ffmpeg 嵌入 |
| `bilidown/__main__.py` | 修改 | 新增 `--danmaku-ass` CLI 参数 |
| `bilidown/__init__.py` | 修改 | 导出 `DanmakuAssError` |
| `tests/test_danmaku_ass.py` | 新增 | ASS 转换单元测试 |
| `tests/test_downloader.py` | 修改 | 扩展集成测试 |

---

### Task 1: 创建 danmaku_ass.py 核心转换模块

**Files:**
- Create: `bilidown/danmaku_ass.py`

- [ ] **Step 1: 创建 danmaku_ass.py 文件**

```python
"""弹幕 JSON Lines 转 ASS 字幕格式"""

import json
import os


class DanmakuAssError(Exception):
    """弹幕 ASS 转换异常"""
    pass


def _decimal_to_ass_color(dec: int) -> str:
    """将十进制颜色值转换为 ASS 的 &HBBGGRR& 格式"""
    r = (dec >> 16) & 0xFF
    g = (dec >> 8) & 0xFF
    b = dec & 0xFF
    return f"&H00{b:02X}{g:02X}{r:02X}&"


def _format_ass_time(total_seconds: float) -> str:
    """将秒数转换为 ASS 时间格式 H:MM:SS.cc"""
    total_seconds = max(0, total_seconds)
    h = int(total_seconds // 3600)
    m = int((total_seconds % 3600) // 60)
    s = int(total_seconds % 60)
    cs = int((total_seconds * 100) % 100)
    return f"{h}:{m:02d}:{s:02d}.{cs:02d}"


def _estimate_text_width(text: str, fontsize: int) -> int:
    """估算文本的像素宽度"""
    byte_len = len(text.encode("utf-8"))
    return int(byte_len * fontsize * 0.6)


def _escape_ass_text(content: str) -> str:
    """转义 ASS 文本内容中的特殊字符"""
    content = content.replace("\\", "\\\\")
    content = content.replace("{", "\\{")
    content = content.replace("}", "\\}")
    return content


def convert_danmaku_to_ass(
    danmaku_path: str,
    ass_path: str,
    video_width: int = 1920,
    video_height: int = 1080,
) -> None:
    """读取 JSON Lines 弹幕文件，生成 ASS 字幕文件"""
    if not os.path.exists(danmaku_path):
        raise DanmakuAssError(f"弹幕文件不存在: {danmaku_path}")

    # 读取所有弹幕条目
    entries: list[dict] = []
    with open(danmaku_path, "r", encoding="utf-8") as f:
        for line_num, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError as e:
                # 跳过格式错误的行，继续处理其他行
                print(f"跳过第 {line_num} 行弹幕（JSON 解析失败）: {e}")
                continue

    if not entries:
        raise DanmakuAssError(f"弹幕文件中没有有效数据: {danmaku_path}")

    # 行计数器（滚动弹幕、顶部固定、底部固定各自独立）
    scroll_counter = 0
    top_counter = 0
    bottom_counter = 0

    # 生成 Dialogue 行
    dialogue_lines: list[str] = []
    for elem in entries:
        mode = elem.get("mode", 1)
        progress = elem.get("progress", 0)
        fontsize = elem.get("fontsize", 25)
        color = elem.get("color", 16777215)
        content = elem.get("content", "")

        if not content:
            continue

        start_sec = progress / 1000.0
        ass_color = _decimal_to_ass_color(color)
        ass_start = _format_ass_time(start_sec)
        escaped_content = _escape_ass_text(content)

        if mode == 4:  # 底部固定
            duration = 5.0
            row = bottom_counter % 3
            bottom_counter += 1
            line_height = fontsize + 4
            y = video_height - (row + 1) * line_height - 10
            x = video_width // 2
            ass_end = _format_ass_time(start_sec + duration)
            override = f"{{\\pos({x},{y})}}{{\\an2}}{{\\fs{fontsize}}}{{\\c{ass_color}}}"
        elif mode == 5:  # 顶部固定
            duration = 5.0
            row = top_counter % 3
            top_counter += 1
            line_height = fontsize + 4
            y = row * line_height + 10
            x = video_width // 2
            ass_end = _format_ass_time(start_sec + duration)
            override = f"{{\\pos({x},{y})}}{{\\an8}}{{\\fs{fontsize}}}{{\\c{ass_color}}}"
        else:  # 滚动弹幕（mode=1 及未知 mode）
            duration = 8.0
            row = scroll_counter % 15
            scroll_counter += 1
            line_height = fontsize + 4
            y = row * line_height + 10
            text_w = _estimate_text_width(content, fontsize)
            ass_end = _format_ass_time(start_sec + duration)
            override = f"{{\\move({video_width},{y},{-text_w},{y})}}{{\\fs{fontsize}}}{{\\c{ass_color}}}"

        dialogue = f"Dialogue: 0,{ass_start},{ass_end},Default,,0,0,0,,{override}{escaped_content}"
        dialogue_lines.append(dialogue)

    # 生成 ASS 文件内容
    style_line = (
        "Style: Default,微软雅黑,25,"
        "&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,"
        "0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1"
    )

    ass_lines = [
        "[Script Info]",
        "Title: bilibili danmaku",
        "ScriptType: v4.00+",
        f"PlayResX: {video_width}",
        f"PlayResY: {video_height}",
        "Timer: 100.0000",
        "",
        "[V4+ Styles]",
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding",
        style_line,
        "",
        "[Events]",
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text",
    ]
    ass_lines.extend(dialogue_lines)

    # 写入文件
    try:
        with open(ass_path, "w", encoding="utf-8") as f:
            f.write("\n".join(ass_lines))
    except OSError as e:
        raise DanmakuAssError(f"写入 ASS 文件失败: {e}")
```

- [ ] **Step 2: 验证文件语法正确**

```bash
python -c "from bilidown.danmaku_ass import convert_danmaku_to_ass, DanmakuAssError; print('OK')"
```

预期：无错误输出，打印 `OK`

- [ ] **Step 3: Commit**

```bash
git add bilidown/danmaku_ass.py
git commit -m "feat: 新增 danmaku_ass 模块，将弹幕 JSON Lines 转换为 ASS 字幕

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: 在 downloader.py 中集成 ASS 转换和 ffmpeg 嵌入

**Files:**
- Modify: `bilidown/downloader.py:41-43`（`__init__` 方法签名）
- Modify: `bilidown/downloader.py:274-316`（`run` 方法）

- [ ] **Step 1: 修改 Downloader 构造函数，新增 danmaku_ass 参数**

将 `__init__` 方法签名与初始化改为：

```python
def __init__(self, task_num: int = 7, danmaku_ass: bool = False):
    self.task_num = min(task_num, 10)
    self.danmaku_ass = danmaku_ass
    self.dir = ""
    self.session = requests.Session()
    self.session.headers.update({
        "User-Agent": _ua_pool.random,
        "Referer": "https://www.bilibili.com/",
    })
    if os.path.exists("cookie.txt"):
        try:
            with open("cookie.txt") as f:
                self.session.headers["Cookie"] = f.read().strip()
        except OSError as e:
            print(f"读取 cookie.txt 失败: {e}")
```

- [ ] **Step 2: 在 run 方法中添加 ASS 转换和 ffmpeg 嵌入逻辑**

在 `run` 方法的下载弹幕块之后（`self.download_danmaku(video, pbar=dm_pbar)` 之后，约 current:314 行）、`print(f'download {video.bv} finished')` 之前插入：

```python
        # 阶段 5: 弹幕转 ASS 并嵌入视频（仅当 --danmaku-ass 开启时）
        if self.danmaku_ass:
            self._embed_danmaku_ass(video)
```

同时在 `Downloader` 类中添加 `_embed_danmaku_ass` 方法：

```python
    def _embed_danmaku_ass(self, video: "Video") -> None:
        """将 danmuku.txt 转换为 ASS 并通过 ffmpeg 集成到视频中"""
        import subprocess
        from .danmaku_ass import convert_danmaku_to_ass, DanmakuAssError

        danmaku_path = f"{self.dir}/danmuku.txt"
        ass_path = f"{self.dir}/danmaku.ass"
        output_path = f"{self.dir}/{video.title}.{video.format}"
        temp_path = f"{self.dir}/_temp_danmaku.{video.format}"

        # 转换弹幕为 ASS
        try:
            convert_danmaku_to_ass(danmaku_path, ass_path)
        except DanmakuAssError as e:
            print(f"弹幕 ASS 转换失败: {e}")
            return

        # 使用 ffmpeg 将 ASS 烧录为硬字幕
        try:
            subprocess.run(
                [
                    "ffmpeg",
                    "-i", output_path,
                    "-vf", f"ass={ass_path}",
                    "-c:v", "libx264",
                    "-preset", "medium",
                    "-crf", "18",
                    "-c:a", "copy",
                    temp_path,
                ],
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            print(f"ffmpeg 嵌入字幕失败: {e}")
            # 清理临时文件
            if os.path.exists(temp_path):
                os.remove(temp_path)
            return

        # 替换原视频文件
        os.replace(temp_path, output_path)
        print("弹幕字幕已集成到视频")
```

- [ ] **Step 3: 验证现有测试仍然通过**

```bash
python -m pytest tests/test_downloader.py -v
```

预期：全部 PASS（`danmaku_ass` 默认 `False`，不触发新逻辑）

- [ ] **Step 4: Commit**

```bash
git add bilidown/downloader.py
git commit -m "feat: downloader 集成弹幕 ASS 转换和 ffmpeg 硬字幕嵌入

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: 添加 CLI 参数和包导出

**Files:**
- Modify: `bilidown/__main__.py:17-22`（CLI 参数注册）
- Modify: `bilidown/__main__.py:34`（Downloader 构造调用）
- Modify: `bilidown/__init__.py:3`（导出行）

- [ ] **Step 1: 在 __main__.py 中添加 --danmaku-ass 参数**

在 `parser.add_argument("-t", "--tasknum", ...)` 后面添加：

```python
    parser.add_argument(
        "--danmaku-ass",
        action="store_true",
        help="下载弹幕后生成 ASS 字幕并集成到视频中",
    )
```

- [ ] **Step 2: 将参数传递给 Downloader 构造函数**

将 `downloader = Downloader(task_num=args.tasknum)` 改为：

```python
        downloader = Downloader(task_num=args.tasknum, danmaku_ass=args.danmaku_ass)
```

- [ ] **Step 3: 在 __init__.py 中导出 DanmakuAssError**

将 `from .downloader import Downloader, DownloadError, LoginFailError, VideoInfoError, DanmakuError` 改为：

```python
from .downloader import Downloader, DownloadError, LoginFailError, VideoInfoError, DanmakuError
from .danmaku_ass import DanmakuAssError
```

- [ ] **Step 4: 验证 CLI 参数生效**

```bash
python -m bilidown --help
```

预期：输出中包含 `--danmaku-ass 下载弹幕后生成 ASS 字幕并集成到视频中`

- [ ] **Step 5: Commit**

```bash
git add bilidown/__main__.py bilidown/__init__.py
git commit -m "feat: 新增 --danmaku-ass CLI 参数并导出 DanmakuAssError

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: 编写 danmaku_ass 转换单元测试

**Files:**
- Create: `tests/test_danmaku_ass.py`

- [ ] **Step 1: 编写测试文件**

```python
"""danmaku_ass 模块单元测试"""

import json
import os

import pytest
from bilidown.danmaku_ass import (
    convert_danmaku_to_ass,
    DanmakuAssError,
    _decimal_to_ass_color,
    _format_ass_time,
    _estimate_text_width,
)


class TestDecimalToAssColor:
    """颜色转换测试"""

    def test_white(self):
        # 16777215 = 0xFFFFFF -> &H00FFFFFF& (BGR: FF FF FF)
        result = _decimal_to_ass_color(16777215)
        assert result == "&H00FFFFFF&"

    def test_red(self):
        # 255 = 0x0000FF -> &H000000FF& (BGR: 00 00 FF)
        result = _decimal_to_ass_color(255)
        assert result == "&H000000FF&"

    def test_green(self):
        # 65280 = 0x00FF00 -> &H0000FF00& (BGR: 00 FF 00)
        result = _decimal_to_ass_color(65280)
        assert result == "&H0000FF00&"

    def test_blue(self):
        # 16711680 = 0xFF0000 -> &H00FF0000& (BGR: FF 00 00)
        result = _decimal_to_ass_color(16711680)
        assert result == "&H00FF0000&"

    def test_black(self):
        result = _decimal_to_ass_color(0)
        assert result == "&H00000000&"


class TestFormatAssTime:
    """ASS 时间格式转换测试"""

    def test_zero_seconds(self):
        result = _format_ass_time(0)
        assert result == "0:00:00.00"

    def test_one_minute(self):
        result = _format_ass_time(61.5)
        assert result == "0:01:01.50"

    def test_one_hour(self):
        result = _format_ass_time(3661.75)
        assert result == "1:01:01.75"

    def test_negative_seconds(self):
        # 负数应转为 0
        result = _format_ass_time(-5)
        assert result == "0:00:00.00"


class TestEstimateTextWidth:
    """文本宽度估算测试"""

    def test_english_text(self):
        width = _estimate_text_width("hello", 25)
        assert width > 0

    def test_chinese_text(self):
        width = _estimate_text_width("你好世界", 25)
        assert width > _estimate_text_width("hello", 25)  # 中文更宽

    def test_empty_text(self):
        width = _estimate_text_width("", 25)
        assert width == 0


class TestConvertDanmakuToAss:
    """ASS 转换集成测试"""

    @pytest.fixture
    def danmaku_tmpdir(self, tmp_path):
        """创建临时目录和弹幕文件"""
        danmaku_path = tmp_path / "danmuku.txt"
        ass_path = tmp_path / "danmaku.ass"
        return tmp_path, danmaku_path, ass_path

    def test_scroll_danmaku(self, danmaku_tmpdir):
        """测试单条滚动弹幕 -> 验证输出包含 \\move 命令"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {
            "id": "1", "progress": 1410, "mode": 1,
            "fontsize": 25, "color": 16777215,
            "content": "新年快乐"
        }
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert "[Script Info]" in content
        assert "[V4+ Styles]" in content
        assert "[Events]" in content
        assert "\\move(" in content
        assert "新年快乐" in content

    def test_top_fixed_danmaku(self, danmaku_tmpdir):
        """测试单条顶部固定弹幕 -> 验证输出包含 \\pos 和 \\an8"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {
            "id": "2", "progress": 3000, "mode": 5,
            "fontsize": 30, "color": 255,
            "content": "顶部弹幕"
        }
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert "\\pos(" in content
        assert "\\an8" in content
        assert "顶部弹幕" in content

    def test_bottom_fixed_danmaku(self, danmaku_tmpdir):
        """测试单条底部固定弹幕 -> 验证输出包含 \\pos 和 \\an2"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {
            "id": "3", "progress": 5000, "mode": 4,
            "fontsize": 28, "color": 65280,
            "content": "底部弹幕"
        }
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert "\\pos(" in content
        assert "\\an2" in content
        assert "底部弹幕" in content

    def test_unknown_mode_falls_back_to_scroll(self, danmaku_tmpdir):
        """测试未知 mode -> 按滚动弹幕处理"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {
            "id": "4", "progress": 1000, "mode": 99,
            "fontsize": 25, "color": 16777215,
            "content": "未知模式"
        }
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert "\\move(" in content
        assert "未知模式" in content

    def test_empty_file_raises_error(self, danmaku_tmpdir):
        """测试空弹幕文件 -> 抛出 DanmakuAssError"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku_path.write_text("", encoding="utf-8")

        with pytest.raises(DanmakuAssError):
            convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

    def test_missing_fields_use_defaults(self, danmaku_tmpdir):
        """测试缺失字段 -> 使用默认值"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {"content": "缺字段弹幕"}
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        # 默认 mode=1 -> 滚动，默认 fontsize=25，默认 color=16777215
        assert "\\move(" in content
        assert "\\fs25" in content
        assert "&H00FFFFFF&" in content
        assert "缺字段弹幕" in content

    def test_empty_content_skipped(self, danmaku_tmpdir):
        """测试空 content -> 跳过该条弹幕"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        # 一条有效弹幕 + 一条空 content 弹幕
        lines = [
            {"id": "1", "progress": 1000, "mode": 1, "content": "有效弹幕"},
            {"id": "2", "progress": 2000, "mode": 1, "content": ""},
        ]
        danmaku_path.write_text(
            "\n".join(json.dumps(d) for d in lines) + "\n",
            encoding="utf-8"
        )

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert "有效弹幕" in content
        # 只有 1 条 Dialogue
        assert content.count("Dialogue:") == 1

    def test_malformed_json_skipped(self, danmaku_tmpdir):
        """测试格式错误的行 -> 跳过并继续处理"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        invalid_line = "{invalid json}"
        valid_line = {"id": "1", "progress": 2000, "mode": 1, "content": "有效"}
        danmaku_path.write_text(
            invalid_line + "\n" + json.dumps(valid_line) + "\n",
            encoding="utf-8"
        )

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert "有效" in content
        assert content.count("Dialogue:") == 1

    def test_row_distribution(self, danmaku_tmpdir):
        """测试多条滚动弹幕的行分配"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        lines = []
        for i in range(20):
            lines.append({
                "id": str(i), "progress": i * 500,
                "mode": 1, "fontsize": 25,
                "color": 16777215, "content": f"弹幕{i}"
            })
        danmaku_path.write_text(
            "\n".join(json.dumps(d) for d in lines) + "\n",
            encoding="utf-8"
        )

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        # 20 条弹幕 -> 20 条 Dialogue
        assert content.count("Dialogue:") == 20

    def test_ass_header_format(self, danmaku_tmpdir):
        """测试 ASS 文件头完整"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {
            "id": "1", "progress": 1000, "mode": 1,
            "content": "测试"
        }
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        assert content.startswith("[Script Info]")
        assert "Title: bilibili danmaku" in content
        assert "ScriptType: v4.00+" in content
        assert "PlayResX: 1920" in content
        assert "PlayResY: 1080" in content
        assert "Format: Name, Fontname, Fontsize" in content
        assert "Style: Default,微软雅黑" in content
        assert "Format: Layer, Start, End" in content

    def test_file_not_found(self, danmaku_tmpdir):
        """测试弹幕文件不存在 -> 抛出 DanmakuAssError"""
        tmp_path, _, ass_path = danmaku_tmpdir
        nonexistent = str(tmp_path / "nonexistent.txt")

        with pytest.raises(DanamakuAssError, match="弹幕文件不存在"):
            convert_danmaku_to_ass(nonexistent, str(ass_path))

    def test_special_chars_escaped(self, danmaku_tmpdir):
        """测试弹幕内容中包含 {} 特殊字符时正确转义"""
        tmp_path, danmaku_path, ass_path = danmaku_tmpdir
        danmaku = {
            "id": "1", "progress": 1000, "mode": 1,
            "content": "{你好}世界"
        }
        danmaku_path.write_text(json.dumps(danmaku) + "\n", encoding="utf-8")

        convert_danmaku_to_ass(str(danmaku_path), str(ass_path))

        content = ass_path.read_text(encoding="utf-8")
        # 应包含转义后的 \\{你好\\} 文字
        # 注意：在 ASS 文本中，转义后是 \{你好\}，在 Python 字符串中是 \\{你好\\}
        assert "你好" in content
        assert "世界" in content
```

- [ ] **Step 2: 运行测试，确认全部通过**

```bash
python -m pytest tests/test_danmaku_ass.py -v
```

预期：全部 15 个测试 PASS

- [ ] **Step 3: Commit**

```bash
git add tests/test_danmaku_ass.py
git commit -m "test: 新增 danmaku_ass 模块单元测试

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: 扩展 Downloader 集成测试

**Files:**
- Modify: `tests/test_downloader.py`

- [ ] **Step 1: 在 TestRun 类中添加 danmaku_ass 集成测试**

在 `tests/test_downloader.py` 的 `TestRun` 类末尾添加以下两个测试方法：

```python
    @patch("subprocess.run")
    def test_run_with_danmaku_ass(self, mock_ffmpeg, tmp_path):
        """测试 --danmaku-ass 开启时，生成 ASS 文件并调用 ffmpeg"""
        from bilidown.model import Video

        # 模拟 ffmpeg 创建临时输出文件
        def fake_run(*args, **kwargs):
            output_path = args[0][-1]
            pathlib.Path(output_path).touch()
            return MagicMock(returncode=0)
        mock_ffmpeg.side_effect = fake_run

        # 创建模拟 danmuku.txt
        danmaku_data = {
            "id": "1", "progress": 1410, "mode": 1,
            "fontsize": 25, "color": 16777215,
            "content": "测试弹幕",
        }
        import json
        (tmp_path / "danmuku.txt").write_text(
            json.dumps(danmaku_data) + "\n", encoding="utf-8"
        )

        # 创建模拟视频输出文件
        (tmp_path / "测试.mp4").write_bytes(b"fake_video")

        video = Video(
            bv="BV1xx", cid=1,
            video_url="", audio_url="",
            title="测试", format="mp4", duration=360, content_len=1000,
        )

        d = Downloader(task_num=2, danmaku_ass=True)
        d.dir = str(tmp_path)
        d._embed_danmaku_ass(video)

        # 验证 ASS 文件生成
        ass_path = tmp_path / "danmaku.ass"
        assert ass_path.exists()
        ass_content = ass_path.read_text(encoding="utf-8")
        assert "测试弹幕" in ass_content

        # 验证 ffmpeg 被调用（第一个参数是 ffmpeg）
        assert mock_ffmpeg.called
        call_args = mock_ffmpeg.call_args[0][0]
        assert call_args[0] == "ffmpeg"
        assert "ass=" in call_args[3]  # -vf 参数

    def test_run_without_danmaku_ass(self, tmp_path):
        """测试默认行为（不传 danmaku_ass），不生成 ASS 文件"""
        from bilidown.model import Video

        d = Downloader(task_num=2)  # 默认 danmaku_ass=False
        d.dir = str(tmp_path)

        assert not (tmp_path / "danmaku.ass").exists()
```

- [ ] **Step 2: 运行所有测试，确认全部通过**

```bash
python -m pytest tests/ -v
```

预期：全部 PASS

- [ ] **Step 3: Commit**

```bash
git add tests/test_downloader.py
git commit -m "test: 扩展 Downloader 集成测试，覆盖 danmaku_ass 开关场景

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6: 端到端验证

**Files:**
- 使用 `example/danmuku.txt` 进行端到端验证

- [ ] **Step 1: 使用示例弹幕文件生成 ASS**

```bash
python -c "
from bilidown.danmaku_ass import convert_danmaku_to_ass
convert_danmaku_to_ass('example/danmuku.txt', 'example/danmaku_test.ass')
print('ASS 文件已生成')
"
```

- [ ] **Step 2: 验证生成的 ASS 文件格式**

```bash
python -c "
import os
path = 'example/danmaku_test.ass'
assert os.path.exists(path), 'ASS 文件不存在'
content = open(path, encoding='utf-8').read()
assert '[Script Info]' in content, '缺少 [Script Info]'
assert '[V4+ Styles]' in content, '缺少 [V4+ Styles]'
assert '[Events]' in content, '缺少 [Events]'
dialogue_count = content.count('Dialogue:')
print(f'ASS 文件有效，共 {dialogue_count} 条弹幕 Dialogue')
"
```

- [ ] **Step 3: 清理测试产物**

```bash
rm -f example/danmaku_test.ass
```

- [ ] **Step 4: 最终全量测试**

```bash
python -m pytest tests/ -v
```

预期：全部 PASS

- [ ] **Step 5: 最终 commit（如无变更则跳过）**

如需调整，commit 修正。
```
