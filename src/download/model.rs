//! 使用自 https://github.com/gwy15/danmu2ass/blob/main/src/bilibili/model.rs
//!
//! 弹幕的 probuf

use prost::Message;

#[derive(Debug, Clone)]
pub struct Video {
    pub bv: String,
    pub cid: u64,
    pub url: String,
    pub title: String,
    pub format: String,
    pub content_lenth: u64,
}

impl Video {
    pub fn new(
        bv: String,
        cid: u64,
        url: String,
        title: String,
        format: String,
        content_lenth: u64,
    ) -> Self {
        Video {
            bv,
            cid,
            url,
            title,
            format,
            content_lenth,
        }
    }
}

/// 弹幕 pb 定义
#[derive(Clone, Message)]
pub struct Bullet {
    /// 弹幕 dmid
    #[prost(int64, tag = "1")]
    pub id: i64,

    /// 弹幕出现位置（单位 ms）
    #[prost(int32, tag = "2")]
    pub progress: i32,

    /// 弹幕类型
    #[prost(int32, tag = "3")]
    pub mode: i32,

    /// 弹幕字号
    #[prost(int32, tag = "4")]
    pub fontsize: i32,

    /// 弹幕颜色
    #[prost(uint32, tag = "5")]
    pub color: u32,

    /// 发送者 mid hash
    #[prost(string, tag = "6")]
    pub mid_hash: String,

    /// 弹幕正文
    #[prost(string, tag = "7")]
    pub content: String,

    /// 弹幕发送时间
    #[prost(int64, tag = "8")]
    pub ctime: i64,

    /// 弹幕权重
    #[prost(int32, tag = "9")]
    pub weight: i32,

    /// 动作？
    #[prost(string, tag = "10")]
    pub action: String,

    /// 弹幕池
    #[prost(int32, tag = "11")]
    pub pool: i32,

    /// 弹幕 dmid str
    #[prost(string, tag = "12")]
    pub dmid_str: String,

    /// 弹幕属性
    #[prost(int32, tag = "13")]
    pub attr: i32,
}

#[derive(Clone, Message)]
pub struct BulletSegment {
    #[prost(message, repeated, tag = "1")]
    pub elems: Vec<Bullet>,
}
