
use crate::types::{StarRatingResult, CalculationError};

pub fn rao_quadratic_entropy_log(values: &[f64], log_iterations: usize) -> StarRatingResult<f64> {
    // Determine the unique categories and their counts
    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).expect("Valeurs finies attendues"));
    
    let mut unique = Vec::new();
    let mut counts = Vec::new();
    
    if !sorted_values.is_empty() {
        let mut current_value = sorted_values[0];
        let mut current_count = 1;
        
        for &value in &sorted_values[1..] {
            if (value - current_value).abs() < 1e-10 {
                current_count += 1;
            } else {
                unique.push(current_value);
                counts.push(current_count);
                current_value = value;
                current_count = 1;
            }
        }
        unique.push(current_value);
        counts.push(current_count);
    }
    
    let total_count: usize = counts.iter().sum();
    let p: Vec<f64> = counts.iter().map(|&count| count as f64 / total_count as f64).collect();
    
    let distance_func = |x: f64, y: f64, log_it: usize| -> f64 {
        let mut acc = (x - y).abs();
        for _ in 0..log_it {
            acc = (1.0 + acc).ln();
        }
        acc
    };
    
    // Compute the distance (dissimilarity) matrix for the unique values
    let n = unique.len();
    let mut dist_matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            dist_matrix[i][j] = distance_func(unique[i], unique[j], log_iterations);
        }
    }
    
    // Compute Rao's Quadratic Entropy: Q = sum_{i,j} p_i * p_j * d(i, j)
    let mut q = 0.0;
    for i in 0..n {
        for j in 0..n {
            q += p[i] * p[j] * dist_matrix[i][j];
        }
    }
    Ok(q)
}

pub fn variety(note_seq: &[(i32, i32, i32)], note_seq_by_column: &[Vec<(i32, i32, i32)>]) -> StarRatingResult<f64> {
    // assume that note_seq already is sorted by head
    let heads: Vec<i32> = note_seq.iter().map(|n| n.1).collect();
    let mut tails: Vec<i32> = note_seq.iter().map(|n| n.2).collect(); // -1 for rice is included
    tails.sort();
    
    let head_gaps: Vec<f64> = (0..heads.len()-1).map(|i| (heads[i+1] - heads[i]) as f64).collect();
    let tail_gaps: Vec<f64> = (0..tails.len()-1).map(|i| (tails[i+1] - tails[i]) as f64).collect();
    
    let head_variety = rao_quadratic_entropy_log(&head_gaps, 1)?;
    let tail_variety = rao_quadratic_entropy_log(&tail_gaps, 1)?;
    
    let mut all_head_gaps = Vec::new();
    for k in 0..note_seq_by_column.len() {
        let column_heads: Vec<i32> = note_seq_by_column[k].iter().map(|n| n.1).collect();
        let column_gaps: Vec<f64> = (0..column_heads.len()-1).map(|i| (column_heads[i+1] - column_heads[i]) as f64).collect();
        all_head_gaps.extend(column_gaps);
    }
    let col_variety = 2.5 * rao_quadratic_entropy_log(&all_head_gaps, 2)?;
    
    Ok(0.5 * head_variety + 0.11 * tail_variety + 0.45 * col_variety)
}

pub fn spikiness(d_sorted: &[f64], w_sorted: &[f64]) -> StarRatingResult<f64> {
    let total_weight = w_sorted.iter().sum::<f64>();
    if total_weight.abs() < 1e-10 {
        return Err(CalculationError::DivisionByZero("spikiness: total weight".to_string()).into());
    }
    
    let weighted_mean = (d_sorted.iter().zip(w_sorted.iter())
        .map(|(d, w)| d.powf(5.0) * w)
        .sum::<f64>() / total_weight).powf(0.2);
    
    let weighted_variance = (d_sorted.iter().zip(w_sorted.iter())
        .map(|(d, w)| (d.powf(8.0) - weighted_mean.powf(8.0)).powi(2) * w)
        .sum::<f64>() / total_weight).powf(0.125);
    
    if weighted_mean.abs() < 1e-10 {
        return Err(CalculationError::DivisionByZero("spikiness: weighted mean".to_string()).into());
    }
    
    Ok(weighted_variance.sqrt() / weighted_mean)
}

