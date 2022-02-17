use transient_hashmap::{StandardTimer, Timer, TransientHashMap};

pub type PollId = usize;
/// Indexes all poll requests.
/// Lazily garbage collects unused polls info.
pub struct PollManager<F, T = StandardTimer>
where
    T: Timer,
{
    polls:             TransientHashMap<PollId, F, T>,
    next_available_id: PollId,
}

impl<F> PollManager<F, StandardTimer> {
    /// Creates new instance of indexer
    pub fn new(lifetime: u32) -> Self {
        PollManager::new_with_timer(Default::default(), lifetime)
    }
}

impl<F, T> PollManager<F, T>
where
    T: Timer,
{
    pub fn new_with_timer(timer: T, lifetime: u32) -> Self {
        PollManager {
            polls:             TransientHashMap::new_with_timer(lifetime, timer),
            next_available_id: 0,
        }
    }

    pub fn create_poll(&mut self, filter: F) -> PollId {
        self.polls.prune();

        let id = self.next_available_id;
        self.polls.insert(id, filter);

        self.next_available_id += 1;
        id
    }

    /// Get a mutable reference to stored poll filter
    pub fn poll_mut(&mut self, id: &PollId) -> Option<&mut F> {
        self.polls.prune();
        self.polls.get_mut(id)
    }

    /// Removes poll info.
    pub fn remove_poll(&mut self, id: &PollId) -> bool {
        self.polls.remove(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use transient_hashmap::Timer;
    struct TestTimer<'a> {
        time: &'a Cell<i64>,
    }

    impl<'a> Timer for TestTimer<'a> {
        fn get_time(&self) -> i64 {
            self.time.get()
        }
    }
}
