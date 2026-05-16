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
    total_cs = int(round(total_seconds * 100))
    h = total_cs // 360000
    m = (total_cs % 360000) // 6000
    s = (total_cs % 6000) // 100
    cs = total_cs % 100
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
    content = content.replace("\n", "\\N")
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