pub fn switch(note_seq: &[(i32, i32, i32)], tail_seq: &[(i32, i32, i32)], all_corners: &[f64], ks_arr: &[f64], weights: &[f64]) -> StarRatingResult<f64> {
    let heads: Vec<i32> = note_seq.iter().map(|n| n.1).collect();
    let idx_list: Vec<usize> = heads.iter().map(|&head| {
        match all_corners.iter().position(|&val| val >= head as f64) {
            Some(pos) => pos,
            None => all_corners.len() - 1,
        }
    }).collect();
    
    let ks_arr_at_note: Vec<f64> = idx_list[..idx_list.len()-1].iter().map(|&idx| ks_arr[idx]).collect();
    let weights_at_note: Vec<f64> = idx_list[..idx_list.len()-1].iter().map(|&idx| weights[idx]).collect();
    
    let head_gaps: Vec<f64> = (0..heads.len()-1).map(|i| (heads[i+1] - heads[i]) as f64 / 1000.0).collect();
    
    let avgs: Vec<f64> = (0..head_gaps.len()).map(|i| {
        let start = (i as i32 - 50).max(0) as usize;
        let end = (i + 50).min(head_gaps.len() - 1);
        head_gaps[start..=end].iter().sum::<f64>() / (end - start + 1) as f64
    }).collect();
    
    let signature_head: f64 = head_gaps.iter().zip(avgs.iter()).zip(weights_at_note.iter()).zip(ks_arr_at_note.iter())
        .map(|(((gap, avg), weight), ks)| (gap / avg / head_gaps.len() as f64 * weight).sqrt() * ks.powf(0.25))
        .sum();
    
    let ref_signature_head: f64 = head_gaps.iter().zip(avgs.iter()).zip(weights_at_note.iter())
        .map(|((gap, avg), weight)| gap / avg * weight)
        .sum::<f64>().sqrt();
    
    let tails: Vec<i32> = tail_seq.iter().map(|n| n.2).collect();
    let tail_idx_list: Vec<usize> = tails.iter().map(|&tail| {
        match all_corners.iter().position(|&val| val >= tail as f64) {
            Some(pos) => pos,
            None => all_corners.len() - 1,
        }
    }).collect();
    
    let tail_ks_arr_at_note: Vec<f64> = tail_idx_list[..tail_idx_list.len()-1].iter().map(|&idx| ks_arr[idx]).collect();
    let tail_weights_at_note: Vec<f64> = tail_idx_list[..tail_idx_list.len()-1].iter().map(|&idx| weights[idx]).collect();
    
    let tail_gaps: Vec<f64> = (0..tails.len()-1).map(|i| (tails[i+1] - tails[i]) as f64 / 1000.0).collect();
    
    let mut signature_tail = 0.0;
    let mut ref_signature_tail = 0.0;
    
    if !tails.is_empty() && tails[tails.len()-1] > tails[0] {
        let tail_avgs: Vec<f64> = (0..tail_gaps.len()).map(|i| {
            let start = (i as i32 - 50).max(0) as usize;
            let end = (i + 50).min(tail_gaps.len() - 1);
            tail_gaps[start..=end].iter().sum::<f64>() / (end - start + 1) as f64
        }).collect();
        
        signature_tail = tail_gaps.iter().zip(tail_avgs.iter()).zip(tail_weights_at_note.iter()).zip(tail_ks_arr_at_note.iter())
            .map(|(((gap, avg), weight), ks)| (gap / avg / tail_gaps.len() as f64 * weight).sqrt() * ks.powf(0.25))
            .sum();
        
        ref_signature_tail = tail_gaps.iter().zip(tail_avgs.iter()).zip(tail_weights_at_note.iter())
            .map(|((gap, avg), weight)| gap / avg * weight)
            .sum::<f64>().sqrt();
    }
    
    let switches = (signature_head * head_gaps.len() as f64 + signature_tail * tail_gaps.len() as f64) / 
                   (ref_signature_head * head_gaps.len() as f64 + ref_signature_tail * tail_gaps.len() as f64);
    
    Ok(switches / 2.0 + 0.5)
}
