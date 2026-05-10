"""Downloader 初始化与登录验证测试"""

from unittest.mock import MagicMock, patch

import pytest
import requests
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


class TestBuildVideo:
    @patch.object(requests.Session, "get")
    def test_build_video_dash(self, mock_get):
        from bilidown.model import Video

        # 响应0: check_login 返回
        resp0 = MagicMock()
        resp0.json.return_value = {"data": {"isLogin": True}}

        # 响应1: 视频基本信息
        resp1 = MagicMock()
        resp1.json.return_value = {
            "data": {
                "title": "测试视频/标题",
                "cid": 12345,
                "duration": 360,
            }
        }

        # 响应2: DASH 播放地址
        resp2 = MagicMock()
        resp2.json.return_value = {
            "data": {
                "dash": {
                    "video": [{"baseUrl": "https://example.com/v.m4s", "mimeType": "video/mp4"}],
                    "audio": [{"baseUrl": "https://example.com/a.m4s"}],
                }
            }
        }

        # 响应3: Range 请求获取 content length
        resp3 = MagicMock()
        resp3.headers = {"Content-Range": "bytes 0-1024/104857600"}

        mock_get.side_effect = [resp0, resp1, resp2, resp3]

        d = Downloader()
        video = d.build_video("BV1xx")

        assert video.bv == "BV1xx"
        assert video.cid == 12345
        assert video.title == "测试视频╱标题"
        assert video.format == "mp4"
        assert video.duration == 360
        assert video.content_len == 104857600
        assert video.audio_url == "https://example.com/a.m4s"

    @patch.object(requests.Session, "head")
    @patch.object(requests.Session, "get")
    def test_build_video_flv(self, mock_get, mock_head):
        from bilidown.model import Video

        # 响应0: check_login
        resp0 = MagicMock()
        resp0.json.return_value = {"data": {"isLogin": True}}
        resp0.headers = {}

        # 响应1: 视频基本信息
        resp1 = MagicMock()
        resp1.json.return_value = {
            "data": {"title": "FLV视频", "cid": 67890, "duration": 180}
        }
        resp1.headers = {}

        # 响应2: FLV 播放地址 (无 dash 字段)
        resp2 = MagicMock()
        resp2.json.return_value = {
            "data": {
                "durl": [{"url": "https://example.com/v.flv"}],
            }
        }
        resp2.headers = {}

        # 响应3: HEAD 请求获取 content length
        resp3 = MagicMock()
        resp3.headers = {"Content-Type": "video/x-flv", "Content-Length": "999999"}
        resp3.json.side_effect = ValueError  # not called, just in case

        mock_get.side_effect = [resp0, resp1, resp2]
        mock_head.return_value = resp3

        d = Downloader()
        video = d.build_video("BV1yy")

        assert video.bv == "BV1yy"
        assert video.cid == 67890
        assert video.title == "FLV视频"
        assert video.format == "flv"
        assert video.duration == 180
        assert video.content_len == 999999
        assert video.audio_url == ""
        assert video.video_url == "https://example.com/v.flv"
