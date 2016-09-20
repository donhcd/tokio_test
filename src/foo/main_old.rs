extern crate curl;
extern crate futures;
extern crate tokio_core;
extern crate tokio_curl;

use std::io::{self, Write};
use std::sync::mpsc::channel;
use std::{thread, time};

use curl::easy::Easy;
use futures::{Future, empty, done};
use tokio_core::reactor::{Core, Remote};
use tokio_curl::Session;

fn bar() -> Remote {
    let (sender, receiver) = channel();
    // let mut core_handle = None;
    // core.run(empty::<(), ()>());
    thread::spawn(move || {
        // Create an event loop that we'll run on, as well as an HTTP `Session`
        // which we'll be routing all requests through.
        let mut core = Core::new().unwrap();
        sender.send(core.remote());

        core.run(empty::<(), ()>());
    });
    receiver.recv().unwrap()
}

fn foo(remote: Remote) {
    remote.spawn(|h| {
        // Prepare the HTTP request to be sent.
        let session = Session::new(h.clone());
        let mut req = Easy::new();
        req.get(true).unwrap();
        req.url("https://www.rust-lang.org").unwrap();
        let mut resp_data = Vec::new();
        {
            let mut transfer = req.transfer();
            // XXX don: is it okay to keep these unwraps? for reference:
            // http://alexcrichton.com/curl-rust/src/curl/src/easy.rs.html#2938-3041
            transfer.write_function(|new_data| {
                    resp_data.extend_from_slice(new_data);
                    println!("got some stuff: {}",
                             unsafe { String::from_utf8_unchecked(resp_data.clone()) });
                    Ok(new_data.len())
                })
                .unwrap();
            if let Err(err) = transfer.perform() {
                println!("failed to make request {:?}: {}", "foo", err);
            }
        }

        // Once we've got our session, issue an HTTP request to download the
        // rust-lang home page
        let request = session.perform(req);

        // Execute the request, and print the response code as well as the error
        // that happened (if any).
        h.spawn(request.then(|res| {
            match res {
                Ok(mut easy) => println!("resp code: {:?}", easy.response_code()),
                Err(err) => println!("booo: {}", err),
            }
            Ok(())
        }));
        // let mut req = h.clone().run(request).unwrap();
        // println!("{:?}", req.response_code());
        done(Ok(()))
    });
}

fn main() {
    let mut core_handle = bar();
    foo(core_handle);
    thread::sleep(time::Duration::from_secs(10));
}
