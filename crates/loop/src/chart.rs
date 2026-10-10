//! The main result chart: training and held-out scores against tokens spent,
//! with code size on the same timeline. Written as a standalone SVG.

use std::fmt::Write as _;
use std::path::Path;

use crate::state::{self, Paths, Scores};

/// One point on the timeline: the accepted version after this many tokens.
struct Point {
    tokens: f64,
    scores: Scores,
}

fn pct(right: usize, turns: usize) -> f64 {
    if turns == 0 {
        0.0
    } else {
        100.0 * right as f64 / turns as f64
    }
}

pub fn write(paths: &Paths, out: &Path) -> Result<(), String> {
    let history = state::history(paths);
    let mut points: Vec<Point> = Vec::new();
    let mut current: Option<Scores> = None;
    for e in &history {
        if e.accepted {
            current = Some(e.scores.clone());
        }
        if let Some(s) = &current {
            points.push(Point {
                tokens: e.tokens_cumulative as f64 / 1e6,
                scores: s.clone(),
            });
        }
    }
    if points.is_empty() {
        return Err("no history yet".into());
    }
    let (w, h, left, right, top, bottom) = (900.0, 480.0, 70.0, 80.0, 40.0, 60.0);
    let max_x = points
        .iter()
        .map(|p| p.tokens)
        .fold(0.0_f64, f64::max)
        .max(0.001);
    let max_size = points
        .iter()
        .map(|p| p.scores.size as f64)
        .fold(0.0_f64, f64::max)
        .max(1.0)
        * 1.1;
    let x = |t: f64| left + (w - left - right) * t / max_x;
    let y_pct = |p: f64| top + (h - top - bottom) * (1.0 - p / 100.0);
    let y_size = |s: f64| top + (h - top - bottom) * (1.0 - s / max_size);
    let series = |f: &dyn Fn(&Scores) -> f64, y: &dyn Fn(f64) -> f64| -> String {
        let mut d = String::new();
        let mut last_y = None;
        for (i, p) in points.iter().enumerate() {
            let py = y(f(&p.scores));
            let px = x(p.tokens);
            if i == 0 {
                let _ = write!(d, "M{px:.1},{py:.1}");
            } else {
                // A step: the score holds until the next accepted change.
                if let Some(ly) = last_y {
                    let _ = write!(d, " L{px:.1},{ly:.1}");
                }
                let _ = write!(d, " L{px:.1},{py:.1}");
            }
            last_y = Some(py);
        }
        d
    };
    let train = series(&|s| pct(s.train_right, s.train_turns), &y_pct);
    let held = series(&|s| pct(s.heldout_right, s.heldout_turns), &y_pct);
    let valid = series(&|s| pct(s.valid_right, s.valid_turns), &y_pct);
    let size = series(&|s| s.size as f64, &y_size);
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" font-family="sans-serif" font-size="12">
<rect width="100%" height="100%" fill="white"/>
<text x="{left}" y="22" font-size="15" font-weight="bold">glassmind: share of turns answered right, and code size, against agent tokens spent</text>"#
    );
    for p in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let yy = y_pct(p);
        let _ = writeln!(
            svg,
            r##"<line x1="{left}" x2="{}" y1="{yy:.1}" y2="{yy:.1}" stroke="#ddd"/><text x="{}" y="{:.1}" text-anchor="end">{p}%</text>"##,
            w - right,
            left - 6.0,
            yy + 4.0
        );
    }
    for i in 0..=4 {
        let s = max_size * f64::from(i) / 4.0;
        let _ = writeln!(
            svg,
            r##"<text x="{}" y="{:.1}" fill="#777">{:.0}</text>"##,
            w - right + 6.0,
            y_size(s) + 4.0,
            s
        );
        let t = max_x * f64::from(i) / 4.0;
        let _ = writeln!(
            svg,
            r##"<text x="{:.1}" y="{}" text-anchor="middle">{t:.1}M</text>"##,
            x(t),
            h - bottom + 18.0
        );
    }
    let _ = writeln!(
        svg,
        r##"<text x="{:.1}" y="{}" text-anchor="middle">agent tokens spent (millions)</text>
<text x="{}" y="{:.1}" fill="#777" transform="rotate(90 {} {:.1})" text-anchor="middle">code size (syntax tokens)</text>
<path d="{size}" fill="none" stroke="#999" stroke-width="1.5" stroke-dasharray="4 3"/>
<path d="{train}" fill="none" stroke="#2563eb" stroke-width="2.5"/>
<path d="{held}" fill="none" stroke="#dc2626" stroke-width="2.5"/>
<path d="{valid}" fill="none" stroke="#f59e0b" stroke-width="1.5"/>
<g transform="translate({left},{}) "><rect width="12" height="3" y="-4" fill="#2563eb"/><text x="16">training (train generators, train repos)</text>
<rect x="260" width="12" height="3" y="-4" fill="#dc2626"/><text x="276">held-out (held-out generator and repos)</text>
<rect x="540" width="12" height="3" y="-4" fill="#f59e0b"/><text x="556">validation</text>
<rect x="640" width="12" height="3" y="-4" fill="#999"/><text x="656">code size</text></g>
</svg>"##,
        (left + w - right) / 2.0,
        h - 12.0,
        w - 20.0,
        h / 2.0,
        w - 20.0,
        h / 2.0,
        h - bottom + 38.0
    );
    std::fs::write(out, svg).map_err(|e| e.to_string())
}
