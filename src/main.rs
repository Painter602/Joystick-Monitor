#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// hide console window on Windows in release
#[cfg(feature = "logging")]
use chrono::Utc;
use crate::device::Message;
use eframe::egui;
use egui::{Ui, Vec2, Color32, Sense, };
use egui_extras::image::RetainedImage;
use std::cmp;
use std::time::{Duration, Instant, };
use walkdir::WalkDir;

#[cfg(feature = "logging")]
use log::info;
use ::function_name::named;

mod about;
mod device;

const DARK_FILL: egui::Color32 = egui::Color32::from_rgb(0x11, 0x00, 0x11); //:DARK_GRAY;  // :DARK_BLUE;  // :BLACK;
const FILL_COLOUR: egui::Color32 = egui::Color32::GREEN;
const FRAME_RATE: u64 = 60; // 30; //120; // 60; //30; // 60;              // frames per second
const IMG_SIZE: f32 = 240.0;
const MAX_ERR_LIST: usize = 7; // 100;
const PROGRAM_TITLE: &str = "Joystick Monitor";
const TICK: Duration = Duration::from_millis(1000 / FRAME_RATE - 3);

#[derive(Copy, Clone, PartialEq)]
enum State {
    About,
    Initialising,
    IsNew,
    PrepScreen,
    Running,
}

struct MyApp {
    state: State,
    // old_state: State,
    err_list: Vec<String>,
    images: Vec<RetainedImage>,
    img_sizes: Vec<egui::Vec2>,
    now: Instant,
    show_buttons: bool,
    best_width: f32,
    // tint: [u8; 3],
    // recover: bool,
}

