use std::iter::repeat;

mod pen;

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

pub struct Canvas {
    pub drawer: Box<dyn Fn(&SingleVecImage, (usize, usize))>,
    viewport_size: (usize, usize),
    canvas_size: (usize, usize),
    pub image: SingleVecImage,
    #[allow(dead_code)]
    background_color: RGB,
    previous_input: Option<PenInput>,
}

impl Canvas {
    pub fn new(
        drawer: Box<dyn Fn(&SingleVecImage, (usize, usize))>,
        canvas_size: (usize, usize),
    ) -> Canvas {
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
        }
    }

    pub fn pen_stroke(&mut self, input: PenInput) {
        let canvas_w = self.canvas_size.0 as i32;
        let canvas_y = self.canvas_size.1 as i32;
        let _changed_pixels = pen::circle_pen(&input, &self.previous_input)
            .filter(|((x, y), _)| {
                0 <= *x && *x < canvas_w && 0 <= *y && *y < canvas_y
            })
            .map(|((x, y), color)| ((x as u32, y as u32), color))
            .map(|((x, y), color)| {
                self.image.set(x as usize, y as usize, color);
                ((x, y), color)
            })
            .collect::<Vec<_>>();
        self.previous_input = Some(input);
        (self.drawer)(&self.image, self.viewport_size);
    }

    pub fn pen_stroke_end(&mut self) {
        self.previous_input = None;
    }

    pub fn reflect_all(&mut self) {
        (self.drawer)(&self.image, self.viewport_size);
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
}
