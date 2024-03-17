use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Error;
use std::string::ToString;
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub order_id: String,
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
    pub buyer_info: String, //“2,1”为实名认证
    pub need_contact: i32,  //需要联系人表单吗
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
    pub id: i64,
    pub delivery_type: i32, //配送方式，1为电子票，3为纸质票
    start_time: u64,
    pub name: String,
    #[serde(rename = "type")]
    type_: i32,
    ticket_type: i32,
    screen_type: i32,
    pub ticket_list: Vec<Ticket>,
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Ticket {
    pub id: i64,
    pub anonymous_buy: bool, //匿名购买
    pub price: u64,
    pub desc: String,
    sale_start: String,
    sale_end: String,
    sale_type: i32,
    pub is_sale: i32,
    screen_name: String,
    #[serde(rename = "clickable")]
    click_able: bool,
}
#[derive(Serialize, Clone, Default)]
pub struct PrepareForm {
    pub project_id: i64,
    pub screen_id: i64,
    pub sku_id: i64,
    pub order_type: i32,
    pub count: u8,
}

pub struct CreateForm {
    pub project_id: i64,
    pub screen_id: i64,
    pub sku_id: i64,
    pub count: u8,
    pub pay_money: u64,
    pub order_type: i32,
    pub timestamp: u128,
    pub token: String,
    //#[serde(rename = "deviceId")]
    pub device_id: String,
    //#[serde(rename = "clickPosition")]
    pub click_position: ClickPosition,
    //#[serde(rename = "new_risk")]
    pub new_risk: bool,
    //#[serde(rename = "requestSource")]
    pub request_source: String, //电脑为pc-new
    pub buyer: String,          //联系人姓名
    pub tel: String,            //联系人电话
}

#[derive(Serialize)]
pub struct ClickPosition {
    pub x: u32,
    pub y: u32,
    pub origin: u128, //点击按钮时候的时间戳
    pub now: u128,    //发送请求时候的时间戳
}

impl Serialize for CreateForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("createForm", 14)?;
        state.serialize_field("project_id", &self.project_id)?;
        state.serialize_field("screen_id", &self.screen_id)?;
        state.serialize_field("sku_id", &self.sku_id)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("pay_money", &self.pay_money)?;
        state.serialize_field("order_type", &self.order_type)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("token", &self.token)?;
        state.serialize_field("deviceId", &self.device_id)?;
        state.serialize_field(
            "clickPosition",
            &serde_json::to_string(&self.click_position).unwrap(),
        )?;
        state.serialize_field("newRisk", &self.new_risk)?;
        state.serialize_field("requestSource", &self.request_source)?;
        state.serialize_field("buyer", &self.buyer)?;
        state.serialize_field("tel", &self.tel)?;
        state.end()
    }
}

pub async fn cancel_order(
    client: &Client,
    headers: HeaderMap,
    order_id: &String,
) -> Result<(), ()> {
    let res = client
        .get("https://show.bilibili.com/api/ticket/order/cancel?order_id=".to_string() + order_id)
        .headers(headers)
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let errno = json.get("errno").unwrap();
    if errno.as_i64().unwrap() == 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub async fn pay_param(
    client: &Client,
    headers: HeaderMap,
    order_id: &String,
) -> Result<String, ()> {
    let res = client
        .get(
            "https://show.bilibili.com/api/ticket/order/getPayParam?order_id=".to_string()
                + order_id,
        )
        .headers(headers)
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    if let Some(url) = data.get("code_url") {
        return Ok(url.as_str().unwrap().to_string());
    } else {
        return Err(());
    }
}

pub async fn order_info(client: &Client, headers: HeaderMap, order_id: String) -> Order {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let res = client
        .get(format!(
            "https://show.bilibili.com/api/ticket/order/info?order_id={}&timestamp={}",
            order_id, timestamp
        ))
        .headers(headers)
        .send()
        .await
        .unwrap();
    res.json::<Order>().await.unwrap()
}

pub async fn order_prepare(
    client: &Client,
    headers: HeaderMap,
    prepare_form: &PrepareForm,
) -> Result<String, ()> {
    let res = client
        .post("https://show.bilibili.com/api/ticket/order/prepare")
        .headers(headers)
        .form(prepare_form)
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    let token = data.get("token").unwrap().as_str().unwrap().to_string();
    Ok(token)
}

pub async fn order_create(
    client: &Client,
    headers: HeaderMap,
    create_form: &CreateForm,
) -> Result<u64, String> {
    let res = client
        .post("https://show.bilibili.com/api/ticket/order/createV2")
        .headers(headers)
        .form(create_form)
        .send()
        .await
        .unwrap();
    let json = res.json::<serde_json::Value>().await.unwrap();
    let data = json.get("data").unwrap();
    if let Some(order_id) = data.get("orderId") {
        Ok(order_id.as_u64().unwrap())
    } else {
        Err(json.get("msg").unwrap().as_str().unwrap().to_string())
    }
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