#![feature(step_by)]

extern crate wifi_drone;

use std::fmt;
use std::convert::AsMut;
use wifi_drone::video::VideoListener;
use wifi_drone::network::gamepad::CommandListener;
use wifi_drone::controls::command::{Command, DroneMode};
use wifi_drone::constants;

#[macro_use]
extern crate cpp;

#[macro_use]
extern crate arrayref;

static mut INITIAL_BOUNDING_BOX: Rect = Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0, array: [0.0; 4] };
static mut BOUNDING_BOX: Rect = Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0, array: [0.0; 4] };
static mut SCREEN: Rect = Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0, array: [0.0; 4] };

static mut CMD_ERROR: Rect = Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0, array: [0.0; 4] };
static mut CMD_ERROR_NORM: Rect = Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0, array: [0.0; 4] };

struct Rect { x: f32, y: f32, width: f32, height: f32, array: [f32; 4] }

impl Rect {
    pub fn to_array(&mut self) {
        self.array[0] = self.x;
        self.array[1] = self.y;
        self.array[2] = self.width;
        self.array[3] = self.height;
    }

    pub fn from_array(&mut self, other: Option<&Rect>) {
        let mut array = self.array;
        if other.is_some() {
            array = other.unwrap().array;
        }

        self.x = array[0];
        self.y = array[1];
        self.width = array[2];
        self.height = array[3];
    }
}

impl fmt::Debug for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x = {}, y = {}, width = {}, height = {}", self.x, self.y, self.width, self.height)
    }
}

cpp!{{
    // tracking
	#include <opencv2/core/utility.hpp>
	#include <opencv2/tracking.hpp>
	#include <opencv2/videoio.hpp>
	#include <opencv2/highgui.hpp>

	// detection
	#include <opencv2/objdetect.hpp>
    #include <opencv2/highgui.hpp>
    #include <opencv2/imgproc.hpp>

	#include <iostream>
	#include <cstring>
    #include <chrono>

	using namespace std;
	using namespace cv;
	using namespace chrono;

    static unsigned long lastTracking = 0;
    static unsigned long lastTrackingDuration = 50;

    // tracking
	static Rect2d boundingBox;
	static bool detected = false;
	static bool trackerInitialized = false;
	static String trackerAlgorithm = "BOOSTING";
	static Ptr<Tracker> tracker = Tracker::create(trackerAlgorithm);

	// detection
	static string cascadeName = "out/haarcascade.xml";
    static CascadeClassifier cascade;
    static int numDetections = 0;
    static const int maxNumDetections = 5;
    static vector<Rect> detections [maxNumDetections];

    void retrieveBestBoundingBox() {
        int bestBoundingBox [3] = {0, 0, 0};

        for (int i = 0; i < maxNumDetections; i++) {
            vector<Rect> currentVector = detections[i];

            for (int j = 0; j < currentVector.size(); j++) {
                Rect currentBox = currentVector.at(j);
                int contains = 0;

                for (int _i = 0; _i < maxNumDetections; _i++) {
                    if (_i == i) continue;
                    vector<Rect> otherVector = detections[_i];

                    for (int _j = 0; _j < otherVector.size(); _j++) {
                        Rect otherBox = otherVector.at(_j);
                        Point otherCenter = Point(
                            otherBox.x + (otherBox.width / 2),
                            otherBox.y + (otherBox.height / 2)
                        );
                        if (currentBox.contains(otherCenter)) {
                            contains++;
                        }
                    }
                }

                cout << i << "|" << j << " contains " << contains << " boxes" << endl;
                if (contains > bestBoundingBox[0]) {
                    bestBoundingBox[0] = contains;
                    bestBoundingBox[1] = i;
                    bestBoundingBox[2] = j;
                }
            }
        }

        boundingBox = detections[bestBoundingBox[1]].at(bestBoundingBox[2]);
    }

    void detect(float * & result, Mat frame) {
        if (trackerInitialized || detected) {
            return;
        }

        if (!cascade.load(cascadeName)) {
            cout << "***Could not load classifier cascade...***\n";
        }

        Mat grayFrame, smallFrame;
        vector<Rect> faces;

        cvtColor(frame, grayFrame, COLOR_BGR2GRAY);
        resize(grayFrame, smallFrame, Size(), 1, 1, INTER_LINEAR);
        equalizeHist(smallFrame, smallFrame);
        cascade.detectMultiScale(smallFrame, faces, 1.1, 2, 0 | CASCADE_SCALE_IMAGE, Size(30, 30));

        if (faces.size() > 0) {
            boundingBox = faces.front();

            if (numDetections < maxNumDetections) {
                detections[numDetections] = faces;
                numDetections++;
                for (int i = 0; i < faces.size(); i++) {
                    rectangle(frame, faces.at(i), Scalar(0, 0, 255), 2, 1);
                }
            } else {
                retrieveBestBoundingBox();
                detected = true;
                rectangle(frame, boundingBox, Scalar(0, 255, 255), 2, 1);
            }
        } else {
            boundingBox.x = 0;
            boundingBox.y = 0;
            boundingBox.width = 0;
            boundingBox.height = 0;
        }
    }

    void track(float * & result, Mat frame) {
        if (!trackerInitialized && detected) {
            if (!tracker->init(frame, boundingBox)) {
                cout << "***Could not initialize tracker...***\n";
            }

            trackerInitialized = true;
        } else if (trackerInitialized) {
            tracker->update(frame, boundingBox);
            rectangle(frame, boundingBox, Scalar(0, 255, 0), 2, 1);
        }
    }

    void doWork(float * & result, const int8_t * & buf, const int32_t width, const int32_t height) {
		Mat frame = Mat(height, width, CV_8UC4, &buf);
		frame.data = (uchar*)buf; // wtf, this should not be necessary

        milliseconds ms = duration_cast<milliseconds>(system_clock::now().time_since_epoch());
        if (ms.count() - lastTracking < lastTrackingDuration) {
            rectangle(frame, boundingBox, Scalar(0, 255, 0), 2, 1);
            return;
        }
        lastTracking = ms.count();

        double t = (double)getTickCount();
        if (detected) {
            track(result, frame);
            t = (double)getTickCount() - t;
            lastTrackingDuration = (t*1000/getTickFrequency());
//            cout << "tracking time = " << lastTrackingDuration << " ms" << endl;
            lastTrackingDuration += 50;
        } else {
            detect(result, frame);
            t = (double)getTickCount() - t;
            cout << "detection time = " << (t*1000/getTickFrequency()) << " ms" << endl;
        }

        result[0] = boundingBox.x;
        result[1] = boundingBox.y;
        result[2] = boundingBox.width;
        result[3] = boundingBox.height;
    }
}}

