use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::Hasher,
};

use crate::compiler::asm::{AsmValue, Number, Var};

use super::{Mir, MirCodeBlock};

trait Optimizer<T>: Sized {
    fn optimize(self, state: &mut T) -> Vec<Self>;
}

#[derive(Clone, Debug)]
enum VarValue {
    VarRef(usize),
    Unknown,
    // TODO: use small vec
    Values(Vec<u8>),
}

fn intersect(values: &[u8], values1: &[u8]) -> Vec<u8> {
    let mut v = values.to_vec();
    values1.iter().for_each(|x| v.push(*x));
    v
}

impl VarValue {}

#[derive(Default, Clone, Debug)]
struct OptimizerContext {
    used: HashSet<usize>,
    variables: HashMap<usize, VarValue>,
}

impl OptimizerContext {
    pub fn set_var(&mut self, var: usize, value: VarValue) {
        // TODO: Remove this clone
        self.variables.clone().iter().for_each(|(y, x)| {
            if let VarValue::VarRef(a) = x {
                if *a == var {
                    self.set_var(
                        *y,
                        self.variables.get(a).cloned().unwrap_or(VarValue::Unknown),
                    );
                }
            }
        });
        self.variables.insert(var, value);
    }

    pub fn get_raw_var(&self, var: usize) -> VarValue {
        self.variables
            .get(&var)
            .cloned()
            .unwrap_or(VarValue::Unknown)
    }

    pub fn get_var(&self, var: usize) -> VarValue {
        match self.get_raw_var(var) {
            VarValue::VarRef(a) => self.get_var(a),
            e => e,
        }
    }

    /* pub fn add_values_to_var(&mut self, var: usize, values: &[u8]) {
        match self.variables.get(&var) {
            Some(a) => match a {
                VarValue::VarRef(a) => {
                    let a = self.get_var(*a);
                    return self.set_var(
                        var,
                        match a {
                            VarValue::VarRef(_) => unreachable!(),
                            VarValue::Unknown => VarValue::Unknown,
                            VarValue::Values(g) => VarValue::Values(intersect(&g, values)),
                        },
                    );
                }
                VarValue::Unknown => return,
                VarValue::Values(a) => {
                    self.set_var(var, VarValue::Values(intersect(a, values)));
                }
            },
            None => return self.set_var(var, VarValue::Values(values.to_vec())),
        }
    }

    fn get_var_meta_failible(&self, i: usize) -> VarValue {
        match self.get_raw_var(i) {
            VarValue::VarRef(a) => self.get_var_meta(a),
            VarValue::Unknown => VarValue::Unknown,
            VarValue::Values(a) => VarValue::Values(a),
        }
    } */

    fn get_var_meta(&self, i: usize) -> VarValue {
        match self.get_raw_var(i) {
            VarValue::VarRef(a) => self.get_var_meta(a),
            VarValue::Unknown => VarValue::VarRef(i),
            VarValue::Values(a) => VarValue::Values(a),
        }
    }

    fn merge(&self, oc: &Self) -> Self {
        let keys = self
            .variables
            .keys()
            .chain(oc.variables.keys())
            .cloned()
            .collect::<HashSet<_>>();
        let mut map = HashMap::new();
        for i in keys {
            match (self.get_raw_var(i), oc.get_raw_var(i)) {
                // TODO: Check if one is a subref of the other
                (VarValue::VarRef(a), VarValue::VarRef(b)) => {
                    if a == b {
                        map.insert(i, VarValue::VarRef(a));
                    }
                }
                (VarValue::VarRef(_), VarValue::Values(b)) => {
                    if let VarValue::Values(a) = self.get_var(i) {
                        map.insert(i, VarValue::Values(intersect(&a, &b)));
                    }
                }
                (VarValue::Values(a), VarValue::VarRef(_)) => {
                    if let VarValue::Values(b) = oc.get_var(i) {
                        map.insert(i, VarValue::Values(intersect(&a, &b)));
                    }
                }
                (VarValue::Values(a), VarValue::Values(b)) => {
                    map.insert(i, VarValue::Values(intersect(&a, &b)));
                }
                _ => (),
            }
        }
        Self {
            variables: map,
            used: self.used.clone(),
        }
    }
}

