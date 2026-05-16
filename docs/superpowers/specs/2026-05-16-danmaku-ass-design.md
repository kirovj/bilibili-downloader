# 弹幕转 ASS 字幕并集成到视频 - 设计文档

## 目标

在弹幕下载完成后，将 danmuku.txt（JSON Lines）转换为 ASS 字幕格式，并通过 ffmpeg 硬字幕方式集成到最终视频中。

## 触发方式

通过 CLI 参数 `--danmaku-ass` 可选开启。不传则保持原有行为（只保存 danmuku.txt）。

## 模块架构

新增 `bilidown/danmaku_ass.py` 模块，职责单一：读取 danmuku.txt -> 生成 ASS 字幕文件。

```
bilidown/
├── danmaku_ass.py       # [新增] 弹幕 JSON Lines -> ASS 格式转换
├── downloader.py        # [修改] run() 中调用 danmaku_ass + ffmpeg 嵌入
├── __main__.py          # [修改] 新增 --danmaku-ass 参数
└── ...
```

## 模块接口

```python
# danmaku_ass.py

class DanmakuAssError(Exception):
    """弹幕 ASS 转换异常"""
    pass

def convert_danmaku_to_ass(
    danmaku_path: str,     # danmuku.txt 路径
    ass_path: str,         # 输出的 .ass 文件路径
    video_width: int = 1920,
    video_height: int = 1080,
) -> None:
    """读取 JSON Lines 弹幕文件，生成 ASS 字幕文件"""
    ...
```

## 文件产物（`--danmaku-ass` 开启时）

```
{title}_{bv}/
├── video.mp4          # 最终视频（含硬字幕弹幕）
├── danmuku.txt        # 原始弹幕 JSON Lines（保留）
└── danmaku.ass        # 生成的 ASS 字幕文件（保留）
```

---

## ASS 格式设计

### 弹幕模式映射

| 弹幕 mode | 含义 | ASS 实现 |
|---|---|---|
| 1 | 滚动弹幕 | `\move(x1,y1,x2,y2)` 从右边缘移动到左边缘 |
| 4 | 底部固定 | `\pos(x,y)` + `\an2` 底部居中，持续 5 秒 |
| 5 | 顶部固定 | `\pos(x,y)` + `\an8` 顶部居中，持续 5 秒 |
| 未知 mode | 兜底 | 按滚动弹幕处理 |

### 滚动弹幕布局

- 使用轮询分行策略，共 15 行可用
- 第 i 条滚动弹幕分配到 `row = i % 15`
- 行高 = fontsize + 4px
- Y 坐标 = `row * line_height + 10`（顶部留 10px 边距）
- 从右边缘移动到文本完全超出左边缘：`\move(video_width, y, -text_width_estimate, y)`
- 文本宽度估算：`len(content.encode('utf-8')) * fontsize * 0.6`（按字符宽度近似计算）
- 移动时长固定 8 秒（标准速度）
- 开始时间 = `progress / 1000`（毫秒转秒），结束时间 = 开始时间 + 8

### 固定弹幕布局

- 顶部固定和底部固定使用各自独立的行计数器
- 顶部固定：Y 坐标从 10px 开始，`row = top_counter % 3`，`Y = row * line_height + 10`
- 底部固定：`row = bottom_counter % 3`，`Y = video_height - (row + 1) * line_height - 10`
- 停留时长固定 5 秒
- 水平居中：`X = video_width / 2`
- 使用 `\an8`（顶部居中）或 `\an2`（底部居中）对齐
- 开始时间 = `progress / 1000`，结束时间 = 开始时间 + 5

### 颜色转换

danmuku.txt 中的 `color` 为十进制整数，需转换为 ASS 的 `&HBBGGRR` 十六进制格式（BGR 低位在前）。

```
16777215 (白色) -> 0xFFFFFF -> &HFFFFFF&
```

`&H` 前缀 + 6 位十六进制 BGR + `&` 后缀。

### 字号

直接使用 danmuku.txt 中的 `fontsize` 值。

### 字体

ASS 头部 `[V4+ Styles]` 中默认字体设为 `微软雅黑`。

### 缺省值处理

| 缺失字段 | 默认值 |
|---|---|
| progress | 0 |
| fontsize | 25 |
| color | 16777215（白色） |

---

## ffmpeg 嵌入设计

### 硬字幕嵌入命令

```bash
ffmpeg -i video.mp4 -vf "ass=danmaku.ass" \
  -c:v libx264 -preset medium -crf 18 -c:a copy \
  temp_output.mp4
```

