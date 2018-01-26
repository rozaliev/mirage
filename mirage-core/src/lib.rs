extern crate libc;
extern crate mio;
extern crate mirage_async;

use std::os::unix::io::{AsRawFd, RawFd};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::HashMap;

use mirage_async::{Async, Await};

thread_local!(static THREAD_CONTEXT: RefCell<Option<Context>> =  RefCell::new(None) );

const MAIN_TASK: mio::Token = mio::Token(1);

pub struct Core {
    tasks: HashMap<mio::Token, Box<Async<()>>>,

    poll: mio::Poll,
    events: mio::Events,
    new_async_interests: Rc<Cell<Option<(RawFd, mio::Ready)>>>,

    last_task_id: usize,
    awaiting: HashMap<mio::Token, Box<Async<()>>>,
}

#[derive(Clone)]
pub struct Context {
    new_tasks: Rc<RefCell<Vec<Box<Async<()>>>>>,
    new_async_interests: Rc<Cell<Option<(RawFd, mio::Ready)>>>,
    current_token: Rc<Cell<mio::Token>>,
}

pub fn context() -> Context {
    THREAD_CONTEXT.with(|c| {
        c.borrow()
            .as_ref()
            .expect("there is no core in this thread")
            .clone()
    })
}

impl Core {
    pub fn new() -> Core {
        let new_tasks = Rc::new(RefCell::new(Vec::new()));
        let new_async_interests = Rc::new(Cell::new(None));

        THREAD_CONTEXT.with(|c| {
            let mut b = c.borrow_mut();
            *b = Some(Context {
                new_tasks: new_tasks.clone(),
                new_async_interests: new_async_interests.clone(),
                current_token: Rc::new(Cell::new(MAIN_TASK)),
            });
        });

        Core {
            tasks: HashMap::new(),
            poll: mio::Poll::new().unwrap(),
            events: mio::Events::with_capacity(1024),
            new_async_interests: new_async_interests,
            last_task_id: 1,
            awaiting: HashMap::new(),
        }
    }

    pub fn run<T, A: Async<T>>(&mut self, mut g: A) -> T {
        let mut compl = Vec::new();
        let mut new_awaiting = Vec::new();

        'main: loop {
            set_current_token(MAIN_TASK);
            if let Await::Done(r) = unsafe { g.poll() } {
                return r;
            }

            if let Some(v) = self.new_async_interests.get().take() {
                match self.poll.register(
                    &mio::unix::EventedFd(&v.0),
                    MAIN_TASK,
                    v.1,
                    mio::PollOpt::edge(),
                ) {
                    Ok(_) => {}
                    Err(ref e) if e.raw_os_error() == Some(libc::EEXIST) => {
                        self.poll
                            .reregister(
                                &mio::unix::EventedFd(&v.0),
                                MAIN_TASK,
                                v.1,
                                mio::PollOpt::edge(),
                            )
                            .unwrap();
                    }
                    Err(err) => panic!(err),
                }
            }

            // trace!("main task not ready");

            loop {
                // trace!("inner loop");
                'inner_tasks: for (tok, task) in self.tasks.iter_mut() {
                    // trace!("inner tasks: {:?}", tok);
                    set_current_token(*tok);
                    match unsafe { task.poll() } {
                        Await::NotReady => {
                            if let Some(v) = self.new_async_interests.get().take() {
                                match self.poll.register(
                                    &mio::unix::EventedFd(&v.0),
                                    *tok,
                                    v.1,
                                    mio::PollOpt::edge(),
                                ) {
                                    Ok(_) => {}
                                    Err(ref e) if e.raw_os_error() == Some(libc::EEXIST) => {
                                        self.poll
                                            .reregister(
                                                &mio::unix::EventedFd(&v.0),
                                                *tok,
                                                v.1,
                                                mio::PollOpt::edge(),
                                            )
                                            .unwrap();
                                    }
                                    Err(err) => panic!(err),
                                }
                            }
                            new_awaiting.push(*tok);
                        }
                        Await::Done(()) => {
                            compl.push(*tok);
                        }
                    }
                }

                for i in compl.drain(..) {
                    // trace!("task {:?} completed", i);
                    self.tasks.remove(&i);
                }

                for tok in new_awaiting.drain(..) {
                    // trace!("task {:?} scheduled to await", tok);
                    if let Some(task) = self.tasks.remove(&tok) {
                        self.awaiting.insert(tok, task);
                    }
                }

                THREAD_CONTEXT.with(|c| {
                    let mut b = c.borrow_mut();
                    let new_tasks_ref = &mut b.as_mut().unwrap().new_tasks;
                    let mut new_tasks = new_tasks_ref.borrow_mut();

                    for t in new_tasks.drain(..) {
                        let next_tok = self.next_tok();
                        // trace!("new task added {:?}", next_tok);
                        self.tasks.insert(next_tok, t);
                    }
                });

                // main fd
                {
                    let mut main_fired = false;
                    let dur = if self.tasks.len() == 0 {
                        // trace!("no active tasks, awaiting io");
                        None
                    } else {
                        Some(::std::time::Duration::from_millis(1))
                    };
                    self.poll.poll(&mut self.events, dur).unwrap();
                    for event in &self.events {
                        let tok = event.token();
                        if tok == MAIN_TASK {
                            // trace!("main task awaken");
                            main_fired = true;
                        } else {
                            if let Some(task) = self.awaiting.remove(&tok) {
                                // trace!("task {:?} awaken", tok);
                                self.tasks.insert(tok, task);
                            }
                        }
                    }

                    if main_fired {
                        continue 'main;
                    }
                }
            }
        }
    }

    fn next_tok(&mut self) -> mio::Token {
        self.last_task_id += 1;
        mio::Token(self.last_task_id)
    }
}

impl Context {
    pub fn spawn<A: Async<()> + 'static>(&self, a: A) {
        let mut tasks = self.new_tasks.borrow_mut();
        tasks.push(Box::new(a));
    }

    pub fn register_read<T: AsRawFd>(&self, fd: &T) {
        self.new_async_interests
            .set(Some((fd.as_raw_fd(), mio::Ready::readable())));
    }

    pub fn register_write<T: AsRawFd>(&self, fd: &T) {
        self.new_async_interests
            .set(Some((fd.as_raw_fd(), mio::Ready::writable())));
    }

    pub fn register_all<T: AsRawFd>(&self, fd: &T) {
        self.new_async_interests.set(Some((
            fd.as_raw_fd(),
            mio::Ready::readable() | mio::Ready::writable(),
        )));
    }
}

fn set_current_token(t: mio::Token) {
    THREAD_CONTEXT.with(|c| c.borrow().as_ref().unwrap().current_token.set(t))
}
