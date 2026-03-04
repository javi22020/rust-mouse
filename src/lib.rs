use enigo::{Coordinate, Enigo, Mouse, Settings};
use pyo3::prelude::*;
use rand::{rng, Rng};
use std::f64::consts::PI;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Clone, Copy, Debug)]
enum EasingFunction {
    QuarticOut,
    CubicOut,
    QuintOut,
    ExpoOut,
    SineOut,
    CircOut,
}

impl EasingFunction {
    fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..6) {
            0 => EasingFunction::QuarticOut,
            1 => EasingFunction::CubicOut,
            2 => EasingFunction::QuintOut,
            3 => EasingFunction::ExpoOut,
            4 => EasingFunction::SineOut,
            _ => EasingFunction::CircOut,
        }
    }
    
    fn apply(&self, x: f64) -> f64 {
        match self {
            EasingFunction::QuarticOut => 1.0 - (1.0 - x).powi(4),
            EasingFunction::CubicOut => 1.0 - (1.0 - x).powi(3),
            EasingFunction::QuintOut => 1.0 - (1.0 - x).powi(5),
            EasingFunction::ExpoOut => {
                if x >= 1.0 { 1.0 } else { 1.0 - 2.0_f64.powf(-10.0 * x) }
            }
            EasingFunction::SineOut => (x * PI / 2.0).sin(),
            EasingFunction::CircOut => (1.0 - (1.0 - x).powi(2)).sqrt(),
        }
    }
}

fn cubic_bezier(t: f64, p0: Point, p1: Point, p2: Point, p3: Point) -> Point {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let x = uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x;
    let y = uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y;

    Point::new(x, y)
}

fn calculate_fitts_duration(distance: f64, target_width: f64, rng: &mut impl Rng) -> u64 {
    let a = 200.0;
    let b = 250.0;
    let id = (distance / target_width + 1.0).log2();
    let base_duration = a + b * id;
    let random_factor = rng.random_range(0.85..1.15);
    (base_duration * random_factor).round() as u64
}

