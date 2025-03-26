# Quartz Scheduler for Rust

Port of the original [Quartz Scheduler](https://www.quartz-scheduler.org/) from 
[Java](https://github.com/quartz-scheduler/quartz) to Rust.

## Status

All very much still work in progress... 

`v0.0.z` are to be considered experimental towards the first "beta", i.e. `v0.1.0`

To see the roadmap ahead in details, see the [milestones on Github](https://github.com/alexsnaps/quartz-rs/milestones)

## About

### Architecture


A `Scheduler` runs off a main scheduler thread that will dispatch `Job`s for execution to workers
from a thread pool, which is configurable in size. The dispatch occurs based off a `Trigger`
defining the actual schedule for a `Job` to fire.

### But... why?

I worked on the Java version of the Quartz Scheduler in a previous life. I already liked it as a user for its 
simplicity and enjoyed it even more for the exact same reason when I started extending it at Terracotta. I hope to 
bring the same joy to Rust developers looking for an easy to use and reliable scheduler.
