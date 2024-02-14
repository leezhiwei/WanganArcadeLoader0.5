use libc::*;
use poll::*;
use std::ffi::{CStr, CString};
use std::mem::transmute;

pub mod adm;
pub mod al;
pub mod gl;
pub mod hook;
pub mod jamma;
pub mod poll;

#[derive(serde::Deserialize)]
pub struct Config {
	fullscreen: bool,
	input_emu: bool,
	deadzone: f32,
}

pub struct KeyConfig {
	test: KeyBindings,
	service: KeyBindings,
	quit: KeyBindings,

	gear_next: KeyBindings,
	gear_previous: KeyBindings,
	gear_neutral: KeyBindings,
	gear_first: KeyBindings,
	gear_second: KeyBindings,
	gear_third: KeyBindings,
	gear_fourth: KeyBindings,
	gear_fifth: KeyBindings,
	gear_sixth: KeyBindings,

	perspective: KeyBindings,
	intrude: KeyBindings,
	gas: KeyBindings,
	brake: KeyBindings,
	wheel_left: KeyBindings,
	wheel_right: KeyBindings,
}

pub static mut CONFIG: Option<Config> = None;
pub static mut KEYCONFIG: Option<KeyConfig> = None;

pub extern "C" fn adachi() -> c_int {
	true as c_int
}

#[no_mangle]
unsafe extern "C" fn system(command: *const c_char) -> c_int {
	let cstr = CStr::from_ptr(command);
	let str = cstr.to_str().unwrap();
	dbg!(str);
	if str.starts_with("perl") {
		if str.ends_with("/tmp/ifconfig.txt") {
			0
		} else {
			0
		}
	} else {
		let system = CString::new("system").unwrap();
		let _original = dlsym(RTLD_NEXT, system.as_ptr());
		0
	}
}

#[no_mangle]
unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *const () {
	let filename = CStr::from_ptr(filename).to_str().unwrap();
	let filename = filename.replace("/tmp/", "./tmp/");
	let filename = CString::new(filename).unwrap();

	let fopen = CString::new("fopen").unwrap();
	let fopen = dlsym(RTLD_NEXT, fopen.as_ptr());
	let fopen: extern "C" fn(*const c_char, *const c_char) -> *const () = transmute(fopen);
	fopen(filename.as_ptr(), mode)
}

// Redirect clLog to std::cout
static CL_MAIN_ORIGINAL: [u8; 19] = [
	0xB8, 0xE4, 0x5D, 0x8C, 0x08, 0x55, 0x89, 0xE5, 0x57, 0x56, 0x53, 0x81, 0xEC, 0x8C, 0x00, 0x00,
	0x00, 0xFF, 0xE0,
];

static mut CL_MAIN_IMPLEMENTATION: [u8; 28] = [
	0x8B, 0x5C, 0x24, 0x04, 0x83, 0xEC, 0x08, 0x53, 0xB8, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xD0, 0x83,
	0xC4, 0x0C, 0x53, 0xB8, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xD0, 0x5B, 0xC3,
];

unsafe extern "C" fn cl_main(address: *mut ()) {
	hook::write_memory(
		address,
		&(hook::get_symbol("_ZSt4cout") as usize).to_le_bytes(),
	);
}

unsafe fn hook_cl_main() {
	let original = CL_MAIN_ORIGINAL.as_ptr();
	region::protect(
		original,
		CL_MAIN_ORIGINAL.len(),
		region::Protection::READ_WRITE_EXECUTE,
	)
	.unwrap();

	for (i, data) in (original as usize).to_le_bytes().iter().enumerate() {
		CL_MAIN_IMPLEMENTATION[i + 9] = *data;
	}
	let func = cl_main as *const () as usize;
	for (i, data) in func.to_le_bytes().iter().enumerate() {
		CL_MAIN_IMPLEMENTATION[i + 20] = *data;
	}

	let implementation = CL_MAIN_IMPLEMENTATION.as_ptr();
	region::protect(
		implementation,
		CL_MAIN_IMPLEMENTATION.len(),
		region::Protection::READ_WRITE_EXECUTE,
	)
	.unwrap();

	hook::hook(
		hook::get_symbol("_ZN6clMainC1Ev"),
		implementation as *const (),
	);
}

#[ctor::ctor]
unsafe fn init() {
	let exe = std::env::current_exe().unwrap();
	if !exe.ends_with("main") {
		panic!("Not 3DX+");
	}

	if let Ok(toml) = std::fs::read_to_string("config.toml") {
		CONFIG = Some(toml::from_str(&toml).unwrap());
	}

	// Really what I should do is implement a custom serde::Deserialize for KeyBindings
	// but serdes documentation is really confusing when it comes to this
	#[derive(serde::Deserialize)]
	#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
	struct KeyConfigTemp {
		test: Vec<String>,
		service: Vec<String>,
		quit: Vec<String>,

		gear_next: Vec<String>,
		gear_previous: Vec<String>,
		gear_neutral: Vec<String>,
		gear_first: Vec<String>,
		gear_second: Vec<String>,
		gear_third: Vec<String>,
		gear_fourth: Vec<String>,
		gear_fifth: Vec<String>,
		gear_sixth: Vec<String>,

		perspective: Vec<String>,
		intrude: Vec<String>,
		gas: Vec<String>,
		brake: Vec<String>,
		wheel_left: Vec<String>,
		wheel_right: Vec<String>,
	}

	let toml = std::fs::read_to_string("keyconfig.toml").unwrap();
	let keyconfig: KeyConfigTemp = toml::from_str(&toml).unwrap();
	let keyconfig = KeyConfig {
		test: parse_keybinding(keyconfig.test),
		service: parse_keybinding(keyconfig.service),
		quit: parse_keybinding(keyconfig.quit),

		gear_next: parse_keybinding(keyconfig.gear_next),
		gear_previous: parse_keybinding(keyconfig.gear_previous),
		gear_neutral: parse_keybinding(keyconfig.gear_neutral),
		gear_first: parse_keybinding(keyconfig.gear_first),
		gear_second: parse_keybinding(keyconfig.gear_second),
		gear_third: parse_keybinding(keyconfig.gear_third),
		gear_fourth: parse_keybinding(keyconfig.gear_fourth),
		gear_fifth: parse_keybinding(keyconfig.gear_fifth),
		gear_sixth: parse_keybinding(keyconfig.gear_sixth),

		perspective: parse_keybinding(keyconfig.perspective),
		intrude: parse_keybinding(keyconfig.intrude),
		gas: parse_keybinding(keyconfig.gas),
		brake: parse_keybinding(keyconfig.brake),
		wheel_left: parse_keybinding(keyconfig.wheel_left),
		wheel_right: parse_keybinding(keyconfig.wheel_right),
	};
	KEYCONFIG = Some(keyconfig);

	hook::hook_symbol("_ZNK6clHaspcvbEv", adachi as *const ());
	hook::hook_symbol("_ZNK7clHasp2cvbEv", adachi as *const ());
	hook::hook_symbol("_ZN18clSeqBootNetThread3runEPv", adachi as *const ());
	adm::init();
	jamma::init();
	al::load_al_funcs();
	hook_cl_main();
}
