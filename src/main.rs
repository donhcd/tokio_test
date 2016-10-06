extern crate curl;
extern crate futures;
extern crate tokio_core;
extern crate tokio_curl;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use curl::easy::Easy;
use futures::{Future, empty, finished};
use tokio_core::reactor::{Core, Remote};
use tokio_curl::Session;

fn get_remote() -> Remote {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        // Create an event loop that we'll run on
        let mut core = Core::new().unwrap();
        sender.send(core.remote()).unwrap();

        core.run(empty::<(), ()>()).unwrap();
    });
    receiver.recv().unwrap()
}

fn foo(remote: Remote) {
    let (outside_sender, outside_receiver) = mpsc::channel();
    remote.spawn(move |h| {
        // Prepare the HTTP request to be sent.
        let session = Session::new(h.clone());
        let mut req = Easy::new();
        let response = Arc::new(Mutex::new(Vec::new()));
        req.get(true).unwrap();
        req.url("https://www.rust-lang.org").unwrap();
        let response2 = response.clone();
        req.write_function(move |new_data| {
                response2.lock().unwrap().extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();

        // Once we've got our session, issue an HTTP request to download the
        // rust-lang home page
        let request = session.perform(req);

        // Execute the request, and print the response code as well as the error
        // that happened (if any).
        h.spawn(request.then(move |req_res| {
            match req_res {
                Ok(mut easy) => println!("resp code: {:?}", easy.response_code()),
                Err(err) => println!("booo: {}", err),
            }
            outside_sender.send(format!("got page: {}", unsafe {
                    String::from_utf8_unchecked(::std::mem::replace(&mut *response.lock().unwrap(),
                                                                    vec![]))
                }))
                .unwrap();
            Ok(())
        }));
        finished(())
    });
    println!("outside: {}", outside_receiver.recv().unwrap())
}

fn main() {
    let core_handle = get_remote();
    foo(core_handle);
}
