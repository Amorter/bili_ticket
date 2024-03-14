use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::rc::Rc;
use std::string::ToString;

#[derive(Serialize, Deserialize, Clone)]
pub struct Buyer {
    id: i64,                 //购票人id
    uid: i64,                //b站id
    account_channel: String, //未知
    personal_id: String,     //身份证号
    name: String,            //真实姓名,
    id_card_front: String,
    id_card_back: String,
    is_default: i8,     //是否为默认账户
    tel: String,        //手机号码
    error_code: i64,    //错误码，无错误则为0
    id_type: i64,       //未知，为0
    verify_status: i64, //未知，为1
    #[serde(rename = "accountId")]
    account_id: i64, //同uid
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ItemInfo {
    pub name: String,
    img: String,
    screen_id: i32,
    screen_name: String,
    express_fee: i32,
    express_free_flag: i32,
    deliver_type: i32,
    screen_type: i32,
    //project_ver_id: i64,
    link_id: i32,
    ticket_type: i32,
    time: i32,
    ticket_type_name: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Img {
    url: String,
    desc: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Order {
    order_id: String,
    uid: String,
    order_type: i32,
    item_id: i32,
    #[serde(rename = "item_info")]
    pub item_info: ItemInfo,
    count: i32,
    total_money: i32,
    pay_money: i32,
    express_fee: i32,
    pay_channel: i32,
    status: i32,
    sub_status: i32,
    refund_status: i32,
    pay_time: i32,
    ctime: String,
    source: String,
    ticket_agent: String,
    img: Img,
    current_time: i32,
    deliver_type_name: String,
    free_deliver: bool,
    create_at: i32,
    pay_remain_time: i32,
    pub sub_status_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    status: i32,
    is_sale: i32,
    start_time: u64,
    end_time: u64,
    sale_begin: i64,
    sale_end: u64,
    sale_start: u64,
    pub performance_image: String,
    pub screen_list: Vec<Screen>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Screen {
    id: i64,
    start_time: u64,
    pub name: String,
    #[serde(rename = "type")]
    type_: i32,
    ticket_type: i32,
    screen_type: i32,
    pub ticket_list: Vec<Ticket>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Ticket {
    id: i64,
    pub price: i64,
    pub desc: String,
    sale_start: String,
    sale_end: String,
    sale_type: i32,
    pub is_sale: i32,
    screen_name: String,
    #[serde(rename = "clickable")]
    click_able: bool,
}

pub async fn order_prepare(client: &Client, headers: HeaderMap) {
    client.get("https://show.bilibili.com/api/ticket/order/prepare");
}

pub async fn order_create(client: &Client, headers: HeaderMap) {
    client.get("https://show.bilibili.com/api/ticket/order/createV2");
}

pub async fn nav_info(client: &Client, headers: HeaderMap) -> (String, String) {
    let res = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .headers(headers)
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    (
        data.get("uname").unwrap().as_str().unwrap().to_string(),
        data.get("face").unwrap().as_str().unwrap().to_string(),
    )
}

pub async fn order_list_shows(client: &Client, headers: HeaderMap) -> Vec<Order> {
    let res = client
        .get("https://show.bilibili.com/api/ticket/order/list?page=0&page_size=20")
        .headers(headers.clone())
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    let order_list: Vec<Order> = serde_json::from_value(data.get("list").unwrap().clone()).unwrap();
    order_list
}

pub async fn buyer_info(client: Client, headers: HeaderMap) -> Vec<Buyer> {
    let res = client
        .get("https://show.bilibili.com/api/ticket/buyer/list")
        .headers(headers)
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    let buyer_list: Vec<Buyer> = serde_json::from_value(data.get("list").unwrap().clone()).unwrap();
    buyer_list
}
pub async fn generate_qrcode(client: &Client) -> (String, String) {
    let res = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    (
        data.get("url").unwrap().as_str().unwrap().to_string(),
        data.get("qrcode_key")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    )
}

pub async fn qrcode_login(client: &Client, qrcode_key: &String) -> (i64, String, Option<String>) {
    let res = client
        .get(
            "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key="
                .to_string()
                + qrcode_key,
        )
        .send()
        .await
        .unwrap();
    let head = res.headers().clone();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    let re_cookie: Option<String>;
    if let Some(cookie) = head.get("Set-Cookie") {
        re_cookie = Option::from(cookie.to_str().unwrap().to_string());
    } else {
        re_cookie = None;
    }

    (
        data.get("code").unwrap().as_i64().unwrap(),
        data.get("message").unwrap().as_str().unwrap().to_string(),
        re_cookie,
    )
}

pub async fn project_info(client: &Client, project_id: u64) -> Result<Project, Error> {
    let res = client
        .get(
            "https://show.bilibili.com/api/ticket/project/get?id=".to_string()
                + &project_id.to_string(),
        )
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    let mut project: Project = serde_json::from_value(data.clone())?;
    let performance_image: serde_json::Value =
        serde_json::from_str(&project.performance_image).unwrap();
    let performance_image_url = "http:".to_string()
        + performance_image
            .get("first")
            .unwrap()
            .get("url")
            .unwrap()
            .as_str()
            .unwrap();
    project.performance_image = performance_image_url;

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let (url, qrcode_key) = generate_qrcode(&Client::new()).await;
        println!("url: {}\nqrcode_key: {}", url, qrcode_key);
    }
}