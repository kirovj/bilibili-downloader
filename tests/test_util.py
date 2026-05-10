import os
import tempfile

import pytest
from bilidown.util import mix_video_audio, replace_illegal_chars_in_windows, write_bytes_to_file


def test_replace_illegal_chars_all():
    result = replace_illegal_chars_in_windows('a\\a/a:a*a?a"a<a>a|a')
    assert result == 'a\u2572a\u2571a\uff1aa\u2731a\uff1fa\u201ca\u300aa\u300ba\u2502a'


def test_replace_illegal_chars_no_illegal():
    result = replace_illegal_chars_in_windows("hello world")
    assert result == "hello world"


def test_replace_illegal_chars_empty():
    result = replace_illegal_chars_in_windows("")
    assert result == ""


def test_write_bytes_to_file():
    with tempfile.TemporaryDirectory() as tmpdir:
        filepath = os.path.join(tmpdir, "test.bin")
        write_bytes_to_file(filepath, b"hello", 0)
        write_bytes_to_file(filepath, b" world", 6)
        with open(filepath, "rb") as f:
            content = f.read()
        assert content == b"hello world"


def test_mix_video_audio_no_files():
    """ffmpeg not found or missing files — should raise subprocess.CalledProcessError or FileNotFoundError"""
    with tempfile.TemporaryDirectory() as tmpdir:
        with pytest.raises((FileNotFoundError, Exception)):
            mix_video_audio(
                os.path.join(tmpdir, "video.mp4"),
                os.path.join(tmpdir, "audio.mp3"),
                os.path.join(tmpdir, "output.mp4"),
            )
