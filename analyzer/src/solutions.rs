use crate::context::*;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use log;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::sync::Arc;

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
    map: HashMap<Vec<HistoryAlias<T>>, Vec<Arc<Solution<T>>>, CommonHasher>,
    path: &'static str,
    previews: &'static str,
    best_file: &'static str,
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
        best_file: &'static str,
    ) -> io::Result<SolutionCollector<T>> {
        Ok(SolutionCollector {
            map: new_hashmap(),
            file: File::create(sols_file)?,
            path: sols_file,
            previews: previews_file,
            best_file,
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
    pub fn insert<W, L, E, Wp>(
        &mut self,
        elapsed: u32,
        history: Vec<HistoryAlias<T>>,
    ) -> (bool, Option<Arc<Solution<T>>>)
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        let loc_history: Vec<HistoryAlias<T>> = history
            .iter()
            .filter_map(|h| {
                if matches!(h, History::G(_, _) | History::H(_, _)) {
                    Some(*h)
                } else {
                    None
                }
            })
            .collect();
        if self.count == 0 || elapsed < self.best {
            self.best = elapsed;
        } else if elapsed - self.best > self.best / 10 {
            log::info!(
                "Excluding solution as too slow: {} > 1.1 * {}",
                elapsed,
                self.best
            );
            return (false, None);
        }

        self.count += 1;
        let sol = Arc::new(Solution { elapsed, history });
        if let Some(vec) = self.map.get_mut(&loc_history) {
            vec.push(sol.clone());
            self.write_previews().unwrap();
            self.write_best().unwrap();
            (false, Some(sol))
        } else {
            let mut locs = loc_history.clone();
            locs.reverse();
            self.map.insert(loc_history, vec![sol.clone()]);
            self.write_previews().unwrap();
            self.write_best().unwrap();
            (true, Some(sol))
        }
    }

    pub fn get_best(&self) -> Arc<Solution<T>> {
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
                if matches!(h, History::G(_, _) | History::H(_, _)) {
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
        writeln!(
            file,
            "in short:\n{}",
            history_summary::<T, _>(sol.history.iter().copied())
        )?;
        writeln!(
            file,
            "in full:\n{}\n\n",
            history_str::<T, _>(sol.history.iter().copied())
        )
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
        writeln!(
            file,
            "{}\n\n",
            history_summary::<T, _>(sol.history.iter().copied())
        )
    }

    pub fn write_previews(&mut self) -> io::Result<()> {
        let mut file = File::create(self.previews)?;
        for vec in self.map.values_mut() {
            vec.sort_unstable_by_key(|el| el.elapsed);
        }
        let mut vec: Vec<_> = self.map.values().filter_map(|v| v.first()).collect();
        vec.sort_by_key(|c| c.elapsed);
        for (i, c) in vec.iter().enumerate() {
            Self::write_one_preview(&mut file, i, c, self.best)?;
        }
        Ok(())
    }

    pub fn write_best(&mut self) -> io::Result<()> {
        let mut file = File::create(self.best_file)?;
        let sol = self.get_best();
        Self::write_one(&mut file, 0, 0, &sol, self.best)
    }

    pub fn sort_and_clean(&mut self) {
        for vec in self.map.values_mut() {
            vec.sort_unstable_by_key(|el| el.elapsed);
            while let Some(last) = vec.last() {
                if last.elapsed - self.best > self.best / 10 {
                    assert!(
                        vec.len() > 1,
                        "Eliminated all solutions! best={} but first={}",
                        self.best,
                        last.elapsed
                    );
                    vec.pop();
                } else {
                    break;
                }
            }
        }
    }

    pub fn export(&mut self) -> io::Result<()> {
        if self.count == 0 {
            log::info!("No solutions");
            return Ok(());
        }
        self.sort_and_clean();
        let mut vecs: Vec<Vec<_>> = self.map.values().cloned().collect();
        vecs.sort_by_key(|v| v[0].elapsed);
        let mut total = 0;
        let mut types = 0;
        // TODO: add a cutoff of some percentage of the fastest?
        for (i, vec) in vecs.iter().enumerate() {
            let mut minor = 0;
            let first = vec.first().unwrap();
            Self::write_one(&mut self.file, i, minor, first, self.best)?;
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
                Self::write_one(&mut self.file, i, minor, similar, self.best)?;
            }
        }
        log::info!(
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
