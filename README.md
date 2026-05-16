# bilibili-downloader

B站视频与弹幕下载器

bilibili downloader and Danmaku restorer. Inspired by [danmu2ass](https://github.com/gwy15/danmu2ass)

因为众所周知的原因，B站的视频会被经常下架、消失。当没有及时缓存，作者也没有在其他平台发布，那么就会很遗憾。因此写了这个程序，用于及时下载，同时也保存一份弹幕文件。

## 功能

- 下载 B 站视频（支持 DASH 流和传统单流格式）
- 下载弹幕并保存为 JSON Lines 文件
- 支持将弹幕转换为 ASS 软字幕，通过 `-c copy` 零重编码混入 MKV 视频

## 使用方法

### 环境要求

- Python 3.11+
- ffmpeg（需在 PATH 中，或放在可执行文件同一目录下）

### 获取 Cookie

先登录 B 站，复制自己的 cookie 到 `cookie.txt`，放在项目根目录。建议使用浏览器隐私模式登录，过期时间更长。

### 安装

```bash
python -m pip install -r requirements.txt
```

### 运行

```bash
python -m bilidown <BV_ID>                    # 基本下载
python -m bilidown <BV_ID> -t 5               # 指定并发数（最大 10）
python -m bilidown <BV_ID> -d                 # 下载并输出软字幕 MKV 视频
python -m bilidown <BV_ID> --danmaku-ass      # 同上（长参数）
```

## LICENSE

MIT
