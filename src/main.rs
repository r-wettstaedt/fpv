#![feature(test)]

extern crate core_foundation;
extern crate core_graphics;
extern crate image;

extern crate test;

use std::{thread, time};
use std::path::Path;
use std::net::{TcpStream, UdpSocket};
use std::io::{Write, Read};
use std::fs::File;

use core_foundation::base::TCFType;
use core_graphics::{display, geometry};

use test::Bencher;

struct PixelImage<'a> {
    pixels: &'a [u8],
    width: usize,
    height: usize,
}

static MAGIC_BYTES_CTRL: &'static [u8] = &[
    0x49, 0x54, 0x64, 0x00, 0x00, 0x00, 0x5D, 0x00, 0x00, 0x00, 0x81, 0x85, 0xFF, 0xBD, 0x2A, 0x29, 0x5C, 0xAD, 0x67, 0x82, 0x5C, 0x57, 0xBE, 0x41, 0x03, 0xF8, 0xCA, 0xE2, 0x64, 0x30, 0xA3, 0xC1,
    0x5E, 0x40, 0xDE, 0x30, 0xF6, 0xD6, 0x95, 0xE0, 0x30, 0xB7, 0xC2, 0xE5, 0xB7, 0xD6, 0x5D, 0xA8, 0x65, 0x9E, 0xB2, 0xE2, 0xD5, 0xE0, 0xC2, 0xCB, 0x6C, 0x59, 0xCD, 0xCB, 0x66, 0x1E, 0x7E, 0x1E,
    0xB0, 0xCE, 0x8E, 0xE8, 0xDF, 0x32, 0x45, 0x6F, 0xA8, 0x42, 0xEE, 0x2E, 0x09, 0xA3, 0x9B, 0xDD, 0x05, 0xC8, 0x30, 0xA2, 0x81, 0xC8, 0x2A, 0x9E, 0xDA, 0x7F, 0xD5, 0x86, 0x0E, 0xAF, 0xAB, 0xFE,
    0xFA, 0x3C, 0x7E, 0x54, 0x4F, 0xF2, 0x8A, 0xD2, 0x93, 0xCD
];
static MAGIC_BYTES_VIDEO_1_1: &'static [u8] = &[
    0x49, 0x54, 0x64, 0x00, 0x00, 0x00, 0x52, 0x00, 0x00, 0x00, 0x0F, 0x32, 0x81, 0x95, 0x45, 0x2E, 0xF5, 0xE1, 0xA9, 0x28, 0x10, 0x86, 0x63, 0x17, 0x36, 0xC3, 0xCA, 0xE2, 0x64, 0x30, 0xA3, 0xC1,
    0x5E, 0x40, 0xDE, 0x30, 0xF6, 0xD6, 0x95, 0xE0, 0x30, 0xB7, 0xC2, 0xE5, 0xB7, 0xD6, 0x5D, 0xA8, 0x65, 0x9E, 0xB2, 0xE2, 0xD5, 0xE0, 0xC2, 0xCB, 0x6C, 0x59, 0xCD, 0xCB, 0x66, 0x1E, 0x7E, 0x1E,
    0xB0, 0xCE, 0x8E, 0xE8, 0xDF, 0x32, 0x45, 0x6F, 0xA8, 0x42, 0xB7, 0x33, 0x0F, 0xB7, 0xC9, 0x57, 0x82, 0xFC, 0x3D, 0x67, 0xE7, 0xC3, 0xA6, 0x67, 0x28, 0xDA, 0xD8, 0xB5, 0x98, 0x48, 0xC7, 0x67,
    0x0C, 0x94, 0xB2, 0x9B, 0x54, 0xD2, 0x37, 0x9E, 0x2E, 0x7A
];
static MAGIC_BYTES_VIDEO_1_2: &'static [u8] = &[
    0x49, 0x54, 0x64, 0x00, 0x00, 0x00, 0x52, 0x00, 0x00, 0x00, 0x54, 0xB2, 0xD1, 0xF6, 0x63, 0x48, 0xC7, 0xCD, 0xB6, 0xE0, 0x5B, 0x0D, 0x1D, 0xBC, 0xA8, 0x1B, 0xCA, 0xE2, 0x64, 0x30, 0xA3, 0xC1,
    0x5E, 0x40, 0xDE, 0x30, 0xF6, 0xD6, 0x95, 0xE0, 0x30, 0xB7, 0xC2, 0xE5, 0xB7, 0xD6, 0x5D, 0xA8, 0x65, 0x9E, 0xB2, 0xE2, 0xD5, 0xE0, 0xC2, 0xCB, 0x6C, 0x59, 0xCD, 0xCB, 0x66, 0x1E, 0x7E, 0x1E,
    0xB0, 0xCE, 0x8E, 0xE8, 0xDF, 0x32, 0x45, 0x6F, 0xA8, 0x42, 0xB7, 0x33, 0x0F, 0xB7, 0xC9, 0x57, 0x82, 0xFC, 0x3D, 0x67, 0xE7, 0xC3, 0xA6, 0x67, 0x28, 0xDA, 0xD8, 0xB5, 0x98, 0x48, 0xC7, 0x67,
    0x0C, 0x94, 0xB2, 0x9B, 0x54, 0xD2, 0x37, 0x9E, 0x2E, 0x7A
];
static MAGIC_BYTES_VIDEO_2: &'static [u8] = &[
    0x49, 0x54, 0x64, 0x00, 0x00, 0x00, 0x58, 0x00, 0x00, 0x00, 0x80, 0x86, 0x38, 0xC3, 0x8D, 0x13, 0x50, 0xFD, 0x67, 0x41, 0xC2, 0xEE, 0x36, 0x89, 0xA0, 0x54, 0xCA, 0xE2, 0x64, 0x30, 0xA3, 0xC1,
    0x5E, 0x40, 0xDE, 0x30, 0xF6, 0xD6, 0x95, 0xE0, 0x30, 0xB7, 0xC2, 0xE5, 0xB7, 0xD6, 0x5D, 0xA8, 0x65, 0x9E, 0xB2, 0xE2, 0xD5, 0xE0, 0xC2, 0xCB, 0x6C, 0x59, 0xCD, 0xCB, 0x66, 0x1E, 0x7E, 0x1E,
    0xB0, 0xCE, 0x8E, 0xE8, 0xDF, 0x32, 0x45, 0x6F, 0xA8, 0x42, 0xEB, 0x20, 0xBE, 0x38, 0x3A, 0xAB, 0x05, 0xA8, 0xC2, 0xA7, 0x1F, 0x2C, 0x90, 0x6D, 0x93, 0xF7, 0x2A, 0x85, 0xE7, 0x35, 0x6E, 0xFF,
    0xE1, 0xB8, 0xF5, 0xAF, 0x09, 0x7F, 0x91, 0x47, 0xF8, 0x7E
];
static GAMEPAD_DATA: &'static [u8] = &[
    0xCC, 0x7F, 0x7F, 0x0, 0x7F, 0x0, 0x7F, 0x33
];

