use hound::WavReader;
use reqwest::ClientBuilder;
use rustfft::{
    num_complex::{Complex, Complex64},
    num_traits::FromPrimitive,
    FftPlanner,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::Hackattic;

pub struct TouchToneDialing;

#[derive(Deserialize, Debug)]
pub struct TouchToneDialingProblem {
    wav_url: String,
}

#[derive(Serialize, Debug)]
pub struct TouchToneDialingAnswer {
    sequence: String,
}

impl Hackattic for TouchToneDialing {
    const NAME: &'static str = "touch_tone_dialing";
    type Problem = TouchToneDialingProblem;
    type Answer = TouchToneDialingAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        info!("Received WAV url {}", problem.wav_url);

        let client = ClientBuilder::new().cookie_store(true).build()?;

        let response = client.get(problem.wav_url).send().await?;

        let bytes = response.bytes().await?;

        let reader = WavReader::new(bytes.as_ref())?;

        let spec = reader.spec();
        info!("WavSpec: {:?}", spec);

        let sample_rate = spec.sample_rate;
        let data: Vec<i16> = reader.into_samples().flatten().collect();

        info!("Found {} data points", data.len());

        let mut count = 0;

        let mut break_points = find_zero_positions(&data)
            .into_iter()
            .filter(|(s, e)| e - s > 10)
            .flat_map(|(s, e)| [s, e])
            .collect::<Vec<usize>>();
        break_points.insert(0, 0);
        break_points.push(break_points.len());

        let mut v = vec![];

        for chunk in break_points.chunks_exact(2) {
            if let &[mut s, mut e] = chunk {
                if e > s + 10 {
                    while s < data.len() && data[s] == 0 {
                        s += 1;
                    }

                    while e < data.len() && data[e] == 0 {
                        e -= 1;
                    }
                    count += 1;
                    // debug!("Start {:?}", &data[s..s+10]);
                    // debug!("End {:?}", &data[e - 10..e]);
                    debug!("Signal Length: {}", e - s + 1);
                    let (low, high) = {
                        let (f1, f2) = find_top_two_frequencies(&data[s..=e], sample_rate as f64);
                        if f1 < f2 {
                            (f1, f2)
                        } else {
                            (f2, f1)
                        }
                    };
                    info!("low = {low}; high = {high}");
                    let c = find_closest_dtmf_mapping(low, high);
                    v.push(c);
                }
            }
        }
        info!("Found {} signals above zero threshold {}", count, 10);
        let s: String = v.into_iter().collect();
        Ok(TouchToneDialingAnswer { sequence: s })
    }
}

fn find_closest_dtmf_mapping(low: i32, high: i32) -> char {
    let matrix = [
        ['1', '2', '3', 'A'],
        ['4', '5', '6', 'B'],
        ['7', '8', '9', 'C'],
        ['*', '0', '#', 'D'],
    ];

    let i = [697, 770, 852, 941]
        .into_iter()
        .enumerate()
        .fold((None, i32::MAX), |(p, d), (i, v)| {
            let delta = (low - v).abs();
            if delta < d {
                (Some(i), delta)
            } else {
                (p, d)
            }
        })
        .0
        .unwrap();

    let j = [1209, 1336, 1477, 1633]
        .into_iter()
        .enumerate()
        .fold((None, i32::MAX), |(p, d), (i, v)| {
            let delta = (high - v).abs();
            if delta < d {
                (Some(i), delta)
            } else {
                (p, d)
            }
        })
        .0
        .unwrap();

    matrix[i][j]
}

fn find_top_two_frequencies(signal: &[i16], sample_rate: f64) -> (i32, i32) {
    let n = signal.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);

    let mut comp_signal: Vec<Complex<_>> = signal
        .iter()
        .flat_map(|&i| Complex64::from_i16(i))
        .collect();

    fft.process(&mut comp_signal);

    let real_signal_strength: Vec<f64> = comp_signal
        .into_iter()
        .take(n / 2 + 1)
        .map(Complex::norm)
        .collect();

    let real_bins: Vec<f64> = (0..n / 2 + 1)
        .map(|k| (k as f64 * sample_rate) / n as f64)
        .collect();

    assert_eq!(real_signal_strength.len(), real_bins.len());

    let mut pairs: Vec<_> = real_signal_strength.into_iter().zip(real_bins).collect();

    pairs.retain(|(s, f)| !s.is_nan() && !f.is_nan());
    pairs.sort_by(|a, b| b.partial_cmp(a).unwrap());

    debug!("{:?}", &pairs[..2]);

    (pairs[0].1.round() as i32, pairs[1].1.round() as i32)
}

fn find_zero_positions(data: &[i16]) -> Vec<(usize, usize)> {
    let mut positions = vec![];

    let mut start = 0;
    let mut count = 0;

    for (i, &v) in data.iter().enumerate() {
        if v == 0 {
            if count == 0 {
                start = i;
            }
            count += 1;
        } else if count != 0 {
            count = 0;
            positions.push((start, i));
            start = 0;
        }
    }

    if count != 0 {
        positions.push((start, data.len()));
    }

    positions
}

#[test]
fn find_zero_positions_test() {
    assert_eq!(find_zero_positions(&[]), vec![]);
    assert_eq!(find_zero_positions(&[0]), vec![(0, 1)]);
    assert_eq!(find_zero_positions(&[0, 0]), vec![(0, 2)]);
    assert_eq!(find_zero_positions(&[0, 0, 0]), vec![(0, 3)]);
    assert_eq!(find_zero_positions(&[0, 1, 0]), vec![(0, 1), (2, 3)]);
    assert_eq!(find_zero_positions(&[0, 1, 1, 0]), vec![(0, 1), (3, 4)]);
    assert_eq!(
        find_zero_positions(&[0, 1, 1, 0, 0, 0, 1]),
        vec![(0, 1), (3, 6)]
    );
}
