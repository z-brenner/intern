//! Reading order and blocks from the order network's outputs (spec §8.5,
//! reference `order_post.py`).
//!
//! The reference keeps line boxes in `float32` arrays and mixes NumPy
//! `float32` arithmetic with Python floats. Under NumPy 2's promotion rules
//! (NEP 50) a Python float meeting a `float32` array or scalar is cast to
//! `float32`, so every box computation here is done in `f32`, the way the
//! reference does it, and the results are bit-identical rather than merely
//! close. The sums follow NumPy's own orders: a reduction along a row is
//! NumPy's pairwise sum, one down a column and a cumulative sum add in
//! sequence ([`pairwise_sum_f32`]).
//!
//! NumPy's `argsort` (its default "quicksort") is not stable, and on a CPU
//! with AVX2 or AVX-512 it is a SIMD sorting network whose order of equal
//! keys depends on the machine, so the reference itself orders ties
//! differently from one computer to the next. Here every sort is stable
//! (equal keys keep their index order). Where it is used this does not change
//! the result: in XY-cut, equal low edges fall on the same side of every cut
//! (the cut is always at the first of a run of equal edges), and the
//! leaf order is a stable sort on keys; the initial ranking of `decode_order`
//! sorts sums of real-valued logits, where ties have measure zero. Replaying
//! the five golden pages with NumPy's `argsort` replaced by a checking
//! wrapper found one call with tied keys, and its order agreed with the
//! stable one.

use super::types::{BlockClass, OrderOutputs};

/// Features a line (spec §8.5): ten of geometry, six of text.
pub const N_FEATS: usize = 16;

/// Characters that end a sentence (`order_post.SENTENCE_END`).
const SENTENCE_END: &str = ".!?:;。！？：；．";

/// Characters that start a list item, as `line_features` sees them.
const ITEM_START: &str = "•-–▪◦*·→";

/// The block classes whose lines go row by row (`GRID_LABELS`).
const GRID: [BlockClass; 3] = [
    BlockClass::Table,
    BlockClass::Form,
    BlockClass::TableOfContents,
];

/// `line_features(boxes, wh, texts)`: sixteen features a line, `[n, 16]`
/// row-major. Boxes are cast to `f32` first, as the reference casts them.
pub fn line_features(boxes: &[[f64; 4]], width: f64, height: f64, texts: &[String]) -> Vec<f32> {
    let boxes: Vec<[f32; 4]> = boxes.iter().map(|b| to_f32(*b)).collect();
    let texts: Vec<&str> = texts.iter().map(String::as_str).collect();
    line_features_f32(&boxes, width, height, Some(&texts))
}

/// [`line_features`] on `f32` boxes; `texts` `None` leaves the six text
/// features zero (`texts=None` in the reference).
pub fn line_features_f32(
    boxes: &[[f32; 4]],
    width: f64,
    height: f64,
    texts: Option<&[&str]>,
) -> Vec<f32> {
    let n = boxes.len();
    if n == 0 {
        return Vec::new();
    }
    // `W, H` are Python ints; dividing a float32 array by one stays float32.
    let (w32, h32) = (width as f32, height as f32);
    let ws: Vec<f32> = boxes.iter().map(|b| (b[2] - b[0]).max(1e-3)).collect();
    let hs: Vec<f32> = boxes.iter().map(|b| (b[3] - b[1]).max(1e-3)).collect();
    let med_h = f64::from(median_f32(&hs));
    let med_h32 = med_h as f32;
    let mut out = Vec::with_capacity(n * N_FEATS);
    for (i, b) in boxes.iter().enumerate() {
        let (w, h) = (ws[i], hs[i]);
        out.extend_from_slice(&[
            b[0] / w32,
            b[1] / h32,
            b[2] / w32,
            b[3] / h32,
            (b[0] + b[2]) / (2.0 * w32),
            (b[1] + b[3]) / (2.0 * h32),
            w / w32,
            (h / med_h32).ln(),
            (w / h).ln() / 4.0,
            // np.full(n, med_h / H * 20.0): computed in float64.
            (med_h / height * 20.0) as f32,
        ]);
        out.extend_from_slice(&text_features(texts.map(|t| t[i])));
    }
    out
}

