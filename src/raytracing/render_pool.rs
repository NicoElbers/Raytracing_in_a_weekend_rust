use std::{
    num::NonZeroUsize,
    sync::{
        mpsc::{self, channel, Sender},
        Arc, Mutex,
    },
    thread::{self, available_parallelism},
};

type ThreadPoolFunction = Box<dyn FnOnce() + Send>;

#[derive(Debug)]
struct InternalState {
    job_transmitter: Sender<ThreadPoolFunction>,
}

impl InternalState {
    fn new(job_transmitter: Sender<ThreadPoolFunction>) -> Self {
        Self { job_transmitter }
    }
}

struct SharedState {
    jobs_queued: usize,
    jobs_running: usize,
}

impl SharedState {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            jobs_running: 0,
            jobs_queued: 0,
        }))
    }

    fn job_starting(&mut self) {
        debug_assert!(self.jobs_queued > 0, "Negative jobs queued");

        self.jobs_queued -= 1;
        self.jobs_running += 1;
    }

    fn job_finished(&mut self) {
        debug_assert!(self.jobs_running > 0, "Negative jobs running");

        self.jobs_running -= 1;
    }

    fn job_queued(&mut self) {
        self.jobs_queued += 1;
    }
}

#[allow(dead_code)]
pub struct ThreadPool {
    max_threads: usize,
    internal_state: InternalState,
    shared_state: Arc<Mutex<SharedState>>,
}

impl Default for ThreadPool {
    fn default() -> Self {
        let max_threads = available_parallelism().expect("Cannot find available threads");

        Self::new(max_threads)
    }
}

impl ThreadPool {
    pub fn new(thread_amount: NonZeroUsize) -> Self {
        let thread_amount = thread_amount.get();
        let (tx_jobs, rx_jobs) = channel::<ThreadPoolFunction>();
        let job_reciever = Arc::new(Mutex::new(rx_jobs));
        let internal_state = InternalState::new(tx_jobs);
        let shared_state = SharedState::new();

        // Create the threads
        for _ in 0..thread_amount {
            let job_reciever = job_reciever.clone();
            let shared_state = shared_state.clone();

            thread::spawn(move || {
                while let Ok(job) = job_reciever.lock().expect("Cannot get job reciever").recv() {
                    shared_state
                        .lock()
                        .expect("Couldn't get shared state")
                        .job_starting();
                    job();
                    shared_state
                        .lock()
                        .expect("Couldn't get shared state")
                        .job_finished();
                }
            });
        }

        Self {
            max_threads: thread_amount,
            internal_state,
            shared_state,
        }
    }

    pub fn send_job(
        &mut self,
        job: ThreadPoolFunction,
    ) -> Result<(), mpsc::SendError<ThreadPoolFunction>> {
        self.internal_state.job_transmitter.send(job)?;
        self.shared_state
            .lock()
            .expect("Couldn't add job to shared state")
            .job_queued();
        Ok(())
    }

    pub fn jobs_running(&self) -> usize {
        self.shared_state
            .lock()
            .expect("Couldn't get shared state")
            .jobs_running
    }

    pub fn jobs_queued(&self) -> usize {
        self.shared_state
            .lock()
            .expect("Couldn't get shared state")
            .jobs_queued
    }

    pub fn is_finished(&self) -> bool {
        self.jobs_running() == 0 && self.jobs_queued() == 0
    }

    #[allow(dead_code)]
    pub const fn threads(&self) -> usize {
        self.max_threads
    }
}
