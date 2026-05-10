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