fn multi_octave_noise(t: f64, phase_x: f64, phase_y: f64, octaves: u8) -> Point {
    let mut x_noise = 0.0;
    let mut y_noise = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    
    for _ in 0..octaves {
        x_noise += ((t * frequency * 2.0 * PI + phase_x).sin() 
                   + (t * frequency * 7.0 * PI + phase_x * 1.3).sin() * 0.5) * amplitude;
        y_noise += ((t * frequency * 3.0 * PI + phase_y).cos() 
                   + (t * frequency * 5.0 * PI + phase_y * 0.7).cos() * 0.5) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    Point::new(x_noise, y_noise)
}

fn calculate_tremor(t: f64, magnitude: f64, phase_x: f64, phase_y: f64, speed_factor: f64) -> Point {
    let noise = multi_octave_noise(t, phase_x, phase_y, 3);
    
    let drift_x = (t * 2.0 * PI + phase_x).sin();
    let drift_y = (t * 3.0 * PI + phase_y).cos();
    
    let jitter_x = (t * 12.0 * PI + phase_x).sin() * 0.3;
    let jitter_y = (t * 12.0 * PI + phase_y).cos() * 0.3;
    
    let damping = 1.0 - t.powi(2);
    let speed_multiplier = 1.0 + speed_factor * 0.5;
    
    Point::new(
        (drift_x + jitter_x + noise.x * 0.3) * magnitude * damping * speed_multiplier,
        (drift_y + jitter_y + noise.y * 0.3) * magnitude * damping * speed_multiplier,
    )
}

fn apply_arrival_drift(enigo: &mut Enigo, target: Point, rng: &mut impl Rng) -> PyResult<()> {
    if rng.random_bool(0.2) {
        return Ok(());
    }
    
    let drift_count = rng.random_range(2..=5);
    let pattern = rng.random_range(0..3);
    
    for i in 0..drift_count {
        let t = i as f64 / drift_count as f64;
        let drift_range = rng.random_range(1.0..3.0);
        
        let (dx, dy) = match pattern {
            0 => {
                let angle = t * 2.0 * PI + rng.random_range(0.0..PI);
                (angle.cos() * drift_range, angle.sin() * drift_range)
            }
            1 => {
                let offset = rng.random_range(-drift_range..drift_range);
                if i % 2 == 0 { (offset, 0.0) } else { (0.0, offset) }
            }
            _ => {
                let angle = t * 4.0 * PI;
                (angle.cos() * drift_range * 0.5, angle.sin() * drift_range)
            }
        };
        
        let new_x = target.x + dx;
        let new_y = target.y + dy;
        
        let _ = enigo.move_mouse(new_x as i32, new_y as i32, Coordinate::Abs);
        spin_sleep::sleep(Duration::from_millis(rng.random_range(30..=80)));
    }
    
    let _ = enigo.move_mouse(target.x as i32, target.y as i32, Coordinate::Abs);
    Ok(())
}

fn should_add_mistake(rng: &mut impl Rng, overshoot: bool) -> bool {
    !overshoot && rng.random_bool(0.05)
}

fn calculate_pause_points(distance: f64, rng: &mut impl Rng) -> Vec<f64> {
    if distance < 500.0 {
        return vec![];
    }
    
    let pause_count = if distance > 1000.0 {
        rng.random_range(1..=2)
    } else {
        if rng.random_bool(0.5) { 1 } else { 0 }
    };
    
    let mut pauses = Vec::new();
    for _ in 0..pause_count {
        pauses.push(rng.random_range(0.3..0.7));
    }
    pauses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    pauses
}

fn apply_pause_with_drift(enigo: &mut Enigo, current: Point, rng: &mut impl Rng) -> PyResult<()> {
    let pause_duration = rng.random_range(50..=200);
    let wobble_count = rng.random_range(2..=4);
    
    for _ in 0..wobble_count {
        let wobble_x = current.x + rng.random_range(-3.0..3.0);
        let wobble_y = current.y + rng.random_range(-3.0..3.0);
        let _ = enigo.move_mouse(wobble_x as i32, wobble_y as i32, Coordinate::Abs);
        spin_sleep::sleep(Duration::from_millis(pause_duration / wobble_count));
    }
    
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (target_x, target_y, duration_ms=None, deviation=50.0, overshoot=false, allow_pauses=None))]
fn move_mouse_human(
    target_x: f64,
    target_y: f64,
    duration_ms: Option<u64>,
    deviation: f64,
    overshoot: bool,
    allow_pauses: Option<bool>,
) -> PyResult<()> {
    let settings = Settings::default();
    let mut enigo = Enigo::new(&settings)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    
    let mut rng = rng();
    
    let init_delay = rng.random_range(50..=200);
    spin_sleep::sleep(Duration::from_millis(init_delay));
    
    let (current_x, current_y) = enigo.location()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    
    let start = Point::new(current_x as f64, current_y as f64);
    let end = Point::new(target_x, target_y);

    if start.x == end.x && start.y == end.y {
        return Ok(());
    }

    let distance = start.distance_to(&end);
    
    let movement_duration = match duration_ms {
        Some(dur) => dur,
        None => {
            let target_width = if distance < 100.0 { 50.0 } else { 100.0 };
            calculate_fitts_duration(distance, target_width, &mut rng)
        }
    };
    
    let dist_x = end.x - start.x;
    let dist_y = end.y - start.y;
    
    let should_mistake = should_add_mistake(&mut rng, overshoot);
    let mistake_direction = if should_mistake {
        Some((rng.random_range(-0.15..0.15), rng.random_range(-0.15..0.15)))
    } else {
        None
    };
    
    let p1_factor = rng.random_range(0.15..0.35);
    let p2_factor = rng.random_range(0.75..0.90);
    
    let arc_direction = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
    let wide_arc = rng.random_bool(0.15);
    let arc_deviation = if wide_arc { deviation * 2.0 } else { deviation };
    
    let p1 = if let Some((dx, dy)) = mistake_direction {
        Point::new(
            start.x + dist_x * dx,
            start.y + dist_y * dy,
        )
    } else {
        Point::new(
            start.x + dist_x * p1_factor + rng.random_range(0.0..arc_deviation) * arc_direction,
            start.y + dist_y * p1_factor + rng.random_range(0.0..arc_deviation) * arc_direction,
        )
    };

    let overshoot_factor = if overshoot || should_mistake { 
        rng.random_range(1.08..1.18) 
    } else { 
        1.0 
    };
    
    let p2 = Point::new(
        start.x + dist_x * p2_factor * overshoot_factor + rng.random_range(-10.0..10.0),
        start.y + dist_y * p2_factor * overshoot_factor + rng.random_range(-10.0..10.0),
    );

    let phase_x = rng.random_range(0.0..2.0 * PI);
    let phase_y = rng.random_range(0.0..2.0 * PI);
    
    let easing_function = EasingFunction::random(&mut rng);
    
    let pauses_enabled = allow_pauses.unwrap_or(distance > 500.0);
    let pause_points = if pauses_enabled {
        calculate_pause_points(distance, &mut rng)
    } else {
        vec![]
    };
    let mut pause_idx = 0;

    let base_step_time = if distance < 100.0 { 6.0 } else { 8.0 };
    let steps = (movement_duration as f64 / base_step_time).ceil() as u64;

    for i in 1..=steps {
        let t_linear = i as f64 / steps as f64;
        
        if pause_idx < pause_points.len() {
            let pause_threshold = pause_points[pause_idx];
            if t_linear >= pause_threshold && t_linear < pause_threshold + 0.05 {
                let current_pos = cubic_bezier(easing_function.apply(t_linear), start, p1, p2, end);
                apply_pause_with_drift(&mut enigo, current_pos, &mut rng)?;
                pause_idx += 1;
            }
        }
        
        let jitter_factor = rng.random_range(0.95..1.05);
        let t_eased = easing_function.apply(t_linear * jitter_factor);

        let mut point = cubic_bezier(t_eased, start, p1, p2, end);

        let speed_factor = if distance < 100.0 { 0.5 } else if distance > 500.0 { 1.5 } else { 1.0 };
        let tremor_mag = if overshoot || should_mistake { 
            rng.random_range(2.5..4.0) 
        } else { 
            rng.random_range(1.0..2.0) 
        };
        
        let noise = calculate_tremor(t_linear, tremor_mag, phase_x, phase_y, speed_factor);
        
        point.x += noise.x;
        point.y += noise.y;

        let _ = enigo.move_mouse(point.x as i32, point.y as i32, Coordinate::Abs);

        let step_delay = rng.random_range(6..=12);
        let micro_delay = if rng.random_bool(0.1) { 
            rng.random_range(10..=30) 
        } else { 
            0 
        };
        
        spin_sleep::sleep(Duration::from_millis(step_delay + micro_delay));
    }

    let _ = enigo.move_mouse(target_x as i32, target_y as i32, Coordinate::Abs);
    
    apply_arrival_drift(&mut enigo, end, &mut rng)?;

    Ok(())
}

#[pymodule]
fn rust_mouse(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(move_mouse_human, m)?)?;
    Ok(())
}