fn main() {
    connect_drone();
//    save_screenshot();
}

fn connect_drone() {
    connect_controls();
    connect_video_1();
}

fn connect_controls() {
    if let Ok(mut stream) = TcpStream::connect("172.16.10.1:8888") {
        println!("Connected to the controls! {:?}", stream);
        send_magic_packets(&stream);

        thread::spawn(move || {
            println!("Created thread");
            connect_gamepad();
        });
    } else {
        println!("Couldn't connect to controls...");
    }
}

fn connect_video_1() {
    if let Ok(mut stream) = TcpStream::connect("172.16.10.1:8888") {
        println!("Connected to the video 1! {:?}", stream);
        let mut magic_video_idx: i8 = 0;
        send_magic_packets_video_1(&stream, magic_video_idx);
    } else {
        println!("Couldn't connect to video 1...");
    }
}

fn connect_video_2() {
    if let Ok(mut stream) = TcpStream::connect("172.16.10.1:8888") {
        println!("Connected to the video 2! {:?}", stream);
        send_magic_packets_video_2(&stream);
    } else {
        println!("Couldn't connect to video 2...");
    }
}

fn connect_gamepad() {
    if let Ok(mut socket) = UdpSocket::bind("0.0.0.0:0") {
        println!("Bound to gamepad! {:?}", socket);
        send_gamepad_data(&socket);
    } else {
        println!("Couldn't connect to gamepad...");
    }
}

fn send_magic_packets(mut stream: &TcpStream) {
    stream.write(MAGIC_BYTES_CTRL);
    println!("Sent magic packets");
}

