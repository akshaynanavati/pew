use libc;

pub struct Clock {
    is_paused: bool,
    start_time: u64,   // Start time in ns
    elapsed_time: u64, // Elapsed time in ns
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            is_paused: false,
            start_time: Clock::now(),
            elapsed_time: 0,
        }
    }

    pub fn pause(&mut self) {
        let now = Clock::now();
        if self.is_paused {
            panic!("Cannot pause an already paused clock");
        }

        self.elapsed_time += now - self.start_time;
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        if !self.is_paused {
            panic!("Cannot resume an already running clock");
        }

        self.is_paused = false;
        self.start_time = Clock::now();
    }

    pub fn stop(self) -> u64 {
        let now = Clock::now();
        if self.is_paused {
            panic!("Cannot stop a paused clock");
        }

        self.elapsed_time + now - self.start_time
    }

    fn now() -> u64 {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        unsafe {
            if libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts) == -1 {
                panic!("Error getting timespec");
            }
        }
        (ts.tv_sec * 1_000_000_000 + ts.tv_nsec) as u64
    }
}
