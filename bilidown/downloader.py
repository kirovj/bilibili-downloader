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

    def __init__(self, task_num: int = 7):
        self.task_num = min(task_num, 10)
        self.dir = ""
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": UA,
            "Referer": "https://www.bilibili.com/",
        })
        if os.path.exists("cookie.txt"):
            with open("cookie.txt") as f:
                self.session.headers["Cookie"] = f.read().strip()

    def check_login(self) -> None:
        """验证 Cookie 是否有效"""
        r = self.session.get(self.API_USERINFO).json()
        if not r.get("data", {}).get("isLogin"):
            raise LoginFailError("Cookie 登录验证失败")
