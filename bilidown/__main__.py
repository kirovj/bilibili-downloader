"""Bilibili 视频下载器 CLI 入口"""

import argparse
import sys

from .downloader import Downloader


def main():
    parser = argparse.ArgumentParser(
        description="Bilibili Video Downloader",
    )
    parser.add_argument(
        "bv",
        help="Bilibili video BV id",
    )
    parser.add_argument(
        "-t", "--tasknum",
        type=int,
        default=7,
        help="Async task num for downloader (max 10)",
    )
    parser.add_argument(
        "-d", "--danmaku-ass",
        action="store_true",
        help="下载弹幕后生成 ASS 字幕并作为软字幕集成到视频中（输出 MKV 格式）",
    )
    args = parser.parse_args()

    if args.tasknum > 10:
        print("task num over 10, please use 1 ~ 10 instead")
        sys.exit(1)

    if not args.bv:
        print("bv id is empty!")
        sys.exit(1)

    try:
        downloader = Downloader(task_num=args.tasknum, danmaku_ass=args.danmaku_ass)
        downloader.run(args.bv)
    except Exception as e:
        print(f"error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
