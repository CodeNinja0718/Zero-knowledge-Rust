/// Permute index for an FFT of `size`
///
/// The permutation is it's own inverse. The permutation is currently
/// a 'bit-reversal' one, where each index has its binary representation
/// reversed.
pub fn permute_index(size: usize, index: usize) -> usize {
    const USIZE_BITS: usize = 0_usize.count_zeros() as usize;
    debug_assert!(index < size);
    if size == 1 {
        0
    } else {
        debug_assert!(size.is_power_of_two());
        let bits = size.trailing_zeros() as usize;
        index.reverse_bits() >> (USIZE_BITS - bits)
    }
}

/// Permute an array of FFT results.
// TODO expose public ifft function which accepts bit-reversed input instead.
pub fn permute<T>(v: &mut [T]) {
    let n = v.len();
    for i in 0..n {
        let j = permute_index(n, i);
        if j > i {
            v.swap(i, j);
        }
    }
}
