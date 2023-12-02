# bilibili-downloader

B站视频与弹幕下载器

bilibili downloader and Danmaku restorer. Inspired by [danmu2ass](https://github.com/gwy15/danmu2ass)

因为众所周知的原因，B站的视频会被经常下架、消失。当没有及时缓存，作者也没有在其他平台发布，那么就会很遗憾。因此写了这个程序，用于及时下载，同时也保存一份弹幕文件。

## 使用方法

先登录B站复制自己的 cookie 到 cookie.txt，不会请百度，这里最好用浏览器隐私模式登录后的 cookie，过期时间长。

```
bilibili-downloader.exe <bv> -t <tasknum>
bilibili-downloader.exe BVasd3... -t 7
```
tasknum 表示异步下载任务数，你也可以直接使用

    bilibili-downloader.exe BVasd3...

## Todo

* 渲染弹幕到视频

## LICENSE
    MIT