impl Optimizer<OptimizerContext> for Mir {
    fn optimize(self, state: &mut OptimizerContext) -> Vec<Self> {
        match self {
            Mir::Copy(a, b) => {
                if !state.used.contains(&a.0) {
                    return Vec::new();
                }
                let bv: AsmValue = match b.clone() {
                    crate::compiler::asm::AsmValue::Var(a) => match state.get_var_meta(a.0) {
                        VarValue::VarRef(a) => AsmValue::Var(Var(a)),
                        VarValue::Unknown => AsmValue::Var(a),
                        VarValue::Values(ab) => {
                            if ab.len() == 1 {
                                AsmValue::Number(Number(ab[0]))
                            } else {
                                AsmValue::Var(a)
                            }
                        }
                    },
                    e => e,
                };
                state.set_var(
                    a.0,
                    match b {
                        crate::compiler::asm::AsmValue::Var(ab) => state.get_var_meta(ab.0),
                        crate::compiler::asm::AsmValue::Number(ab) => VarValue::Values(vec![ab.0]),
                    },
                );
                vec![Mir::Copy(a, bv)]
            }
            Mir::Increment(a) => {
                if !state.used.contains(&a.0) {
                    return Vec::new();
                }
                if let VarValue::Values(d) = state.get_var(a.0) {
                    if d.len() == 1 {
                        state.set_var(a.0, VarValue::Values(vec![(d[0] + 1) % 16]));
                        return vec![Mir::Copy(a, AsmValue::Number(Number((d[0] + 1) % 16)))];
                    }
                }
                state.set_var(
                    a.0,
                    match state.get_var(a.0) {
                        VarValue::VarRef(_) => unreachable!(),
                        VarValue::Unknown => VarValue::Unknown,
                        VarValue::Values(a) => {
                            VarValue::Values(a.iter().map(|x| (x + 1) % 16).collect())
                        }
                    },
                );
                vec![Mir::Increment(a)]
            }
            Mir::Decrement(a) => {
                if !state.used.contains(&a.0) {
                    return Vec::new();
                }
                if let VarValue::Values(d) = state.get_var(a.0) {
                    if d.len() == 1 {
                        state.set_var(a.0, VarValue::Values(vec![(d[0] + 15) % 16]));
                        return vec![Mir::Copy(a, AsmValue::Number(Number((d[0] + 15) % 16)))];
                    }
                }
                state.set_var(
                    a.0,
                    match state.get_var(a.0) {
                        VarValue::VarRef(_) => unreachable!(),
                        VarValue::Unknown => VarValue::Unknown,
                        VarValue::Values(a) => {
                            VarValue::Values(a.iter().map(|x| (x + 15) % 16).collect())
                        }
                    },
                );
                vec![Mir::Decrement(a)]
            }
            Mir::If0(a, b, c) => {
                if b == c {
                    return b
                        .0
                        .into_iter()
                        .map(|x| x.optimize(state))
                        .flatten()
                        .collect();
                }
                if let VarValue::Values(d) = state.get_var(a.0) {
                    if d.contains(&0) {
                        return b
                            .0
                            .into_iter()
                            .map(|x| x.optimize(state))
                            .flatten()
                            .collect();
                    } else {
                        return c
                            .0
                            .into_iter()
                            .map(|x| x.optimize(state))
                            .flatten()
                            .collect();
                    }
                }

                let (k1, st1) = {
                    let mut state = state.clone();
                    state.variables.insert(a.0, VarValue::Values(vec![0]));
                    (
                        b.0.into_iter()
                            .map(|x| x.optimize(&mut state))
                            .flatten()
                            .collect::<Vec<_>>(),
                        state,
                    )
                };
                let (k2, st2) = {
                    let mut state = state.clone();
                    state.variables.insert(
                        a.0,
                        match state.get_var(a.0) {
                            VarValue::VarRef(_) => unreachable!(),
                            VarValue::Unknown => VarValue::Values((1..16).collect()),
                            VarValue::Values(mut a) => {
                                a.remove(0);
                                VarValue::Values(a)
                            }
                        },
                    );
                    (
                        c.0.into_iter()
                            .map(|x| x.optimize(&mut state))
                            .flatten()
                            .collect::<Vec<_>>(),
                        state,
                    )
                };
                *state = st1.merge(&st2);

                return vec![Mir::If0(a, MirCodeBlock(k1), MirCodeBlock(k2))];
            }
            // TODO: Set the vars to be thoses of the last iteration (Where break blocks are)
            Mir::Loop(a) => {
                for i in get_muts_cb(&a) {
                    state.variables.remove(&i);
                }
                let mut k = state.clone();
                return vec![Mir::Loop(MirCodeBlock(
                    a.0.into_iter()
                        .map(|x| x.optimize(&mut k))
                        .flatten()
                        .collect(),
                ))];
            }
            Mir::Break => vec![Mir::Break],
            Mir::Continue => vec![Mir::Continue],
            Mir::Stop => vec![Mir::Stop],
            Mir::ReadRegister(a, b) => {
                if !state.used.contains(&a.0) {
                    return Vec::new();
                }
                vec![Mir::ReadRegister(a, b)]
            }
            Mir::WriteRegister(a, b) => {
                /* println!("--------------------");
                println!("{}", Mir::WriteRegister(a.clone(), b.clone()));
                for (a, b) in &state.variables {
                    println!(
                        "v{} = {}",
                        a,
                        match b {
                            VarValue::VarRef(a) => format!("&v{}", a),
                            VarValue::Values(a) => a
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(" "),
                            VarValue::Unknown => continue,
                        }
                    )
                }
                println!("--------------------");
                println!(); */
                vec![Mir::WriteRegister(
                    a,
                    match b {
                        AsmValue::Var(ab) => match state.get_var_meta(ab.0) {
                            VarValue::VarRef(a) => AsmValue::Var(Var(a)),
                            VarValue::Unknown => AsmValue::Var(ab),
                            VarValue::Values(a) => {
                                if a.len() == 1 {
                                    AsmValue::Number(Number(a[0]))
                                } else {
                                    AsmValue::Var(ab)
                                }
                            }
                        },
                        AsmValue::Number(ab) => AsmValue::Number(ab),
                    },
                )]
            }
        }
    }
}

