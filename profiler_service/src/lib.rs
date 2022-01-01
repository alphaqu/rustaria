use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub static mut PROFILER: Profiler = Profiler {
    methods: None
};

pub fn profile() {
    unsafe {
        PROFILER.methods = Some(HashMap::new());
    }
}

pub fn method_time(method: &'static str, duration: Duration, sub_methods: Vec<&'static str>) {
    unsafe {
        if let Some(profiler) = &mut PROFILER.methods {
            let time = duration.as_millis();
            if let Some(method) = profiler.get_mut(method) {
                method.add(time);
            } else {
                profiler.insert(method, Method { time, invocations: 1, sub_methods });
            }
        }
    }
}

pub fn print() {
    println!("Printing stuff");
    unsafe {
        if let Some(profiler) = &mut PROFILER.methods {
            let mut things = Vec::new();

            for (name, time) in profiler {
                things.push((*name, time.time, time.invocations))
            }

            things.sort_by(|(_, v0, _), (_, v1, _)| v0.cmp(v1));

            for (method_name, time, invoked) in things {
                println!("{}/{}ms: {}", invoked, time, method_name);
            }
        }
    }
}

pub struct Profiler {
    methods: Option<HashMap<&'static str, Method>>,
}

pub struct Method {
    time: u128,
    invocations: u128,
    sub_methods: Vec<&'static str>,
}

impl Method {
    pub fn add(&mut self, time: u128) {
        self.time += time;
        self.invocations += 1;
    }
}