fn main() {
    let video_listener = VideoListener::new(video_callback);
    let command_listener = CommandListener::new(command_callback);
//    let video_listener = VideoListener::new(video_callback_empty);
//    let command_listener = CommandListener::new(command_callback_empty);
//    wifi_drone::connect(constants::get_tcp_path(), video_listener, command_listener);
    wifi_drone::connect("out/ohh.h264".to_owned(), video_listener, command_listener);
}

fn video_callback(data: &mut [u8], width: u32, height: u32) {
//    1658880
    unsafe {
        let mut bounding_box = &mut BOUNDING_BOX.array;
        let buf = data.as_mut_ptr();

        cpp!([mut bounding_box as "float *", buf as "int8_t *", width as "int32_t", height as "int32_t"] {
            doWork(bounding_box, buf, width, height);
        });

        BOUNDING_BOX.from_array(None);

        if SCREEN.width == 0.0 {
            SCREEN.width = width as f32;
            SCREEN.height = height as f32;
        }

        if INITIAL_BOUNDING_BOX.width == 0.0 {
            INITIAL_BOUNDING_BOX.from_array(Some(&BOUNDING_BOX));
        }
    }

    let mut cmd = Command { throttle: 0, yaw: 0, pitch: 0, roll: 0, mode: wifi_drone::controls::command::DroneMode::TookOff, as_array: [0; 8] };
    command_callback(&mut cmd);
    println!("CMD   :  {:?}", cmd);
}

fn command_callback(command: &mut Command) {
    unsafe {
        command.roll = 18;
        command.pitch = 9;

        if SCREEN.width == 0.0 {
            return;
        }
        if command.mode != DroneMode::TookOff {
            return;
        }
        if command.throttle != 0 || (command.pitch != 0 && command.pitch != 9) || (command.roll != 0 && command.roll != 18) || command.yaw != 0 {
            return;
        }

        let screen_width_half: f32 = SCREEN.width / 2.0;
        let box_width_half: f32 = BOUNDING_BOX.width / 2.0;

        let screen_height_half: f32 = SCREEN.height / 2.0;
        let box_height_half: f32 = BOUNDING_BOX.height / 2.0;

        CMD_ERROR.x = screen_width_half  - (BOUNDING_BOX.x + box_width_half);
        CMD_ERROR.y = screen_height_half - (BOUNDING_BOX.y + box_height_half);
        CMD_ERROR_NORM.x = CMD_ERROR.x / screen_width_half;
        CMD_ERROR_NORM.y = CMD_ERROR.y / screen_height_half;

        command.throttle = (90.0 * CMD_ERROR_NORM.y) as i8;
        command.yaw = (90.0 * CMD_ERROR_NORM.x) as i8;

        let area = BOUNDING_BOX.width * BOUNDING_BOX.height;
        let initial_area = INITIAL_BOUNDING_BOX.width * INITIAL_BOUNDING_BOX.height;
        let error_distance = area - initial_area;

//        println!("SCREEN:  {:?}", SCREEN);
//        println!("BOX   :  {:?}", BOUNDING_BOX);
//        println!("ERROR :  {:?}", CMD_ERROR);
//        println!("ERRORN:  {:?}", CMD_ERROR_NORM);
//        println!("ERRDI :  {:?}", error_distance);
//        println!("CMD   :  {:?}", command);
    }
}

fn video_callback_empty(data: &mut [u8], width: u32, height: u32) { }
fn command_callback_empty(command: &mut Command) {
    command.roll = 18;
    command.pitch = 9;
}
