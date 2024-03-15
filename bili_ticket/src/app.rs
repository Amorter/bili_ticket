use crate::task::load_config;
use bili_lib::{Order, Project, Ticket};
use eframe::egui::{vec2, FontData, FontFamily, Image, Vec2};
use eframe::{egui, App, CreationContext};
use egui_extras::install_image_loaders;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct BiliTicket {
    pub runtime: tokio::runtime::Runtime,
    pub terminal_buffer: Arc<Mutex<String>>,
    pub show_login_qr: bool,
    pub login_qr_url: String,
    pub config: Config,
    pub logging: Arc<AtomicBool>,
    pub client: Arc<Client>,
    pub blocking_client: Arc<reqwest::blocking::Client>,
    pub handler_order: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    select_order_id: String,
    is_select_ticket: bool,
    ticket: Ticket,
    screen_name: String,
    is_got_project: bool,
    project_image_url: String,
    pub show_paying_qr: bool,
    pub project: Option<Project>,
    pub target_project: String,
    pub user_name: String,
    pub user_head_img_url: String,
    pub orders: Arc<Mutex<Vec<Order>>>,
    pub cookie: Arc<Mutex<String>>,
    pub is_login: Arc<AtomicBool>,
    pub pay_code: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            select_order_id: String::default(),
            is_select_ticket: false,
            ticket: Ticket::default(),
            screen_name: String::default(),
            is_got_project: false,
            project_image_url: String::default(),
            project: None,
            target_project: String::default(),
            user_name: String::default(),
            user_head_img_url: String::default(),
            orders: Arc::new(Mutex::new(vec![])),
            cookie: Arc::new(Mutex::new(String::default())),
            is_login: Arc::new(AtomicBool::new(false)),
            pay_code: String::default(),
            show_paying_qr: false,
        }
    }
}

