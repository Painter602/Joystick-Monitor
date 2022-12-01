use configparser::ini::Ini;
use ::function_name::named;
use hidapi::{HidApi, HidDevice,};
use log:: info;					// if not using this, comment out all info! calls
// use once_cell::sync::Lazy;	// use unsync in single threaded programs
use once_cell::unsync::Lazy;	// use sync in multi threaded programs
use std::collections::HashMap;

/* ******************************************************************************* */
/* Constants */
const DEV_BUF_LEN: usize = 40;	// Virpil devices use a buff length of 37 bytes
const JS_MAX: u16 = 0xEA60;		// decimal 60,000; nearly 0xffff
const JS_MID: u16 = JS_MAX / 2;
const JS_MAX_F: f32 = JS_MAX as f32;

pub enum Message {
	Err(String),
	None,
}

/* Module static variables */
static mut API: Lazy<Api> = Lazy::new(|| {	Api::new()	});
pub static mut DEVICES_REPORTS: Lazy<HashMap<u32, DeviceReport>> = Lazy::new(|| {
	let m = HashMap::new();
    m
});
static mut JOYSTICKS: Vec<Joystick> = Vec::new();
static mut JS_DEVICES: Lazy<HashMap<u32, hidapi::HidDevice>> = Lazy::new(|| {
	let m = HashMap::new();
    m
});

pub static mut TIME_OUT: i32 = -1;

/* ******************************************************************************* */
/* Structures */
/* ******************************************************************************* */
struct Api {
	hid_api: HidApi,
}

impl Api {
	fn new() -> Api {
		Api { hid_api: HidApi::new()
			.expect( "HidError "), }
	}
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AxisReport {
	pub axis: String,
	pub value: u16,
	pub calibrate: i128,
}

/* ******************************************************************************* */
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DeviceReport {
	pub col: usize,
	pub name: String,
	pub error: bool,
	pub x: u16,
	pub y: u16,
	pub z: u16,
	pub rx: u16,
	pub ry: u16,
	pub rz: u16,
	pub slider_0: u16,
	pub slider_1: u16,
	pub buttons: Vec<u8>,
	pub x_calibrate: i128,
	pub y_calibrate: i128,
	pub z_calibrate: i128,
	pub rx_calibrate: i128,
	pub ry_calibrate: i128,
	pub rz_calibrate: i128,
	pub slider_0_calibrate: i128,
	pub slider_1_calibrate: i128,
}

impl DeviceReport {
	fn new( js: &Joystick ) -> DeviceReport {
			// x_calibrate: i128, y_calibrate: i128, z_calibrate: i128) -> DeviceReport {
		DeviceReport {
			name : js.name.clone(),
			col : js.col,
			error: true,
			x : 0,
			y : 0,
			z : 0,
			rx : 0,
			ry : 0,
			rz : 0,
			slider_0 : 0,
			slider_1 : 0,
			buttons : Vec::new(),
			x_calibrate: js.x.calibrate,
			y_calibrate: js.y.calibrate,
			z_calibrate: js.z.calibrate,
			rx_calibrate: js.rx.calibrate,
			ry_calibrate: js.ry.calibrate,
			rz_calibrate: js.rz.calibrate,
			slider_0_calibrate: js.slider_0.calibrate,
			slider_1_calibrate: js.slider_1.calibrate,
		}
	}

	pub fn x_f32( &self ) -> f32 {
		self.x as f32 / JS_MAX_F
	}

	pub  fn y_f32( &self ) -> f32 {
		self.y as f32 / JS_MAX_F
	}

	pub fn z_f32( &self ) -> f32 {
		self.z as f32 / JS_MAX_F
	}
}

/* ******************************************************************************* */
#[derive(Debug, Clone, )]
pub struct JoystickAxis {
	a0 : usize,
	a1 : usize,
	label : String,
	invert: bool,
	calibrate: i128,
}

impl JoystickAxis {
	fn new() -> JoystickAxis {
		JoystickAxis {
			a0: 0,
			a1: 0,
			label: "".to_string(),
			invert: false,
			calibrate: 0,
		}
	}

