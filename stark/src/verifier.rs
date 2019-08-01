#[allow(unused_imports)]
use crate::{channel::*, fft::*, merkle::*, polynomial::eval_poly, proofs::*, utils::*};
use itertools::Itertools;
use primefield::FieldElement;
use std::collections::HashMap;
use u256::U256;

pub fn check_proof(
    proposed_proof: ProverChannel,
    constraints: &Constraint,
    claim_index: usize,
    claim_value: FieldElement,
    params: &ProofParams,
    trace_cols: usize,
    trace_len: usize,
) -> bool {
    let omega = FieldElement::root(U256::from((trace_len * params.blowup) as u64)).unwrap();
    let eval_domain_size = trace_len * params.blowup;

    let eval_x = geometric_series(&FieldElement::ONE, &omega, eval_domain_size);

    let mut public_input = [claim_index.to_be_bytes()].concat();
    public_input.extend_from_slice(&claim_value.0.to_bytes_be());
    let mut proof_check =
        VerifierChannel::new(public_input.as_slice(), proposed_proof.proof.clone());

    // Get the low degree root commitment, and constraint root commitment
    let low_degree_extension_root: [u8; 32] = proof_check.replay();
    let mut constraint_coefficients: Vec<FieldElement> =
        Vec::with_capacity(constraints.num_constraints);
    for _i in 0..constraints.num_constraints {
        constraint_coefficients.push(proof_check.get_random());
    }
    let constraint_evaluated_root: [u8; 32] = proof_check.replay();

    // Get the oods information from the proof and random
    let oods_point: FieldElement = proof_check.get_random();
    let mut oods_values: Vec<FieldElement> = Vec::with_capacity(2 * trace_cols + 1);
    for _ in 0..=2 * trace_cols {
        oods_values.push(proof_check.replay());
    }
    let mut oods_coefficients: Vec<FieldElement> = Vec::with_capacity(2 * trace_cols + 1);
    for _ in 0..=2 * trace_cols {
        oods_coefficients.push(proof_check.get_random());
    }

    let mut fri_roots: Vec<[u8; 32]> = Vec::with_capacity(params.fri_layout.len() + 1);
    let mut eval_points: Vec<FieldElement> = Vec::with_capacity(params.fri_layout.len() + 1);
    // Get first fri root:
    fri_roots.push(proof_check.replay());
    // Get fri roots and eval points from the channel random
    let mut halvings = 0;
    for &x in params.fri_layout.iter().dropping_back(1) {
        let eval_point = if x == 0 {
            FieldElement::ONE
        } else {
            proof_check.get_random()
        };
        eval_points.push(eval_point);
        fri_roots.push(proof_check.replay());
        halvings += x;
    }
    // Gets the last layer and the polynomial coefficients
    eval_points.push(proof_check.get_random());
    halvings += params.fri_layout[params.fri_layout.len() - 1];
    let last_layer_degree_bound = trace_len / (2_usize.pow(halvings as u32));
    let last_layer_coefficient: Vec<FieldElement> =
        proof_check.replay_many(last_layer_degree_bound);

    // Gets the proof of work from the proof, without moving the random forward.
    let mut holder = [0_u8; 8];
    holder.copy_from_slice(proof_check.read_without_replay(8));
    let proof_of_work = u64::from_be_bytes(holder);
    if !proof_check.pow_verify(proof_of_work, params.pow_bits) {
        return false;
    }
    let recorded_work: u64 = proof_check.replay();
    assert_eq!(recorded_work, proof_of_work);

    // Gets queries from channel
    let eval_domain_size = trace_len * (params.blowup as usize);
    let queries = get_indices(
        params.queries,
        eval_domain_size.trailing_zeros(),
        &mut proof_check,
    );
    let merkle_proof_length = decommitment_size(queries.as_slice(), eval_domain_size);

    // Get values and check decommitment of low degree extension
    let mut led_values: Vec<(usize, Vec<U256>)> = queries
        .iter()
        .map(|&index| {
            let held = proof_check.replay_many(trace_cols);
            (index, held)
        })
        .collect();
    let lde_decommitment: Vec<[u8; 32]> = proof_check.replay_many(merkle_proof_length);
    if !verify(
        low_degree_extension_root,
        eval_domain_size.trailing_zeros(),
        led_values.as_mut_slice(),
        lde_decommitment,
    ) {
        return false;
    }

    // Gets the values and checks the constraint decommitment
    let mut constraint_values: Vec<(usize, U256)> = queries
        .iter()
        .map({ |&index| (index, proof_check.replay()) })
        .collect();
    let constraint_decommitment: Vec<[u8; 32]> = proof_check.replay_many(merkle_proof_length);

    if !verify(
        constraint_evaluated_root,
        eval_domain_size.trailing_zeros(),
        constraint_values.as_mut_slice(),
        constraint_decommitment,
    ) {
        return false;
    }

    let mut fri_indices: Vec<usize> = queries
        .to_vec()
        .iter()
        .map(|x| x / 2_usize.pow((params.fri_layout[0]) as u32))
        .collect();

    // Folded fri values from the previous layer
    let mut fri_folds: HashMap<usize, FieldElement> = HashMap::new();

    let mut previous_indices = queries.to_vec().clone();
    let mut step = 1;
    let mut len = eval_domain_size;
    for (k, _) in fri_roots.iter().enumerate() {
        let mut fri_layer_values = Vec::new();

        fri_indices.dedup();
        for i in fri_indices.iter() {
            let mut coset: Vec<FieldElement> = Vec::new();
            for j in 0..2_usize.pow(params.fri_layout[k] as u32) {
                let n = i * 2_usize.pow(params.fri_layout[k] as u32) + j;

                let has_index = previous_indices.binary_search(&n);
                match has_index {
                    Ok(z) => {
                        if k > 0 {
                            coset.push(fri_folds.get(&n).unwrap().clone());
                        } else {
                            let z_reverse = queries[z].bit_reverse_at(eval_domain_size);
                            coset.push(out_of_domain_element(
                                led_values[z].1.as_slice(),
                                &constraint_values[z].1,
                                &eval_x[z_reverse],
                                &oods_point,
                                oods_values.as_slice(),
                                oods_coefficients.as_slice(),
                                eval_domain_size,
                                params.blowup,
                            ));
                        }
                    }
                    Err(_) => {
                        coset.push(proof_check.replay());
                    }
                }
            }
            fri_layer_values.push((*i, coset));
        }
        // Fold and record foldings
        let mut layer_folds = HashMap::new();
        for (i, coset) in fri_layer_values.iter() {
            layer_folds.insert(
                *i,
                fri_fold(
                    coset.as_slice(),
                    &eval_points[k],
                    step,
                    2_usize.pow((params.fri_layout[k] - 1) as u32) * i,
                    len,
                    eval_x.as_slice(),
                ),
            );
        }

        let merkle_proof_length = decommitment_size(
            fri_indices.as_slice(),
            len / 2_usize.pow(params.fri_layout[k] as u32),
        );
        let decommitment = proof_check.replay_many(merkle_proof_length);
        fri_folds = layer_folds;

        for _ in 0..params.fri_layout[k] {
            step *= 2;
        }
        len /= 2_usize.pow(params.fri_layout[k] as u32);

        if !verify(
            fri_roots[k],
            len.trailing_zeros(),
            &fri_layer_values,
            decommitment,
        ) {
            return false;
        }

        previous_indices = fri_indices.clone();
        if k + 1 < params.fri_layout.len() {
            fri_indices = fri_indices
                .iter()
                .map(|ind| ind / 2_usize.pow((params.fri_layout[k + 1]) as u32))
                .collect();
        }
    }
    if !proof_check.at_end() {
        return false;
    }

    // Checks that the calculated fri folded queries are the points interpolated by
    // the decommited polynomial.
    let interp_root = FieldElement::root(U256::from(len as u64)).unwrap();
    for key in previous_indices.iter() {
        let calculated = fri_folds[key].clone();
        let x_pow = interp_root.pow(U256::from(key.bit_reverse_at(len) as u64));
        let committed = eval_poly(x_pow, last_layer_coefficient.as_slice());

        if committed != calculated.clone() {
            return false;
        }
    }

    // Checks that the oods point calculation matches the constrain calculation
    // TODO

    true
}

