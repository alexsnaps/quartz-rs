/*
 * Copyright 2025 Alex Snaps
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime};

use super::{Job, Trigger};

pub struct JobStore {
  signal: Arc<Condvar>,
  #[allow(dead_code)]
  data: Arc<Mutex<BTreeSet<TriggerWrapper>>>,
}

impl JobStore {
  pub fn new() -> Self {
    Self {
      signal: Arc::new(Default::default()),
      data: Arc::new(Mutex::new(Default::default())),
    }
  }

  pub fn next_job(&self) -> Option<Arc<Job>> {
    let mut guard = self.data.lock().unwrap();
    guard.pop_first().map(|wrapper| {
      let (job, next) = wrapper.compute_next();
      if let Some(next) = next {
        guard.insert(next);
      }
      job
    })
  }

  pub fn add(&self, job: Job, trigger: Trigger) {
    let mut store = self.data.lock().unwrap();
    store.insert((job, trigger).into());
    self.signal.notify_one()
  }

  pub fn next_fire(&self) -> Option<Duration> {
    self.data.lock().unwrap().first().map(|j| {
      j.next_fire()
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO)
    })
  }
}

impl Default for JobStore {
  fn default() -> Self {
    JobStore::new()
  }
}

impl From<(Job, Trigger)> for TriggerWrapper {
  fn from((job, trigger): (Job, Trigger)) -> Self {
    Self {
      trigger: trigger.into(),
      job: job.into(),
      repeated: 0,
      last_triggered: None,
    }
  }
}

#[derive(Debug)]
struct TriggerWrapper {
  trigger: Arc<Trigger>,
  job: Arc<Job>,
  repeated: u32,
  last_triggered: Option<SystemTime>,
}

impl TriggerWrapper {
  fn compute_next(self) -> (Arc<Job>, Option<Self>) {
    let TriggerWrapper {
      trigger,
      job,
      repeated,
      last_triggered: _,
    } = self;

    if let Some(end) = trigger.end_time {
      if end <= SystemTime::now() + trigger.interval.unwrap_or_default() {
        return (job, None);
      }
    }

    if let Some(repeat_count) = trigger.repeat_count {
      if repeated < repeat_count - 1 {
        // we need to repeat
        let next = TriggerWrapper {
          trigger,
          job: job.clone(),
          repeated: repeated + 1,
          last_triggered: Some(SystemTime::now()),
        };
        return (job, Some(next));
      }
    }
    (job, None)
  }

  fn next_fire(&self) -> SystemTime {
    if let Some(last_triggered) = self.last_triggered {
      if let Some(interval) = self.trigger.interval {
        return last_triggered + interval;
      }
    }
    self.trigger.next_fire()
  }
}

impl Eq for TriggerWrapper {}

impl PartialEq<Self> for TriggerWrapper {
  fn eq(&self, other: &Self) -> bool {
    self.job.eq(&other.job) && self.trigger.eq(&other.trigger)
  }
}

impl PartialOrd<Self> for TriggerWrapper {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for TriggerWrapper {
  fn cmp(&self, other: &Self) -> Ordering {
    self.trigger.next_fire().cmp(&other.trigger.next_fire())
  }
}
