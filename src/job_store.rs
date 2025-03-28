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
  data: Arc<Mutex<BTreeSet<JobDetails>>>,
}

impl JobStore {
  pub fn new() -> Self {
    Self {
      signal: Arc::new(Default::default()),
      data: Arc::new(Mutex::new(Default::default())),
    }
  }

  pub fn next_job(&self) -> Option<Arc<Job>> {
    self.data.lock().unwrap().pop_first().map(|details| details.job.clone())
  }

  pub fn add(&self, job: Job, trigger: Trigger) {
    let mut store = self.data.lock().unwrap();
    store.insert((job, trigger).into());
    self.signal.notify_one()
  }

  pub fn next_fire(&self) -> Option<Duration> {
    self.data.lock().unwrap().first().map(|j| {
      j.trigger
        .next_fire()
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

impl From<(Job, Trigger)> for JobDetails {
  fn from((job, trigger): (Job, Trigger)) -> Self {
    JobDetails {
      trigger: trigger.into(),
      job: job.into(),
    }
  }
}

struct JobDetails {
  trigger: Arc<Trigger>,
  job: Arc<Job>,
}

impl Eq for JobDetails {}

impl PartialEq<Self> for JobDetails {
  fn eq(&self, other: &Self) -> bool {
    self.job.eq(&other.job) && self.trigger.eq(&other.trigger)
  }
}

impl PartialOrd<Self> for JobDetails {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for JobDetails {
  fn cmp(&self, other: &Self) -> Ordering {
    self.trigger.next_fire().cmp(&other.trigger.next_fire())
  }
}
