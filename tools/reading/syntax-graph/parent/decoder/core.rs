type Matrix = Vec<Vec<Option<f64>>>;

// Each recursive level strictly reduces node count. Missing edges never use infinity.
fn edmonds(s: &Matrix) -> Result<Vec<usize>> {
    let n = s.len();
    let mut parents = vec![0; n];
    for d in 1..n {
        let mut selected = None;
        for (h, row) in s.iter().enumerate() {
            if let Some(v) = row[d]
                && selected.is_none_or(|(_, old)| v > old)
            {
                selected = Some((h, v));
            }
        }
        parents[d] = selected.ok_or("disconnected score graph")?.0;
    }
    let mut cycle = Vec::new();
    for start in 1..n {
        let mut path = Vec::new();
        let mut v = start;
        while v != 0 {
            if let Some(pos) = path.iter().position(|&x| x == v) {
                cycle = path[pos..].to_vec();
                break;
            }
            path.push(v);
            v = parents[v];
        }
        if !cycle.is_empty() {
            break;
        }
    }
    if cycle.is_empty() {
        return Ok(parents);
    }
    let mut inside = vec![false; n];
    for &v in &cycle {
        inside[v] = true;
    }
    let outside: Vec<_> = (0..n).filter(|&v| !inside[v]).collect();
    let c = outside.len();
    let mut map = vec![c; n];
    for (i, &v) in outside.iter().enumerate() {
        map[v] = i;
    }
    let mut contracted = vec![vec![None; c + 1]; c + 1];
    let mut endpoints = vec![vec![(0, 0); c + 1]; c + 1];
    for h in 0..n {
        for d in 1..n {
            if map[h] == map[d] {
                continue;
            }
            if let Some(mut value) = s[h][d] {
                if inside[d] {
                    value -= s[parents[d]][d].ok_or("missing cycle edge")?;
                }
                if !value.is_finite() {
                    return Err("nonfinite contraction".into());
                }
                let cell = &mut contracted[map[h]][map[d]];
                if cell.is_none_or(|old| value > old) {
                    *cell = Some(value);
                    endpoints[map[h]][map[d]] = (h, d);
                }
            }
        }
    }
    let small = edmonds(&contracted)?;
    for d in 1..small.len() {
        let (h, v) = endpoints[small[d]][d];
        parents[v] = h;
    }
    Ok(parents)
}
fn decode(s: &Matrix) -> Result<Vec<usize>> {
    let n = s.len();
    if !(2..=MAX_TOKENS + 1).contains(&n) || s.iter().any(|r| r.len() != n) {
        return Err("score dimensions".into());
    }
    for (h, row) in s.iter().enumerate() {
        for (d, v) in row.iter().enumerate() {
            if v.is_some_and(|v| !v.is_finite()) || ((h == d || d == 0) && v.is_some()) {
                return Err("invalid score matrix".into());
            }
        }
    }
    let initial = edmonds(s)?;
    if initial[1..].iter().filter(|&&h| h == 0).count() == 1 {
        return Ok(initial);
    }
    let mut best_tree = None;
    let mut best_score = None;
    for root in 1..n {
        if s[0][root].is_none() {
            continue;
        }
        let mut constrained = s.clone();
        for d in 1..n {
            if d != root {
                constrained[0][d] = None;
            } else {
                for row in constrained.iter_mut().skip(1) {
                    row[d] = None;
                }
            }
        }
        let Ok(tree) = edmonds(&constrained) else {
            continue;
        };
        let mut total = 0.;
        for d in 1..n {
            total += s[tree[d]][d].ok_or("missing decoded edge")?;
        }
        if !total.is_finite() {
            return Err("nonfinite tree score".into());
        }
        if best_score.is_none_or(|old| total > old) {
            best_score = Some(total);
            best_tree = Some(tree);
        }
    }
    best_tree.ok_or_else(|| "no single-root tree".into())
}
