use crate::Constraints;
#[cfg(feature = "prover")]
use crate::TraceTable;

pub trait Verifiable {
    fn constraints(&self) -> Constraints;

    // fn verify(&self, proof) -> Result<(), VerifierError> {
    //
    // }
}

#[cfg(feature = "prover")]
pub trait Provable<T>: Verifiable {
    fn trace(&self, witness: T) -> TraceTable;

    // fn prove(&self, witness: &T) -> Result<Proof, ProverError> {
    //     let seed = Vec::from(self);
    //     let constraints = self.constraints();
    //     let trace = self.trace(witness);
    //     let domain_size = constraints.trace_nrows().trailing_zeros();
    //     let params = ProofParams::suggested(domain_size);
    //     proof(&seed, &constraints, &trace, &params)
    // }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::RationalExpression;
    use primefield::FieldElement;

    #[derive(Clone, PartialEq, Debug)]
    pub(crate) struct Claim {
        pub(crate) index: usize,
        pub(crate) value: FieldElement,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub(crate) struct Witness {
        pub(crate) secret: FieldElement,
    }

    impl Verifiable for Claim {
        fn constraints(&self) -> Constraints {
            use RationalExpression::*;

            // Seed
            let mut seed = self.index.to_be_bytes().to_vec();
            seed.extend_from_slice(&self.value.as_montgomery().to_bytes_be());

            // Constraint repetitions
            let trace_length = self.index.next_power_of_two();
            let trace_generator = FieldElement::root(trace_length).unwrap();
            let g = Constant(trace_generator);
            let on_row = |index| (X - g.pow(index)).inv();
            let reevery_row = || (X - g.pow(trace_length - 1)) / (X.pow(trace_length) - 1.into());

            // Constraints
            Constraints::from_expressions((trace_length, 2), seed, vec![
                (Trace(0, 1) - Trace(1, 0)) * reevery_row(),
                (Trace(1, 1) - Trace(0, 0) - Trace(1, 0)) * reevery_row(),
                (Trace(0, 0) - 1.into()) * on_row(0),
                (Trace(0, 0) - (&self.value).into()) * on_row(self.index),
            ])
            .unwrap()
        }
    }

    impl Provable<&Witness> for Claim {
        fn trace(&self, witness: &Witness) -> TraceTable {
            let trace_length = self.index.next_power_of_two();
            let mut trace = TraceTable::new(trace_length, 2);
            trace[(0, 0)] = 1.into();
            trace[(0, 1)] = witness.secret.clone();
            for i in 0..(trace_length - 1) {
                trace[(i + 1, 0)] = trace[(i, 1)].clone();
                trace[(i + 1, 1)] = &trace[(i, 0)] + &trace[(i, 1)];
            }
            dbg!(&trace[(self.index, 0)]);
            trace
        }
    }
}
