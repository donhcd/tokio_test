extern crate curl;
extern crate futures;
extern crate tokio_core;
extern crate tokio_curl;

use std::sync::mpsc;
use std::thread;

use curl::easy::Easy;
use futures::{Future, empty, finished};
use futures::stream::Stream;
use tokio_core::reactor::{Core, Remote};
use tokio_core::channel;
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
        let (sender, receiver) = channel::channel::<Vec<u8>>(h).unwrap();

        // Prepare the HTTP request to be sent.
        let session = Session::new(h.clone());
        let mut req = Easy::new();
        req.get(true).unwrap();
        req.url("https://www.rust-lang.org").unwrap();
        req.write_function(move |new_data| {
                let mut resp_data = Vec::with_capacity(new_data.len());
                resp_data.extend_from_slice(new_data);
                sender.send(resp_data).unwrap();
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
            receiver.collect().then(move |resp_bodies| {
                println!("ok it's done: ok? {}", resp_bodies.is_ok());
                outside_sender.send(format!("got page: {}", unsafe {
                        String::from_utf8_unchecked(resp_bodies.unwrap().swap_remove(0))
                    }))
                    .unwrap();
                Ok(())
            })
        }));
        finished(())
    });
    println!("outside: {}", outside_receiver.recv().unwrap())
}

fn main() {
    let core_handle = get_remote();
    foo(core_handle);
    // thread::sleep(time::Duration::from_secs(10));
}