impl Default for BiliTicket {
    fn default() -> Self {
        BiliTicket {
            handler_order: false,
            blocking_client: Arc::new(reqwest::blocking::Client::new()),
            client: Arc::new(Client::new()),
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            terminal_buffer: Arc::new(Mutex::new(String::default())),
            show_login_qr: false,
            login_qr_url: String::default(),
            config: load_config(),
            logging: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl BiliTicket {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        install_image_loaders(&cc.egui_ctx);

        let mut fonts = eframe::egui::FontDefinitions::default();
        if let Ok(buf) = fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
            fonts
                .font_data
                .insert("微软雅黑".to_owned(), FontData::from_owned(buf));
            fonts
                .families
                .insert(FontFamily::Monospace, vec!["微软雅黑".to_owned()]);
            fonts
                .families
                .insert(FontFamily::Proportional, vec!["微软雅黑".to_owned()]);
        } else {
            println!("Failed to load font 微软雅黑");
        }
        cc.egui_ctx.set_fonts(fonts);
        let mut bili_ticket = Self::default();

        bili_ticket.first_loading();

        bili_ticket
    }

    fn first_loading(&mut self) {
        if let Ok(mut f) = File::open("./config.json") {
            if let Ok(config) = serde_json::from_reader(f) {
                self.config = config;
            }
        }
    }

    fn ui_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("账户", |ui| {
                        if ui.button("更换账户").clicked() {
                            self.config = Config::default();
                        }
                    });
                });
            });
    }
    fn ui_ticket(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if self.config.is_login.load(Ordering::Relaxed) == true {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("请输入票品id");
                            ui.text_edit_singleline(&mut self.config.target_project);
                            if ui.button("确认").clicked() {
                                self.print_terminal("加载票品信息...\n");
                                let mut flag = true;
                                self.get_project().unwrap_or_else(|_| {
                                    self.print_terminal("载入票品信息失败,可能是票品不存在\n");
                                    flag = false;
                                });

                                if flag {
                                    ctx.forget_image(&self.config.project_image_url);
                                    self.config.project_image_url =
                                        self.config.project.clone().unwrap().performance_image;
                                    self.config.is_got_project = true;
                                    self.print_terminal("载入商品信息完成\n");
                                }
                            }
                        });
                    }
                    if self.config.is_got_project {
                        ui.add(
                            Image::from_uri(self.config.project_image_url.clone())
                                .fit_to_exact_size(Vec2::new(405.0, 720.0)),
                        );
                    }
                });
                ui.vertical(|ui| {
                    if self.config.is_got_project {
                        for screen in self.config.project.clone().unwrap().screen_list {
                            if ui.button(screen.name.clone()).clicked() {
                                self.config.screen_name = screen.name.clone();
                            }
                            if screen.name == self.config.screen_name {
                                ui.horizontal(|ui| {
                                    for ticket in screen.ticket_list {
                                        if ui.button(ticket.desc.clone()).clicked() {
                                            self.config.ticket = ticket;
                                            self.config.is_select_ticket = true;
                                        }
                                    }
                                });
                            }
                        }
                    }
                });
            });
        });
    }
    fn ui_order(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("order panel")
            .resizable(true)
            .default_height(100.0)
            .show(ctx, |ui| {
                if self.config.is_login.load(Ordering::Relaxed) == false {
                    if self.logging.load(Ordering::Relaxed) == false {
                        self.do_login();
                        self.logging.store(true, Ordering::Relaxed);
                    }
                    ctx.request_repaint();
                    ui.add(Image::from_uri(self.login_qr_url.clone()));
                }
                if self.config.is_login.load(Ordering::Relaxed) == true {
                    ctx.forget_image(&self.login_qr_url);
                    if self.handler_order == false {
                        self.print_terminal("加载用户昵称和头像...\n");
                        self.print_terminal("加载订单数据...\n");
                        self.get_user_head();
                        self.handler_orders();
                        self.handler_order = true;
                    }

                    egui::SidePanel::left("user_head panel").show_inside(ui, |ui| {
                        ui.heading(self.config.user_name.clone());
                        ui.add(Image::from_uri(self.config.user_head_img_url.clone()));
                    });
                    let height = ui.available_size().y;
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    let orders = self.config.orders.lock().unwrap().clone();
                                    let mut no_pay_wait = true;
                                    for order in orders {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(order.item_info.name.clone());
                                            ui.label(order.sub_status_name.clone());
                                            if order.sub_status_name.clone() == "待支付" {
                                                no_pay_wait = false;
                                                if self.config.select_order_id != order.order_id {
                                                    if ui.link("点此显示付款二维码").clicked()
                                                    {
                                                        ctx.forget_image(&self.config.pay_code);
                                                        self.print_terminal("请求付款二维码...\n");
                                                        if self.do_paying(order.order_id.clone()) {
                                                            self.config.show_paying_qr = true;
                                                            self.config.select_order_id = order.order_id.clone();
                                                        }
                                                    }
                                                } else {
                                                    if ui.link("隐藏付款码").clicked() {
                                                        ctx.forget_image(&self.config.pay_code);
                                                        self.print_terminal("删除缓存\n");
                                                        self.config.show_paying_qr = false;
                                                        self.config.pay_code = String::default();
                                                        self.config.select_order_id = String::default();
                                                    }
                                                }

                                                if ui.link("取消订单").clicked() {
                                                    self.cancel_order(&order.order_id);
                                                }
                                            }
                                        });
                                    }
                                    if no_pay_wait {
                                        self.config.show_paying_qr = false;
                                        self.config.pay_code = String::default();
                                        self.config.select_order_id = String::default();
                                    }
                                });
                                ui.vertical(|ui| {
                                    if self.config.show_paying_qr {
                                        ui.add_sized(
                                            vec2(height, height),
                                            Image::from_uri(&self.config.pay_code),
                                        );
                                    }
                                });
                            });
                        });
                }
            });
    }
    fn ui_terminal(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("terminal panel")
            .resizable(true)
            .default_height(100.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(
                                &mut self.terminal_buffer.lock().unwrap().as_str(),
                            ),
                        );
                    });
            });
    }
    fn ui_argument(&self, ctx: &egui::Context) {
        egui::SidePanel::right("argument panel")
            .resizable(true)
            .show(ctx, |ui| {});
    }
}

impl App for BiliTicket {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ui_menu(ctx);
        self.ui_ticket(ctx);
        self.ui_terminal(ctx);
        self.ui_argument(ctx);
        self.ui_order(ctx);
        if ctx.input(|i| i.viewport().close_requested()) {
            let mut file = File::create("./config.json").unwrap();
            let json = serde_json::to_string(&self.config).unwrap();
            file.write_all(json.as_ref()).unwrap();
        }
    }
}