fn get_indices(num: usize, bits: u32, proof: &mut VerifierChannel) -> Vec<usize> {
    let mut query_indices = Vec::with_capacity(num + 3);
    while query_indices.len() < num {
        let val: U256 = proof.get_random();
        query_indices.push(((val.clone() >> (0x100 - 0x040)).c0 & (2_u64.pow(bits) - 1)) as usize);
        query_indices.push(((val.clone() >> (0x100 - 0x080)).c0 & (2_u64.pow(bits) - 1)) as usize);
        query_indices.push(((val.clone() >> (0x100 - 0x0C0)).c0 & (2_u64.pow(bits) - 1)) as usize);
        query_indices.push((val.c0 & (2_u64.pow(bits) - 1)) as usize);
    }
    query_indices.truncate(num);
    (&mut query_indices).sort_unstable();
    query_indices
}

fn fri_fold(
    coset: &[FieldElement],
    eval_point: &FieldElement,
    mut step: usize,
    mut index: usize,
    mut len: usize,
    eval_x: &[FieldElement],
) -> FieldElement {
    let mut mutable_eval_copy = eval_point.clone();
    let mut coset_full: Vec<FieldElement> = coset.to_vec();
    while coset_full.len() > 1 {
        let mut next_coset = Vec::with_capacity(coset.len() / 2);

        for (k, pair) in coset_full.chunks(2).enumerate() {
            let x = &eval_x[(index + k).bit_reverse_at(len / 2) * step];
            next_coset.push(fri_single_fold(&pair[0], &pair[1], x, &mutable_eval_copy));
        }
        len /= 2;
        index /= 2;
        step *= 2;
        mutable_eval_copy = mutable_eval_copy.square();
        coset_full = next_coset;
    }
    coset_full[0].clone()
}