/// The six text features of one line (`line_features`, the `txt` block).
fn text_features(text: Option<&str>) -> [f32; 6] {
    use super::assemble::pystr;
    let mut f = [0.0f32; 6];
    let Some(text) = text else { return f };
    let t = pystr::strip(text);
    let chars: Vec<char> = t.chars().collect();
    let (Some(&first), Some(&last)) = (chars.first(), chars.last()) else {
        return f;
    };
    let len = chars.len();
    f[0] = ((pystr::display_len(t) as f64).ln_1p() / 5.0) as f32;
    f[1] = (chars.iter().filter(|&&c| pystr::is_digit(c)).count() as f64 / len as f64) as f32;
    let head: String = chars.iter().take(3).collect();
    let numbered = pystr::str_isdigit(head.trim_end_matches(['.', ')'])) && len > 3;
    f[2] = f32::from(u8::from(ITEM_START.contains(first) || numbered));
    f[3] = f32::from(u8::from(last == '-'));
    f[4] = f32::from(u8::from(SENTENCE_END.contains(last)));
    f[5] = f32::from(u8::from(pystr::is_upper(first)));
    f
}

/// `decode_blocks_order(a, same, boxes, cls)`: the lines' reading order.
pub fn decode_blocks_order(outputs: &OrderOutputs, boxes: &[[f64; 4]]) -> Vec<usize> {
    let boxes: Vec<[f32; 4]> = boxes.iter().map(|b| to_f32(*b)).collect();
    decode_blocks_order_f32(outputs, &boxes)
}

/// [`decode_blocks_order`] on `f32` boxes (`order_post.py:318-352`): blocks
/// by average-linkage clustering of the same-block logits, lines inside a
/// block by XY-cut (row by row in a grid block), blocks by the mean order
/// logit between them.
pub fn decode_blocks_order_f32(outputs: &OrderOutputs, boxes: &[[f32; 4]]) -> Vec<usize> {
    let n = outputs.n;
    if n <= 1 {
        return (0..n).collect();
    }
    let blocks = cluster(&outputs.same, n);
    let inner: Vec<Vec<usize>> = blocks
        .iter()
        .map(|b| {
            let bx: Vec<[f32; 4]> = b.iter().map(|&i| boxes[i]).collect();
            let local = if GRID.contains(&block_class(&outputs.cls, b)) {
                order_top_down_f32(&bx)
            } else {
                order_xycut_f32(&bx)
            };
            local.into_iter().map(|k| b[k]).collect()
        })
        .collect();
    // up = triu(G a G^T, 1) with G[k, i] = 1 / |block k|; ab = up - up^T.
    let m = blocks.len();
    let mut ab = vec![0.0f32; m * m];
    let ga: Vec<Vec<f64>> = blocks
        .iter()
        .map(|b| {
            let g = 1.0 / b.len() as f64;
            let mut row = vec![0.0f64; n];
            for &i in b {
                for (j, v) in row.iter_mut().enumerate() {
                    *v += g * f64::from(outputs.a[i * n + j]);
                }
            }
            row
        })
        .collect();
    for p in 0..m {
        for q in p + 1..m {
            let g = 1.0 / blocks[q].len() as f64;
            let up: f64 = blocks[q].iter().map(|&j| ga[p][j] * g).sum();
            ab[p * m + q] = up as f32;
            ab[q * m + p] = (-up) as f32;
        }
    }
    decode_order(&ab, m)
        .into_iter()
        .flat_map(|k| inner[k].iter().copied())
        .collect()
}

/// The class of the block holding `lines`: the argmax of its lines' summed
/// class logits (`cls[b].sum(0).argmax()`).
pub fn block_class(cls: &[f32], lines: &[usize]) -> BlockClass {
    let mut sums = [0.0f32; 10];
    for &i in lines {
        for (s, v) in sums.iter_mut().zip(&cls[i * 10..i * 10 + 10]) {
            *s += *v;
        }
    }
    BlockClass::from_index(argmax_f32(&sums))
}

