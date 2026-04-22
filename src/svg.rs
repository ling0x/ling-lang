use std::fs;
use crate::vm::Op;

const SIZE: usize = 800;
const CX: f64 = 400.0;
const CY: f64 = 400.0;
const RING_GAP: f64 = 28.0;
const INNER_RADIUS: f64 = 30.0;
const DOT_RADIUS: f64 = 5.0;
const SECTORS: usize = 12;

fn sector_angle(s: usize) -> f64 {
    s as f64 * 2.0 * std::f64::consts::PI / SECTORS as f64
}

fn dot_at(ring: usize, sector: usize) -> (f64, f64) {
    let r = INNER_RADIUS + ring as f64 * RING_GAP;
    let a = sector_angle(sector);
    (CX + r * a.cos(), CY + r * a.sin())
}

fn ring_color(op: &Op) -> &'static str {
    match op {
        Op::Push(_) => "#45a29e",
        Op::Add => "#66fcf1",
        Op::Sub => "#f18f01",
        Op::Mul => "#c3073f",
        Op::Dup => "#950740",
        Op::Print => "#e0e0e0",
        Op::Halt => "#ff0000",
    }
}

pub fn ops_to_svg(ops: &[Op], out: &str) {
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        SIZE, SIZE, SIZE, SIZE
    );
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#0b0c10\"/>\n");

    for (i, op) in ops.iter().enumerate() {
        let r = INNER_RADIUS + i as f64 * RING_GAP;
        let col = ring_color(op);
        svg.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>\n",
            CX, CY, r, col
        ));

        let sectors: Vec<usize> = match op {
            Op::Push(n) => vec![(*n as usize) % SECTORS],
            Op::Add => vec![0, 6],
            Op::Sub => vec![0, 3],
            Op::Mul => vec![0, 4, 8],
            Op::Dup => vec![0, 1],
            Op::Print => vec![0, 3, 6, 9],
            Op::Halt => (0..SECTORS).collect(),
        };

        for s in sectors {
            let (dx, dy) = dot_at(i, s);
            svg.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>\n",
                dx, dy, DOT_RADIUS, col
            ));
        }
    }

    svg.push_str("</svg>");
    fs::write(out, svg).unwrap();
}
