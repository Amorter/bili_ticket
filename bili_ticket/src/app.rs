use crate::task::load_config;
use bili_lib::{Order, Project};
use eframe::egui;
use eframe::egui::{FontData, FontFamily, Image, Sense, Vec2};
use egui_extras::install_image_loaders;
use reqwest::Client;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
pub struct BiliTicket {
    pub runtime: Arc<tokio::runtime::Runtime>,
    pub terminal_buffer: Arc<Mutex<String>>,
    pub show_qr: bool,
    pub login_qr_url: String,
    pub handler_order: bool,
    pub config: Config,
    pub logging: Arc<AtomicBool>,
}
pub struct Config {
    is_got_project: bool,
    project_image_url: String,
    pub project: Option<Project>,
    pub target_project: String,
    pub user_name: String,
    pub user_head_img_url: String,
    pub orders: Arc<Mutex<Vec<Order>>>,
    pub cookie: Arc<Mutex<String>>,
    pub client: Arc<Client>,
    pub blocking_client: Arc<reqwest::blocking::Client>,
    pub is_login: Arc<AtomicBool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            is_got_project: false,
            project_image_url: String::default(),
            project: None,
            target_project: String::default(),
            user_name: String::default(),
            user_head_img_url: String::default(),
            orders: Arc::new(Mutex::new(vec![])),
            cookie: Arc::new(Mutex::new(String::default())),
            client: Arc::new(Client::new()),
            blocking_client: Arc::new(reqwest::blocking::Client::new()),
            is_login: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for BiliTicket {
    fn default() -> Self {
        BiliTicket {
            runtime: Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            ),
            terminal_buffer: Arc::new(Mutex::new(String::default())),
            show_qr: false,
            login_qr_url: String::default(),
            handler_order: false,
            config: load_config(),
            logging: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl BiliTicket {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

    fn first_loading(&mut self) {}

    fn ui_menu(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu panel")
            .resizable(true)
            .show(ctx, |ui| {});
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
                                    self.print_terminal("载入商品信息失败,可能是票品不存在\n");
                                    flag = false;
                                });

                                if flag {
                                    self.config.project_image_url =
                                        self.config.project.clone().unwrap().performance_image;
                                    self.config.is_got_project = true;
                                    self.print_terminal("载入商品信息完成\n");
                                }
                            }
                        });
                    }
                    if self.config.is_got_project == true {
                        ui.add(
                            Image::from_uri(self.config.project_image_url.clone())
                                .fit_to_exact_size(Vec2::new(405.0, 720.0)),
                        );
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

                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            for order in &*self.config.orders.lock().unwrap() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.add(egui::Label::new(order.item_info.name.clone()));
                                    ui.add(egui::Label::new(order.sub_status_name.clone()));
                                });
                            }
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

impl eframe::App for BiliTicket {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ui_menu(ctx);
        self.ui_ticket(ctx);
        self.ui_terminal(ctx);
        self.ui_argument(ctx);
        self.ui_order(ctx);
    }
}
