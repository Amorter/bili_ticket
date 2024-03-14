use crate::app::{BiliTicket, Config};
use bili_lib::{generate_qrcode, nav_info, order_list_shows, project_info, qrcode_login};
use reqwest::header::{HeaderMap, COOKIE};
use serde_json::Error;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub fn load_config() -> Config {
    Config::default()
}

impl BiliTicket {
    pub fn print_terminal(&self, str: &str) {
        let tb = Arc::clone(&self.terminal_buffer);
        if !tb.lock().unwrap().ends_with("\n") {
            tb.lock().unwrap().push('\n');
        }
        tb.lock().unwrap().push_str(str);
    }

    pub fn do_login(&mut self) {
        let (url, qrcode_key) = self.runtime.block_on(generate_qrcode(&self.config.client));
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
        self.show_qr = true;
        let logging = Arc::clone(&self.logging);
        let tb = Arc::clone(&self.terminal_buffer);
        let c = Arc::clone(&self.config.cookie);
        let rt = Arc::clone(&self.runtime);
        let cl = Arc::clone(&self.config.client);
        let is_l = Arc::clone(&self.config.is_login);
        rt.spawn(async move {
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
        let rt = Arc::clone(&self.runtime);
        let cl = Arc::clone(&self.config.client);
        let orders = Arc::clone(&self.config.orders);
        let headers = self.build_headers();
        rt.spawn(async move {
            loop {
                let res = order_list_shows(&cl, headers.clone()).await;
                *orders.lock().unwrap() = res;
                sleep(Duration::from_secs(3)).await;
            }
        });
    }

    pub fn get_user_head(&mut self) {
        let (uname, face_img) = self
            .runtime
            .block_on(nav_info(&self.config.client, self.build_headers()));
        self.config.user_name = uname;
        self.config.user_head_img_url = face_img;
    }

    pub fn get_project(&mut self) -> Result<(), Error> {
        let project = self.runtime.block_on(project_info(
            &self.config.client,
            self.config.target_project.parse().unwrap(),
        ))?;
        self.config.project = Option::from(project.clone());
        Ok(())
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, self.config.cookie.lock().unwrap().parse().unwrap());
        headers
    }
}