/// `decode_order(a)` (`order_post.py:73-103`): an order from pairwise logits
/// `a[i * n + j]` (> 0: i before j). The row-sum ranking, then up to four
/// sweeps of insertion moves that raise the summed log-probability of the
/// pairwise relations.
pub fn decode_order(a: &[f32], n: usize) -> Vec<usize> {
    if n <= 1 {
        return (0..n).collect();
    }
    // lp = -logaddexp(0, -a), in float32.
    let lp: Vec<f32> = a.iter().map(|&v| -logaddexp_f32(0.0, -v)).collect();
    let row_sums: Vec<f32> = (0..n)
        .map(|i| pairwise_sum_f32(&lp[i * n..(i + 1) * n]))
        .collect();
    let mut col_sums = vec![0.0f32; n];
    for i in 0..n {
        for (s, v) in col_sums.iter_mut().zip(&lp[i * n..(i + 1) * n]) {
            *s += *v;
        }
    }
    let score: Vec<f32> = (0..n).map(|i| -row_sums[i] + col_sums[i]).collect();
    let mut order = argsort_f32(&score);
    for _ in 0..4 {
        let mut improved = false;
        for x in order.clone() {
            let rest: Vec<usize> = order.iter().copied().filter(|&r| r != x).collect();
            let m = rest.len();
            // gains[k] = cumsum(before)[k - 1] + reversed cumsum(after)[k],
            // each cumulative sum in float32, their sum in float64.
            let mut before = vec![0.0f64; m + 1];
            let mut acc = 0.0f32;
            for (k, &r) in rest.iter().enumerate() {
                acc += lp[r * n + x];
                before[k + 1] = f64::from(acc);
            }
            let mut after = vec![0.0f64; m + 1];
            let mut acc = 0.0f32;
            for (k, &r) in rest.iter().enumerate().rev() {
                acc += lp[x * n + r];
                after[k] = f64::from(acc);
            }
            let gains: Vec<f64> = before.iter().zip(&after).map(|(b, a)| b + a).collect();
            let k = argmax_f64(&gains);
            let cur = order.iter().position(|&o| o == x).unwrap_or(0);
            if gains[k] > gains[cur] + 1e-6 {
                let mut next = rest;
                next.insert(k, x);
                order = next;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    order
}

/// `_cluster(logits)` (`order_post.py:265-315`): average-linkage clustering
/// of pairwise "same group" logits `[n * n]`. The two groups with the
/// highest mean logit between them merge while that mean is positive; the
/// smaller index survives. This keeps the reference's incremental
/// best-partner bookkeeping step for step, so exact ties resolve as there.
/// Groups come back as sorted members, by surviving index.
pub fn cluster(logits: &[f32], n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return Vec::new();
    }
    let ninf = f64::NEG_INFINITY;
    let mut s: Vec<f64> = logits.iter().map(|&v| f64::from(v)).collect();
    for i in 0..n {
        s[i * n + i] = 0.0;
    }
    let mut size = vec![1.0f64; n];
    let mut alive = vec![true; n];
    let mut members: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    let mut mm = s.clone();
    for i in 0..n {
        mm[i * n + i] = ninf;
    }
    let mut best: Vec<usize> = (0..n)
        .map(|i| argmax_f64(&mm[i * n..(i + 1) * n]))
        .collect();
    let mut best_v: Vec<f64> = (0..n).map(|i| mm[i * n + best[i]]).collect();
    for _ in 0..n - 1 {
        let a0 = argmax_f64(&best_v);
        let b0 = best[a0];
        if best_v[a0] <= 0.0 {
            break;
        }
        let (a, b) = (a0.min(b0), a0.max(b0));
        // S[a] += S[b]; S[:, a] += S[:, b]; S[a, a] = 0.
        for j in 0..n {
            s[a * n + j] += s[b * n + j];
        }
        for i in 0..n {
            s[i * n + a] += s[i * n + b];
        }
        s[a * n + a] = 0.0;
        size[a] += size[b];
        alive[b] = false;
        let moved = std::mem::take(&mut members[b]);
        members[a].extend(moved);
        let mut row: Vec<f64> = (0..n).map(|j| s[a * n + j] / (size[a] * size[j])).collect();
        for (j, v) in row.iter_mut().enumerate() {
            if !alive[j] {
                *v = ninf;
            }
        }
        row[a] = ninf;
        for j in 0..n {
            mm[a * n + j] = row[j];
            mm[j * n + a] = row[j];
        }
        for j in 0..n {
            mm[b * n + j] = ninf;
            mm[j * n + b] = ninf;
        }
        best_v[b] = ninf;
        best[a] = argmax_f64(&row);
        best_v[a] = row[best[a]];
        // Other rows: (j, a) changed and (j, b) is gone.
        for j in 0..n {
            if j != a && alive[j] && (best[j] == a || best[j] == b) {
                best[j] = argmax_f64(&mm[j * n..(j + 1) * n]);
                best_v[j] = mm[j * n + best[j]];
            }
        }
        for j in 0..n {
            if j != a && alive[j] && row[j] > best_v[j] {
                best[j] = a;
                best_v[j] = row[j];
            }
        }
    }
    (0..n)
        .filter(|&i| alive[i])
        .map(|i| {
            let mut g = members[i].clone();
            g.sort_unstable();
            g
        })
        .collect()
}

/// `order_top_down(boxes)` (`order_post.py:111-121`): rows top to bottom
/// (centres within half a median line height share a row), left to right.
pub fn order_top_down_f32(boxes: &[[f32; 4]]) -> Vec<usize> {
    if boxes.is_empty() {
        return Vec::new();
    }
    let cy: Vec<f32> = boxes.iter().map(|b| (b[1] + b[3]) / 2.0).collect();
    let hs: Vec<f32> = boxes.iter().map(|b| b[3] - b[1]).collect();
    let half = median_f32(&hs) * 0.5;
    // max(h * 0.5, 1e-3): the Python float meets a float32 and is cast.
    let unit = if 1e-3f32 > half { 1e-3f32 } else { half };
    let mut idx: Vec<usize> = (0..boxes.len()).collect();
    let key = |i: usize| ((cy[i] / unit).round_ties_even(), boxes[i][0]);
    idx.sort_by(|&i, &j| {
        let (ri, xi) = key(i);
        let (rj, xj) = key(j);
        ri.total_cmp(&rj).then(xi.total_cmp(&xj))
    });
    idx
}

/// `order_xycut(boxes)`: the given lines ordered by recursive cuts,
/// returned as elements of `indices`.
pub fn order_xycut(boxes: &[[f64; 4]], indices: &[usize]) -> Vec<usize> {
    let bx: Vec<[f32; 4]> = indices.iter().map(|&i| to_f32(boxes[i])).collect();
    order_xycut_f32(&bx)
        .into_iter()
        .map(|k| indices[k])
        .collect()
}

/// `order_xycut(boxes)` (`order_post.py:124-154`) on `f32` boxes: split at the
/// widest whitespace gap on either axis (the rows when that gap is at least
/// as wide), upper or left part first; without a gap, top-down order.
/// Returns indices into `boxes`.
pub fn order_xycut_f32(boxes: &[[f32; 4]]) -> Vec<usize> {
    fn best_cut(
        boxes: &[[f32; 4]],
        ids: &[usize],
        axis: usize,
    ) -> (f32, Option<usize>, Vec<usize>) {
        let (lo, hi) = if axis == 1 { (1, 3) } else { (0, 2) };
        let mut o: Vec<usize> = (0..ids.len()).collect();
        o.sort_by(|&p, &q| boxes[ids[p]][lo].total_cmp(&boxes[ids[q]][lo]));
        let mut end = boxes[ids[o[0]]][hi];
        let (mut best_gap, mut best_k) = (0.0f32, None);
        for k in 1..o.len() {
            let b = boxes[ids[o[k]]];
            let gap = b[lo] - end;
            if gap > best_gap {
                best_gap = gap;
                best_k = Some(k);
            }
            if b[hi] > end {
                end = b[hi];
            }
        }
        (best_gap, best_k, o)
    }
    fn rec(boxes: &[[f32; 4]], ids: Vec<usize>, out: &mut Vec<usize>) {
        if ids.len() <= 1 {
            out.extend(ids);
            return;
        }
        let (gy, ky, oy) = best_cut(boxes, &ids, 1);
        let (gx, kx, ox) = best_cut(boxes, &ids, 0);
        let (k, o) = match (ky, kx) {
            (None, None) => {
                let b: Vec<[f32; 4]> = ids.iter().map(|&i| boxes[i]).collect();
                out.extend(order_top_down_f32(&b).into_iter().map(|k| ids[k]));
                return;
            }
            _ if gy >= gx => (ky, oy),
            _ => (kx, ox),
        };
        // A gap is positive, so the chosen axis always has its cut.
        let k = k.unwrap_or(0);
        rec(boxes, o[..k].iter().map(|&i| ids[i]).collect(), out);
        rec(boxes, o[k..].iter().map(|&i| ids[i]).collect(), out);
    }
    let mut out = Vec::with_capacity(boxes.len());
    rec(boxes, (0..boxes.len()).collect(), &mut out);
    out
}

/// `split_blocks(order, same)` with each block's class (spec §8.6): runs of
/// consecutive ordered lines with a positive same-block logit between
/// neighbours, as indices into the original lines. This is not the
/// clustering that ordered them.
pub fn split_blocks(order: &[usize], outputs: &OrderOutputs) -> Vec<(BlockClass, Vec<usize>)> {
    let n = outputs.n;
    let mut blocks: Vec<Vec<usize>> = Vec::new();
    for &i in order {
        match blocks.last_mut() {
            Some(b) if outputs.same[b[b.len() - 1] * n + i] > 0.0 => b.push(i),
            _ => blocks.push(vec![i]),
        }
    }
    blocks
        .into_iter()
        .map(|b| (block_class(&outputs.cls, &b), b))
        .collect()
}

/// `np.median` of a `float32` array: the middle value, or the `float32`
/// mean of the two middle values; NaN for none.
pub fn median_f32(values: &[f32]) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    let mut v = values.to_vec();
    v.sort_by(f32::total_cmp);
    let m = v.len() / 2;
    if v.len() % 2 == 1 {
        v[m]
    } else {
        (v[m - 1] + v[m]) / 2.0
    }
}

/// `np.median` of `float64` values.
pub fn median_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let m = v.len() / 2;
    if v.len() % 2 == 1 {
        v[m]
    } else {
        (v[m - 1] + v[m]) / 2.0
    }
}

