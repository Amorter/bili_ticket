use crate::app::{NamePhoneForm, BiliTicket, Config, OrderType};
use bili_lib::{
    cancel_order, generate_qrcode, nav_info, order_create, order_list_shows, order_prepare,
    pay_param, project_info, qrcode_login, ClickPosition, CreateForm, Order, PrepareForm,
};
use reqwest::header::{HeaderMap, COOKIE};
use serde_json::Error;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

pub fn load_config() -> Config {
    Config::default()
}

impl BiliTicket {
    pub fn buy_ticket_now(&self, prepare_form: &PrepareForm, anonymous_form: &NamePhoneForm) {
        let token = self.prepare_order(prepare_form);
        //let regex = Regex::new(r"deviceFingerprint=<device_id>;").unwrap();
        //let cookie = self.config.cookie.lock().unwrap().to_string();
        //let cap = regex.captures(cookie.as_str()).unwrap();
        //let device_id = cap["device_id"].to_string();
        let device_id = "".to_string();
        let create_form = CreateForm {
            project_id: prepare_form.project_id,
            screen_id: prepare_form.screen_id,
            sku_id: prepare_form.sku_id,
            count: prepare_form.count,
            pay_money: self.config.ticket.price * prepare_form.count as u64,
            order_type: 1,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            token,
            device_id,
            click_position: ClickPosition {
                x: 935,
                y: 786,
                origin: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    - 1000,
                now: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            },
            new_risk: false,
            request_source: "pc_new".to_string(),
            buyer: anonymous_form.name.clone(),
            tel: anonymous_form.phone.clone(),
        };
        match self.runtime.block_on(order_create(
            &self.client,
            self.build_headers(),
            &create_form,
        )) {
            Ok(order_id) => {
                self.print_terminal("购票成功");
            }
            Err(e) => self.print_terminal(format!("购票失败，错误信息: {}\n", e).as_str()),
        }
    }

    pub fn prepare_order(&self, prepare_form: &PrepareForm) -> String {
        let token = self.runtime.block_on(order_prepare(
            &self.client,
            self.build_headers(),
            prepare_form,
        ));
        token.unwrap()
    }

    pub fn cancel_order(&self, order_id: &String) {
        match self
            .runtime
            .block_on(cancel_order(&self.client, self.build_headers(), order_id))
        {
            Ok(_) => {
                self.print_terminal("取消订单成功!\n");
            }
            Err(_) => {
                self.print_terminal("取消订单失败，可能是订单不存在?\n");
            }
        };
    }

    pub fn print_terminal(&self, str: &str) {
        let tb = Arc::clone(&self.terminal_buffer);
        if !tb.lock().unwrap().ends_with("\n") {
            tb.lock().unwrap().push('\n');
        }
        tb.lock().unwrap().push_str(str);
    }

    pub fn do_paying(&mut self, order_id: String) -> bool {
        match self
            .runtime
            .block_on(pay_param(&self.client, self.build_headers(), &order_id))
        {
            Ok(url) => {
                self.config.pay_code = format!(
                    "https://api.pwmqr.com/qrcode/create/?url={}",
                    url.replace("&", "%26")
                );
                true
            }
            Err(_) => {
                self.print_terminal("请求支付码失败，可能是订单不存在?\n");
                false
            }
        }
    }
    pub fn do_login(&mut self) {
        let (url, qrcode_key) = self.runtime.block_on(generate_qrcode(&self.client));
        // let qrcode = QRBuilder::new(url).build().unwrap();
        // self.login_qr = ImageBuilder::default()
        //     .shape(Shape::RoundedSquare)
        //     .background_color([255, 255, 255, 0])
        //     .fit_width(250)
        //     .to_bytes(&qrcode)
        //     .unwrap();
        self.login_qr_url = format!(
            "https://api.pwmqr.com/qrcode/create/?url={}",
            url.replace("&", "%26")
        );
        self.print_terminal("请扫描二维码登录:\n");
        self.show_login_qr = true;
        let logging = Arc::clone(&self.logging);
        let tb = Arc::clone(&self.terminal_buffer);
        let c = Arc::clone(&self.config.cookie);
        let cl = Arc::clone(&self.client);
        let is_l = Arc::clone(&self.config.is_login);
        self.runtime.spawn(async move {
            loop {
                sleep(Duration::from_secs(3)).await;
                let (code, msg, cookie) = qrcode_login(&cl, &qrcode_key).await;
                match code {
                    0 => {
                        *c.lock().unwrap() = cookie.unwrap();
                        is_l.store(true, Ordering::Relaxed);
                        tb.lock().unwrap().push_str("登录成功!\n");
                        logging.store(false, Ordering::Relaxed);
                        break;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        });
    }

    pub fn handler_orders(&self) {
        let cl = Arc::clone(&self.client);
        let orders = Arc::clone(&self.config.orders);
        let headers = self.build_headers();
        let is_handler = Arc::clone(&self.handler_order);
        self.runtime.spawn(async move {
            loop {
                if !is_handler.load(Ordering::Relaxed) {
                    return;
                }
                let res = order_list_shows(&cl, headers.clone()).await;
                *orders.lock().unwrap() = res;
                sleep(Duration::from_millis(1500)).await;
            }
        });
    }

    pub fn get_user_head(&mut self) {
        let (uname, face_img) = self
            .runtime
            .block_on(nav_info(&self.client, self.build_headers()));
        self.config.user_name = uname;
        self.config.user_head_img_url = face_img;
    }

    pub fn get_project(&mut self) -> Result<(), Error> {
        let project = self.runtime.block_on(project_info(
            &self.client,
            self.config.target_project.parse().unwrap(),
        ))?;
        self.config.project = Option::from(project.clone());
        if project.screen_list[0].ticket_list[0].anonymous_buy {
            self.config.order_type = OrderType::Anonymous
        } else if project.screen_list[0].delivery_type == 3 {
            self.config.order_type = OrderType::Deliver;
        } else if project.buyer_info == "2,1" {
            self.config.order_type = OrderType::Buyer;
        } else if project.need_contact == 1 {
            self.config.order_type = OrderType::NamePhone;
        }
        Ok(())
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, self.config.cookie.lock().unwrap().parse().unwrap());
        headers
    }
}