	fn config_split_axis(&mut self, val: &String ) {
		let vals = val.split(",");
	
		let mut counter = 0;
		
		for v in vals {
			match counter {
				0 => {
					match v.trim().parse::<usize>() {
						Ok( n ) => {
							self.a0 = n;
						}
						Err( _e ) => {}
					}
				}
				1 => {
					match v.trim().parse::<usize>() {
						Ok( n ) => {
							self.a1 = n;
						}
						Err( _e ) => {}
					}
				}
				3 => {
					match v.trim().parse::<bool>() {
						Ok( b ) => {
							self.invert = b;
						}
						Err( _e ) => {}
					}
				}
				4 => {
					match v.trim().parse::<i128>() {
						Ok( n ) => {
							self.calibrate = n;
						}
						Err( _e ) => {}
					}
				}
				_ => { // other => {
					self.label = v.trim().clone().to_owned();
				}
			}
			counter += 1;
		}
	}
}


#[derive(Debug, Clone, )]
pub struct Joystick {
	vid: u16,
	pid: u16,
	hash: u32,
	name: String,
	x: JoystickAxis,
	y: JoystickAxis,
	z: JoystickAxis,
	rx: JoystickAxis,
	ry: JoystickAxis,
	rz: JoystickAxis,
	slider_0: JoystickAxis,
	slider_1: JoystickAxis,
	col: usize,
	log_device: bool,
	echo_x : u32,
	echo_y : u32,
	echo_z : u32,
	buttons : Vec< usize >,
}

impl Joystick {
	fn new() -> Joystick {
		let vec: Vec<usize> = Vec::new();
		Joystick {
			vid: 0,
			pid: 0,
			hash: 0,
			name: "".to_string(),
			x: JoystickAxis::new(),
			y: JoystickAxis::new(),
			z: JoystickAxis::new(),
			rx: JoystickAxis::new(),
			ry: JoystickAxis::new(),
			rz: JoystickAxis::new(),
			slider_0: JoystickAxis::new(),
			slider_1: JoystickAxis::new(),
			col: std::usize::MAX,
			log_device: false,
			echo_x: 0,
			echo_y: 0,
			echo_z: 0,
			buttons: vec,
		}
	}

	#[named]
	fn set_config_values(&mut self, value_map: HashMap<String, Option<String>>) -> Vec<Message> {	
		let mut ret: Vec<Message> = Vec::new();
		for (key, value ) in value_map {
			match &value {
				Some( val ) => {
					let key_s = key.as_str();
					match key_s {
						"vid" => {
							self.vid = hex_to_u16( val );
						}
						"pid" => {
							self.pid = hex_to_u16( val );
						}
						"x" => {	self.x.config_split_axis( val );	}
						"y" => {	self.y.config_split_axis( val );	}
						"z" => {	self.z.config_split_axis( val );
									self.z.invert = !self.z.invert;			// dirty, could do better
								}
						"rx" => {	self.rx.config_split_axis( val );	}
						"ry" => {	self.ry.config_split_axis( val );	}
						"rz" => {	self.rz.config_split_axis( val );	}
						"slider_0" => {	self.slider_0.config_split_axis( val );	}
						"slider_1" => {	self.slider_1.config_split_axis( val );	}
						"col" => {	self.col = num_to_usize( val );	}
						"log_device" => {
							match val.parse::<bool>() {
								Ok( b ) => {
									self.log_device = b;
								}
								Err( err ) => {
									ret.push( show_error(module_path!(),
											function_name!(), 
											format!("Error reading 'log_device' {}", err)));
								}
							}
						}
						"buttons" => {	self.set_buttons( value);	}
						"echo_x" => {	self.set_echo( 'x', val );	}
						"echo_y" => {	self.set_echo( 'y', val );	}
						"echo_z" => {	self.set_echo( 'z', val );	}
						"comment" => { /* just consume comments */ }
						other => {
							ret.push( show_error(module_path!(),
												function_name!(),
												format!(
													"Unknown key: {}", other) ) );
						}
					}
				}
				None => {
					ret.push( show_error(module_path!(),
										function_name!(),
										format!("Error for: Key {} -> Value {:?}", key, value)));
				}
			}
		}								
		self.set_hash();
		info!("{}::{} has read {}",
				module_path!(),
				function_name!(),
				self.vid_pid());
		ret
	}

	#[named]
	fn read_device( &mut self, buff : &mut [  u8 ]) -> Vec<Message> {
		let mut ret: Vec<Message> = Vec::new();
		let device: Option< &HidDevice>;
		unsafe {
			device =JS_DEVICES.get(&self.hash);
		}
		
		match device {
			Some( dev ) => {
				let time_out: i32;
				unsafe{
					time_out = TIME_OUT;
				}
				match dev.read_timeout(buff, time_out) { //match dev.read(buff) {
					Ok( bsize ) => {
						if bsize >= DEV_BUF_LEN {
							ret.push(
								show_error(module_path!(), function_name!(),
									format!(
										"Device buffer too small (DEV_BUF_LEN = {})",
										DEV_BUF_LEN) ) );
						}
					}
					Err( err ) => {
						ret.push( show_error(module_path!(), function_name!(),
								format!("{} {}",self.vid_pid(), err))) ;
						self.set_name();
					}
				}
			}
			None => {
				info!("{}::{} {} None",
				module_path!(), function_name!(),
				self.vid_pid() );
				self.set_name();
			}
		}
		
		ret
	}

