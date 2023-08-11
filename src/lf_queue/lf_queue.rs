use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub struct LF_Queue<T: Default + Clone> {
    store: Vec<T>,
    next_write_index: AtomicUsize,
    next_read_index: AtomicUsize,
    num_elements: AtomicUsize,
}

impl<T: Default + Clone> LF_Queue<T> {
    pub fn new(num_elemnts: usize) -> Self {
        LF_Queue {
            store: vec![T::default(); num_elemnts],
            next_write_index: AtomicUsize::new(0),
            next_read_index: AtomicUsize::new(0),
            num_elements: AtomicUsize::new(0),
        }
    }

    pub fn size(&self) -> usize {
        self.num_elements.load(Ordering::Relaxed)
    }

    pub fn get_next_to_write_to(&mut self) -> Option<&mut T> {
        let current_index = self.next_write_index.load(Ordering::Relaxed);
        if current_index < self.store.len() {
            Some(&mut self.store[current_index])
        } else {
            None
        }
    }

    pub fn get_next_to_read(&self) -> Option<&T> {
        if self.size() > 0 {
            let current_index = self.next_read_index.load(Ordering::Relaxed);
            Some(&self.store[current_index])
        } else {
            None
        }
    }

    pub fn update_write_index(&self) {
        let current_index = self.next_write_index.load(Ordering::Relaxed);
        let next_index = (current_index + 1) % self.store.len();
        self.next_write_index.store(next_index, Ordering::Relaxed);
        self.num_elements.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_read_index(&self) {
        let current_index = self.next_read_index.load(Ordering::Relaxed);
        let next_index = (current_index + 1) % self.store.len();
        self.next_read_index.store(next_index, Ordering::Relaxed);

        assert!(
            self.num_elements.load(Ordering::Relaxed) != 0,
            "Read an invalid element in thread: {:?}",
            thread::current().id()
        );

        self.num_elements.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::LF_Queue;
    use std::thread;

    #[test]
    fn test_basic_queue_operations() {
        let mut queue = LF_Queue::<u32>::new(5);
        assert_eq!(queue.size(), 0);

        let elem = queue.get_next_to_write_to().unwrap(); // Get position first
        *elem = 42; // Write to it
        queue.update_write_index(); // Then update the index

        assert_eq!(queue.size(), 1);

        let read_elem = queue.get_next_to_read().unwrap();
        assert_eq!(*read_elem, 42);
        queue.update_read_index();
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn test_multithreaded_queue_operations() {
        let queue = std::sync::Arc::new(std::sync::Mutex::new(LF_Queue::<u32>::new(1000)));
        let mut handles = vec![];

        // Spawn writers
        for _ in 0..5 {
            let queue_clone = queue.clone();
            handles.push(thread::spawn(move || {
                for i in 0..200 {
                    let mut queue_guard = queue_clone.lock().unwrap();
                    queue_guard.update_write_index();
                    let elem = queue_guard.get_next_to_write_to().unwrap();
                    *elem = i;
                }
            }));
        }

        // Spawn readers
        for _ in 0..5 {
            let queue_clone = queue.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..200 {
                    let mut queue_guard = queue_clone.lock().unwrap();
                    while queue_guard.get_next_to_read().is_none() {}
                    queue_guard.update_read_index();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(queue.lock().unwrap().size(), 0);
    }
}
