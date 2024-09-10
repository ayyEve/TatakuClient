pub fn interleave(input: &Vec<Vec<f32>>) -> Vec<f32> {
    if input.len() == 0 { return Vec::new() }
    let samples = input[0].len();
    let channels = input.len();

    let mut output = Vec::with_capacity(samples * channels);

    for sample in 0..samples {
        for channel in 0..channels {
            output.push(input[channel][sample])
        }
    }

    output
}

pub fn _deinterleave(input: &Vec<f32>, channels: usize) -> Vec<Vec<f32>> {
    let samples = input.len() / channels;

    let mut output = vec![Vec::with_capacity(samples); channels];

    for (i, sample) in input.into_iter().cloned().enumerate() {
        output[i % channels].push(sample);
    }

    output
}

// todo: write tests for these
