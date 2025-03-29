/*
 * Copyright 2024 Alex Snaps
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

//! # Quartz Scheduler
//!
//! This is a Rust port of the [Quartz Scheduler](https://www.quartz-scheduler.org/) originally written in
//! Java. Quartz can be integrated within pretty much any Rust application that targets a
//! multithreaded architecture.
//!
//! ## Highlevel architecture
//!
//! A [`Scheduler`] runs off a main scheduler thread that will dispatch [`Job`]s for execution to workers
//! from a thread pool, which is configurable in size. The dispatch occurs based off a [`Trigger`]
//! defining the actual schedule for a [`Job`] to fire.
//! ## Examples
//!
//! Basic usage example showing how to set up a simple job and trigger within the Quartz scheduler:
//!
//! ```rust
//! use quartz::{Scheduler, Job, Trigger};
//!
//! // Create a new scheduler instance
//! let scheduler = Scheduler::new();
//!
//! // Define a job with an `id`, `group` and a function to execute
//! let job = Job::with_identity(
//!     "basic_job",
//!     "default_group",
//!     || println!("Executing the basic job!")
//! );
//!
//! // Create a trigger with an identifier and a group
//! // to execute immediately and repeat twice, every 200ms
//! let trigger = Trigger::with_identity("basic_trigger", "default_group")
//!     .repeat(2)
//!     .every(std::time::Duration::from_millis(200));
//!
//! // Schedule the job using the trigger
//! scheduler.schedule_job(job, trigger);
//!
//! // Note: this example assumes the scheduler implementation is handling
//! // job executions based on its triggers appropriately in the background.
//! // Give it some time to execute
//! std::thread::sleep(std::time::Duration::from_secs(1));
//!
//! // finally shutting the scheduler down
//! scheduler.shutdown();
//! ```

mod job_store;
mod threading;

use crate::job_store::JobStore;
use crate::threading::SchedulerThread;

use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Entry point in Quartz, which also controls the lifecycle of the necessary resources.
/// The `Scheduler` is the entry point of the Quartz Scheduler, responsible for managing the
/// lifecycle of scheduling resources and orchestrating the execution of tasks.
///
/// # Examples
///
/// ```rust
/// use quartz::{Scheduler, Job, Trigger};
///
/// let scheduler = Scheduler::new();
///
/// let job = Job::with_identity(
///     "example_job",
///     "example_group",
///     || println!("Executing job!")
/// );
///
/// let trigger = Trigger::with_identity("trigger_id", "example_group");
/// scheduler.schedule_job(job, trigger);
/// ```
pub struct Scheduler {
  job_store: Arc<JobStore>,
  scheduler_thread: SchedulerThread,
}

impl Scheduler {
  /// Creates a new [`Scheduler`], initializing the storage for [`Job`]s, starts the scheduler
  /// thread and initializes the worker thread pool.
  ///
  /// # Returns
  ///
  /// A new instance of [`Scheduler`].
  pub fn new() -> Self {
    let job_store = Arc::new(JobStore::new());
    let scheduler_thread = SchedulerThread::new(NonZeroUsize::new(2).unwrap(), Arc::clone(&job_store));

    Self {
      job_store,
      scheduler_thread,
    }
  }

  /// Schedule a [`Job`], triggered according to the schedule described by the [`Trigger`]
  ///
  /// # Arguments
  ///
  /// * `job` - A [`Job`] instance describing the task to be executed.
  /// * `trigger` - A [`Trigger`] instance that specifies the schedule for the job.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use quartz::{Scheduler, Job, Trigger};
  ///
  /// let scheduler = Scheduler::new();
  /// let job = Job::with_identity(
  ///     "example_job",
  ///     "example_group",
  ///     || println!("This job is running!")
  /// );
  ///
  /// let trigger = Trigger::with_identity("trigger_id", "example_group");
  /// scheduler.schedule_job(job, trigger);
  /// ```
  pub fn schedule_job(&self, job: Job, trigger: Trigger) {
    self.job_store.add(job, trigger);
  }

  /// Shuts the [`Scheduler`] down, letting any [`Job`] currently executing run to the end
  ///
  /// Initiates the shutdown of the [`Scheduler`].
  ///
  /// This method terminates the scheduler's thread and prevents any
  /// new [`Job`]s from being scheduled. Any currently executing
  /// [`Job`]s will be allowed to complete before the shutdown
  /// process finishes.
  ///
  /// After calling this method, the [`Scheduler`] instance should
  /// no longer be used.
  pub fn shutdown(self) {
    self.scheduler_thread.shutdown();
  }
}

impl Default for Scheduler {
  fn default() -> Self {
    Self::new()
  }
}

/// Describes "what" is to be executed
///
/// A [`Job`] represents a unit of work that can be executed and scheduled by the [`Scheduler`].
/// Each [`Job`] has a unique `id` and belongs to a specific `group`.
/// The actual execution of the job is defined by the `target_fn`, which is a function to execute.
///
/// # Fields
/// - `id`: A unique identifier for the job.
/// - `group`: The group to which the job belongs, used for categorization.
/// - `target_fn`: The function to execute when the job is triggered.
///
/// # Examples
/// ```rust
/// use quartz::{Job, Scheduler, Trigger};
///
/// let scheduler = Scheduler::new();
/// let job = Job::with_identity(
///     "example_job",
///     "example_group",
///     || println!("This job is executing!")
/// );
///
/// scheduler.schedule_job(job, Trigger::with_identity("trigger_id", "example_group"));
/// ```
pub struct Job {
  id: String,
  group: String,
  target_fn: Box<dyn Fn() + Send + Sync>,
}

impl Debug for Job {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    write!(fmt, "Job {}::{}", self.group, self.id)
  }
}

impl Job {
  /// Creates a new [`Job`] that will execute the `target` and can be referenced by `id` and
  /// `target`, once [scheduled](Scheduler::schedule_job())
  pub fn with_identity<S: Into<String>>(id: S, group: S, target: impl Fn() + Send + Sync + 'static) -> Self {
    Self {
      id: id.into(),
      group: group.into(),
      target_fn: Box::new(target),
    }
  }

  /// Accessor to the [`Job`]'s identity
  ///
  /// # Returns
  ///
  /// The id of the [`Job`].
  pub fn id(&self) -> &str {
    &self.id
  }

  /// Accessor to the [`Job`]'s group.
  ///
  /// # Returns
  ///
  /// The group of the [`Job`].
  pub fn group(&self) -> &str {
    &self.group
  }

  /// Executes the [`Job`]'s target function.
  pub fn execute(&self) {
    (self.target_fn)();
  }
}

impl PartialEq for Job {
  fn eq(&self, other: &Self) -> bool {
    self.id.eq(&other.id) && self.group.eq(&other.group)
  }
}

/// Describes the schedule to use when [scheduling](Scheduler::schedule_job()) [`Job`]s with a
/// [`Scheduler`]
#[derive(Debug, PartialEq)]
pub struct Trigger {
  id: String,
  group: String,
  start_time: Option<SystemTime>,
  end_time: Option<SystemTime>,
  interval: Option<Duration>,
  repeat_count: Option<u32>,
}

impl Trigger {
  /// Creates a new [`Trigger`] that describes a schedule and can be referenced by `id` and
  /// `group`, and is used to [schedule](Scheduler::schedule_job()) a [`Job`].
  pub fn with_identity<S: Into<String>>(id: S, group: S) -> Self {
    Self {
      id: id.into(),
      group: group.into(),
      start_time: None,
      end_time: None,
      interval: None,
      repeat_count: None,
    }
  }

  /// Configures the [Trigger] to start execution at the specified `start_time`.
  ///
  /// # Arguments
  ///
  /// * `start_time` - A `SystemTime` value representing when the schedule should begin execution.
  ///
  /// # Returns
  ///
  /// Returns a new `Trigger` instance with the `start_time` configured.
  pub fn start_at(self, start_time: SystemTime) -> Self {
    Self {
      start_time: Some(start_time),
      ..self
    }
  }

  /// Sets the `end_time` at which the schedule [Trigger] is to stop.
  ///
  /// # Arguments
  ///
  /// * `end_time` - A `SystemTime` value representing when the schedule should end execution.
  ///
  /// # Returns
  ///
  /// Returns a new `Trigger` instance with the `end_time` configured.
  pub fn end_at(self, end_time: SystemTime) -> Self {
    Self {
      end_time: Some(end_time),
      ..self
    }
  }

  /// Configures the [Trigger] to execute repeatedly at the given interval.
  ///
  /// # Arguments
  ///
  /// * `interval` - A `Duration` representing the interval at which the trigger should repeat execution.
  ///
  /// # Returns
  ///
  /// Returns a new `Trigger` instance with the `interval` configured.
  pub fn every(self, interval: Duration) -> Self {
    Self {
      interval: Some(interval),
      ..self
    }
  }

  /// Sets the number of times the [Trigger] should repeat execution.
  ///
  /// # Arguments
  ///
  /// * `count` - A `u32` value specifying the number of repetitions for the schedule.
  ///
  /// # Returns
  ///
  /// Returns a new `Trigger` instance with the `count` configured.
  pub fn repeat(self, count: u32) -> Self {
    Self {
      repeat_count: Some(count),
      ..self
    }
  }

  /// Returns the next scheduled fire time for the [Trigger].
  ///
  /// The next fire time is determined based on the `start_time` of the `Trigger`. If no `start_time`
  /// is specified, it defaults to `SystemTime::now`.
  ///
  /// # Returns
  ///
  /// A `SystemTime` value representing when the [Trigger] is scheduled to fire next.
  pub fn next_fire(&self) -> SystemTime {
    self.start_time.unwrap_or_else(SystemTime::now)
  }
}

#[cfg(test)]
mod tests {
  use crate::{Job, Scheduler, Trigger};
  use std::thread;
  use std::time::{Duration, SystemTime};

  const JOB_ID: &str = "job1";

  #[test]
  fn test_basic_api() {
    // First we must get a reference to a scheduler
    let sched = Scheduler::new();

    // computer a time that is 600 ms from now
    let run_time = SystemTime::now() + Duration::from_millis(600);

    println!("------- Scheduling Job  -------------------");

    // define the job and tie it to a closure
    let job = Job::with_identity(JOB_ID, "group1", || println!("Hello, world from {JOB_ID}!"));

    // Trigger the job to run
    let trigger = Trigger::with_identity("trigger1", "group1")
      .start_at(run_time)
      .repeat(2)
      .every(Duration::from_millis(100));

    // Tell quartz to schedule the job using our trigger
    sched.schedule_job(job, trigger);
    println!("{JOB_ID} will run at: {run_time:?}");

    // wait long enough so that the scheduler as an opportunity to
    // run the job!
    println!("------- Waiting 1 second... -------------");
    // wait 1 seconds to show job
    thread::sleep(Duration::from_secs(1));
    // executing...

    // shut down the scheduler
    println!("------- Shutting Down ---------------------");
    sched.shutdown();
  }
}
