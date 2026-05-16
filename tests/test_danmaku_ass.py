"""danmaku_ass 模块单元测试"""

import json

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
        # 255 = 0x0000FF -> B=FF G=00 R=00 -> &H00FF0000&
        result = _decimal_to_ass_color(255)
        assert result == "&H00FF0000&"

    def test_green(self):
        # 65280 = 0x00FF00 -> B=00 G=FF R=00 -> &H0000FF00&
        result = _decimal_to_ass_color(65280)
        assert result == "&H0000FF00&"

    def test_blue(self):
        # 16711680 = 0xFF0000 -> B=00 G=00 R=FF -> &H000000FF&
        result = _decimal_to_ass_color(16711680)
        assert result == "&H000000FF&"

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

        with pytest.raises(DanmakuAssError, match="弹幕文件不存在"):
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
        # 应包含转义后的 \{你好\} 文字
        assert "你好" in content
        assert "世界" in content
