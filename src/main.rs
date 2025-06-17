use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum TimeUnit {
    Hours,
    Days,
    Minutes,
    Seconds,
    Weeks,
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
            weekday: None,
            repeat: None,
        }
    }
}

pub struct JobBuilder<'a> {
    interval: u64,
    time_unit: Option<TimeUnit>,
    at_time: Option<NaiveTime>,
    scheduler: &'a mut Scheduler,
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

    pub fn days(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Days);
        self
    }

    pub fn hours(mut self) -> Self {
        self.time_unit = Some(TimeUnit::Hours);
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
            time_unit: self.time_unit.unwrap(),
            at_time: self.at_time,
            task: Arc::new(job_fn),
            last_run: None,
            weekday: self.weekday,
            remaining_runs: self.repeat,
        };
        self.scheduler.jobs.push(job);
    }
}

impl Scheduler {
    pub fn run_pending(&mut self) {
        let now = Utc::now();
        for job in &mut self.jobs {
            if let Some(0) = job.remaining_runs {
                continue;
            }
            if let Some(wanted_day) = job.weekday {
                if now.weekday() != wanted_day {
                    continue;
                }
            }

            let should_run = match job.last_run {
                None => true,
                Some(last) => {
                    let elapsed = now - last;
                    let interval = match job.time_unit {
                        TimeUnit::Seconds => Duration::seconds(job.interval as i64),
                        TimeUnit::Minutes => Duration::minutes(job.interval as i64),
                        TimeUnit::Hours => Duration::hours(job.interval as i64),
                        TimeUnit::Days => Duration::days(job.interval as i64),
                        TimeUnit::Weeks => Duration::weeks(job.interval as i64),
                    };
                    elapsed >= interval
                }
            };

            if should_run {
                if let Some(at_time) = job.at_time {
                    if now.time() < at_time {
                        continue;
                    }

                    if let Some(last_run) = job.last_run {
                        match job.time_unit {
                            TimeUnit::Days | TimeUnit::Weeks => {
                                if last_run.date_naive() == now.date_naive() {
                                    continue;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                (job.task)();
                job.last_run = Some(now);
                if let Some(ref mut count) = job.remaining_runs {
                    *count -= 1;
                }
            }
        }
    }
}

fn main() {
    let mut scheduler = Scheduler::new();

    //scheduler
    //    .every(3)
    //    .seconds()
    //    .do_(|| println!("task scheduled"));

    //scheduler
    //    .every(1)
    //    .days()
    //    .at("19:24")
    //    .do_(|| println!("task scheduled"));
    //
    //scheduler
    //    .every(1)
    //    .week()
    //    .tuesday()
    //    .at("19:24")
    //    .do_(|| println!("task scheduled"));
    //
    scheduler
        .every(3)
        .seconds()
        .repeat(3)
        .do_(|| println!("task scheduled"));

    loop {
        scheduler.run_pending();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
