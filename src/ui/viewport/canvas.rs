use std::iter::repeat;

mod pen;
use pen::PenSetting;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RGB {
    array: [u8; 4],
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> RGB {
        RGB {
            array: [b, g, r, 0],
        }
    }

    pub fn r(self) -> u8 {
        self.array[2]
    }

    pub fn g(self) -> u8 {
        self.array[1]
    }

    pub fn b(self) -> u8 {
        self.array[0]
    }
}

#[derive(Debug, Clone)]
pub struct SingleVecImage {
    pub width: usize,
    pub height: usize,
    pub vector: Vec<u8>,
}

impl SingleVecImage {
    pub fn new(
        data: impl Iterator<Item = RGB>,
        width: usize,
        height: usize,
    ) -> SingleVecImage {
        SingleVecImage {
            width,
            height,
            vector: data
                .take(width * height)
                .flat_map(|c| c.array.to_vec())
                .collect(),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: RGB) {
        let i = 4 * (x + self.width * y);
        self.vector[i..i + 4].clone_from_slice(&color.array);
    }

    #[allow(dead_code)]
    fn extend(&mut self, dx: usize, dy: usize, background: RGB) {
        if dx > 0 {
            self.vector = (0..self.height)
                .flat_map(|y| {
                    self.vector[y * self.width * 4..(y + 1) * self.width * 4]
                        .iter()
                        .chain(background.array.iter().cycle().take(dx * 4))
                })
                .copied()
                .collect();
            self.width = self.width + dx;
        };
        if dy > 0 {
            self.height = self.height + dy;
            self.vector.append(
                &mut background
                    .array
                    .iter()
                    .cycle()
                    .take(4 * dy * (self.width + dx))
                    .copied()
                    .collect(),
            );
        };
    }
}

#[derive(Clone, Copy)]
pub struct PenInput {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
}

pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub type DrawHandler = Box<dyn Fn(&SingleVecImage, (usize, usize), Rectangle)>;

pub struct Canvas {
    pub drawer: DrawHandler,
    viewport_size: (usize, usize),
    canvas_size: (usize, usize),
    pub image: SingleVecImage,
    #[allow(dead_code)]
    background_color: RGB,
    previous_input: Option<PenInput>,
    pen_setting: PenSetting,
}

impl Canvas {
    pub fn new(drawer: DrawHandler, canvas_size: (usize, usize)) -> Canvas {
        let background_color = RGB::new(0xff, 0xff, 0xff);
        Canvas {
            drawer,
            viewport_size: canvas_size,
            canvas_size,
            image: SingleVecImage::new(
                repeat(background_color),
                canvas_size.0,
                canvas_size.1,
            ),
            background_color,
            previous_input: None,
            pen_setting: PenSetting { size: 20.0 },
        }
    }

    pub fn pen_stroke(&mut self, input: PenInput) {
        let canvas_w = self.canvas_size.0 as i32;
        let canvas_h = self.canvas_size.1 as i32;
        let mut max_x = 0;
        let mut min_x = canvas_w as u32;
        let mut max_y = 0;
        let mut min_y = canvas_h as u32;
        let changed_pixels =
            pen::circle_pen(&input, &self.previous_input, &self.pen_setting)
                .filter(|((x, y), _)| {
                    0 <= *x && *x < canvas_w && 0 <= *y && *y < canvas_h
                })
                .map(|((x, y), color)| ((x as u32, y as u32), color))
                .inspect(|((x, y), color)| {
                    self.image.set(*x as usize, *y as usize, *color);
                    max_x = max_x.max(*x);
                    min_x = min_x.min(*x);
                    max_y = max_y.max(*y);
                    min_y = min_y.min(*y);
                })
                .collect::<Vec<_>>();
        self.previous_input = Some(input);
        if !changed_pixels.is_empty() {
            let changed_area = Rectangle {
                x: min_x as f64,
                y: min_y as f64,
                width: (max_x - min_x) as f64,
                height: (max_y - min_y) as f64,
            };
            (self.drawer)(&self.image, self.viewport_size, changed_area);
        }
    }

    pub fn pen_stroke_end(&mut self) {
        self.previous_input = None;
    }

    pub fn reflect_all(&mut self) {
        (self.drawer)(
            &self.image,
            self.viewport_size,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: self.viewport_size.0 as f64,
                height: self.viewport_size.1 as f64,
            },
        );
    }

    #[allow(dead_code)]
    pub fn set_viewport_size(&mut self, width: usize, height: usize) {
        let dx = (width as i32) - (self.canvas_size.0 as i32);
        let dy = (height as i32) - (self.canvas_size.1 as i32);
        if dx > 0 {
            self.image.extend(dx as usize, 0, self.background_color);
            self.canvas_size.0 = width;
        }
        if dy > 0 {
            self.image.extend(0, dy as usize, self.background_color);
            self.canvas_size.1 = height;
        }
        self.viewport_size = (width, height);
    }

    pub fn get_size(&self) -> (usize, usize) {
        self.canvas_size
    }

    pub fn set_pen_size(&mut self, size: f64) {
        self.pen_setting.size = size;
    }
}
