use std::cell::RefCell;

thread_local! {
    static STDOUT: RefCell<String> = RefCell::new(String::new());
}

pub fn clear_stdout() {
    STDOUT.with(|s| s.borrow_mut().clear());
}

pub fn get_stdout() -> String {
    STDOUT.with(|s| s.borrow().clone())
}

pub fn append_stdout(s: &str) {
    STDOUT.with(|stdout| stdout.borrow_mut().push_str(s));
}

pub fn append_stdout_ln(s: &str) {
    STDOUT.with(|stdout| {
        let mut b = stdout.borrow_mut();
        b.push_str(s);
        b.push('\n');
    });
}
