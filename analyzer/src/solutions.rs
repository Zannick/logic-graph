use crate::context::*;
use crate::new_hashset_with;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use lazy_static::lazy_static;
use log;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tera::{Context, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = Tera::new("../templates/*.tera").unwrap();
    pub static ref SOLUTIONS_DIR: PathBuf = PathBuf::from("solutions/");
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

pub fn write_graph<W, T>(
    world: &W,
    startctx: &T,
    history: &[HistoryAlias<T>],
) -> anyhow::Result<PathBuf>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    let now = Instant::now();
    let mut edges = Vec::new();
    let mut spots = Vec::new();
    let mut ctx = ContextWrapper::new(startctx.clone());
    let mut spot = ctx.get().position();
    let mut recent = Vec::new();
    let mut last_real_spot = Default::default();
    let mut i = 0;
    for h in history.iter() {
        match h {
            History::G(item, _) | History::H(item, _) => {
                if world.should_draw_spot(ctx.get().position()) {
                    recent.push(format!("{}", item));
                    spots.push((format!("{:?}", ctx.get().position()), i, recent.join("\\n")));
                    i += 1;
                    recent.clear();
                } else {
                    recent.push(format!("{}", item));
                }
            }
            History::C(..) | History::L(..) => (),
            History::W(w, ..) => {
                recent.push(format!("{}Warp", w));
            }
            _ => {
                recent.push(format!("{}", h));
            }
        }
        ctx.replay(world, *h);
        let is_warp = matches!(h, History::W(..));
        if ctx.get().position() != spot || matches!(h, History::A(..)) {
            if world.should_draw_spot(spot) {
                if world.should_draw_spot(ctx.get().position()) {
                    edges.push((
                        format!("{:?}", spot),
                        format!("{:?}", ctx.get().position()),
                        i,
                        is_warp,
                        recent.join("\\n"),
                    ));
                    i += 1;
                    recent.clear();
                    if is_warp {
                        spots.push((format!("{:?}", ctx.get().position()), i, String::from("")));
                    }
                    last_real_spot = ctx.get().position();
                }
            } else if last_real_spot != Default::default()
                && !recent.is_empty()
                && world.should_draw_spot(ctx.get().position())
            {
                // Draw the edge
                edges.push((
                    format!("{:?}", last_real_spot),
                    format!("{:?}", ctx.get().position()),
                    i,
                    true,
                    recent.join("\\n"),
                ));
                i += 1;
                recent.clear();
            }
        }
        spot = ctx.get().position();
    }
    let mut context = Context::new();
    context.insert("edges", &edges);
    context.insert("spots", &spots);
    let res = TEMPLATES.render("solution_graph.m4.tera", &context)?;
    let mut path = SOLUTIONS_DIR.clone();
    path.push(format!("{}.m4", ctx.elapsed()));
    let mut i = 0;
    while path.exists() {
        i += 1;
        path.set_file_name(format!("{}_{}.m4", ctx.elapsed(), i));
    }
    let mut file = File::create(&path).unwrap();
    write!(file, "{}", res)?;
    log::info!(
        "Wrote route of {}ms to {:?} in {:?}",
        ctx.elapsed(),
        path,
        now.elapsed()
    );
    Ok(path)
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Solution<T: Ctx> {
    pub elapsed: u32,
    pub history: Vec<HistoryAlias<T>>,
    // TODO: generate a solution-specific filename on new()
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    processing_queue: PriorityQueue<Arc<Solution<T>>, Reverse<u32>, CommonHasher>,
    path: &'static str,
    previews: &'static str,
    best_file: &'static str,
    startctx: T,
    file: File,
    count: usize,
    best: u32,
    pending: bool,
}

impl<T> SolutionCollector<T>
where
    T: Ctx,
{
    pub fn new(
        sols_file: &'static str,
        previews_file: &'static str,
        best_file: &'static str,
        startctx: &T,
    ) -> io::Result<SolutionCollector<T>> {
        // Clear entries out of the solutions dir before starting
        for entry in std::fs::read_dir(&*SOLUTIONS_DIR)? {
            let path = entry?.path();
            if path.is_file() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(SolutionCollector {
            map: new_hashmap(),
            processing_queue: PriorityQueue::default(),
            file: File::create(sols_file)?,
            path: sols_file,
            previews: previews_file,
            startctx: startctx.clone(),
            best_file,
            count: 0,
            best: 0,
            pending: false,
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

    pub fn cutoff(&self) -> u32 {
        self.best + self.best / 10
    }

    /// Inserts a solution into the collection and returns a status detailing
    /// whether this solution was accepted and if it's unique.
    pub fn insert_solution<W, L, E, Wp>(
        &mut self,
        solution: Arc<Solution<T>>,
        world: &W,
    ) -> SolutionResult
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        let loc_history: Vec<HistoryAlias<T>> =
            collection_history::<T, W, L, _>(solution.history.iter().copied()).collect();
        let best = if self.count == 0 || solution.elapsed < self.best {
            self.best = solution.elapsed;
            write_graph(world, &self.startctx, &solution.history).unwrap();
            true
        } else if solution.elapsed > self.cutoff() {
            log::info!(
                "Excluding solution as too slow: {} > 1.1 * {}",
                solution.elapsed,
                self.best
            );
            return SolutionResult::TooSlow;
        } else {
            false
        };

        self.count += 1;
        if let Some(set) = self.map.get_mut(&loc_history) {
            if set.contains(&solution) {
                SolutionResult::Duplicate
            } else {
                self.processing_queue.push(solution.clone(), Reverse(solution.elapsed));
                set.insert(solution);
                if best {
                    self.write_previews().unwrap();
                    self.write_best().unwrap();
                    self.pending = false;
                }
                SolutionResult::Included
            }
        } else {
            let mut locs = loc_history.clone();
            locs.reverse();
            self.processing_queue.push(solution.clone(), Reverse(solution.elapsed));
            self.map.insert(loc_history, new_hashset_with(solution));
            if best {
                self.write_previews().unwrap();
                self.write_best().unwrap();
                self.pending = false;
            } else {
                self.pending = true;
            }
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

    pub fn next_unprocessed(&mut self) -> Option<Arc<Solution<T>>> {
        self.processing_queue.pop().map(|x| x.0)
    }

    pub fn has_unprocessed(&self) -> bool {
        self.processing_queue.is_empty()
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
        let len = vec.len();
        for (i, c) in vec.into_iter().enumerate() {
            Self::write_one_preview(&mut file, i, c, self.best)?;
        }
        log::debug!("Wrote {} solution previews into {}", len, self.previews);
        Ok(())
    }

    pub fn write_previews_if_pending(&mut self) -> io::Result<()> {
        if self.pending {
            self.pending = false;
            self.write_best().unwrap();
            self.write_previews()
        } else {
            Ok(())
        }
    }

    pub fn write_best(&mut self) -> io::Result<()> {
        let mut file = File::create(self.best_file)?;
        let sol = self.get_best();
        Self::write_one(&mut file, 0, 0, &sol, self.best)
    }

    pub fn clean(&mut self) {
        let mut keys_to_drop = Vec::new();
        let cutoff = self.cutoff();
        if let Some(min) = self
            .map
            .values()
            .map(|set| set.iter().map(|sol| sol.elapsed).min())
            .flatten()
            .min()
        {
            for (key, set) in self.map.iter_mut() {
                set.retain(|sol| sol.elapsed <= cutoff);
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
