#![feature(step_by)]

extern crate wifi_drone;

use std::convert::AsMut;
use wifi_drone::video::VideoListener;

#[macro_use]
extern crate cpp;

cpp!{{
	#include <opencv2/core/utility.hpp>
	#include <opencv2/tracking.hpp>
	#include <opencv2/videoio.hpp>
	#include <opencv2/highgui.hpp>
	#include <iostream>
	#include <cstring>

	using namespace std;
	using namespace cv;

	static Mat image;
	static Rect2d boundingBox;
	static bool paused;
	static bool selectObject = false;
	static bool startSelection = false;
	static bool initialized = false;
	static Ptr<Tracker> tracker;
	static String tracker_algorithm = "BOOSTING";

	void init() {
	    tracker = Tracker::create(tracker_algorithm);

		boundingBox.x = 347;
		boundingBox.y = 85;
		boundingBox.width = 73;
		boundingBox.height = 70;

		selectObject = true;
	}

    void track(int32_t * & result, const int32_t * & buf, const int32_t width, const int32_t height) {
		Mat frame = Mat(height, width, CV_8UC4, &buf);

        if (!initialized && selectObject) {
            if (!tracker->init(frame, boundingBox)) {
                cout << "***Could not initialize tracker...***\n";
            }

            initialized = true;
        } else if (initialized) {
            tracker->update(frame, boundingBox);

            result[0] = boundingBox.x;
            result[1] = boundingBox.y;
            result[2] = boundingBox.width;
            result[3] = boundingBox.height;
        }
    }
}}

fn main() {
    unsafe {
        cpp!([] {
            init();
        });
    }

    let listener = VideoListener::new(cb);
    wifi_drone::connect(listener);
}

fn cb(data: &mut [u8], width: u32, height: u32) {
//    1658880

    let mut buf: &mut [u32; 414720] = &mut [0; 414720];
    let mut _buf: [u8; 414720] = [0; 414720];
    let mut index = 0;
    let mut pos = 0;

    for b in _buf.into_iter() {
        pos = index * 4;
        buf[index] = data[pos + 0] as u32;
        buf[index] = (buf[index] << 8) + (data[pos + 1] as u32);
        buf[index] = (buf[index] << 8) + (data[pos + 2] as u32);
        index = index + 1;
    }

    let mut bounding_box: &mut [u32; 4] = &mut [0; 4];
    unsafe {
        cpp!([mut bounding_box as "int32_t *", buf as "int32_t *", width as "int32_t", height as "int32_t"] {
            track(bounding_box, buf, width, height);
        });
    }

    let _x = bounding_box[0] * 4;
    let _y = bounding_box[1] * 4;
    let _width = bounding_box[2] * 4;
    let _height = bounding_box[3] * 4;

    for y in (_y.._y + _height).step_by(4) {
        for x in (_x.._x + _width).step_by(4) {
            if x > _x && x < _x + _width - 4 && y > _y && y < _y + _height - 4 {
                continue;
            }

            let pos = y * width + x;
            data[(pos + 0) as usize] = 255;
            data[(pos + 1) as usize] = 255;
            data[(pos + 2) as usize] = 0;
            data[(pos + 3) as usize] = 255;
        }
    }
}