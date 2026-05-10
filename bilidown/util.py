import os
import subprocess


def replace_illegal_chars_in_windows(value: str) -> str:
    """替换 Windows 文件名中的非法字符"""
    replacements = {
        "\\": "\u2572",
        "/": "\u2571",
        ":": "\uff1a",
        "*": "\u2731",
        "?": "\uff1f",
        '"': "\u201c",
        "<": "\u300a",
        ">": "\u300b",
        "|": "\u2502",
    }
    for old, new in replacements.items():
        value = value.replace(old, new)
    return value


def write_bytes_to_file(filepath: str, data: bytes, offset: int) -> None:
    """带偏移量的文件写入"""
    if not os.path.exists(filepath):
        with open(filepath, "wb") as f:
            pass
    with open(filepath, "r+b") as f:
        f.seek(offset)
        f.write(data)


def mix_video_audio(video_path: str, audio_path: str, output_path: str) -> None:
    """使用 ffmpeg 混流视频和音频"""
    subprocess.run(
        [
            "ffmpeg",
            "-i",
            video_path,
            "-i",
            audio_path,
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-strict",
            "experimental",
            output_path,
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