impl MyApp {
    fn about_screen(&mut self, ui: &mut Ui ) {
        let loading = self.state != State::About;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.centered_and_justified( |ui|{
                let mut about = "";
                if !loading {
                    about = "About ";
                }
                ui.heading(format!("{}{}", about, PROGRAM_TITLE,));
                });
            });
            if loading {
                ui.horizontal(|ui| {
                    ui.centered_and_justified( |ui|{
                        ui.add(egui::Label::new("-- Loading, please wait --"));
                    });
                });
            }
            for line in about::about() {
                ui.horizontal(|ui| {
                    ui.centered_and_justified( |ui|{
                        ui.add(egui::Label::new(line));
                    });
                });
            }
            
            if !loading {
                ui.horizontal(|ui| {
                    let blank: String = "         ".to_string();
                    ui.centered_and_justified( |ui|{
                        ui.label(&blank);
                        if ui.button("OK")
                            .clicked() {
                            self.state = State::Running;
                        }
                        ui.label(&blank);
                    });
                });            
            }
        });
    }

    #[named]
    fn check_devices(&mut self) {
		self.err_list = Vec::new();
        for mssg in device::check_devices() {
            match mssg {
                Message::Err( mssg ) => {
                    self.err_list.insert( 0, show_error( // .push( show_error(
                        module_path!(),
                        function_name!(),
                        format!("{}", mssg) ) );
                }
                Message::None => {}
            }
        }       
    }

    fn find_image(&mut self, dir_entry: walkdir::DirEntry) {
        let path = dir_entry.path();
        match path.extension(){
            Some( ext ) => {
                if ext == "svg" {
                    match path.to_str() {
                        Some( img_path ) => {
                            self.load_image( img_path );
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
    }
    
    #[named]
    fn init( &mut self ) {
		#[cfg(feature = "logging")] {
			info!("{}::{}", module_path!(), function_name!());
        }
        self.now = Instant::now();
        while self.err_list.len() > MAX_ERR_LIST {
            self.err_list.remove( MAX_ERR_LIST );
        }
        match self.state {
            State::PrepScreen => {
                self.state = State::Initialising;
                self.set_api();
                
                for entry in
                    WalkDir::new("img").sort_by_key(
                        |a| a.file_name().to_owned()) {
                            
                    match entry {
                        Ok( dir_entry ) => {
                            if dir_entry.file_type().is_dir() {
                                continue;
                            }
                            self.find_image(dir_entry);
                        }
                        Err( err ) => {
                            self.err_list.insert(0, show_error( //   .push( show_error(
                                                    module_path!(),
                                                    function_name!(),
                                                    format!("{}", err) ) );
                        }
                    }
                }
                self.state = State::Running;
            }
            _ => {}
        }
        
		#[cfg(feature = "logging")] {
			info!("{}::{} done", module_path!(), function_name!());
        }
    }

    fn joystick_screen(&mut self, ui: &mut Ui, ctx: &egui::Context, menu_height: f32) -> f32 {
        self.check_devices();

        let outer = ui.horizontal_centered(|ui| {
            let dev_repts: &mut Vec<device::DeviceReport> = &mut Vec::new();
            unsafe {
                for (_, dev_report) in device::DEVICES_REPORTS.iter() {
                    if dev_report.col < std::usize::MAX {
                        dev_repts.push( dev_report.clone() );
                    }
                }
            }
            
            dev_repts.sort_unstable();
            for dev_report in dev_repts {
                if dev_report.col < std::usize::MAX {
                    ui.vertical(|ui| {
                        let texture = self.images[ dev_report.col ].texture_id(ctx);
                        
                        //let tint: egui::Color32;
                        // tint = Color32::LIGHT_GREEN ; // :LIGHT_GRAY; // :GRAY;
                        // tint = Color32::WHITE;
                        
                        let img = egui::widgets::Image::new( 
                                    texture, 
                                    self.img_sizes[dev_report.col ])
                                .rotate((dev_report.z_f32()-0.5)*2.0 + dev_report.z_calibrate as f32,
                                Vec2::splat(0.5))
                                 // .tint(Color32::WHITE);
                                 // .tint(tint)
                                 ;
                        img.paint_at(ui, egui::Rect::from_center_size(
                                        egui::Pos2::new( 
                                            (dev_report.x_f32()+0.55)*self.img_sizes[dev_report.col ].x * 0.5 +
                                            dev_report.col as f32 * self.img_sizes[dev_report.col ].x +
                                            dev_report.x_calibrate as f32, 
                                            (dev_report.y_f32()+0.55)*self.img_sizes[dev_report.col ].y * 0.5 +
                                            dev_report.y_calibrate as f32 + menu_height,),
                                        egui::Vec2::new( 
                                            self.img_sizes[dev_report.col ].x, 
                                            self.img_sizes[dev_report.col ].y)));
                        
                        
                        /*
                        dbg!( dev_report.x );
                        if dev_report.x < 2 || dev_report.x > 59998 {
                            dbg!( dev_report.x_f32() );
                            dbg!( (dev_report.x_f32()+0.55)*self.img_sizes[dev_report.col ].x * 0.5 +
                            dev_report.col as f32 * self.img_sizes[dev_report.col ].x +
                            dev_report.x_calibrate as f32 );
                        }
                        */
                        
                        let texture = self.images[ 2 ].texture_id(ctx);     // hard coded, wrong!!!!
                        let img = egui::widgets::Image::new( 
                                    texture, 
                                    self.img_sizes[dev_report.col ]);
                        ui.add( img);
                        
                        if self.show_buttons {
                            let mut first_row: bool = true;
                            for btn_row in &dev_report.buttons {
                                ui.horizontal(|ui| { 
                                    let mut val: u8 = 0b00000001;
                                    loop {
                                        let mut btn_texture = self.images[self.images.len()-1].texture_id(ctx);
                                        if 0 < (btn_row & val) {
                                            if first_row & (val == 0b00000001) {
                                                btn_texture = self.images[self.images.len()-3].texture_id(ctx);                                                
                                            } else {
                                                btn_texture = self.images[self.images.len()-2].texture_id(ctx);                                                
                                            }
                                        }
                                        
                                        ui.image(
                                            btn_texture,
                                            self.img_sizes[self.images.len()-1 ]);
                                            
                                        if val > 0b01000001 {
                                            break;
                                        }
                                        val *= 2;
                                        first_row = false;
                                    };
                                });     
                            }
                        }
                    });
                }
            }
        });
        
        ui.vertical(|ui| {
            for str in &self.err_list {
                ui.add(egui::Label::new(str));
            }
            for _n in self.err_list.len()..MAX_ERR_LIST {
                ui.add(egui::Label::new(""));
            }
        });
        let response = outer.response.interact(Sense::click());
        response.context_menu(|ui| {
            let mut btn_txt = "Show Buttons";
            if self.show_buttons { btn_txt = "Hide Buttons";}
            if ui.button(btn_txt).clicked() {
                self.show_buttons = !self.show_buttons;
                ui.close_menu();
            }
            if ui.button("About").clicked() {
                self.state = State::About;
                ui.close_menu();
            }
        });
        
        outer.response.rect.width()
    }
    
    #[named]
    fn load_image(&mut self, img_path: &str ) {
        match std::fs::read_to_string(img_path) {
            Ok( svg_str ) => {
                match RetainedImage::from_svg_str(
                    img_path, &svg_str ) {
                    Ok( img ) => {
                        let size: egui::Vec2 = egui::Vec2{
                            x: img.width() as f32,
                            y: img.height() as f32,};
                            
                        self.images.push( img );
                        self.img_sizes.push( size );
                    }
                    Err( err ) => {
                        self.err_list.insert(0, show_error(         //) .push( show_error(
                                module_path!(),
                                function_name!(),
                                format!("init, load image file '{}', error {}",
                                        img_path, err) ) );
                    }
                }
            }
            Err(_) => {
                self.err_list.insert(0, show_error(
                        module_path!(),
                        function_name!(),
                        format!(
                            "Should have been able to read {}",
                            img_path) ) );
            }
        }
    }

    #[named]
    fn set_api(&mut self) {
        for mssg in device::load_devices( FRAME_RATE as i32 ) {
            match mssg {
                Message::Err( mssg ) => {
                    self.err_list.insert(0, show_error(
                            module_path!(),
                            function_name!(),
                            mssg ) );
                }
                _ => {}
            }
        }
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: State::IsNew,
            err_list: Vec::new(),
            images: Vec::new(),
            img_sizes: Vec::new(),
            now: Instant::now(),
            show_buttons: false,
            best_width: 0.0,
            // tint: [255; 3],
            // recover: true,
        }
    }
}

impl eframe::App for MyApp {
    //#[named]
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let menu_height: f32 = 0.0; // f64;
        let mut win_height: f32 = 0.0; // 240.0;
        let mut win_width: f32 = 240.0;
        ctx.request_repaint_after(TICK);

        let fill_colour: Color32;
        match self.state {
            State::Running => { fill_colour = FILL_COLOUR; }
            _ => { fill_colour = DARK_FILL; }
        }

        let cen_pan = egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::none()
                    .inner_margin(egui::style::Margin::same( 4.0 ))
                    .outer_margin(egui::style::Margin::same( -6.0 ))
                    .fill( fill_colour) // .fill(FILL_COLOUR) // :DARK_GRAY) // :GRAY) // :GREEN)
                    .stroke(egui::Stroke::none())
                    .show(ui, |ui| {

                match self.state {
                    //State::About => {   self.about_screen( ui );    }
                    State::IsNew => {
                        self.about_screen( ui );
                        self.state = State::PrepScreen;
                    }
                    State::PrepScreen => {                        
                        self.about_screen( ui );
                        self.init();
                    }
                    State::Running => { win_width = self.joystick_screen(ui, ctx, menu_height); }
                    _ => {  self.about_screen( ui );    }
                }
            });
        });

        win_height = cmp::max(cen_pan.response.rect.height() as u64 - 6, win_height as u64 ) as f32;
        win_width = cmp::max(cen_pan.response.rect.width() as u64 - 6, win_width as u64 ) as f32;
        self.best_width = cmp::max(self.best_width as u64, win_width as u64 ) as f32;

        frame.set_window_size(Vec2 { x: self.best_width - 9.0, y: win_height - 9.0 } );
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        let colour = FILL_COLOUR.to_tuple();
        egui::Color32::from_rgba_unmultiplied(colour.0, colour.1, colour.2, colour.3).into()
    }
}

