extern crate futures;
extern crate tokio_core;

use std::env;
use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;
use tokio_core::io::{copy, Io};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    // Create the event loop that will drive this server
    let mut l = Core::new().unwrap();
    let handle = l.handle();

    // Create a TCP listener which will listen for incoming connections
    let socket = TcpListener::bind(&addr, &handle).unwrap();

    // Once we've got the TCP listener, inform that we have it
    println!("Listening on: {}", addr);

    // Pull out the stream of incoming connections and then for each new
    // one spin up a new task copying data.
    //
    // We use the `io::copy` future to copy all data from the
    // reading half onto the writing half.
    let done = socket.incoming().and_then(|(socket, addr)| {
        // let pair = futures::lazy(|| Ok(socket.split()));
        // let amt = pair.and_then(|(reader, writer)| copy(reader, writer));

        // // Once all that is done we print out how much we wrote, and then
        // // critically we *spawn* this future which allows it to run
        // // concurrently with other connections.
        // handle.spawn(amt.then(move |result| {
        //     println!("wrote {:?} bytes to {}", result, addr);
        //     Ok(())
        // }));

        // ()
        unimplemented!()
    });

    // Execute our server (modeled as a future) and wait for it to
    // complete.
    l.run(done).unwrap();
}