pub fn get_muts_cb(mir: &MirCodeBlock) -> HashSet<usize> {
    let mut muts = HashSet::new();
    mir.0.iter().for_each(|x| get_muts(x, &mut muts));
    muts
}

pub fn opt(mir: Vec<Mir>) -> Vec<Mir> {
    std::fs::write(
        "target/before_opt.mir",
        mir.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let c1 = mir.iter().map(count).sum::<usize>();

    let mut l: Vec<Mir> = mir;

    fn calculate_hash<T>(t: &T) -> u64
    where
        T: std::hash::Hash,
    {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    let mut k = calculate_hash(&l);

    let mut iter = 0;

    loop {
        iter += 1;
        let mut used = HashSet::new();
        l.iter().for_each(|x| get_used(x, &mut used));

        let mut state = OptimizerContext {
            variables: HashMap::new(),
            used,
        };
        l = l
            .into_iter()
            .map(|x| x.optimize(&mut state))
            .flatten()
            .collect();
        let o = calculate_hash(&l);
        if k == o {
            break;
        }
        k = o;
    }
    std::fs::write(
        "target/after_opt.mir",
        l.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let c2 = l.iter().map(count).sum::<usize>();
    println!(
        "Optimized from {} MIR instructions to {} MIR instructions in {} iterations",
        c1, c2, iter
    );
    l
}

pub fn count(mir: &Mir) -> usize {
    match mir {
        Mir::Copy(_, _) => 1,
        Mir::Increment(_) => 1,
        Mir::Decrement(_) => 1,
        Mir::If0(_, a, b) => {
            1 + a.0.iter().map(count).sum::<usize>() + b.0.iter().map(count).sum::<usize>()
        }
        Mir::Loop(a) => 1 + a.0.iter().map(count).sum::<usize>(),
        Mir::Break => 1,
        Mir::Continue => 1,
        Mir::Stop => 1,
        Mir::ReadRegister(_, _) => 1,
        Mir::WriteRegister(_, _) => 1,
    }
}

pub fn get_muts(mir: &Mir, muts: &mut HashSet<usize>) {
    match mir {
        Mir::Copy(a, _) | Mir::Increment(a) | Mir::Decrement(a) | Mir::ReadRegister(a, _) => {
            muts.insert(a.0);
        }
        Mir::If0(_, a, b) => {
            a.0.iter().for_each(|a| get_muts(a, muts));
            b.0.iter().for_each(|a| get_muts(a, muts));
        }
        Mir::Loop(a) => a.0.iter().for_each(|a| get_muts(a, muts)),
        _ => (),
    }
}

pub fn get_used(mir: &Mir, muts: &mut HashSet<usize>) {
    match mir {
        Mir::Copy(_, AsmValue::Var(a)) | Mir::WriteRegister(_, AsmValue::Var(a)) => {
            muts.insert(a.0);
        }
        Mir::If0(c, a, b) => {
            muts.insert(c.0);
            a.0.iter().for_each(|a| get_used(a, muts));
            b.0.iter().for_each(|a| get_used(a, muts));
        }
        Mir::Loop(a) => a.0.iter().for_each(|a| get_used(a, muts)),
        _ => (),
    }
}
