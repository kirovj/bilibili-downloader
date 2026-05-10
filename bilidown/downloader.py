"""Bilibili 视频下载器核心模块"""

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
    """Bilibili 视频下载器"""

    API_USERINFO = "https://api.bilibili.com/x/web-interface/nav"
    API_INFO = "https://api.bilibili.com/x/web-interface/view?bvid="
    API_PLAY = "https://api.bilibili.com/x/player/playurl"

    def __init__(self, task_num: int = 7):
        self.task_num = min(task_num, 10)
        self.dir = ""
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": UA,
            "Referer": "https://www.bilibili.com/",
        })
        if os.path.exists("cookie.txt"):
            try:
                with open("cookie.txt") as f:
                    self.session.headers["Cookie"] = f.read().strip()
            except OSError as e:
                print(f"读取 cookie.txt 失败: {e}")

    def check_login(self) -> None:
        """验证 Cookie 是否有效"""
        try:
            r = self.session.get(self.API_USERINFO, timeout=10)
            r.raise_for_status()
        except requests.RequestException as e:
            raise LoginFailError(f"登录验证请求失败: {e}")
        if not r.json().get("data", {}).get("isLogin"):
            raise LoginFailError("Cookie 登录验证失败")

    def _extract_format(self, content_type: str) -> str:
        """根据 MIME 类型提取文件格式"""
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
        """根据 BV 号构建 Video 对象"""
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