	fn set_buttons( &mut self, value: Option<String>) {
		match value {
			Some( values) => {
				for val in values.split(",") {
					match val.trim().parse::<usize>() {
						Ok( v ) => {
							self.buttons.push(v);
						}
						Err(_) => {}
					}
				}
			}
			None => {}
		}
	}

    #[named]
	fn set_echo( &mut self, axis: char, value: &String ) {
		let mut echo_hash: u32 = 0;
		for val in value.split(" ") {
			echo_hash = echo_hash * 0x10000
							+ hex_to_u16( &val.to_string() ) as u32;
		}
		match axis {
			'x' => {	self.echo_x = echo_hash;	}
			'y' => {	self.echo_y = echo_hash;	}
			'z' => {	self.echo_z = echo_hash;	}
			_ => {}
		}
		info!("{}::{} {} axis from {:04x} {:04x}",
				module_path!(), function_name!(),
				// self.vid,		// nice to have, but may not be known yet
				// self.pid,		// nice to have, but may not be known yet
				&axis,
				echo_hash / 0x10000,
				echo_hash % 0x10000);
	}

	fn set_hash(&mut self) {
		self.hash = self.vid as u32 * 0x10000 + self.pid as u32;
	}

	#[named]
	fn set_name(&mut self ) -> Message {
		let api: &HidApi;
		unsafe{ 
			api = &API.hid_api;
		}
		let mut ret: Message = Message::None;
		match api.open( self.vid, self.pid ) {
			Ok( device ) => {
				match device.get_product_string() {
					Ok( pr_string ) => {
						match pr_string {
							Some(name) => {
								self.name = name;
							}
							None => {}
						}
					}
					Err( err ) => {
						ret = show_error(module_path!(), function_name!(), 
							format!("Error {} - could not name device {}",
											err, self.vid_pid()) );
					}
				}				
				unsafe {
					JS_DEVICES.insert(self.hash, device);
				}
			}
			Err( err ) => {
				ret = show_error(module_path!(), function_name!(),
								format!("{} - could not open {}",
											err, self.vid_pid()));
			}
		}
		ret
	}

