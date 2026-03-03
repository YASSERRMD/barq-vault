use crate::lzma::sys;
use approx::assert_relative_eq;

#[test]
fn test_delta_all_zeros() {
    let input = vec![0.0f32; 10];
    let mut encoded = vec![0.0f32; 10];
    unsafe {
        sys::barq_delta_encode_f32(input.as_ptr(), encoded.as_mut_ptr(), 10);
    }
    assert_eq!(encoded, input);

    let mut decoded = vec![0.0f32; 10];
    unsafe {
        sys::barq_delta_decode_f32(encoded.as_ptr(), decoded.as_mut_ptr(), 10);
    }
    assert_eq!(decoded, input);
}

#[test]
fn test_delta_monotonically_increasing() {
    let input: Vec<f32> = (0..10).map(|x| x as f32).collect();
    let mut encoded = vec![0.0f32; 10];
    unsafe {
        sys::barq_delta_encode_f32(input.as_ptr(), encoded.as_mut_ptr(), 10);
    }
    // [0, 1, 2, 3...] -> [0, 1, 1, 1...]
    assert_eq!(encoded[0], 0.0);
    for i in 1..10 {
        assert_eq!(encoded[i], 1.0);
    }

    let mut decoded = vec![0.0f32; 10];
    unsafe {
        sys::barq_delta_decode_f32(encoded.as_ptr(), decoded.as_mut_ptr(), 10);
    }
    assert_eq!(decoded, input);
}

#[test]
fn test_delta_round_trip_random() {
    let input = vec![1.2, 5.5, -3.3, 0.0, 10.1];
    let mut encoded = vec![0.0f32; 5];
    unsafe {
        sys::barq_delta_encode_f32(input.as_ptr(), encoded.as_mut_ptr(), 5);
    }

    let mut decoded = vec![0.0f32; 5];
    unsafe {
        sys::barq_delta_decode_f32(encoded.as_ptr(), decoded.as_mut_ptr(), 5);
    }
    
    for (a, b) in input.iter().zip(decoded.iter()) {
        assert_relative_eq!(a, b, epsilon = 1e-6);
    }
}
