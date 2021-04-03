use std::collections::HashSet;

use super::*;

pub struct PenSetting {
    pub size: f64,
}

fn rectangle(r: i32) -> impl Iterator<Item = (i32, i32)> {
    (-r..=r).flat_map(move |x| (-r..=r).map(move |y| (x, y)))
}

fn circle(r: f64, x: f64, y: f64) -> impl Iterator<Item = (i32, i32)> {
    let p_x = x;
    let p_y = y;
    rectangle(r.ceil() as i32)
        .filter(move |(x, y)| {
            let dx = x.abs();
            let dy = y.abs();
            ((dx.pow(2) + dy.pow(2)) as f64) < r.powi(2)
        })
        .map(move |(x, y)| {
            let x = x + p_x.floor() as i32;
            let y = y + p_y.floor() as i32;
            (x, y)
        })
}

fn rect(
    xr: std::ops::RangeInclusive<i32>,
    yr: std::ops::RangeInclusive<i32>,
) -> impl Iterator<Item = (i32, i32)> {
    xr.flat_map(move |x| yr.clone().map(move |y| (x, y)))
}

fn line(
    r1: f64,
    x1: f64,
    y1: f64,
    r2: f64,
    x2: f64,
    y2: f64,
) -> impl Iterator<Item = (i32, i32)> {
    let x1 = x1.floor();
    let x2 = x2.floor();
    let y1 = y1.floor();
    let y2 = y2.floor();
    let min = |a1: f64, a2: f64| {
        (if a1 < a2 { a1 - r1 } else { a2 - r2 }).floor() as i32
    };
    let max = |a1: f64, a2: f64| {
        (if a1 < a2 { a2 + r2 } else { a1 + r1 }).ceil() as i32
    };
    let dx = x2 - x1;
    let dy = y2 - y1;
    let hx = dy;
    let hy = -dx;
    rect(min(x1, x2)..=max(x1, x2), min(y1, y2)..=max(y1, y2)).filter(
        move |(x, y)| {
            let relative_x = *x as f64 - x1;
            let relative_y = *y as f64 - y1;
            let len_pow_2 = dx.powi(2) + dy.powi(2);
            let ratio_mul_len_pow_2 = dx * relative_x + dy * relative_y;
            if ratio_mul_len_pow_2 < 0.0 || len_pow_2 < ratio_mul_len_pow_2 {
                false
            } else {
                let border_d_mul_len_pow_2 = ratio_mul_len_pow_2 * r2
                    + (len_pow_2 - ratio_mul_len_pow_2) * r1;
                let d_mul_len = (hx * relative_x + hy * relative_y).abs();
                border_d_mul_len_pow_2 > d_mul_len * len_pow_2.sqrt()
            }
        },
    )
}

fn pressure_to_radias(p: f64, size: f64) -> f64 {
    p * size
}

fn circle_pen_outline(
    input: &PenInput,
    previous_input: &Option<PenInput>,
    setting: &PenSetting,
) -> HashSet<(i32, i32)> {
    match previous_input {
        None => circle(
            pressure_to_radias(input.pressure, setting.size),
            input.x,
            input.y,
        )
        .collect(),
        Some(previous_input) => {
            let previous_size =
                pressure_to_radias(previous_input.pressure, setting.size);
            let size = pressure_to_radias(input.pressure, setting.size);
            circle(previous_size, previous_input.x, previous_input.y)
                .chain(circle(size, input.x, input.y))
                .chain(line(
                    previous_size,
                    previous_input.x,
                    previous_input.y,
                    size,
                    input.x,
                    input.y,
                ))
                .collect()
        }
    }
}

pub fn circle_pen(
    input: &PenInput,
    previous_input: &Option<PenInput>,
    setting: &PenSetting,
) -> impl Iterator<Item = ((i32, i32), RGB)> {
    let h = circle_pen_outline(input, previous_input, setting);
    h.into_iter().map(|p| (p, RGB::new(0, 0, 0)))
}

#[allow(dead_code)]
pub fn debug_pen(
    input: &PenInput,
    previous_input: &Option<PenInput>,
    setting: &PenSetting,
) -> impl Iterator<Item = ((i32, i32), RGB)> {
    let blue = RGB::new(0x03, 0xfc, 0xcf);
    match previous_input {
        None => circle(
            pressure_to_radias(input.pressure, setting.size),
            input.x,
            input.y,
        )
        .map(|p| (p, blue))
        .collect::<Vec<_>>()
        .into_iter(),
        Some(previous_input) => {
            let previous_size =
                pressure_to_radias(previous_input.pressure, setting.size);
            let size = pressure_to_radias(input.pressure, setting.size);
            line(
                previous_size,
                previous_input.x,
                previous_input.y,
                size,
                input.x,
                input.y,
            )
            .map(|p| (p, RGB::new(0, 0, 0)))
            .chain(
                circle(previous_size, previous_input.x, previous_input.y)
                    .map(|p| (p, blue)),
            )
            .chain(circle(size, input.x, input.y).map(|p| (p, blue)))
            .collect::<Vec<_>>()
            .into_iter()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_round() {
        assert_eq!(2.6f64.round(), 3.0);
    }
    #[test]
    fn test_rectangle_pen() {
        assert_eq!(
            rectangle(1).collect::<HashSet<_>>(),
            [
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ]
            .iter()
            .copied()
            .collect::<HashSet<_>>()
        );
    }
}