参数说明：
- `ass=danmaku.ass`：烧录 ASS 字幕到视频画面
- `-c:v libx264`：重新编码视频流
- `-crf 18`：高品质编码，接近无损
- `-c:a copy`：音频流直接复制，不重新编码

成功后将临时文件重命名覆盖原 `video.mp4`，删除临时文件。

### 集成位置（Downloader.run() 内）

```
download_danmaku(video)           # 已有，生成 danmuku.txt
    │
    ▼
if self.danmaku_ass:              # [新增] 条件判断
    convert_danmaku_to_ass()      # [新增] 转换 ASS
    调用 ffmpeg 嵌入字幕          # [新增] 烧录到视频
```

---

## 错误处理

核心原则：**字幕嵌入是可选的增强功能，任何环节失败都不影响核心下载结果。**

```
convert_danmaku_to_ass()
    │
    ├── 失败 ──► DanmakuAssError ──► print 警告 ──► 跳过嵌入
    │
    ▼
ffmpeg 嵌入
    │
    ├── 失败 ──► DanmakuAssError ──► print 警告 ──► 删除临时文件
    │
    └── 成功 ──► 替换原视频 ──► print 完成信息
```

---

## CLI 参数

```python
# __main__.py
parser.add_argument(
    "--danmaku-ass",
    action="store_true",
    help="下载弹幕后生成 ASS 字幕并集成到视频中"
)
```

---

## 测试策略

### 新建 `tests/test_danmaku_ass.py`

| 测试用例 | 测试内容 |
|---|---|
| `test_convert_single_scroll` | 1 条滚动弹幕 -> 验证 Dialogue + \move 命令 |
| `test_convert_single_top_fixed` | 1 条顶部固定弹幕 -> 验证 Dialogue + \pos + \an8 |
| `test_convert_single_bottom_fixed` | 1 条底部固定弹幕 -> 验证 Dialogue + \pos + \an2 |
| `test_convert_unknown_mode` | mode 不为 1/4/5 -> 按滚动弹幕处理 |
| `test_convert_color_white` | color=16777215 -> 验证输出 `&HFFFFFF&` |
| `test_convert_color_red` | color=255 (纯红) -> 验证输出 `&H0000FF&` |
| `test_convert_empty_file` | 空 danmuku.txt -> 正常完成，无 Dialogue |
| `test_convert_missing_progress` | 缺 progress 字段 -> 使用 0 |
| `test_convert_missing_fontsize` | 缺 fontsize 字段 -> 使用 25 |
| `test_convert_missing_color` | 缺 color 字段 -> 使用 16777215 |
| `test_convert_row_distribution` | 多条滚动弹幕 -> 验证行分配正确轮询 |
| `test_convert_ass_header` | 验证 ASS 文件包含完整的 `[Script Info]` 和 `[V4+ Styles]` 头部 |
| `test_convert_malformed_json` | 某行不是有效 JSON -> 跳过该行，继续处理其他行 |

### 扩展现有测试 `tests/test_downloader.py`

| 测试用例 | 测试内容 |
|---|---|
| `test_run_with_danmaku_ass` | Mock ffmpeg，验证生成 .ass 文件和 ffmpeg 调用 |
| `test_run_without_danmaku_ass` | 默认行为，验证不生成 .ass 文件 |

---

## 修改文件清单

| 文件 | 修改 |
|---|---|
| `bilidown/danmaku_ass.py` | [新增] 弹幕转 ASS 核心逻辑 |
| `bilidown/downloader.py` | Downloader 构造函数新增 `danmaku_ass` 参数；run() 方法末尾新增 ASS 转换和 ffmpeg 嵌入逻辑 |
| `bilidown/__main__.py` | 新增 `--danmaku-ass` CLI 参数 |
| `bilidown/__init__.py` | 导出 `DanmakuAssError` |
| `bilidown/util.py` | [可选] 如需要，新增 `embed_subtitle()` 函数封装 ffmpeg 调用 |
| `tests/test_danmaku_ass.py` | [新增] ASS 转换单元测试 |
| `tests/test_downloader.py` | 扩展集成测试 |

## 不涉及

- 不修改 protobuf 解析逻辑（danmaku_pb2.py）
- 不修改弹幕下载逻辑（download_danmaku）
- 不修改 Video 数据模型
- 不修改 DASH/FLV 流下载和混流逻辑
- 不做弹幕密度过滤或碰撞检测
