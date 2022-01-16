
// https://github.com/WeirdConstructor/HexoDSP/blob/master/tests/common/mod.rs#L735-L783

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum FFT {
    F16,
    F32,
    F64,
    F128,
    F512,
    F1024,
    F2048,
    F4096,
    F8192,
    F16384,
    F65535,
}

impl FFT {
    pub fn size(&self) -> usize {
        match self {
            FFT::F16      => 16,
            FFT::F32      => 32,
            FFT::F64      => 64,
            FFT::F128     => 128,
            FFT::F512     => 512,
            FFT::F1024    => 1024,
            FFT::F2048    => 2048,
            FFT::F4096    => 4096,
            FFT::F8192    => 8192,
            FFT::F16384   => 16384,
            FFT::F65535   => 65535,
        }
    }
}

/// (frequency, amplitude)
pub fn fft(buf: &mut [f32], size: FFT) -> Vec<(f32, f32)> {
    let len = size.size();
    let mut res = vec![];

    if len > buf.len() {
        println!("len > buf.len");
        return res;
    }

    // Hann window:
    for (i, s) in buf[0..len].iter_mut().enumerate() {
        let w =
            0.5
            * (1.0 
               - ((2.0 * std::f32::consts::PI * i as f32)
                  / (len as f32 - 1.0))
                 .cos());
        *s *= w;
    }

    use rustfft::{FftPlanner, num_complex::Complex};

    let mut complex_buf =
        buf.iter()
           .map(|s| Complex { re: *s, im: 0.0 })
           .collect::<Vec<Complex<f32>>>();

    let mut p = FftPlanner::<f32>::new();
    let fft = p.plan_fft_forward(len);


    fft.process(&mut complex_buf[0..len]);


    let amplitudes: Vec<_> =
        complex_buf[0..len]
        .iter()
        .map(|c| c.norm())
        .collect();
//    println!("fft: {:?}", &complex_buf[0..len]);


    for (i, amp) in amplitudes.iter().enumerate() {
        let freq = (i as f32 * super::AUDIO.sample_rate as f32) / len as f32;
        if freq > 22050.0 {
            // no freqency images above nyquist...
            continue;
        }
//        println!("{:6.0} {}", freq, *amp);
        res.push((freq.round(), *amp));
    }

    // println!("fft -> len: {}, complex: {}, amplitudes: {}, res: {}", len, complex_buf.len(), amplitudes.len(), res.len());
    res
}