	fn vid_pid( &self ) -> String {
		format!("{:04x} {:04x}", self.vid, self.pid)
	}
}

/* ******************************************************************************* */

#[named]
pub fn check_devices() -> Vec<Message> {
	let mut ret: Vec<Message> = Vec::new();
	
	let mut joysticks: Vec<Joystick>;
	unsafe {
		joysticks = JOYSTICKS.clone();
	}
	
	for js in &mut joysticks {
		let buff: &mut [u8] = &mut [0; DEV_BUF_LEN];
		ret.append(&mut js.read_device( buff) );
		
		if js.log_device {
			let mut buff_st:String = "".to_string();
			for n in 0..buff.len() {
				buff_st = format!("{}\t{}", buff_st, buff[ n ]).to_string();
			}
			info!("{}::{} -> {:04x} {:04x}: {}", module_path!(), function_name!(), js.vid, js.pid, buff_st);
		}
		make_device_report( &js, buff );
	}
	
	do_echo(&joysticks);
	ret
}

/* ******************************************************************************* */

fn do_echo(joysticks: &Vec<Joystick>) {
	for js in joysticks {
		unsafe {
			match DEVICES_REPORTS.get(&js.hash) {
				Some( dr ) => {
					let mut dr = dr.clone();
					match DEVICES_REPORTS.get(&js.echo_x) {
						Some( xjs ) => { dr.x = xjs.x; }
						_ => {}
					}
					match DEVICES_REPORTS.get(&js.echo_y) {
						Some( yjs ) => { dr.y = yjs.y; }
						_ => {}
					}
					match DEVICES_REPORTS.get(&js.echo_z) {
						Some( zjs ) => { dr.z = zjs.z; }
						_ => {}
					}
					DEVICES_REPORTS.insert(js.hash, dr);
				}
				_ => {}
			}
		}
	}
}

/* ******************************************************************************* */

fn device_report_axis( axis: JoystickAxis, buff : &[u8] ) -> u16 { // a_0 : usize, a_1 : usize, invert: bool, buff : &[u8] ) -> u16 {
	if buff[ 0 ] == 0	{ return JS_MID; }
	if axis.a0 <= 0			{ return JS_MID; }
	if axis.a1 <= 0			{ return JS_MID; }

	let len = buff.len();
	if axis.a0 >= len		{ return JS_MID; }
	if axis.a1 >= len		{ return JS_MID; }

	let ret = buff[ axis.a0 ] as u16 * 0x100 + buff[ axis.a1 ] as u16;

	if axis.invert {
		return JS_MAX - ret;
	}
	ret
}

/* ******************************************************************************* */

fn hex_to_u16( value: &String) -> u16 {
	match u16::from_str_radix( value.trim(), 16) {
		Ok( val) => { val }
		Err( _e ) => { 0 }
	}
}

/* ******************************************************************************* */

#[named]
pub fn load_devices( frame_rate: i32 ) -> Vec<Message> { // (api: &HidApi) -> Vec<Message> {
	let mut ret: Vec<Message> = Vec::new();
	let mut config = Ini::new();

	match config.load("./config/joystick_monitor.ini") {
		Ok( res ) => {
			for (k,v) in res {
				if k.to_lowercase() == "comment" { continue; }
				let mut js = Joystick::new();
				ret.append( &mut js.set_config_values(v) );
				unsafe {
					ret.push( js.set_name() );
					JOYSTICKS.push(js);
				}
			}
		}
		Err(err) => {
			ret.push( show_error(module_path!(), function_name!(), err));
		}
	}
	unsafe {
		TIME_OUT = frame_rate / (JOYSTICKS.len() as i32 + 1);
	}
	ret
}

/* ******************************************************************************* */

fn make_device_report(js : &Joystick, buff : &[u8]) {
	unsafe {
		match DEVICES_REPORTS.get(&js.hash) {
			None => {
				let dr = DeviceReport::new( js ) ; // DeviceReport::new( js.name.clone(), js.col, js.x_calibrate, js.y_calibrate, js.z_calibrate);
				DEVICES_REPORTS.insert(js.hash, dr);
			}

			Some( dr ) => {
				let js = js.clone();
				let mut dr = dr.clone();
				dr.x = device_report_axis(js.x, buff);	//(js.x.a0, js.x.a1, js.x.invert, buff);
				dr.y = device_report_axis(js.y, buff);	//(js.y.a0, js.y.a1, js.y.invert, buff);
				dr.z = device_report_axis(js.z, buff);	//(js.z.a0, js.z.a1, js.z.invert, buff);
				dr.rx = device_report_axis(js.rx, buff);	//(js.rx.a0, js.rx_1, js.rx.invert, buff);
				dr.ry = device_report_axis(js.ry, buff);	//(js.ry.a0, js.ry_1, js.ry.invert, buff);
				dr.rz = device_report_axis(js.rz, buff);	//(js.rz_0, js.rz_1, js.rz.invert, buff);
				dr.slider_0 = device_report_axis(js.slider_0, buff);	//(js.slider_0_0, js.slider_0_1, js.slider_0_invert, buff);
				dr.slider_1 = device_report_axis(js.slider_1, buff);	//(js.y_0, js.y_1, js.y_invert, buff);

				if buff[ 0 ] == 0 {
					dr.error = true;
				} else {
					if dr.error {
						dr.error = (dr.x == JS_MID) & (dr.y == JS_MID) & (dr.z == JS_MID) ;
					// } else {
					//	dr.error = false;		// no change
					}
				}

				dr.buttons = Vec::new();

				for b in 0..js.buttons.len() {
					dr.buttons.push(buff[ js.buttons[ b ]]); // .push( buff[ b ]);
				}
				DEVICES_REPORTS.insert(js.hash, dr);
			}
		}
	}
}

/* ******************************************************************************* */

fn num_to_usize( value: &String) -> usize {
	match i8::from_str_radix( value.as_str(), 10) {
		Ok( val) => {
			if val >= 0 {
				return val as usize;
			}
			return std::usize::MAX;
		}
		Err( _e ) => { 0 }
	}
}

/* ******************************************************************************* */

pub fn show_error(module_path: &str, function_name: &str, message: String) -> Message {
	let inf = format!("{}::{} {}", module_path, function_name, message );
	info!( "{}", &inf );
	Message::Err( inf )
}

/* ******************************************************************************* *
 *		*** End ***
 * ******************************************************************************* */