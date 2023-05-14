use crate::context::*;
use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};

fn is_subset<T, Iter>(mut subset: Iter, mut superset: Iter) -> bool
where
    T: Eq,
    Iter: Iterator<Item = T>,
{
    'eq: for n in subset.by_ref() {
        // This only iterates forward once
        for v in superset.by_ref() {
            if n == v {
                continue 'eq;
            }
        }
        return false;
    }
    true
}

#[derive(Clone, Debug)]
pub struct Solution<T: Ctx> {
    pub elapsed: u32,
    pub history: Vec<HistoryAlias<T>>,
}

pub struct SolutionCollector<T>
where
    T: Ctx,
{
    map: HashMap<Vec<HistoryAlias<T>>, Vec<Solution<T>>, CommonHasher>,
    path: &'static str,
    previews: &'static str,
    file: File,
    count: usize,
    best: u32,
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

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn unique(&self) -> usize {
        self.map.len()
    }

    pub fn best(&self) -> u32 {
        self.best
    }

    /// Inserts a solution into the collection and returns whether this solution
    /// is unique compared to the prior solutions.
    pub fn insert(
        &mut self,
        elapsed: u32,
        history: Vec<HistoryAlias<T>>,
    ) -> Option<Vec<HistoryAlias<T>>> {
        let loc_history: Vec<HistoryAlias<T>> = history
            .iter()
            .filter_map(|h| {
                if matches!(h, History::Get(_, _) | History::MoveGet(_, _)) {
                    Some(*h)
                } else {
                    None
                }
            })
            .collect();
        if self.count == 0 || elapsed < self.best {
            self.best = elapsed;
        }
        self.count += 1;
        if let Some(vec) = self.map.get_mut(&loc_history) {
            vec.push(Solution { elapsed, history });
            None
        } else {
            let mut locs = loc_history.clone();
            locs.reverse();
            self.map
                .insert(loc_history, vec![Solution { elapsed, history }]);
            self.write_previews().unwrap();
            Some(locs)
        }
    }

    pub fn get_best(&self) -> Solution<T> {
        self.map
            .values()
            .map(|v| v.iter().min_by_key(|c| c.elapsed).unwrap())
            .min_by_key(|c| c.elapsed)
            .unwrap()
            .clone()
    }

    pub fn get_best_unique(&self) -> Vec<HistoryAlias<T>> {
        self.map
            .values()
            .map(|v| v.iter().min_by_key(|c| c.elapsed).unwrap())
            .min_by_key(|c| c.elapsed)
            .unwrap()
            .history
            .iter()
            .filter_map(|h| {
                if matches!(h, History::Get(_, _) | History::MoveGet(_, _)) {
                    Some(*h)
                } else {
                    None
                }
            })
            .collect()
    }

    fn write_one(
        file: &mut File,
        num: usize,
        minor_num: usize,
        sol: &Solution<T>,
        comp: u32,
    ) -> io::Result<()> {
        let diff = sol.elapsed - comp;
        writeln!(
            file,
            "Solution #{}-{}, est. {}ms{}:",
            num,
            minor_num,
            sol.elapsed,
            if diff > 0 {
                format!(" (+{}ms)", diff)
            } else {
                "".to_string()
            }
        )?;
        writeln!(file, "in short:\n{}", history_summary::<T>(&sol.history))?;
        writeln!(file, "in full:\n{}\n\n", history_str::<T>(&sol.history))
    }

    fn write_one_preview(
        file: &mut File,
        num: usize,
        sol: &Solution<T>,
        comp: u32,
    ) -> io::Result<()> {
        let diff = sol.elapsed - comp;
        writeln!(
            file,
            "Solution #{}, est. {}ms{}:",
            num,
            sol.elapsed,
            if diff > 0 {
                format!(" (+{}ms)", diff)
            } else {
                "".to_string()
            }
        )?;
        writeln!(file, "{}\n\n", history_summary::<T>(&sol.history))
    }

    pub fn write_previews(&self) -> io::Result<()> {
        let mut file = File::create(self.previews)?;
        let mut vec: Vec<&Solution<T>> = self
            .map
            .values()
            .map(|v| v.iter().min_by_key(|c| c.elapsed).unwrap())
            .collect();
        vec.sort_by_key(|c| c.elapsed);
        for (i, c) in vec.iter().enumerate() {
            if c.elapsed - self.best > self.best / 10 {
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
        let mut vecs: Vec<Vec<Solution<T>>> = self.map.into_values().collect();
        let mut file = self.file;
        for vec in vecs.iter_mut() {
            vec.sort_unstable_by_key(|el| el.elapsed);
        }
        vecs.sort_by_key(|v| v[0].elapsed);
        let mut total = 0;
        let mut types = 0;
        // TODO: add a cutoff of some percentage of the fastest?
        for (i, vec) in vecs.iter().enumerate() {
            let mut minor = 0;
            let first = vec.first().unwrap();
            if first.elapsed - self.best > self.best / 10 {
                break;
            }
            Self::write_one(&mut file, i, minor, first, self.best)?;
            total += 1;
            types += 1;
            for (j, similar) in vec.iter().enumerate().skip(1) {
                if vec[..j]
                    .iter()
                    .any(|sol| is_subset(sol.history.iter(), similar.history.iter()))
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