/* ******************************************************************************* *
 *
 * ******************************************************************************* */

fn load_icon(path: &str) -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

/* ******************************************************************************* */

#[named]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    // tracing_subscriber::fmt::init();
    #[cfg(feature = "logging")] {
        setup_logger().expect("failed to set up logger");
        info!("{}::{}", module_path!(), function_name!());
    }
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(   IMG_SIZE * 2.0, 
                                                IMG_SIZE + 20.0)),
        icon_data: Some(load_icon("./assets/JSIcon.png")),
        resizable: true,
        ..Default::default()
    };
    
    eframe::run_native(
        PROGRAM_TITLE,
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

/* ******************************************************************************* */

#[cfg(feature = "logging")] 
fn prog() -> Option<String> {
    std::env::current_exe()
        .ok()?
        .file_name()?
        .to_str()?
        .to_owned()
        .into()
}

/* ******************************************************************************* */
#[cfg(feature = "logging")]        
fn setup_logger() -> Result<(), fern::InitError> {
    let file: String;
    let now = Utc::now().format("%Y-%m-%d %H.%M.%S");
    let log_dir = "log/";
    match prog() {
        Some( file_s ) => {
            let file_v: Vec< &str> = file_s.as_str().split(".").collect();
            match file_v.len() {
                0 => {file = format!("{}{} log.log", log_dir, now).to_string();}
                1 => {file = format!("{}{} {}.log", log_dir, now, file_v[ 0 ]).to_string();}
                _ => {file = format!("{}{} {}.log", log_dir, now, file_v[ file_v.len() - 2 ]).to_string();}
            }
        }
        _ => {
            file = format!("{}{} log.log", log_dir, now).to_string();
        }
    }
    
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(file.as_str())?)
        .apply()?;
    Ok(())
}

/* ******************************************************************************* */

fn show_error(_module_path: &str, _function_name: &str, message: String ) -> String{
    #[cfg(feature = "logging")] {
        info!( "{}::{} {}", _module_path, _function_name, message );
    }
    message
}

/* ******************************************************************************* */

/* ******************************************************************************* *
 *              *** END ***
 * ******************************************************************************* */
