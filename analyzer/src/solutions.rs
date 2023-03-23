use crate::context::*;
use crate::{CommonHasher, new_hashmap};
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
    map: HashMap<Vec<History<T>>, Vec<ContextWrapper<T>>, CommonHasher>,
    path: &'static str,
    previews: &'static str,
    file: File,
    count: usize,
    best: i32,
}

impl<T> SolutionCollector<T>
where
    T: Ctx,
{
    pub fn new(
        sols_file: &'static str,
        previews_file: &'static str,
    ) -> io::Result<SolutionCollector<T>> {
        Ok(SolutionCollector {
            map: new_hashmap(),
            file: File::create(sols_file)?,
            path: sols_file,
            previews: previews_file,
            count: 0,
            best: 0,
        })
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn unique(&self) -> usize {
        self.map.len()
    }

    pub fn best(&self) -> i32 {
        self.best
    }

    pub fn insert(&mut self, ctx: ContextWrapper<T>) {
        let loc_history: Vec<History<T>> = ctx
            .history_rev()
            .filter(|h| match h {
                History::Get(_, _) | History::MoveGet(_, _) => true,
                _ => false,
            })
            .collect();
        if self.count == 0 || ctx.elapsed() < self.best {
            self.best = ctx.elapsed();
        }
        if let Some(vec) = self.map.get_mut(&loc_history) {
            vec.push(ctx);
        } else {
            self.map.insert(loc_history, vec![ctx]);
            self.write_previews().unwrap();
        }
        self.count += 1;
    }

    fn write_one(
        file: &mut File,
        num: usize,
        minor_num: usize,
        ctx: &ContextWrapper<T>,
        comp: i32,
    ) -> io::Result<()> {
        let diff = ctx.elapsed() - comp;
        writeln!(
            file,
            "Solution #{}-{}, est. {}ms{}:",
            num,
            minor_num,
            ctx.elapsed(),
            if diff > 0 {
                format!(" (+{}ms)", diff)
            } else {
                "".to_string()
            }
        )?;
        writeln!(file, "in short:\n{}", ctx.history_summary())?;
        writeln!(file, "in full:\n{}\n\n", ctx.history_str())
    }

    fn write_one_preview(
        file: &mut File,
        num: usize,
        ctx: &ContextWrapper<T>,
        comp: i32,
    ) -> io::Result<()> {
        let diff = ctx.elapsed() - comp;
        writeln!(
            file,
            "Solution #{}, est. {}ms{}:",
            num,
            ctx.elapsed(),
            if diff > 0 {
                format!(" (+{}ms)", diff)
            } else {
                "".to_string()
            }
        )?;
        writeln!(file, "{}\n\n", ctx.history_summary())
    }

    pub fn write_previews(&self) -> io::Result<()> {
        let mut file = File::create(self.previews)?;
        let mut vec: Vec<&ContextWrapper<T>> = self
            .map
            .values()
            .map(|v| v.iter().min_by_key(|c| c.elapsed()).unwrap())
            .collect();
        vec.sort_by_key(|c| c.elapsed());
        for (i, c) in vec.iter().enumerate() {
            if c.elapsed() - self.best > self.best / 10 {
                break;
            }
            Self::write_one_preview(&mut file, i, c, self.best)?
        }
        Ok(())
    }

    pub fn export(self) -> io::Result<()> {
        if self.count == 0 {
            println!("No solutions");
            return Ok(());
        }
        let mut vecs: Vec<Vec<ContextWrapper<T>>> = self.map.into_values().collect();
        let mut file = self.file;
        for vec in vecs.iter_mut() {
            vec.sort();
        }
        vecs.sort_by_key(|v| v[0].elapsed());
        let mut total = 0;
        let mut types = 0;
        // TODO: add a cutoff of some percentage of the fastest?
        for (i, vec) in vecs.iter().enumerate() {
            let mut minor = 0;
            let first = vec.first().unwrap();
            if first.elapsed() - self.best > self.best / 10 {
                break;
            }
            Self::write_one(&mut file, i, minor, first, self.best)?;
            total += 1;
            types += 1;
            for (j, similar) in vec.iter().enumerate().skip(1) {
                if vec[..j]
                    .iter()
                    .any(|ctx| is_subset(&mut ctx.history_rev(), &mut similar.history_rev()))
                {
                    continue;
                }
                minor += 1;
                total += 1;
                Self::write_one(&mut file, i, minor, similar, self.best)?;
            }
        }
        println!(
            "Wrote {} solutions ({} types, reduced from {} total/{} types) to {}",
            total,
            types,
            self.count,
            vecs.len(),
            self.path
        );
        Ok(())
    }
}