fn send_magic_packets_video_1(mut stream: &TcpStream, mut magic_video_idx: i8) {
    magic_video_idx = magic_video_idx + 1;

    if magic_video_idx == 1 {
        stream.write(MAGIC_BYTES_VIDEO_1_1);
        println!("Sent magic packets video 1 2");
        send_magic_packets_video_1(&stream, magic_video_idx);
    } else {
        stream.write(MAGIC_BYTES_VIDEO_1_2);
        println!("Sent magic packets video 1 2");
        connect_video_2();
    }
}

fn send_magic_packets_video_2(mut stream: &TcpStream) {
    stream.write(MAGIC_BYTES_VIDEO_2);
    println!("Sent magic packets video 2");

    listen_video_2(&stream);
}

fn send_gamepad_data(mut socket: &UdpSocket) {
    while true {
        let fifty_millis = time::Duration::from_millis(50);
        thread::sleep(fifty_millis);

        socket.send_to(GAMEPAD_DATA, "172.16.10.1:8895");
        println!("Sent gamepad data");
    }
}

fn listen_video_2(mut stream: &TcpStream) {
    let len = 500000;
    let mut data = [0; 500000];
    let mut data_idx = 0;

    while true {
        let mut buffer = [0; 8192];
        let mut buffer_size = 0;

        match stream.take_error() {
            Ok(option) => println!("what {:?}", option),
            Err(e) => println!("Received err: {:?}", e),
        }

        match stream.read(&mut buffer[..]) {
            Ok(size) => buffer_size = size,
            Err(e) => println!("Error reading stream: {:?}", e),
        }

//        println!("Buffer size: {:?}", buffer_size);
        if buffer_size > 0 && buffer_size != 106 {
            for i in 0..buffer_size {
//                print!("{} ", buffer[i]);

                data[data_idx] = buffer[i];
                data_idx = data_idx + 1;
                if data_idx >= len {
                    println!("Greater");
                    break;
                }
            }
        }

//        println!();
        println!("{}", data_idx);
        if data_idx >= len {
            println!("Greater");
            break;
        }
    }

    let mut file = File::create("./out/data.h264").unwrap();
    file.write(&data);
}







fn save_screenshot() {
    unsafe {
        let cg_display_id = display::CGMainDisplayID();

        let cg_point = geometry::CGPoint{x: 0.0, y: 0.0};
        let cg_size = geometry::CGSize{width: 400.0, height: 1050.0};
        let cg_rect = geometry::CGRect{origin: cg_point, size: cg_size};
//        let cg_rect = display::CGDisplayBounds(cg_display_id);

        let pixel_image = get_pixels(&cg_display_id, &cg_rect);
    }
}

unsafe fn get_pixels<'a>(cg_display_id: &'a display::CGDirectDisplayID, cg_rect: &'a display::CGRect) /*-> PixelImage<'a>*/ {
    let cg_image_ref = display::CGWindowListCreateImage(*cg_rect, display::kCGWindowListOptionAll, *cg_display_id, display::kCGWindowImageDefault);

    let cg_image = core_graphics::image::CGImage::wrap_under_get_rule(cg_image_ref);
    let cf_data = cg_image.data();

    let data = cf_data.bytes();

    println!("{} {}", data.len(), cg_image.bits_per_pixel());
    println!("{} {}", cg_rect.size.width, cg_rect.size.height);
    println!("{} {}", cg_image.width(), cg_image.height());

    image::save_buffer(&Path::new("./out/image.png"), data, cg_image.width() as u32, cg_image.height() as u32, image::RGBA(8)).unwrap();

//    PixelImage {
//        pixels: data,
//        width: cg_image.width(),
//        height: cg_image.height(),
//    }
}

#[bench]
fn bench_get_display(b: &mut Bencher) {
    unsafe {
        b.iter(|| {
            let cg_display_id = display::CGMainDisplayID();
            let cg_rect = display::CGDisplayBounds(cg_display_id);
        });
    }
}

#[bench]
fn bench_get_pixels(b: &mut Bencher) {
    unsafe {
        let cg_display_id = display::CGMainDisplayID();
        let cg_rect = display::CGDisplayBounds(cg_display_id);

        b.iter(|| {
            get_pixels(&cg_display_id, &cg_rect);
        });
    }
}