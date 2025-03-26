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

mod thread_pool;

use crate::threading::thread_pool::Executable;
use crate::{Job, JobStore};

use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Acquire;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use thread_pool::WorkerPool;

const DEFAULT_WAIT_NO_WORK: Duration = Duration::from_secs(1);

pub(super) struct SchedulerThread {
  halted: Arc<AtomicBool>,
  workers: Arc<WorkerPool<Arc<Job>>>,
  #[allow(dead_code)]
  store: Arc<JobStore>,
  handle: JoinHandle<()>,
}

impl SchedulerThread {
  pub fn new(pool_size: NonZeroUsize, store: Arc<JobStore>) -> Self {
    let halted = Arc::new(AtomicBool::default());
    let workers = Arc::new(WorkerPool::new(pool_size));
    let handle = {
      let halted = halted.clone();
      let store = store.clone();
      let workers = workers.clone();

      thread::Builder::new()
        .name("Quartz Scheduler Thread".to_string())
        .spawn(move || {
          while !halted.load(Acquire) {
            let next_fire = store.next_fire().unwrap_or(DEFAULT_WAIT_NO_WORK);
            if !next_fire.is_zero() {
              thread::sleep(next_fire);
            }
            if let Some(job) = store.next_job() {
              #[allow(clippy::unit_arg)]
              if let Err(_task) = workers.submit(job) {
                // FIXME: no worker available! reschedule task
              }
            }
          }
        })
        .unwrap()
    };

    Self {
      halted,
      workers,
      store,
      handle,
    }
  }

  pub fn shutdown(self) {
    self.halted.store(true, std::sync::atomic::Ordering::Release);
    self.handle.join().expect("Scheduler thread panicked");
    Arc::try_unwrap(self.workers)
      .expect("worker pool is still being used!")
      .shutdown();
  }
}

impl Executable for Arc<Job> {
  fn exec(&self) {
    (self.target_fn)()
  }
}

#[cfg(test)]
mod tests {
  use crate::threading::SchedulerThread;
  use crate::JobStore;
  use std::num::NonZeroUsize;
  use std::sync::Arc;
  use std::thread;
  use std::time::Duration;

  #[test]
  fn api() {
    let t = SchedulerThread::new(NonZeroUsize::new(3).unwrap(), Arc::new(JobStore::new()));
    thread::sleep(Duration::from_millis(3));
    t.shutdown();
  }
}
