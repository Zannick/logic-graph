use crate::context::*;
use crate::new_hashset_with;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use lazy_static::lazy_static;
use log;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tera::{Context, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = Tera::new("../templates/*.tera").unwrap();
}

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

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
pub struct Solution<T: Ctx> {
    pub elapsed: u32,
    pub history: Vec<HistoryAlias<T>>,
}

impl<T> Solution<T>
where
    T: Ctx,
{
    pub fn write_graph<W>(&self, world: &W, startctx: &T) -> anyhow::Result<()>
    where
        W: World,
        T: Ctx<World = W>,
        W::Location: Location<Context = T>,
    {
        let now = Instant::now();
        let mut edges = Vec::new();
        let mut ctx = ContextWrapper::new(startctx.clone());
        for h in &self.history {
            let spot = ctx.get().position();
            ctx.replay(world, *h);
            if ctx.get().position() != spot {
                edges.push((format!("{:?}", spot), format!("{:?}", ctx.get().position())));
            }
        }
        let mut context = Context::new();
        context.insert("edges", &edges);
        let res = TEMPLATES.render("solution_graph.m4.tera", &context)?;
        let mut path = PathBuf::from(format!("solutions/{}.m4", self.elapsed));
        let mut i = 0;
        while path.exists() {
            i += 1;
            path.set_file_name(format!("{}_{}.m4", self.elapsed, i));
        }
        let mut file = File::create(&path).unwrap();
        write!(file, "{}", res)?;
        log::info!(
            "Wrote solution of {}ms to {:?} in {:?}",
            self.elapsed,
            path,
            now.elapsed()
        );
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SolutionSuffix<T>(pub Arc<Solution<T>>, pub usize)
where
    T: Ctx;

impl<T> SolutionSuffix<T>
where
    T: Ctx,
{
    pub fn suffix(&self) -> &[HistoryAlias<T>] {
        &self.0.history[self.1..]
    }
}

impl<T> PartialEq for SolutionSuffix<T>
where
    T: Ctx,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.history[self.1..] == other.0.history[other.1..]
    }
}
impl<T> Eq for SolutionSuffix<T> where T: Ctx {}
impl<T> std::hash::Hash for SolutionSuffix<T>
where
    T: Ctx,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.history[self.1..].hash(state);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SolutionResult {
    TooSlow,
    Included,
    IsUnique,
    Duplicate,
}

impl SolutionResult {
    pub fn accepted(self) -> bool {
        self == Self::Included || self == Self::IsUnique
    }
}

pub struct SolutionCollector<T>
where
    T: Ctx,
{
    map: HashMap<Vec<HistoryAlias<T>>, HashSet<Arc<Solution<T>>, CommonHasher>, CommonHasher>,
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

    /// Inserts a solution into the collection and returns a status detailing
    /// whether this solution was accepted and if it's unique.
    pub fn insert_solution<W, L, E, Wp>(&mut self, solution: Arc<Solution<T>>) -> SolutionResult
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        let loc_history: Vec<HistoryAlias<T>> = solution
            .history
            .iter()
            .filter_map(|h| {
                if matches!(h, History::G(_, _) | History::H(_, _)) {
                    Some(*h)
                } else {
                    None
                }
            })
            .collect();
        if self.count == 0 || solution.elapsed < self.best {
            self.best = solution.elapsed;
        } else if solution.elapsed - self.best > self.best / 10 {
            log::info!(
                "Excluding solution as too slow: {} > 1.1 * {}",
                solution.elapsed,
                self.best
            );
            return SolutionResult::TooSlow;
        }

        self.count += 1;
        if let Some(set) = self.map.get_mut(&loc_history) {
            if set.contains(&solution) {
                SolutionResult::Duplicate
            } else {
                set.insert(solution);
                self.write_previews().unwrap();
                self.write_best().unwrap();
                SolutionResult::Included
            }
        } else {
            let mut locs = loc_history.clone();
            locs.reverse();
            self.map.insert(loc_history, new_hashset_with(solution));
            self.write_previews().unwrap();
            self.write_best().unwrap();
            SolutionResult::IsUnique
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

    pub fn iter(&self) -> impl Iterator<Item = Arc<Solution<T>>> + '_ {
        self.map.values().map(|v| v.iter().cloned()).flatten()
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

    pub fn write_previews(&self) -> io::Result<()> {
        let mut file = File::create(self.previews)?;
        let mut vec: Vec<_> = self
            .map
            .values()
            .filter_map(|set| set.iter().min_by_key(|sol| sol.elapsed))
            .collect();
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

    pub fn clean(&mut self) {
        let mut keys_to_drop = Vec::new();
        if let Some(min) = self
            .map
            .values()
            .map(|set| set.iter().map(|sol| sol.elapsed).min())
            .flatten()
            .min()
        {
            for (key, set) in self.map.iter_mut() {
                set.retain(|sol| sol.elapsed - self.best <= self.best / 10);
                if set.is_empty() {
                    keys_to_drop.push(key.clone());
                }
            }
            for key in keys_to_drop {
                self.map.remove_entry(&key);
            }
            assert!(
                !self.map.is_empty(),
                "Eliminated all solutions! best={} but min={:?}",
                self.best,
                min
            );
            self.count = self.map.values().map(|set| set.len()).sum();
        }
    }

    pub fn export(&mut self) -> io::Result<()> {
        if self.count == 0 {
            log::info!("No solutions");
            return Ok(());
        }
        self.clean();
        let mut vecs: Vec<Vec<_>> = self
            .map
            .values()
            .map(|set| {
                let mut vec: Vec<_> = set.iter().cloned().collect();
                vec.sort_by_key(|sol| sol.elapsed);
                vec
            })
            .collect();
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
