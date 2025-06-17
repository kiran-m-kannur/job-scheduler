use chrono::{DateTime, Duration, NaiveTime, Utc};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone)]
pub enum TimeUnit {
    Hours,
    Days,
    Minutes,
    Seconds,
}

pub struct Job {
    interval: u64,
    time_unit: TimeUnit,
    at_time: Option<NaiveTime>,
    task: Arc<dyn Fn() + Send + Sync>,
    last_run: Option<DateTime<Utc>>,
}

//scheduler
pub struct Scheduler {
    jobs: Vec<Job>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler { jobs: vec![] }
    }

    pub fn every(&mut self, interval: u64) -> JobBuilder {
        JobBuilder {
            interval,
            scheduler: self,
            time_unit: None,
            at_time: None,
        }
    }
}

pub struct JobBuilder<'a> {
    interval: u64,
    time_unit: Option<TimeUnit>,
    at_time: Option<NaiveTime>,
    scheduler: &'a mut Scheduler,
}

impl<'a> JobBuilder<'a> {
    pub fn seconds(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Seconds);
        self
    }
    pub fn minutes(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Minutes);
        self
    }
    pub fn days(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Days);
        self
    }
    pub fn hours(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Hours);
        self
    }
    pub fn do_<F>(self, job_fn: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let job = Job {
            interval: self.interval,
            time_unit: self.time_unit.unwrap(),
            at_time: self.at_time,
            task: Arc::new(job_fn),
            last_run: None,
        };
        self.scheduler.jobs.push(job);
    }
}

impl Scheduler {
    pub fn run_pending(&mut self) {
        let now = Utc::now();

        for job in &mut self.jobs {
            let should_run = match job.last_run {
                None => true,
                Some(last) => {
                    let elapsed = now - last;
                    let interval = match job.time_unit {
                        TimeUnit::Seconds => Duration::seconds(job.interval as i64),
                        TimeUnit::Minutes => Duration::minutes(job.interval as i64),
                        TimeUnit::Hours => Duration::hours(job.interval as i64),
                        TimeUnit::Days => Duration::days(job.interval as i64),
                    };

                    elapsed >= interval
                }
            };

            if should_run {
                if let Some(at) = job.at_time {
                    if now.time() < at {
                        continue;
                    }
                }

                (job.task)();
                job.last_run = Some(now);
            }
        }
    }
}

fn main() {
    let mut scheduler = Scheduler::new();
    scheduler
        .every(5)
        .seconds()
        .do_(|| println!("task scheduled"));

    loop {
        scheduler.run_pending();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