/// NumPy's pairwise summation of a contiguous `float32` run
/// (`pairwise_sum_FLOAT`): what `arr.sum()` and a sum along a row compute.
pub fn pairwise_sum_f32(a: &[f32]) -> f32 {
    let n = a.len();
    if n < 8 {
        let mut res = 0.0f32;
        for &v in a {
            res += v;
        }
        res
    } else if n <= 128 {
        let mut r = [0.0f32; 8];
        r.copy_from_slice(&a[..8]);
        let mut i = 8;
        while i < n - n % 8 {
            for (j, acc) in r.iter_mut().enumerate() {
                *acc += a[i + j];
            }
            i += 8;
        }
        let mut res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]));
        for &v in &a[i..] {
            res += v;
        }
        res
    } else {
        let mut n2 = n / 2;
        n2 -= n2 % 8;
        pairwise_sum_f32(&a[..n2]) + pairwise_sum_f32(&a[n2..])
    }
}

/// `np.mean` of a `float32` array: the pairwise sum over the count, in
/// `float32`.
pub fn mean_f32(a: &[f32]) -> f32 {
    pairwise_sum_f32(a) / a.len() as f32
}

/// NumPy's `logaddexp` on `float32` (`npy_logaddexpf`).
fn logaddexp_f32(x: f32, y: f32) -> f32 {
    if x == y {
        return x + std::f32::consts::LN_2;
    }
    let tmp = x - y;
    if tmp > 0.0 {
        x + (-tmp).exp().ln_1p()
    } else if tmp <= 0.0 {
        y + tmp.exp().ln_1p()
    } else {
        tmp
    }
}

/// The indices that sort `v` ascending, equal keys in index order.
fn argsort_f32(v: &[f32]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|&i, &j| v[i].total_cmp(&v[j]));
    idx
}

/// `np.argmax`: the first maximum (the first NaN, if any); 0 when empty.
pub fn argmax_f64(v: &[f64]) -> usize {
    let mut best = 0;
    for (i, &x) in v.iter().enumerate() {
        if x.is_nan() {
            return i;
        }
        if x > v[best] {
            best = i;
        }
    }
    best
}

/// `np.argmax` on `float32` values.
pub fn argmax_f32(v: &[f32]) -> usize {
    let mut best = 0;
    for (i, &x) in v.iter().enumerate() {
        if x.is_nan() {
            return i;
        }
        if x > v[best] {
            best = i;
        }
    }
    best
}

fn to_f32(b: [f64; 4]) -> [f32; 4] {
    [b[0] as f32, b[1] as f32, b[2] as f32, b[3] as f32]
}
