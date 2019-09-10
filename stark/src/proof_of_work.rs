use macros_decl::hex;
use std::convert::TryFrom;
use tiny_keccak::Keccak;
use u256::U256;

#[cfg(all(feature = "std", feature = "prover"))]
use rayon::prelude::*;

// Difficulty threshold after which a multi-threaded solver is used.
// Note: tests should use a difficulty below this  threshold .
#[cfg(all(feature = "std", feature = "prover"))]
const THREADED_THRESHOLD: usize = 10;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ChallengeSeed([u8; 32]);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Challenge {
    seed:       [u8; 32],
    difficulty: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Response {
    nonce: u64,
}

impl ChallengeSeed {
    pub fn from_bytes(seed: [u8; 32]) -> Self {
        Self(seed)
    }

    pub fn with_difficulty(self, difficulty: usize) -> Challenge {
        let mut seed = [0_u8; 32];
        let mut keccak = Keccak::new_keccak256();
        keccak.update(&hex!("0123456789abcded"));
        keccak.update(&self.0);
        keccak.update(&[u8::try_from(difficulty).unwrap()]);
        keccak.finalize(&mut seed);
        Challenge { difficulty, seed }
    }
}

impl Challenge {
    pub fn verify(&self, response: Response) -> bool {
        // TODO: return Result<()>
        // OPT: Inline Keccak256 and work directly on buffer using 'keccakf'
        let mut keccak = Keccak::new_keccak256();
        let mut digest = [0_u8; 32];
        keccak.update(&self.seed);
        keccak.update(&(response.nonce.to_be_bytes()));
        keccak.finalize(&mut digest);
        // OPT: Check performance impact of conversion
        let work = U256::from_bytes_be(&digest).leading_zeros();
        work >= self.difficulty
    }
}

#[cfg(feature = "prover")]
impl Challenge {
    pub fn solve(&self) -> Response {
        #[cfg(features = "std")]
        {
            if self.difficulty > THREADED_THRESHOLD {
                return self.solve_threaded();
            }
        }

        // We assume a nonce exists and will be found in reasonable time.
        #[allow(clippy::maybe_infinite_iter)]
        (0_u64..)
            .map(|nonce| Response { nonce })
            .find(|&response| self.verify(response))
            .expect("No valid nonce found")
    }

    // TODO: Make tests compatible with the proof of work values from this function
    #[cfg(feature = "std")]
    fn solve_threaded(&self) -> Response {
        // NOTE: Rayon does not support open ended ranges, so we need to use a closed
        // one.
        (0..u64::max_value())
            .into_par_iter()
            .map(|nonce| Response { nonce })
            .find_any(|&response| self.verify(response))
            .expect("No valid nonce found")
    }
}

impl Response {
    pub fn from_nonce(nonce: u64) -> Self {
        Self { nonce }
    }

    pub fn nonce(self) -> u64 {
        self.nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threaded_proof_of_work_test() {
        let challenge = ChallengeSeed::from_bytes(hex!(
            "0123456789abcded0123456789abcded0123456789abcded0123456789abcded"
        ))
        .with_difficulty(8);
        let response = challenge.solve();
        assert!(challenge.verify(response));
    }

    #[test]
    fn ver_threaded_proof_of_work_test() {
        let challenge = ChallengeSeed::from_bytes(hex!(
            "0123456789abcded0123456789abcded0123456789abcded0123456789abcded"
        ))
        .with_difficulty(8);
        let response = challenge.solve_threaded();
        assert!(challenge.verify(response));
    }
}
