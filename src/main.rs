use chrono::{DateTime, Datelike, Duration, NaiveTime, Timelike, Utc, Weekday};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

pub trait Scheduler {
    fn run_if_due(&mut self, now: DateTime<Utc>);
}

pub struct Job {
    interval: u64,
    time_unit: TimeUnit,
    at_time: Option<NaiveTime>,
    task: Arc<dyn Fn() + Send + Sync>,
    last_run: Option<DateTime<Utc>>,
    weekday: Option<Weekday>,
    remaining_runs: Option<i32>,
}

impl Scheduler for Job {
    fn run_if_due(&mut self, now: DateTime<Utc>) {
        if let Some(0) = self.remaining_runs {
            return;
        }

        if let Some(wanted_day) = self.weekday {
            if now.weekday() != wanted_day {
                return;
            }
        }

        let should_run = match self.last_run {
            None => true,
            Some(last) => {
                let elapsed = now - last;
                let interval = match self.time_unit {
                    TimeUnit::Seconds => Duration::seconds(self.interval as i64),
                    TimeUnit::Minutes => Duration::minutes(self.interval as i64),
                    TimeUnit::Hours => Duration::hours(self.interval as i64),
                    TimeUnit::Days => Duration::days(self.interval as i64),
                    TimeUnit::Weeks => Duration::weeks(self.interval as i64),
                };
                elapsed >= interval
            }
        };

        if should_run {
            if let Some(at_time) = self.at_time {
                if now.time() < at_time {
                    return;
                }

                if let Some(last_run) = self.last_run {
                    match self.time_unit {
                        TimeUnit::Days | TimeUnit::Weeks => {
                            if last_run.date_naive() == now.date_naive() {
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            }

            (self.task)();
            self.last_run = Some(now);
            if let Some(ref mut count) = self.remaining_runs {
                *count -= 1;
            }
        }
    }
}

pub struct JobRunner {
    jobs: Vec<Box<dyn Scheduler>>,
}

impl JobRunner {
    pub fn new() -> Self {
        JobRunner { jobs: vec![] }
    }

    pub fn every(&mut self, interval: u64) -> JobBuilder {
        JobBuilder {
            interval,
            job_runner: self,
            time_unit: None,
            at_time: None,
            weekday: None,
            repeat: None,
        }
    }

    pub fn run_pending(&mut self) {
        let now = Utc::now();
        for job in &mut self.jobs {
            job.run_if_due(now);
        }
    }
}

pub struct JobBuilder<'a> {
    interval: u64,
    time_unit: Option<TimeUnit>,
    at_time: Option<NaiveTime>,
    job_runner: &'a mut JobRunner,
    weekday: Option<Weekday>,
    repeat: Option<i32>,
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
    pub fn hours(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Hours);
        self
    }
    pub fn days(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Days);
        self
    }
    pub fn week(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Weeks);
        self
    }

    pub fn at(mut self, time_str: &str) -> Self {
        self.at_time = Some(NaiveTime::parse_from_str(time_str, "%H:%M").unwrap());
        self
    }

    pub fn monday(mut self) -> Self {
        self.weekday = Some(Weekday::Mon);
        self
    }
    pub fn tuesday(mut self) -> Self {
        self.weekday = Some(Weekday::Tue);
        self
    }
    pub fn wednesday(mut self) -> Self {
        self.weekday = Some(Weekday::Wed);
        self
    }
    pub fn thursday(mut self) -> Self {
        self.weekday = Some(Weekday::Thu);
        self
    }
    pub fn friday(mut self) -> Self {
        self.weekday = Some(Weekday::Fri);
        self
    }
    pub fn saturday(mut self) -> Self {
        self.weekday = Some(Weekday::Sat);
        self
    }
    pub fn sunday(mut self) -> Self {
        self.weekday = Some(Weekday::Sun);
        self
    }

    pub fn repeat(mut self, count: i32) -> Self {
        self.repeat = Some(count);
        self
    }

    pub fn do_<F>(self, job_fn: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let job = Job {
            interval: self.interval,
            time_unit: self.time_unit.expect("TimeUnit required"),
            at_time: self.at_time,
            task: Arc::new(job_fn),
            last_run: None,
            weekday: self.weekday,
            remaining_runs: self.repeat,
        };

        self.job_runner.jobs.push(Box::new(job));
    }
}

fn main() {
    let mut runner = JobRunner::new();

    runner
        .every(3)
        .seconds()
        .repeat(3)
        .do_(|| println!("task scheduled"));

    loop {
        runner.run_pending();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
