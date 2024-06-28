use std::{
    io,
    num::NonZeroUsize,
    panic::UnwindSafe,
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, available_parallelism, current},
};

type ThreadPoolFunctionBoxed = Box<dyn FnOnce() + Sync + Send + UnwindSafe>;

struct SharedState {
    jobs_queued: usize,
    jobs_running: usize,
    job_paniced: usize,
}

impl SharedState {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            jobs_running: 0,
            jobs_queued: 0,
            job_paniced: 0,
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

    fn job_paniced(&mut self) {
        self.job_paniced += 1;
    }
}

#[allow(dead_code)]
pub struct ThreadPool {
    thread_amount: usize,
    job_sender: Sender<ThreadPoolFunctionBoxed>,
    shared_state: Arc<Mutex<SharedState>>,
}

impl ThreadPool {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn default() -> io::Result<Self> {
        let max_threads = available_parallelism()?;

        Self::new(max_threads)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn new(thread_amount: NonZeroUsize) -> io::Result<Self> {
        let thread_amount = thread_amount.get();

        let (job_sender, job_receiver) = channel::<ThreadPoolFunctionBoxed>();
        let job_reciever = Arc::new(Mutex::new(job_receiver));

        let shared_state = SharedState::new();

        // Create the threads
        for thread_num in 0..thread_amount {
            let job_reciever = job_reciever.clone();
            let shared_state = shared_state.clone();

            let thread_name = format!("Threadpool {thread_num}");

            thread::Builder::new()
                .name(thread_name)
                .spawn(Self::thread_function(job_reciever, shared_state))?;
        }

        Ok(Self {
            thread_amount,
            job_sender,
            shared_state,
        })
    }

    fn thread_function(
        job_reciever: Arc<Mutex<Receiver<ThreadPoolFunctionBoxed>>>,
        shared_state: Arc<Mutex<SharedState>>,
    ) -> impl FnOnce() + Send + 'static {
        move || loop {
            // TODO: Actually test this maybe
            let job = job_reciever.lock().expect("Cannot get reciever").recv();
            match job {
                Ok(job) => {
                    shared_state
                        .lock()
                        .expect("Couldn't get shared state")
                        .job_starting();

                    let result = std::panic::catch_unwind(job);

                    shared_state
                        .lock()
                        .expect("Couldn't get shared state")
                        .job_finished();

                    if let Err(err) = result {
                        shared_state
                            .lock()
                            .expect("Couldn't get shared state")
                            .job_paniced();

                        eprintln!(
                            "Job {:?} errored. Thread {} is panicing",
                            err,
                            current().name().unwrap_or("Unnamed")
                        );
                    }
                }
                Err(_) => break,
            }
        }
    }

    pub fn send_job(
        &mut self,
        job: impl FnOnce() + Sync + Send + UnwindSafe + 'static,
    ) -> Result<(), mpsc::SendError<ThreadPoolFunctionBoxed>> {
        // NOTE: It is essential that the shared state is updated FIRST otherwise
        // we have a race condidition that the job is transmitted and read before
        // the shared state is updated, leading to a negative amount of jobs queued
        self.shared_state
            .lock()
            .expect("Couldn't add job to shared state")
            .job_queued();
        self.job_sender.send(Box::new(job))?;
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

    pub fn threads_paniced(&self) -> usize {
        self.shared_state
            .lock()
            .expect("Couldn't get shared state")
            .job_paniced
    }

    pub fn has_paniced(&self) -> bool {
        self.threads_paniced() != 0
    }

    pub fn is_finished(&self) -> bool {
        self.jobs_running() == 0 && self.jobs_queued() == 0
    }

    #[allow(dead_code)]
    pub const fn threads(&self) -> usize {
        self.thread_amount
    }
}
