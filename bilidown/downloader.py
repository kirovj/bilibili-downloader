"""Bilibili 视频下载器核心模块"""

import os

import requests

from .model import Video


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
    API_BULLET = "http://api.bilibili.com/x/v2/dm/web/seg.so"

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

    def build_video(self, bv: str) -> "Video":
        """根据 BV 号构建 Video 对象"""
        from .model import Video
        from .util import replace_illegal_chars_in_windows

        self.check_login()

        # Step 1: 获取视频基本信息
        info_url = f"{self.API_INFO}{bv}"
        try:
            resp = self.session.get(info_url, timeout=10)
            resp.raise_for_status()
            info = resp.json()["data"]
        except requests.RequestException as e:
            raise VideoInfoError(f"请求视频信息失败: {e}")
        except (KeyError, ValueError, TypeError) as e:
            raise VideoInfoError(f"解析视频信息响应失败: {e}")

        title = replace_illegal_chars_in_windows(info.get("title", bv))
        cid = info["cid"]
        duration = info["duration"]

        # Step 2: 获取播放地址
        try:
            resp = self.session.get(
                self.API_PLAY,
                params={"bvid": bv, "cid": cid, "fnval": "2000"},
                timeout=10,
            )
            resp.raise_for_status()
            play = resp.json()["data"]
        except requests.RequestException as e:
            raise VideoInfoError(f"请求播放地址失败: {e}")
        except (KeyError, ValueError, TypeError) as e:
            raise VideoInfoError(f"解析播放地址响应失败: {e}")

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

    def download_chunk(self, video: "Video", range_tuple: tuple, index: int) -> None:
        """下载单个视频分块"""
        from .util import write_bytes_to_file

        start, end = range_tuple
        r = self.session.get(
            video.video_url,
            headers={"Range": f"bytes={start}-{end}"},
            stream=True,
            timeout=30,
        )
        r.raise_for_status()
        filepath = f"{self.dir}/chunk_{index}"
        offset = 0
        for chunk in r.iter_content(chunk_size=8192):
            if chunk:
                write_bytes_to_file(filepath, chunk, offset)
                offset += len(chunk)

    def download_chunks(self, video: "Video") -> int:
        """并发分块下载视频"""
        from concurrent.futures import ThreadPoolExecutor, as_completed

        chunk_size = 10 * 1024 * 1024  # 10MB
        futures = []
        start = 0
        index = 0

        with ThreadPoolExecutor(max_workers=self.task_num) as executor:
            while start < video.content_len:
                end = min(start + chunk_size, video.content_len) - 1
                if end < start:
                    end = start
                f = executor.submit(self.download_chunk, video, (start, end), index)
                futures.append(f)
                start = end + 1
                index += 1

            for f in as_completed(futures):
                f.result()  # 传播异常

        return index

    def download_audio(self, video: "Video") -> None:
        """下载音频流"""
        if not video.audio_url:
            return
        r = self.session.get(video.audio_url, stream=True, timeout=30)
        r.raise_for_status()
        filepath = f"{self.dir}/audio.mp3"
        with open(filepath, "wb") as f:
            for chunk in r.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)

    def build_final_video(self, video: "Video", chunk_count: int) -> None:
        """合并分块并调用 ffmpeg 混流"""
        import os
        from .util import mix_video_audio

        video_path = f"{self.dir}/video.{video.format}"

        # 合并所有 chunk
        with open(video_path, "wb") as out:
            for i in range(chunk_count):
                chunk_path = f"{self.dir}/chunk_{i}"
                with open(chunk_path, "rb") as f:
                    out.write(f.read())
                os.remove(chunk_path)

        audio_path = f"{self.dir}/audio.mp3"
        output_path = f"{self.dir}/{video.title}.{video.format}"

        if os.path.exists(audio_path) and os.path.getsize(audio_path) > 0:
            mix_video_audio(video_path, audio_path, output_path)
            os.remove(video_path)
            os.remove(audio_path)
        else:
            os.rename(video_path, output_path)

    def download_danmaku_segment(self, video: "Video", seg_index: int):
        """下载单个弹幕分段"""
        from .danmaku_pb2 import DanmakuSegment

        r = self.session.get(
            self.API_BULLET,
            params={"oid": video.cid, "segment_index": seg_index, "type": 1},
            timeout=30,
        )
        r.raise_for_status()
        segment = DanmakuSegment().parse(r.content)
        return segment

    def download_danmaku(self, video: "Video") -> None:
        """下载并解析弹幕，写入 JSON Lines 文件"""
        from concurrent.futures import ThreadPoolExecutor, as_completed
        import json

        bags = (video.duration + 359) // 360  # 每 6 分钟一个 segment
        results = {}

        with ThreadPoolExecutor(max_workers=self.task_num) as executor:
            futures = {
                executor.submit(self.download_danmaku_segment, video, i + 1): i
                for i in range(bags)
            }
            for f in as_completed(futures):
                idx = futures[f]
                results[idx] = f.result()

        with open(f"{self.dir}/danmuku.txt", "w", encoding="utf-8") as f:
            for i in range(bags):
                for elem in results[i].elems:
                    d = elem.to_dict()
                    f.write(json.dumps(d, ensure_ascii=False) + "\n")
