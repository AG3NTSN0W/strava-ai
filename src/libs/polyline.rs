pub struct Polyline {
    pub points_js: Vec<Vec<f64>>,
}

const INTERVAL_METER: f64 = 5.0;
// Merge within 25px equivalent (~25m at this scale)
const MERGE_DISTANCE: f64 = 0.00025; // ~25m in degrees

impl Polyline {
    pub fn new(polylines: Vec<String>) -> Self {
        let mut all_points: Vec<(f64, f64)> = Vec::new();
        for encoded in &polylines {
            let decoded = Self::decode_polyline(encoded);
            all_points.extend(Self::interpolate(&decoded, INTERVAL_METER));
        }

        let points_js = Self::points_js(&all_points);
        Self { points_js }
    }

    fn decode_polyline(encoded: &str) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        let (mut lat, mut lng) = (0i32, 0i32);
        let mut i = 0;
        let bytes: Vec<u8> = encoded.bytes().collect();
        while i < bytes.len() {
            for val in [&mut lat, &mut lng] {
                let mut shift = 0;
                let mut result = 0u32;
                loop {
                    let b = (bytes[i] - 63) as u32;
                    i += 1;
                    result |= (b & 0x1f) << shift;
                    shift += 5;
                    if b < 0x20 {
                        break;
                    }
                }
                *val += if result & 1 != 0 {
                    !(result >> 1) as i32
                } else {
                    (result >> 1) as i32
                };
            }
            points.push((lat as f64 / 1e5, lng as f64 / 1e5));
        }
        points
    }

    fn haversine(a: (f64, f64), b: (f64, f64)) -> f64 {
        let (dlat, dlng) = ((b.0 - a.0).to_radians(), (b.1 - a.1).to_radians());
        let h = (dlat / 2.0).sin().powi(2)
            + a.0.to_radians().cos() * b.0.to_radians().cos() * (dlng / 2.0).sin().powi(2);
        6371000.0 * 2.0 * h.sqrt().asin()
    }

    fn interpolate(points: &[(f64, f64)], interval_m: f64) -> Vec<(f64, f64)> {
        let mut result = Vec::new();
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let dist = Self::haversine(a, b);
            let steps = (dist / interval_m).ceil() as usize;
            for s in 0..steps {
                let t = s as f64 / steps as f64;
                result.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        if let Some(&last) = points.last() {
            result.push(last);
        }
        result
    }

    fn points_js(all_points: &Vec<(f64, f64)>) -> Vec<Vec<f64>> {
        let mut merged: Vec<(f64, f64, u32)> = Vec::new();
        for &pt in all_points {
            let mut found = false;
            for m in merged.iter_mut() {
                let dlat = pt.0 - m.0;
                let dlng = pt.1 - m.1;
                if (dlat * dlat + dlng * dlng).sqrt() <= MERGE_DISTANCE {
                    m.2 += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                merged.push((pt.0, pt.1, 1));
            }
        }

        let max_freq = merged.iter().map(|m| m.2).max().unwrap_or(1);

        // Build JS data array
        merged
            .iter()
            .map(|&(lat, lng, count)| {
                let norm = ((count as f64 / max_freq as f64) * 100.0)
                    .ceil()
                    .min(100.0)
                    .max(1.0);
                let hue = 240.0 - (norm * 2.4);
                vec![lat, lng, norm, hue]
            })
            .collect()
    }
}