fn fri_single_fold(
    poly_at_x: &FieldElement,
    poly_at_neg_x: &FieldElement,
    x: &FieldElement,
    eval_point: &FieldElement,
) -> FieldElement {
    (poly_at_x + poly_at_neg_x) + eval_point / x * (poly_at_x - poly_at_neg_x)
}

// TODO - Make sure this is general
#[allow(clippy::too_many_arguments)]
fn out_of_domain_element(
    poly_points_u: &[U256],
    constraint_point_u: &U256,
    x_cord: &FieldElement,
    oods_point: &FieldElement,
    oods_values: &[FieldElement],
    oods_coefficients: &[FieldElement],
    eval_domain_size: usize,
    blowup: usize,
) -> FieldElement {
    let poly_points: Vec<FieldElement> = poly_points_u
        .iter()
        .map(|i| FieldElement(i.clone()))
        .collect();
    let constraint_point = FieldElement(constraint_point_u.clone());
    let x_transform = x_cord * FieldElement::GENERATOR;
    let omega = FieldElement::root(U256::from(eval_domain_size as u64)).unwrap();
    let g = omega.pow(U256::from(blowup as u64));
    let mut r = FieldElement::ZERO;

    for x in 0..poly_points.len() {
        r += &oods_coefficients[2 * x] * (&poly_points[x] - &oods_values[2 * x])
            / (&x_transform - oods_point);
        r += &oods_coefficients[2 * x + 1] * (&poly_points[x] - &oods_values[2 * x + 1])
            / (&x_transform - &g * oods_point);
    }
    r += &oods_coefficients[oods_coefficients.len() - 1]
        * (constraint_point - &oods_values[oods_values.len() - 1])
        / (&x_transform - oods_point);

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fibonacci::*;
    use hex_literal::*;
    use u256::u256h;

    #[test]
    fn verifier_fib_test() {
        let claim_index = 1000;
        let claim_value = FieldElement::from(u256h!(
            "0142c45e5d743d10eae7ebb70f1526c65de7dbcdb65b322b6ddc36a812591e8f"
        ));
        let witness = FieldElement::from(u256h!(
            "00000000000000000000000000000000000000000000000000000000cafebabe"
        ));
        let actual = stark_proof(
            &get_trace_table(1024, witness),
            &get_constraint(),
            claim_index,
            claim_value.clone(),
            &ProofParams {
                blowup:     16,
                pow_bits:   12,
                queries:    20,
                fri_layout: vec![3, 2],
            },
        );

        assert!(check_proof(
            actual,
            &get_constraint(),
            claim_index,
            claim_value,
            &ProofParams {
                blowup:     16,
                pow_bits:   12,
                queries:    20,
                fri_layout: vec![3, 2],
            },
            2,
            1024
        ));
    }
}
