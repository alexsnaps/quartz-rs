# Quartz Scheduler for Rust

Port of the original [Quartz Scheduler](https://www.quartz-scheduler.org/) from 
[Java](https://github.com/quartz-scheduler/quartz) to Rust.

## Status

All very much still work in progress... 

`v0.0.z` are to be considered experimental towards the first "beta", i.e. `v0.1.0`.

To see the roadmap ahead in details, see the [milestones on Github](https://github.com/alexsnaps/quartz-rs/milestones)

## Usage example

```rust
let mut sched = Scheduler::new();

// computer a time that is 600 ms from now
let run_time = SystemTime::now() + Duration::from_millis(600);

println!("------- Scheduling Job  -------------------");

// define the job and tie it to a closure
let job = Job::with_identity(JOB_ID, "group1", || println!("Hello, world from {JOB_ID}!"));

// Trigger the job to run
let trigger = Trigger::with_identity("trigger1", "group1").start_at(run_time);

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
```

## About

### Architecture


A `Scheduler` runs off a main scheduler thread that will dispatch `Job`s for execution to workers
from a thread pool, which is configurable in size. The dispatch occurs based off a `Trigger`
defining the actual schedule for a `Job` to fire.

### But... why?

I worked on the Java version of the Quartz Scheduler in a previous life. I already liked it as a user for its 
simplicity and enjoyed it even more for the exact same reason when I started extending it at Terracotta. I hope to 
bring the same joy to Rust developers looking for an easy to use and reliable scheduler.
