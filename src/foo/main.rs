extern crate curl;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate tokio_core;
extern crate tokio_curl;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::io::stdout;
use std::ops::DerefMut;
use std::sync::mpsc::channel;
use std::{thread, time};

use curl::easy::Easy;
use futures::{Future, empty, done, oneshot};
use tokio_core::reactor::{Core, Remote};
use tokio_curl::Session;

thread_local!{
    static VALUE_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

fn bar() -> Remote {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        // Create an event loop that we'll run on, as well as an HTTP `Session`
        // which we'll be routing all requests through.
        let mut core = Core::new().unwrap();
        sender.send(core.remote());

        core.run(empty::<(), ()>());
    });
    receiver.recv().unwrap()
}

fn foo(remote: Remote, url: &str) {
    let url = url.to_owned();
    println!("hi");
    remote.spawn(move |h| {
        let (c, p) = oneshot::<()>(); // just need to know when the value's in the cache
        VALUE_CACHE.with::<_, ()>(|value_cache| {
            let mut value_cache = value_cache.borrow_mut();
            println!("what's happening");
            if value_cache.contains_key(&url) {
                c.complete(());
                println!("complete 1");
            } else {
                // Prepare the HTTP request to be sent.
                let session = Session::new(h.clone());
                let mut req = Easy::new();
                req.get(true).unwrap();
                req.url(&url).unwrap();
                let mut resp_data = Vec::new();
                {
                    let mut transfer = req.transfer();
                    // XXX don: is it okay to keep these unwraps? for reference:
                    // http://alexcrichton.com/curl-rust/src/curl/src/easy.rs.html#2938-3041
                    transfer.write_function(|new_data| {
                            resp_data.extend_from_slice(new_data);
                            println!("got some stuff");
                            Ok(new_data.len())
                        })
                        .unwrap();
                    if let Err(err) = transfer.perform() {
                        println!("failed to make request {:?}: {}", url, err);
                    }
                }
                // req.write_function(move |data| {
                // io::stdout().write_all(data).unwrap();
                println!("adding some stuff");
                value_cache.insert(url.clone(),
                                   unsafe { String::from_utf8_unchecked(resp_data) });
                println!("added some stuff");
                // String::from_utf8_lossy(data).into_owned();
                // Ok(data.len())
                // })
                // .unwrap();

                // Once we've got our session, issue an HTTP request to download the
                // rust-lang home page
                let request = session.perform(req).wait();
                println!("performed req");
                c.complete(());

                // Execute the request, and print the response code as well as the error
                // that happened (if any).
                // h.spawn(request.then(|res| {
                //     println!("printing resp code");
                //     match res {
                //         Ok(mut easy) => println!("resp code: {:?}", easy.response_code()),
                //         Err(err) => println!("booo: {}", err),
                //     }
                //     c.complete(());
                //     println!("complete 2");
                //     Ok(())
                // }));
            }
            p.map(|res| {
                    println!("got page: {}", value_cache[&url]);
                })
                .wait();
            println!("this will never happen");
        });
        stdout().flush();
        // let mut req = h.clone().run(request).unwrap();
        // println!("{:?}", req.response_code());
        done(Ok(()))
    });
}

fn main() {
    let mut core_handle = bar();
    foo(core_handle, "https://www.rust-lang.org");
    thread::sleep(time::Duration::from_secs(10));
    stdout().flush();
}
