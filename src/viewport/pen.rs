use std::collections::HashSet;

use super::canvas::*;

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
            (dx.pow(2) + dy.pow(2)) as f64 <= r.powf(2.0)
        })
        .map(move |(x, y)| {
            let x = x + p_x.round() as i32;
            let y = y + p_y.round() as i32;
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
            let ratio =
                (dx * relative_x + dy * relative_y) / (dx.powi(2) + dy.powi(2));
            if ratio < 0.0 || 1.0 < ratio {
                false
            } else {
                let border_d = ratio * r2 + (1.0 - ratio) * r1;
                let d = (hx * relative_x + hy * relative_y).abs()
                    / (hx.powi(2) + hy.powi(2)).sqrt();
                border_d > d
            }
        },
    )
}

fn pressure_to_radias(p: f64) -> f64 {
    p * 20.0
}

fn circle_pen_outline(
    input: &PenInput,
    previous_input: &Option<PenInput>,
) -> HashSet<(i32, i32)> {
    match previous_input {
        None => circle(pressure_to_radias(input.pressure), input.x, input.y)
            .collect(),
        Some(previous_input) => {
            let previous_size = pressure_to_radias(previous_input.pressure);
            let size = pressure_to_radias(input.pressure);
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
) -> impl Iterator<Item = ((i32, i32), RGB)> {
    let h = circle_pen_outline(input, previous_input);
    h.into_iter().map(|p| (p, RGB::new(0, 0, 0)))
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
