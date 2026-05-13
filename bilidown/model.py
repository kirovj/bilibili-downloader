from dataclasses import dataclass


@dataclass
class Video:
    """视频元数据模型"""
    bv: str
    cid: int
    video_url: str
    audio_url: str
    title: str
    format: str
    duration: int
    content_len: int
