use c8common::control::{ControlledInterpreter, FrameInfo, InterpreterState};
use c8common::hooks::{HookInternalAccess, InterpreterHook};
use c8common::Display;
use image::{GrayImage, Luma};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Recorder {
    info_file: File,
    step_number: u64,
    frame_number: u64,
    mode: RecorderMode,
}

impl Recorder {
    pub fn images_to_folder(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            info_file: File::create(path.join("frames.json")).unwrap(),
            step_number: 0,
            frame_number: 0,
            mode: RecorderMode::Images { folder: path },
        }
    }

    pub fn compact(path: impl Into<PathBuf>) -> Self {
        Self {
            info_file: File::create(path.into()).unwrap(),
            step_number: 0,
            frame_number: 0,
            mode: RecorderMode::Compact,
        }
    }

    fn write_new_frame(&mut self, frame: Display) {
        match self.mode {
            RecorderMode::Images { ref folder } => {
                let new_image_path = folder.join(format!("{}.png", self.frame_number));
                self.open();
                self.write_common();
                write!(
                    self.info_file,
                    ", \"path\": {}",
                    new_image_path.file_name().unwrap().to_str().unwrap()
                )
                .unwrap();
                self.close();
                let mut image = GrayImage::new(64, 32);
                for (y, row) in frame.raw().iter().enumerate() {
                    for (x, &pixel) in row.iter().enumerate() {
                        image.put_pixel(
                            x as u32,
                            y as u32,
                            if pixel as usize == 1 {
                                Luma([255])
                            } else {
                                Luma([0])
                            },
                        )
                    }
                }
                image.save(new_image_path).unwrap();
            }
            RecorderMode::Compact => {
                self.open();
                self.write_common();
                write!(self.info_file, ", \"data\": [\"").unwrap();
                let mut row_comma = false;
                for row in frame.raw() {
                    if row_comma {
                        write!(self.info_file, "\",\"").unwrap();
                    }
                    for pixel in row {
                        write!(self.info_file, "{}", *pixel as usize).unwrap();
                    }
                    row_comma = true;
                }
                write!(self.info_file, "\"]").unwrap();
                self.close();
            }
        }
        self.frame_number += 1;
    }

    fn write_common(&mut self) {
        write!(
            self.info_file,
            "\"frame\": {}, \"step\": {}, ",
            self.frame_number, self.step_number
        )
        .unwrap();
    }

    fn open(&mut self) {
        write!(self.info_file, "{{").unwrap();
    }

    fn close(&mut self) {
        writeln!(self.info_file, "}}").unwrap();
    }
}

impl<T: ControlledInterpreter> InterpreterHook<T> for Recorder {
    fn after_step(&mut self, int: &mut T, frame: &mut FrameInfo) {
        if self.frame_number == 0
            || <Self as HookInternalAccess<T>>::is_modify_screen(&*self, frame)
        {
            self.write_new_frame(int.display().clone())
        }
    }

    fn post_cycle(&mut self, _: &mut InterpreterState) {
        self.step_number += 1;
    }
}

#[derive(Debug, Clone)]
pub enum RecorderMode {
    Images { folder: PathBuf },
    Compact,
}
