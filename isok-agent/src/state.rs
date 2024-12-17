use crate::jobs::Job;
use std::ops::Deref;
use std::time::Duration;

// Own a job and it's last execution
#[derive(Debug)]
pub struct JobState {
    last_run: Option<Duration>,
    job: Job,
}

impl JobState {
    pub(crate) fn new(job: Job) -> JobState {
        JobState {
            last_run: None,
            job,
        }
    }

    pub fn last_run(&self) -> Option<Duration> {
        self.last_run
    }
}

impl Deref for JobState {
    type Target = Job;
    fn deref(&self) -> &Self::Target {
        &self.job
    }
}
