use crate::context::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

fn is_subset<T, Iter>(subset: &mut Iter, superset: &mut Iter) -> bool
where
    T: Eq,
    Iter: Iterator<Item = T>,
{
    'eq: while let Some(n) = subset.next() {
        while let Some(v) = superset.next() {
            if n == v {
                continue 'eq;
            }
        }
        return false;
    }
    true
}

pub struct SolutionCollector<T>
where
    T: Ctx,
{
    map: HashMap<Vec<History<T>>, Vec<ContextWrapper<T>>>,
    path: &'static str,
    file: File,
}

impl<T> SolutionCollector<T>
where
    T: Ctx,
{
    pub fn new(path: &'static str) -> io::Result<SolutionCollector<T>> {
        Ok(SolutionCollector {
            map: HashMap::new(),
            path,
            file: File::create(path)?,
        })
    }

    pub fn insert(&mut self, ctx: ContextWrapper<T>) {
        let loc_history: Vec<History<T>> = ctx
            .history_rev()
            .filter(|h| match h {
                History::Get(_, _) | History::MoveGet(_, _) => true,
                _ => false,
            })
            .collect();
        if let Some(vec) = self.map.get_mut(&loc_history) {
            vec.push(ctx);
        } else {
            self.map.insert(loc_history, vec![ctx]);
        }
    }

    fn write_one(
        file: &mut File,
        num: usize,
        minor_num: usize,
        ctx: &ContextWrapper<T>,
    ) -> io::Result<()> {
        writeln!(
            file,
            "Solution #{}-{}, est. {}ms:",
            num,
            minor_num,
            ctx.elapsed()
        )?;
        writeln!(file, "in short:\n{}", ctx.history_preview())?;
        writeln!(file, "in full:\n{}\n\n", ctx.history_str())
    }

    pub fn export(self) -> io::Result<()> {
        let mut vecs: Vec<Vec<ContextWrapper<T>>> = self.map.into_values().collect();
        let mut file = self.file;
        for vec in vecs.iter_mut() {
            vec.sort();
        }
        vecs.sort_by_key(|v| v[0].elapsed());
        let mut total = 0;
        for (i, vec) in vecs.iter().enumerate() {
            let mut minor = 0;
            Self::write_one(&mut file, i, minor, vec.first().unwrap())?;
            total += 1;
            for (j, similar) in vec.iter().enumerate().skip(1) {
                if vec[..j]
                    .iter()
                    .any(|ctx| is_subset(&mut ctx.history_rev(), &mut similar.history_rev()))
                {
                    continue;
                }
                minor += 1;
                total += 1;
                Self::write_one(&mut file, i, minor, similar)?;
            }
        }
        println!("Wrote {} solutions ({} types) to {}", total, vecs.len(), self.path);
        Ok(())
    }
}
